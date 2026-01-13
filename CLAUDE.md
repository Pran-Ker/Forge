# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Forge is a minimal, hackable AI agent built in Rust that uses Claude Sonnet 4.5 for reasoning and task execution through a 4-phase workflow.

## Development Commands

```bash
# Build
cargo build --release

# Run the agent
cargo run

# Build and run
cargo build --release && cargo run

# Check for compile errors
cargo check
```

## Environment Setup

Requires `ANTHROPIC_API_KEY` environment variable:
- Create `.env` file from `.env.example`
- Or export in shell: `export ANTHROPIC_API_KEY="your_key"`

## Architecture

### 4-Phase Agent Workflow

The agent operates in a strict sequential phase model (src/main.rs:43-71):

1. **Reasoning** - Analyze request and determine which tools to use
2. **Response** - Generate friendly explanation of intended actions
3. **Tool Call Planning** - Generate structured JSON tool calls
4. **Execution** - Execute each tool call sequentially

Each phase calls the Anthropic API separately. The phases are chained - reasoning feeds into response, both feed into tool planning, and tool calls are executed last.

### Agent Implementation (src/agent.rs)

- `Agent` struct maintains HTTP client, API key, output formatter, and conversation history
- `call_api()` - Core API interaction with Claude Sonnet 4.5 (model: claude-sonnet-4-5-20250929)
- `reason()` - Phase 1: Generates reasoning about tool selection
- `respond()` - Phase 2: Creates user-facing response
- `create_tool_calls()` - Phase 3: Generates JSON array of tool calls with tool name, args array, and description
- `execute_tool_calls()` / `execute_single_tool()` - Phase 4: Executes tools via match statement

### Tool System (src/tools/)

Tools are simple functions, one per file. Each tool is directly invoked by the agent via pattern matching in `execute_single_tool()`. Tools available:

- **read** - Read files with line numbers
- **write** - Create files with automatic directory creation
- **edit** - Search/replace with diff display
- **bash** - Execute shell commands (async)
- **glob** - Pattern-based file finding
- **grep** - Content search using ripgrep
- **websearch** - DuckDuckGo search (async)
- **webfetch** - Fetch and convert web pages to markdown (async)
- **ask** - Interactive user prompts with multiple choice

To add a new tool:
1. Create file in `src/tools/`
2. Implement function
3. Export in `src/tools/mod.rs`
4. Add case to `execute_single_tool()` match in `src/agent.rs`

### Output System (src/output.rs)

`Output` struct provides colored terminal output:
- `success()` - Green bold
- `error()` - Red bold to stderr
- `info()` - Blue
- `tool_header()` - Cyan bold with arrow prefix
- `diff()` - Colored diff output (+green, -red, @cyan)
- `list_item()` - Numbered list with cyan index

## Design Principles

**Core Goal: Minimize lines of code while delivering required functionality.** Keep implementations simple and clear. As features get added, actively resist complexity growth. Prefer straightforward solutions over clever abstractions.

- **Point-to-point implementation** - No unnecessary abstractions
- **One tool per file** - Easy to locate and modify
- **Simple error handling** - Uses `?` operator and `anyhow::Result`
- **Async only where needed** - Agent and web tools use async, file tools are sync
- **No message history** - Each API call is stateless (current implementation)
