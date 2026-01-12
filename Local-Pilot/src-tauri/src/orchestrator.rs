//! AI 编排逻辑模块
//! 处理 "思考 -> 工具调用 -> 反馈" 循环

use crate::mcp::{McpClient, protocol::{Tool, Resource, FileInfo}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

/// 工具调用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCallStatus {
    PendingApproval,
    Approved,
    Executed,
    Failed,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub arguments: Value,
    pub status: ToolCallStatus,
    pub result: Option<Value>,
    pub error: Option<String>,
}

/// 编排器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorStatus {
    Thinking,
    CallingTool,
    WaitingApproval,
    Processing,
    Completed,
}

/// 编排器结构体
pub struct Orchestrator {
    mcp_client: Arc<Mutex<Option<McpClient>>>,
    api_key: String,
    api_base: String,
    model_name: String,
}

/// AI响应结构
#[derive(Debug, Deserialize)]
struct AiResponse {
    pub content: Vec<AiContent>,
    pub stop_reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AiContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { 
        id: String, 
        name: String, 
        input: Value 
    },
}

impl Orchestrator {
    /// 创建新的编排器实例
    pub fn new(api_key: String, api_base: String, model_name: String) -> Self {
        Self {
            mcp_client: Arc::new(Mutex::new(None)),
            api_key,
            api_base,
            model_name,
        }
    }

    /// 设置MCP客户端
    pub async fn set_mcp_client(&mut self, client: McpClient) {
        let mut client_guard = self.mcp_client.lock().await;
        *client_guard = Some(client);
    }

    /// 获取可用工具列表
    pub async fn list_available_tools(&self) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let client_guard = self.mcp_client.lock().await;
        if let Some(ref client) = *client_guard {
            client.list_tools().await.map_err(|e| e.into())
        } else {
            Ok(Vec::new()) // 返回空列表，如果没有客户端
        }
    }

    /// 执行工具调用
    pub async fn execute_tool_call(
        &self,
        tool_name: String,
        arguments: Value,
    ) -> Result<ToolCallResult, Box<dyn std::error::Error>> {
        let requires_approval = crate::mcp::requires_approval(&tool_name);
        
        if requires_approval {
            return Ok(ToolCallResult {
                tool_name,
                arguments,
                status: ToolCallStatus::PendingApproval,
                result: None,
                error: Some("This action requires approval".to_string()),
            });
        }

        let client_guard = self.mcp_client.lock().await;
        if let Some(ref client) = *client_guard {
            match client.call_tool(tool_name.clone(), Some(arguments.as_object().unwrap().clone())).await {
                Ok(result) => {
                    Ok(ToolCallResult {
                        tool_name,
                        arguments,
                        status: ToolCallStatus::Executed,
                        result: Some(result),
                        error: None,
                    })
                }
                Err(e) => {
                    Ok(ToolCallResult {
                        tool_name,
                        arguments,
                        status: ToolCallStatus::Failed,
                        result: None,
                        error: Some(e.to_string()),
                    })
                }
            }
        } else {
            Ok(ToolCallResult {
                tool_name,
                arguments,
                status: ToolCallStatus::Failed,
                result: None,
                error: Some("MCP client not available".to_string()),
            })
        }
    }

    /// 构建系统提示，包含可用工具信息
    fn build_system_prompt(&self, tools: &[Tool]) -> String {
        let tools_json = tools.iter()
            .map(|tool| format!(
                r#"{{"name": "{}", "description": "{}", "input_schema": {}}}"#,
                tool.name, tool.description, tool.input_schema
            ))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            r#"You are an AI assistant that can interact with local system tools through the Model Context Protocol (MCP).
Available tools:
[
{}
]

When responding to user queries, if you need to perform an action, use the appropriate tool by calling it with the required arguments.
Follow these rules:
1. Always use the exact tool names as provided.
2. Provide all required arguments according to the input schema.
3. For potentially destructive actions (containing 'write', 'delete', 'move'), ask for confirmation before executing.
4. To search for local files, use the 'search_local_files' tool with a query parameter.
5. Respond with plain text when providing explanations or summaries."#,
            tools_json
        )
    }

    /// 调用LLM API获取响应
    async fn call_llm_api(
        &self,
        messages: Vec<HashMap<String, Value>>,
        tools: &[Tool],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let system_prompt = self.build_system_prompt(tools);
        
        // 使用reqwest创建HTTP客户端
        let client = reqwest::Client::new();
        
        // 构建请求体
        let mut body = serde_json::Map::new();
        body.insert("model".to_string(), Value::String(self.model_name.clone()));
        body.insert("messages".to_string(), serde_json::to_value(&messages)?);
        body.insert("system".to_string(), Value::String(system_prompt));
        body.insert("max_tokens".to_string(), Value::Number(serde_json::Number::from(1024)));
        body.insert("temperature".to_string(), Value::Number(serde_json::Number::from_f64(0.7).unwrap()));
        
        // 检查是否为Anthropic API
        let is_anthropic = self.api_base.contains("anthropic.com") || self.api_base.contains("openai.com");
        
        let response = if is_anthropic {
            // Anthropic API 请求
            client
                .post(&format!("{}/messages", self.api_base))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?
        } else {
            // 其他API提供商（如OpenAI兼容接口）
            client
                .post(&self.api_base)
                .header("authorization", format!("Bearer {}", &self.api_key))
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?
        };
        
        let response_text = response.text().await?;
        
        // 解析响应
        // 对于Anthropic API，响应格式不同，需要特别处理
        if is_anthropic {
            // Anthropic响应格式
            let anthropic_response: serde_json::Value = serde_json::from_str(&response_text)?;
            
            if let Some(content_array) = anthropic_response.get("content").and_then(|v| v.as_array()) {
                let mut result = String::new();
                for content_item in content_array {
                    if let Some(text_type) = content_item.get("type").and_then(|v| v.as_str()) {
                        match text_type {
                            "text" => {
                                if let Some(text) = content_item.get("text").and_then(|v| v.as_str()) {
                                    result.push_str(text);
                                }
                            }
                            "tool_use" => {
                                if let (Some(name), Some(input)) = (
                                    content_item.get("name").and_then(|v| v.as_str()),
                                    content_item.get("input"),
                                ) {
                                    result.push_str(&format!(
                                        "[TOOL_USE: {} with args: {}]", 
                                        name, 
                                        serde_json::to_string(input)?
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(result)
            } else {
                Ok(response_text)
            }
        } else {
            // 其他API响应格式
            Ok(response_text)
        }
    }

    /// 处理用户消息
    pub async fn process_user_message(
        &self,
        user_message: &str,
    ) -> Result<Vec<ToolCallResult>, Box<dyn std::error::Error>> {
        // 1. 获取可用工具
        let available_tools = self.list_available_tools().await?;
        
        // 2. 准备消息
        let mut messages = Vec::new();
        let mut user_msg = HashMap::new();
        user_msg.insert("role".to_string(), Value::String("user".to_string()));
        user_msg.insert("content".to_string(), Value::String(user_message.to_string()));
        messages.push(user_msg);
        
        // 3. 调用LLM
        let llm_response = self.call_llm_api(messages, &available_tools).await?;
        
        // 4. 解析LLM响应并执行工具调用（如果有的话）
        let mut tool_results = Vec::new();
        
        // 解析LLM响应中的工具调用
        if llm_response.contains("[TOOL_USE:") {
            // 这里解析工具调用命令
            // 简化的解析逻辑，实际实现中需要更复杂的解析
            for line in llm_response.lines() {
                if line.contains("[TOOL_USE:") {
                    // 提取工具名称和参数
                    if let Some(start_idx) = line.find("[TOOL_USE: ") {
                        if let Some(end_idx) = line.find(" with args: ") {
                            let tool_name = &line[start_idx + 11..end_idx]; // 11 is length of "[TOOL_USE: "
                            
                            // 提取参数部分
                            let args_start = end_idx + 10; // 10 is length of " with args: "
                            let args_part = &line[args_start..line.len()-1]; // remove closing ']'
                            
                            if let Ok(args_value) = serde_json::from_str::<Value>(args_part) {
                                // 执行工具调用
                                let result = self.execute_tool_call(tool_name.to_string(), args_value).await?;
                                tool_results.push(result);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(tool_results)
    }

    /// 批准待定的工具调用
    pub async fn approve_tool_call(
        &self,
        tool_name: String,
        arguments: Value,
    ) -> Result<ToolCallResult, Box<dyn std::error::Error>> {
        let client_guard = self.mcp_client.lock().await;
        if let Some(ref client) = *client_guard {
            match client.call_tool(tool_name.clone(), Some(arguments.as_object().unwrap().clone())).await {
                Ok(result) => {
                    Ok(ToolCallResult {
                        tool_name,
                        arguments,
                        status: ToolCallStatus::Approved,
                        result: Some(result),
                        error: None,
                    })
                }
                Err(e) => {
                    Ok(ToolCallResult {
                        tool_name,
                        arguments,
                        status: ToolCallStatus::Failed,
                        result: None,
                        error: Some(e.to_string()),
                    })
                }
            }
        } else {
            Ok(ToolCallResult {
                tool_name,
                arguments,
                status: ToolCallStatus::Failed,
                result: None,
                error: Some("MCP client not available".to_string()),
            })
        }
    }
}