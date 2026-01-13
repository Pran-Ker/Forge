use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{Output, tools};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: Vec<String>,
    pub description: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

pub struct Agent {
    client: Client,
    api_key: String,
    output: Output,
}

impl Agent {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            output: Output::new(),
        }
    }

    async fn call_api(&self, prompt: &str) -> Result<String> {
        let request = AnthropicRequest {
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 2000,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_body: AnthropicResponse = response.json().await?;

        Ok(response_body.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_else(|| "No response".to_string()))
    }

    pub async fn reason(&self, user_input: &str) -> Result<String> {
        self.output.tool_header("Reasoning");

        let prompt = format!(
            "You are an AI agent with these tools:\n\
            - read <path>: Read a file\n\
            - write <path> <content>: Write a file\n\
            - edit <path> <search> <replace>: Edit a file\n\
            - bash <command>: Run a command\n\
            - glob <pattern>: Find files by pattern\n\
            - grep <pattern> <path>: Search file contents\n\n\
            User request: \"{}\"\n\n\
            Think about what tools you need and why. Keep it brief (2-3 sentences).",
            user_input
        );

        let reasoning = self.call_api(&prompt).await?;
        println!("{}", reasoning);
        Ok(reasoning)
    }

    pub async fn respond(&self, user_input: &str, reasoning: &str) -> Result<String> {
        self.output.tool_header("Response");

        let prompt = format!(
            "User: \"{}\"\nReasoning: {}\n\n\
            Provide a brief, friendly response (1 sentence) about what you'll do.",
            user_input, reasoning
        );

        let response_text = self.call_api(&prompt).await?;
        println!("{}", response_text);
        Ok(response_text)
    }

    pub async fn create_tool_calls(&self, user_input: &str, reasoning: &str) -> Result<Vec<ToolCall>> {
        self.output.tool_header("Planning Tool Calls");

        let prompt = format!(
            "User: \"{}\"\nReasoning: {}\n\n\
            Available tools:\n\
            - read <path>: Read a file\n\
            - write <path> <content>: Write a file  \n\
            - edit <path> <search> <replace>: Edit a file\n\
            - bash <command>: Run shell command\n\
            - glob <pattern>: Find files\n\
            - grep <pattern> <path>: Search contents\n\n\
            Generate tool calls as JSON array. Each call needs:\n\
            - tool: tool name\n\
            - args: array of string arguments\n\
            - description: what this call does\n\n\
            Example:\n\
            [{{\"tool\": \"read\", \"args\": [\"Cargo.toml\"], \"description\": \"Read Cargo.toml\"}}]\n\n\
            Return ONLY the JSON array, nothing else.",
            user_input, reasoning
        );

        let json_text = self.call_api(&prompt).await?;

        let tool_calls: Vec<ToolCall> = serde_json::from_str(&json_text)
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse tool calls: {}. Response was:\n{}", e, json_text);
                vec![]
            });

        for (i, call) in tool_calls.iter().enumerate() {
            self.output.list_item(i + 1, &format!("{} {}", call.tool, call.args.join(" ")));
        }

        Ok(tool_calls)
    }

    pub async fn execute_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Result<()> {
        self.output.tool_header("Executing");

        for (i, call) in tool_calls.iter().enumerate() {
            self.output.info(&format!("\n[{}/{}] {}", i + 1, tool_calls.len(), call.description));

            match self.execute_single_tool(&call).await {
                Ok(result) => {
                    if !result.is_empty() {
                        println!("{}", result);
                    }
                    self.output.success("✓ Done");
                }
                Err(e) => {
                    self.output.error(&format!("✗ Error: {}", e));
                }
            }
        }

        println!();
        self.output.success("All tasks completed!");
        Ok(())
    }

    async fn execute_single_tool(&self, call: &ToolCall) -> Result<String> {
        match call.tool.as_str() {
            "read" => {
                let path = call.args.get(0).map(|s| s.as_str()).unwrap_or("");
                let content = tools::read(path, None, None)?;
                Ok(content)
            }
            "write" => {
                let path = call.args.get(0).map(|s| s.as_str()).unwrap_or("");
                let content = call.args[1..].join(" ");
                tools::write(path, &content)?;
                Ok(format!("Wrote to {}", path))
            }
            "edit" => {
                let path = call.args.get(0).map(|s| s.as_str()).unwrap_or("");
                let search = call.args.get(1).map(|s| s.as_str()).unwrap_or("");
                let replace = call.args.get(2).map(|s| s.as_str()).unwrap_or("");
                let diff = tools::edit(path, search, replace, false)?;
                Ok(format!("Edited {}\n{}", path, diff))
            }
            "bash" => {
                let command = call.args.join(" ");
                let result = tools::bash(&command).await?;
                Ok(result.output)
            }
            "glob" => {
                let pattern = call.args.get(0).map(|s| s.as_str()).unwrap_or("");
                let matches = tools::glob(pattern, None)?;
                Ok(format!("Found {} files:\n{}", matches.len(), matches.join("\n")))
            }
            "grep" => {
                let pattern = call.args.get(0).map(|s| s.as_str()).unwrap_or("");
                let path = call.args.get(1).map(|s| s.as_str()).unwrap_or(".");
                let matches = tools::grep(pattern, path, false)?;
                let output = matches.iter()
                    .take(10)
                    .map(|m| format!("{}:{} {}", m.file, m.line_num, m.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(format!("Found {} matches:\n{}", matches.len(), output))
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", call.tool)),
        }
    }
}
