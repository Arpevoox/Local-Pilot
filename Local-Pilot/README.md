# Local-Pilot

Local-Pilot is a desktop application that provides a local AI-powered assistant for interacting with your system. Built with Tauri, it combines a React frontend with a Rust backend to offer secure local operations using the Model Context Protocol (MCP).

## Project Structure

```
Local-Pilot/
├── src/                    # Frontend UI (React + TypeScript)
├── src-tauri/             # Backend (Rust + Tauri)
├── mcp-servers/           # Pre-configured MCP server scripts
├── docs/                  # Architecture diagrams and API docs
├── dist/                  # Build output (ignored by git)
├── node_modules/          # Node.js dependencies (ignored by git)
├── .env.example          # Environment variable template
├── .gitignore            # Git ignore rules
├── package.json          # Node.js dependencies and scripts
├── vite.config.ts        # Vite configuration
└── README.md             # This file
```

## Features

- Secure local system interaction through MCP
- AI-powered assistance with human-in-the-loop for dangerous operations
- Cross-platform desktop application
- Environment variable-based configuration

## Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and fill in your API keys
3. Install dependencies: `npm install`
4. Run the application: `npm run tauri dev`

## Configuration

Create a `.env` file based on `.env.example` with your API keys:

```bash
ANTHROPIC_API_KEY=your_anthropic_api_key_here
DEEPSEEK_API_KEY=your_deepseek_api_key_here
API_BASE=https://api.anthropic.com
MODEL_NAME=claude-3-5-sonnet-20241022
```

## Security

This application is designed with security in mind:
- Dangerous operations require explicit user approval
- API keys are loaded from environment variables
- System access is limited to configured MCP servers



<div align="center">
  <img src="./assets/logo.png" alt="Local-Pilot Logo" width="120" height="120" />
  <h1>Local-Pilot</h1>
  <p><b>Give your LLM a keyboard and a seat in your office.</b></p>

  <!-- GitHub Badges -->
  <p>
    <img src="https://img.shields.io/github/stars/your-username/local-pilot?style=for-the-badge&color=gold" alt="stars" />
    <img src="https://img.shields.io/github/license/your-username/local-pilot?style=for-the-badge&color=blue" alt="license" />
    <img src="https://img.shields.io/badge/MCP-Supported-green?style=for-the-badge" alt="MCP" />
    <img src="https://img.shields.io/badge/Built_with-Rust-orange?style=for-the-badge&logo=rust" alt="rust" />
  </p>
</div>

---

## 💡 为什么选择 Local-Pilot?

1. **打破囚笼**：现有的 LLM 被困在浏览器和对话框里，无法感知也无法操作你本地的 IDE、Excel 或文件系统。
2. **拒绝繁琐**：手动整理文件、跨软件搬运数据是数字化办公中最后的“体力活”，耗时且乏味。
3. **安全桥梁**：Local-Pilot 利用 **MCP (Model Context Protocol)** 协议，在保护隐私的前提下，赋予 AI 操控本地系统的“双手”。

---

## ✨ 核心特性

- 🛡️ **Privacy First (隐私至上)**：所有系统指令均在本地通过 MCP 执行。涉及删除、修改等敏感操作时，强制触发 **Human-in-the-loop** 审计弹窗。
- ⚡ **Native MCP Support**：原生支持 Anthropic 推出的 MCP 协议。你可以随意挂载社区成千上万的 MCP Server（如 SQLite, Google Drive, Docker）。
- 🦀 **High Performance**：基于 **Rust & Tauri 2.0** 构建。极小的内存占用（<50MB），秒级启动，跨平台支持 (macOS, Windows, Linux)。
- 🧠 **Smart Context**：内置本地索引引擎，让 AI 真正“记住”你本地文件夹的结构与内容。

---

## 🎬 演示 (Demo)

> “帮我把 Downloads 文件夹里所有的 PDF 发票按日期重命名，并汇总到一个 Excel 里。”

![Local-Pilot Demo GIF](./assets/demo.gif)
*AI 识别意图 -> 扫描文件 -> 弹出确认框 -> 物理执行 -> 自动生成表格*

---

## 🚀 快速开始

仅需三步，即可让 AI 接管你的本地繁琐任务：

```bash
# 1. 克隆项目
git clone https://github.com/your-username/local-pilot.git

# 2. 安装依赖 (确保已安装 Node.js 和 Rust)
npm install

# 3. 启动开发环境
npm run tauri dev