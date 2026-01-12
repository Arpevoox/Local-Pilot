// Mock Tauri API for browser environment
export const invoke = async (command: string, args?: Record<string, unknown>): Promise<any> => {
  console.log(`Mock Tauri invoke: ${command}`, args);
  
  // Simulate different commands
  switch (command) {
    case 'greet':
      return `Hello, ${args?.name}! You've been greeted from Mock Rust!`;
    
    case 'init_mcp':
      // Simulate MCP initialization
      return 'MCP initialized successfully';
    
    case 'list_mcp_tools':
      // Return mock MCP tools
      return [
        {
          name: "file_reader",
          description: "读取本地文件内容",
          input_schema: {
            "type": "object",
            "properties": {
              "path": {
                "type": "string",
                "description": "文件路径"
              }
            },
            "required": ["path"]
          },
        },
        {
          name: "shell_executor",
          description: "在本地执行shell命令",
          input_schema: {
            "type": "object",
            "properties": {
              "command": {
                "type": "string",
                "description": "要执行的命令"
              }
            },
            "required": ["command"]
          },
        }
      ];
    
    case 'process_user_message':
      // Simulate processing with occasional approval requirement
      const random = Math.random();
      if (random > 0.7) { // 30% chance of needing approval
        return 'PENDING_APPROVAL';
      } else {
        return `Processed message: "${args?.message}"`;
      }
    
    case 'approve_tool_call':
      return `Tool call ${args?.toolName} approved and executed`;
    
    case 'search_local_files':
      // Return mock search results
      return [
        { id: '1', name: 'example.txt', path: '/home/user/example.txt', size: 1024 },
        { id: '2', name: 'document.pdf', path: '/home/user/documents/document.pdf', size: 2048 }
      ];
    
    case 'refresh_file_index':
      return 'File index refreshed successfully';
    
    default:
      // For unknown commands, return a generic response
      return `Mock response for command: ${command}`;
  }
};

// Mock other Tauri APIs if needed
export const listen = async (event: string, handler: (event: any) => void) => {
  console.log(`Listening for event: ${event}`);
  return () => {}; // Unlisten function
};

export const emit = async (event: string, payload?: any) => {
  console.log(`Emitted event: ${event}`, payload);
};