# Local-Pilot Architecture

## Overview
Local-Pilot is a desktop application built with Tauri that provides a local AI-powered assistant for interacting with the local system. The application uses the Model Context Protocol (MCP) to connect to various local services.

## Components

### Frontend (React + TypeScript)
- Located in `/src`
- Provides the user interface for interacting with the AI assistant
- Communicates with the backend via Tauri commands

### Backend (Rust + Tauri)
- Located in `/src-tauri`
- Handles all system interactions and API calls
- Implements Tauri commands for frontend communication
- Manages MCP connections

### MCP Servers
- Located in `/mcp-servers`
- Pre-configured servers for common local services
- Examples: filesystem, SQLite, Docker, etc.

## Data Flow

1. User enters a request in the frontend
2. Frontend invokes a Tauri command to process the request
3. Backend communicates with MCP servers and external APIs
4. Results are returned to the frontend for display

## Security Model

- API keys stored in environment variables
- Dangerous operations require explicit user approval
- Sandboxed execution of potentially harmful commands