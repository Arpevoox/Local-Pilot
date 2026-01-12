import { useState, useEffect } from "react";
import "./App.css";

// Dynamically import Tauri API or use mock
let invoke: any;
try {
  // Try to import the real Tauri API
  const tauriCore = await import("@tauri-apps/api/core");
  invoke = tauriCore.invoke;
} catch (e) {
  // If that fails, use the mock API
  const mockApi = await import("./mock-tauri");
  invoke = mockApi.invoke;
}


interface Message {
  id: string;
  content: string;
  role: "user" | "assistant";
  timestamp: Date;
}

interface OperationPreview {
  id: string;
  title: string;
  description: string;
  status: "pending" | "executing" | "completed" | "failed";
}

interface PendingApproval {
  id: string;
  toolName: string;
  arguments: any;
  description: string;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([
    { id: "1", content: "你好！我是Local Pilot助手，我可以帮助你与本地系统交互。", role: "assistant", timestamp: new Date() },
  ]);
  const [inputValue, setInputValue] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [operationPreviews, setOperationPreviews] = useState<OperationPreview[]>([
    { id: "op1", title: "文件读取", description: "读取本地文件内容", status: "pending" },
    { id: "op2", title: "命令执行", description: "执行本地shell命令", status: "pending" },
  ]);
  const [pendingApproval, setPendingApproval] = useState<PendingApproval | null>(null);
  const [mcpServers, setMcpServers] = useState<string[]>(["Filesystem", "SQLite"]); // 示例MCP服务器列表

  const updateOperationStatus = (id: string, status: OperationPreview['status']) => {
    setOperationPreviews(prev => 
      prev.map(op => op.id === id ? { ...op, status } : op)
    );
  };

  // 处理批准操作
  const handleApproveOperation = async () => {
    if (!pendingApproval) return;
    
    try {
      // 调用后端批准工具调用
      const result = await invoke("approve_tool_call", {
        toolName: pendingApproval.toolName,
        arguments: JSON.stringify(pendingApproval.arguments)
      });
      
      // 添加操作结果到消息列表
      const resultMessage: Message = {
        id: Date.now().toString(),
        content: `操作结果: ${result}`,
        role: "assistant",
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, resultMessage]);
      
      // 清除待批准操作
      setPendingApproval(null);
      
      // 更新操作预览状态
      if (operationPreviews.length > 0) {
        updateOperationStatus(operationPreviews[0].id, "completed");
      }
    } catch (error) {
      console.error("Error approving operation:", error);
      
      // 显示错误消息
      const errorMessage: Message = {
        id: Date.now().toString(),
        content: `批准操作失败: ${(error as Error).message}`,
        role: "assistant",
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, errorMessage]);
      
      if (operationPreviews.length > 0) {
        updateOperationStatus(operationPreviews[0].id, "failed");
      }
    }
  };

  // 拒绝操作
  const handleRejectOperation = () => {
    // 添加拒绝消息到消息列表
    const rejectMessage: Message = {
      id: Date.now().toString(),
      content: `操作已拒绝: ${pendingApproval?.description || '未知操作'}`,
      role: "assistant",
      timestamp: new Date(),
    };
    setMessages(prev => [...prev, rejectMessage]);
    
    // 清除待批准操作
    setPendingApproval(null);
    
    // 更新操作预览状态
    if (operationPreviews.length > 0) {
      updateOperationStatus(operationPreviews[0].id, "failed");
    }
  };

  // 初始化MCP
  useEffect(() => {
    const initMcp = async () => {
      try {
        await invoke("init_mcp");
        console.log("MCP initialized");
      } catch (error) {
        console.error("Failed to initialize MCP:", error);
      }
    };
    
    initMcp();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputValue.trim() || isLoading) return;

    // 添加用户消息
    const userMessage: Message = {
      id: Date.now().toString(),
      content: inputValue,
      role: "user",
      timestamp: new Date(),
    };
    
    setMessages(prev => [...prev, userMessage]);
    setInputValue("");
    setIsLoading(true);

    try {
      // 更新第一个操作的状态为执行中
      if (operationPreviews.length > 0) {
        updateOperationStatus(operationPreviews[0].id, "executing");
      }
      
      // 调用后端处理用户消息
      const result = await invoke("process_user_message", {
        message: inputValue,
        apiKey: process.env.ANTHROPIC_API_KEY || "",
        apiBase: process.env.API_BASE || "https://api.anthropic.com",
        modelName: process.env.MODEL_NAME || "claude-3-5-sonnet-20241022"
      });
      
      if (result === "PENDING_APPROVAL") {
        // 显示确认卡片 - 在实际应用中，后端可能返回更多详细信息
        setPendingApproval({
          id: Date.now().toString(),
          toolName: "dangerous_operation", // 实际应用中应从后端获取
          arguments: { path: "/example/path" }, // 实际应用中应从后端获取
          description: "执行潜在危险操作: 删除 /example/path"
        });
      } else {
        // 显示AI响应
        const aiMessage: Message = {
          id: (Date.now() + 1).toString(),
          content: `AI回复: ${typeof result === 'string' ? result : '操作已完成'}`,
          role: "assistant",
          timestamp: new Date(),
        };
        setMessages(prev => [...prev, aiMessage]);
        
        // 更新第一个操作的状态为已完成
        if (operationPreviews.length > 0) {
          updateOperationStatus(operationPreviews[0].id, "completed");
        }
      }
    } catch (error) {
      console.error("Error processing message:", error);
      
      // 显示错误消息
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        content: `错误: ${(error as Error).message}`,
        role: "assistant",
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, errorMessage]);
      
      if (operationPreviews.length > 0) {
        updateOperationStatus(operationPreviews[0].id, "failed");
      }
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-50">
      {/* 顶部导航栏 */}
      <header className="bg-white shadow-sm py-4 px-6">
        <div className="flex items-center">
          <h1 className="text-xl font-bold text-gray-800">Local Pilot</h1>
          <span className="ml-auto text-sm text-gray-500">MCP就绪</span>
        </div>
      </header>

      {/* 主内容区域 - 分为左右两栏 */}
      <div className="flex flex-1 overflow-hidden">
        {/* 左侧对话区 */}
        <div className="flex-1 flex flex-col border-r border-gray-200">
          <div className="flex-1 overflow-y-auto p-6 bg-white">
            <div className="max-w-3xl mx-auto space-y-6">
              {messages.map((message) => (
                <div 
                  key={message.id} 
                  className={`flex ${message.role === "user" ? "justify-end" : "justify-start"}`}
                >
                  <div 
                    className={`max-w-[80%] rounded-lg px-4 py-2 ${
                      message.role === "user" 
                        ? "bg-blue-500 text-white" 
                        : "bg-gray-200 text-gray-800"
                    }`}
                  >
                    <p>{message.content}</p>
                    <span className={`text-xs mt-1 block ${message.role === "user" ? "text-blue-100" : "text-gray-500"}`}>
                      {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                  </div>
                </div>
              ))}
              {isLoading && (
                <div className="flex justify-start">
                  <div className="max-w-[80%] rounded-lg px-4 py-2 bg-gray-200 text-gray-800">
                    <p>正在思考...</p>
                  </div>
                </div>
              )}
            </div>
          </div>
          
          {/* 输入区域 */}
          <div className="border-t border-gray-200 p-4 bg-white">
            <form onSubmit={handleSubmit} className="flex gap-3">
              <input
                type="text"
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                placeholder="输入您的请求..."
                className="flex-1 border border-gray-300 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                disabled={isLoading}
              />
              <button
                type="submit"
                className="bg-blue-500 hover:bg-blue-600 text-white px-6 py-2 rounded-lg transition disabled:opacity-50"
                disabled={isLoading || !inputValue.trim()}
              >
                发送
              </button>
            </form>
          </div>
        </div>

        {/* 右侧操作预览侧边栏 */}
        <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
          <div className="p-4 border-b border-gray-200">
            <h2 className="font-semibold text-lg text-gray-800">操作预览</h2>
            <p className="text-sm text-gray-500 mt-1">即将执行的操作</p>
          </div>
          
          <div className="flex-1 overflow-y-auto p-4">
            <div className="space-y-4">
              {operationPreviews.map((op) => (
                <div 
                  key={op.id} 
                  className={`p-4 rounded-lg border ${
                    op.status === "completed" ? "border-green-500 bg-green-50" :
                    op.status === "failed" ? "border-red-500 bg-red-50" :
                    op.status === "executing" ? "border-yellow-500 bg-yellow-50" :
                    "border-gray-300 bg-gray-50"
                  }`}
                >
                  <div className="flex items-center">
                    <div className={`w-3 h-3 rounded-full mr-2 ${
                      op.status === "completed" ? "bg-green-500" :
                      op.status === "failed" ? "bg-red-500" :
                      op.status === "executing" ? "bg-yellow-500" :
                      "bg-gray-400"
                    }`} />
                    <h3 className="font-medium text-gray-800">{op.title}</h3>
                  </div>
                  <p className="text-sm text-gray-600 mt-2">{op.description}</p>
                  <div className="mt-3 text-xs text-gray-500">
                    状态: <span className={`capitalize ${
                      op.status === "completed" ? "text-green-600" :
                      op.status === "failed" ? "text-red-600" :
                      op.status === "executing" ? "text-yellow-600" :
                      "text-gray-600"
                    }`}>
                      {op.status === "pending" && "待处理"}
                      {op.status === "executing" && "执行中"}
                      {op.status === "completed" && "已完成"}
                      {op.status === "failed" && "失败"}
                    </span>
                  </div>
                </div>
              ))}
            </div>
            
            {/* MCP服务器状态指示器 */}
            <div className="mt-6 pt-4 border-t border-gray-200">
              <h3 className="font-medium text-gray-800 mb-2">MCP服务器连接</h3>
              <div className="space-y-2">
                {mcpServers.map((server, index) => (
                  <div key={index} className="flex items-center">
                    <div className="w-2 h-2 rounded-full bg-green-500 mr-2"></div>
                    <span className="text-sm text-gray-700">{server}</span>
                  </div>
                ))}
              </div>
              <div className="mt-2 text-xs text-gray-500">
                支持工具: 文件读取, 命令执行, 网络搜索
              </div>
            </div>
          </div>
        </div>
      </div>
      
      {/* 操作确认卡片 */}
      {pendingApproval && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
            <h3 className="text-lg font-semibold text-gray-800 mb-2">操作确认</h3>
            <p className="text-gray-600 mb-4">检测到潜在危险操作，需要您的确认：</p>
            
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
              <div className="flex items-start">
                <div className="flex-shrink-0 mt-1">
                  <svg className="h-5 w-5 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3">
                  <h4 className="text-sm font-medium text-yellow-800">{pendingApproval.toolName}</h4>
                  <p className="text-sm text-yellow-700 mt-1">{pendingApproval.description}</p>
                  <div className="mt-2 text-xs text-yellow-600">
                    参数: {JSON.stringify(pendingApproval.arguments)}
                  </div>
                </div>
              </div>
            </div>
            
            <div className="flex justify-end space-x-3">
              <button
                onClick={handleRejectOperation}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
              >
                拒绝
              </button>
              <button
                onClick={handleApproveOperation}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              >
                确认执行
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;