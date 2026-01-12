# Local-Pilot API Documentation

## Tauri Commands

### `init_mcp`
Initializes the Model Context Protocol (MCP) client.

**Parameters:** None

**Returns:** `Promise<string>` - Success or error message

### `list_mcp_tools`
Lists available MCP tools.

**Parameters:** None

**Returns:** `Promise<Array<Tool>>` where Tool has:
- `name`: string - Tool name
- `description`: string - Tool description
- `input_schema`: object - JSON Schema for tool inputs

### `process_user_message`
Processes a user message with the AI assistant.

**Parameters:**
- `message`: string - The user's message
- `apiKey`: string - API key for the LLM service
- `apiBase`: string - Base URL for the LLM API
- `modelName`: string - Name of the LLM model to use

**Returns:** `Promise<string>` - Response from the AI, or "PENDING_APPROVAL" if human approval is needed

### `approve_tool_call`
Approves a potentially dangerous tool call.

**Parameters:**
- `tool_name`: string - Name of the tool to execute
- `arguments`: string - JSON string of arguments for the tool

**Returns:** `Promise<string>` - Result of the tool execution

### `search_local_files`
Searches for files in the local file system.

**Parameters:**
- `query`: string - Search query

**Returns:** `Promise<Array<FileInfo>>` where FileInfo has:
- `id`: string - Unique identifier
- `name`: string - File name
- `path`: string - Full file path
- `size`: number - File size in bytes

### `refresh_file_index`
Refreshes the local file index.

**Parameters:** None

**Returns:** `Promise<string>` - Success or error message

## Environment Variables

The application uses the following environment variables:

- `ANTHROPIC_API_KEY`: API key for Anthropic's Claude models
- `DEEPSEEK_API_KEY`: API key for DeepSeek models (optional)
- `API_BASE`: Base URL for the LLM API (defaults to Anthropic)
- `MODEL_NAME`: Default model name to use (defaults to claude-3-5-sonnet-20241022)