//! MCP (Model Context Protocol) 协议定义
//! 定义MCP协议的消息格式和数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP请求消息
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "method", content = "params")]
pub enum RequestMessage {
    /// 请求可用的工具列表
    #[serde(rename = "tools/list")]
    ToolsList {},
    
    /// 执行指定工具
    #[serde(rename = "tools/call")]
    ToolCall {
        name: String,
        arguments: Option<HashMap<String, serde_json::Value>>,
    },
    
    /// 请求可用资源列表
    #[serde(rename = "resources/list")]
    ResourcesList {},
    
    /// 获取指定资源内容
    #[serde(rename = "resources/read")]
    ResourceRead {
        uri: String,
    },
    
    /// 订阅资源变更
    #[serde(rename = "resources/subscribe")]
    ResourceSubscribe {
        uri: String,
    },
    
    /// 取消订阅资源变更
    #[serde(rename = "resources/unsubscribe")]
    ResourceUnsubscribe {
        uri: String,
    },
    
    /// 发送心跳
    #[serde(rename = "ping")]
    Ping {},
}

/// MCP响应消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseMessage {
    pub id: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<ResponseError>,
}

/// MCP错误响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// 工具定义
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// 文件信息定义
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: String,
    pub created: String,
    pub is_directory: bool,
}

/// 资源定义
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: String,
}