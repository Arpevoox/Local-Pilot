//! MCP (Model Context Protocol) 客户端实现
//! 用于通过stdio与MCP服务器进行通信

use crate::mcp::protocol::{RequestMessage, ResponseMessage, Tool, Resource};
use serde_json::Value;
use std::collections::HashMap;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use std::sync::Arc;
use uuid::Uuid;

/// MCP客户端结构体
pub struct McpClient {
    child_process: Arc<Mutex<Option<tokio::process::Child>>>,
    stdin_tx: Arc<Mutex<Option<tokio::io::StdinWriteHalf>>>,
    response_channels: Arc<Mutex<HashMap<String, mpsc::Sender<ResponseMessage>>>>,
}

impl McpClient {
    /// 创建新的MCP客户端并启动子进程
    pub async fn new(mcp_server_cmd: Vec<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cmd = Command::new(mcp_server_cmd[0]);
        for arg in &mcp_server_cmd[1..] {
            cmd.arg(arg);
        }
        
        let mut child = cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        
        let (stdin_reader, stdin_writer) = tokio::io::split(stdin);
        let _stdout_reader = stdout;
        
        let response_channels: Arc<Mutex<HashMap<String, mpsc::Sender<ResponseMessage>>>> = 
            Arc::new(Mutex::new(HashMap::new()));
        
        // 启动监听stdout的异步任务
        let channels_clone = Arc::clone(&response_channels);
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(response) = serde_json::from_str::<ResponseMessage>(&line) {
                    if let Some(id) = &response.id {
                        let mut channels = channels_clone.lock().await;
                        if let Some(sender) = channels.remove(id) {
                            let _ = sender.send(response.clone());
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            child_process: Arc::new(Mutex::new(Some(child))),
            stdin_tx: Arc::new(Mutex::new(Some(stdin_writer))),
            response_channels,
        })
    }

    /// 发送请求到MCP服务器并通过stdio接收响应
    pub async fn send_request(&self, request: RequestMessage) -> Result<ResponseMessage, Box<dyn std::error::Error>> {
        let request_id = Uuid::new_v4().to_string();
        
        // 创建响应通道
        let (response_tx, mut response_rx) = mpsc::channel(1);
        {
            let mut channels = self.response_channels.lock().await;
            channels.insert(request_id.clone(), response_tx);
        }
        
        // 序列化请求并发送
        let mut request_map = serde_json::Map::new();
        request_map.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
        request_map.insert("id".to_string(), Value::String(request_id.clone()));
        
        match request {
            RequestMessage::ToolsList {} => {
                request_map.insert("method".to_string(), Value::String("tools/list".to_string()));
            },
            RequestMessage::ToolCall { name, arguments } => {
                request_map.insert("method".to_string(), Value::String("tools/call".to_string()));
                let mut params = serde_json::Map::new();
                params.insert("name".to_string(), Value::String(name));
                if let Some(args) = arguments {
                    params.insert("arguments".to_string(), serde_json::to_value(args)?);
                }
                request_map.insert("params".to_string(), Value::Object(params));
            },
            RequestMessage::ResourcesList {} => {
                request_map.insert("method".to_string(), Value::String("resources/list".to_string()));
            },
            RequestMessage::ResourceRead { uri } => {
                request_map.insert("method".to_string(), Value::String("resources/read".to_string()));
                let mut params = serde_json::Map::new();
                params.insert("uri".to_string(), Value::String(uri));
                request_map.insert("params".to_string(), Value::Object(params));
            },
            RequestMessage::ResourceSubscribe { uri } => {
                request_map.insert("method".to_string(), Value::String("resources/subscribe".to_string()));
                let mut params = serde_json::Map::new();
                params.insert("uri".to_string(), Value::String(uri));
                request_map.insert("params".to_string(), Value::Object(params));
            },
            RequestMessage::ResourceUnsubscribe { uri } => {
                request_map.insert("method".to_string(), Value::String("resources/unsubscribe".to_string()));
                let mut params = serde_json::Map::new();
                params.insert("uri".to_string(), Value::String(uri));
                request_map.insert("params".to_string(), Value::Object(params));
            },
            RequestMessage::Ping {} => {
                request_map.insert("method".to_string(), Value::String("ping".to_string()));
            },
        }
        
        let json_request = serde_json::Value::Object(request_map);
        let request_str = serde_json::to_string(&json_request)?;
        
        // 发送到stdin
        {
            let mut stdin = self.stdin_tx.lock().await;
            if let Some(ref mut writer) = *stdin {
                writer.write_all(request_str.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
        }
        
        // 等待响应
        match tokio::time::timeout(tokio::time::Duration::from_secs(30), response_rx.recv()).await {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Err("Channel closed unexpectedly".into()),
            Err(_) => Err("Timeout waiting for response".into()),
        }
    }

    /// 获取可用工具列表
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let request = RequestMessage::ToolsList {};
        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            let tools: Vec<Tool> = serde_json::from_value(result)?;
            Ok(tools)
        } else {
            Err("No result in response".into())
        }
    }

    /// 调用指定工具
    pub async fn call_tool(
        &self,
        name: String,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let request = RequestMessage::ToolCall { name, arguments };
        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            Ok(result)
        } else {
            Err("No result in response".into())
        }
    }

    /// 获取可用资源列表
    pub async fn list_resources(&self) -> Result<Vec<Resource>, Box<dyn std::error::Error>> {
        let request = RequestMessage::ResourcesList {};
        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            let resources: Vec<Resource> = serde_json::from_value(result)?;
            Ok(resources)
        } else {
            Err("No result in response".into())
        }
    }

    /// 读取指定资源内容
    pub async fn read_resource(&self, uri: String) -> Result<Value, Box<dyn std::error::Error>> {
        let request = RequestMessage::ResourceRead { uri };
        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            Ok(result)
        } else {
            Err("No result in response".into())
        }
    }
}