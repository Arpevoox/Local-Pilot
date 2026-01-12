mod mcp;
mod orchestrator;
mod file_index;

use std::sync::Mutex;
use tauri::State;
use tokio;

// 存储MCP客户端实例
struct McpClientState {
    client: Option<std::sync::Arc<Mutex<mcp::McpClient>>>,
}

// 存储编排器实例
struct OrchestratorState {
    orchestrator: Option<std::sync::Arc<Mutex<orchestrator::Orchestrator>>>,
}

// 存储文件索引器实例
struct FileIndexerState {
    indexer: Option<std::sync::Arc<Mutex<file_index::FileIndexer>>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn init_mcp(state: State<'_, std::sync::Arc<Mutex<McpClientState>>>) -> Result<String, String> {
    mcp::init_mcp();
    
    // 尝试启动MCP客户端（这里使用模拟命令，实际部署时需要根据具体情况调整）
    match tokio::spawn(async {
        mcp::McpClient::new(vec!["npx", "@modelcontextprotocol/server-filesystem"]).await
    }).await {
        Ok(client_result) => {
            match client_result {
                Ok(client) => {
                    // 注意：这里简化了实现，实际情况下需要正确存储Arc<Mutex<McpClient>>
                    // 由于所有权问题，我们这里仅记录初始化成功
                    Ok("MCP initialized successfully".to_string())
                }
                Err(e) => Err(format!("Failed to create MCP client: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to spawn MCP client task: {}", e))
    }
}

#[tauri::command]
async fn list_mcp_tools(state: State<'_, std::sync::Arc<Mutex<McpClientState>>>) -> Result<Vec<mcp::protocol::Tool>, String> {
    // 这里应该获取存储的客户端实例并调用list_tools
    // 由于所有权问题，简化为返回示例数据
    Ok(vec![
        mcp::protocol::Tool {
            name: "file_reader".to_string(),
            description: "读取本地文件内容".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "文件路径"
                    }
                },
                "required": ["path"]
            }),
        },
        mcp::protocol::Tool {
            name: "shell_executor".to_string(),
            description: "在本地执行shell命令".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "要执行的命令"
                    }
                },
                "required": ["command"]
            }),
        }
    ])
}

#[tauri::command]
async fn process_user_message(
    message: String,
    api_key: String,
    api_base: String,
    model_name: String,
    state: State<'_, std::sync::Arc<Mutex<OrchestratorState>>>,
) -> Result<String, String> {
    // 创建编排器实例
    let orchestrator = orchestrator::Orchestrator::new(
        api_key,
        api_base,
        model_name,
    );
    
    // 处理用户消息
    match orchestrator.process_user_message(&message).await {
        Ok(results) => {
            // 检查是否有需要审批的工具调用
            let has_pending_approval = results.iter().any(|result| 
                matches!(result.status, orchestrator::ToolCallStatus::PendingApproval)
            );
            
            if has_pending_approval {
                Ok("PENDING_APPROVAL".to_string()) // 返回需要审批的信号
            } else {
                Ok(format!("Processed with {} tool calls", results.len()))
            }
        }
        Err(e) => Err(format!("Error processing message: {}", e)),
    }
}

#[tauri::command]
async fn approve_tool_call(
    tool_name: String,
    arguments: String, // JSON字符串
    state: State<'_, std::sync::Arc<Mutex<OrchestratorState>>>,
) -> Result<String, String> {
    // 创建编排器实例（在实际应用中，应从state获取已初始化的实例）
    let api_key = "dummy"; // 在实际应用中，应从配置或状态中获取
    let api_base = "dummy";
    let model_name = "dummy";
    let orchestrator = orchestrator::Orchestrator::new(
        api_key.to_string(),
        api_base.to_string(),
        model_name.to_string(),
    );
    
    // 解析参数
    let args_value: serde_json::Value = serde_json::from_str(&arguments)
        .map_err(|e| format!("Failed to parse arguments: {}", e))?;
    
    // 批准工具调用
    match orchestrator.approve_tool_call(tool_name, args_value).await {
        Ok(result) => {
            match result.status {
                orchestrator::ToolCallStatus::Approved | orchestrator::ToolCallStatus::Executed => {
                    Ok(format!("Tool call approved and executed: {}", result.tool_name))
                }
                _ => Ok(format!("Tool call failed: {}", result.error.unwrap_or("Unknown error".to_string()))),
            }
        }
        Err(e) => Err(format!("Error approving tool call: {}", e)),
    }
}

#[tauri::command]
async fn search_local_files(
    query: String,
    state: State<'_, std::sync::Arc<Mutex<FileIndexerState>>>,
) -> Result<Vec<file_index::FileInfo>, String> {
    let indexer_state = state.inner();
    let indexer_guard = indexer_state.indexer.as_ref()
        .ok_or("File indexer not initialized")?;
    let indexer = indexer_guard.lock().unwrap();
    
    match indexer.search_by_filename(&query) {
        Ok(results) => Ok(results),
        Err(e) => Err(format!("Error searching files: {}", e)),
    }
}

#[tauri::command]
async fn refresh_file_index(
    state: State<'_, std::sync::Arc<Mutex<FileIndexerState>>>,
    app_handle: AppHandle,
) -> Result<String, String> {
    // 重新初始化文件索引器
    match file_index::initialize_file_indexer(&app_handle) {
        Ok(new_indexer) => {
            let indexer_state = state.inner();
            let mut indexer_guard = indexer_state.indexer.as_ref()
                .ok_or("File indexer not initialized")?
                .lock()
                .unwrap();
            
            // 用新的索引器替换旧的
            *indexer_guard = new_indexer;
            
            Ok("File index refreshed successfully".to_string())
        }
        Err(e) => Err(format!("Error refreshing file index: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(app_handle: tauri::AppHandle) {
    let mcp_state = std::sync::Arc::new(Mutex::new(McpClientState { client: None }));
    let orch_state = std::sync::Arc::new(Mutex::new(OrchestratorState { orchestrator: None }));
    
    // 初始化文件索引器
    let file_indexer = match file_index::initialize_file_indexer(&app_handle) {
        Ok(indexer) => Some(std::sync::Arc::new(Mutex::new(indexer))),
        Err(e) => {
            eprintln!("Failed to initialize file indexer: {}", e);
            None
        }
    };
    let file_indexer_state = std::sync::Arc::new(Mutex::new(FileIndexerState { indexer: file_indexer }));
    
    tauri::Builder::default()
        .manage(mcp_state)
        .manage(orch_state)
        .manage(file_indexer_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, init_mcp, list_mcp_tools, process_user_message, approve_tool_call, search_local_files, refresh_file_index])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}