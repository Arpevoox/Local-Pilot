//! MCP (Model Context Protocol) 模块
//! 用于处理 MCP 协议相关的逻辑

pub mod protocol;
pub mod client;
pub mod server;

pub use client::McpClient;

/// 初始化MCP功能
pub fn init_mcp() {
    println!("Initializing MCP (Model Context Protocol)...");
}

/// 工具安全检查 - 判断是否需要审批
pub fn requires_approval(tool_name: &str) -> bool {
    let unsafe_keywords = ["write", "delete", "move", "rm", "remove", "mv", "rename", "modify"];
    
    for keyword in &unsafe_keywords {
        if tool_name.to_lowercase().contains(keyword) {
            return true;
        }
    }
    false
}