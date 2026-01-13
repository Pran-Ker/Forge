# Forge

A minimal, hackable AI agent built in Rust. Forge uses Claude Sonnet 4.5 to reason about your requests, create execution plans, and take action using built-in tools.

## Agent Workflow

Forge implements a **4-phase agentic workflow**:

1. **Reasoning** - Claude analyzes your request and determines which tools to use
2. **Response** - Claude explains what it will do in friendly language
3. **Tool Call Planning** - Claude generates structured tool calls (JSON format)
4. **Execution** - Forge executes each tool call and displays results

## Features

### Built-in Tools

- **Read** - Read files with line numbers
- **Write** - Create new files with automatic directory creation
- **Edit** - Search and replace with beautiful diff display
- **Bash** - Execute shell commands with real-time output streaming
- **Glob** - Find files by pattern with modification time sorting
- **Grep** - Search file contents using ripgrep's powerful regex engine
- **WebSearch** - Search the web using DuckDuckGo
- **WebFetch** - Fetch and convert web pages to markdown
- **AskUserQuestion** - Interactive user prompts with multiple choice support

## Setup

### 1. Get an Anthropic API Key

Get your API key from [Anthropic Console](https://console.anthropic.com/)

### 2. Set Environment Variable

Create a `.env` file or add to your shell config:

```bash
export ANTHROPIC_API_KEY="your_api_key_here"
```

Or create a `.env` file:
```bash
cp .env.example .env
# Edit .env and add your API key
```

### 3. Build and Run

```bash
cargo build --release
cargo run
```

## Usage

Simply chat with Forge in natural language:

```
you> read the Cargo.toml file

→ Reasoning
I need to use the read tool to read the Cargo.toml file and display its contents.

→ Response
I'll read the Cargo.toml file for you.

→ Planning Tool Calls
1. read Cargo.toml

→ Executing
[1/1] Read Cargo.toml
     1  [package]
     2  name = "Forge"
     3  version = "0.1.0"
     4  edition = "2021"
     5
     6  [dependencies]
     7  owo-colors = "4.1"
     8  console = "0.15"
     ...
✓ Done

All tasks completed!
```

```
you> find all rust files in src directory

→ Reasoning
I'll use the glob tool with the pattern "src/**/*.rs" to find all Rust files.

→ Response
I'll search for all Rust files in the src directory.

→ Planning Tool Calls
1. glob src/**/*.rs

→ Executing
[1/1] Find Rust files
Found 13 files:
src/main.rs
src/lib.rs
src/agent.rs
src/output.rs
src/types.rs
src/tools/mod.rs
src/tools/read.rs
...
✓ Done

All tasks completed!
```

## Architecture

```
Forge/
├── src/
│   ├── main.rs          # Agent loop (reasoning → response → todos → execute)
│   ├── lib.rs           # Library exports
│   ├── agent.rs         # Agent implementation with Anthropic API calls
│   ├── output.rs        # Terminal formatting
│   ├── types.rs         # Common types
│   └── tools/
│       ├── mod.rs       # Tool exports
│       ├── read.rs      # Read implementation
│       ├── write.rs     # Write implementation
│       ├── edit.rs      # Edit with diffing
│       ├── bash.rs      # Command execution
│       ├── glob.rs      # Pattern matching
│       ├── grep.rs      # Content search
│       ├── websearch.rs # Web search
│       ├── webfetch.rs  # Web fetching
│       └── ask.rs       # User prompts
└── Cargo.toml
```

## How It Works

1. **User Input** - You provide a natural language request
2. **Reasoning Phase** - Claude analyzes the request and determines which tools to use
3. **Response Phase** - Claude generates a friendly explanation
4. **Tool Call Planning** - Claude generates structured tool calls in JSON format
5. **Execution** - Forge executes each tool call using the actual tool implementations
6. **Results** - Tool output is displayed with beautiful colored formatting

## Design Principles

- **Minimal code** - Point-to-point implementation, no unnecessary abstractions
- **Hackable** - One tool per file, easy to locate and modify
- **Simple error handling** - Uses `?` operator and `anyhow`, no complex error handling
- **Async where needed** - Only agent and web tools use async
- **Beautiful output** - Colored terminal output using owo-colors
- **AI-powered** - Leverages Claude Sonnet 4.5 for intelligent task decomposition

## Dependencies

- `reqwest` - HTTP client for Anthropic API and web operations
- `owo-colors` + `console` - Terminal formatting
- `tokio` - Async runtime
- `similar` - Diff generation
- `globset` + `walkdir` - File pattern matching
- `grep-regex` + `grep-searcher` - Ripgrep libraries for searching
- `html2md` - HTML to markdown conversion
- `inquire` - User interaction
- `anyhow` + `thiserror` - Error handling
- `dotenv` - Environment variable loading
- `serde` + `serde_json` - JSON serialization

## Extending Forge

### Adding New Tools

1. Create a new file in `src/tools/` (e.g., `my_tool.rs`)
2. Implement your tool function
3. Export it in `src/tools/mod.rs`
4. The agent will automatically use it when needed!

Example:

```rust
// src/tools/my_tool.rs
use anyhow::Result;

pub fn my_tool(input: &str) -> Result<String> {
    // Your implementation
    Ok(format!("Processed: {}", input))
}
```

### Customizing the Agent

Edit `src/agent.rs` to:
- Change the model (default: `claude-sonnet-4-5-20250929`)
- Modify prompts for reasoning, response, or todo generation
- Adjust max_tokens or other API parameters

## License

MIT
