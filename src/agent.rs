use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{Output, tools};
use std::pin::Pin;
use std::future::Future;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Serialize, Clone)]
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
    messages: Vec<Message>,
}

impl Agent {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            output: Output::new(),
            messages: Vec::new(),
        }
    }

    fn handle_error<'a>(&'a mut self, error: &'a str, failed_tool: &'a ToolCall, depth: u32) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            self.output.tool_header("Error Recovery");

            // Add error context to conversation
            self.messages.push(Message {
                role: "user".to_string(),
                content: format!(
                    "The tool call '{}' with args {:?} failed with error: {}\n\n\
                    Please reason about why this failed and suggest a different approach.",
                    failed_tool.tool, failed_tool.args, error
                ),
            });

            // Get new reasoning with error context
            let reasoning = self.reason_with_context().await?;
            println!();

            // Create new tool calls based on error analysis
            let new_tool_calls = self.create_tool_calls_with_context(&reasoning).await?;

            if !new_tool_calls.is_empty() {
                println!();
                self.execute_tool_calls_with_retry(new_tool_calls, depth + 1).await?;
            }

            Ok(())
        })
    }

    async fn reason_with_context(&mut self) -> Result<String> {
        self.output.tool_header("Reasoning");

        let prompt = "Based on the error above, think about what went wrong and what to try next. Keep it brief (2-3 sentences).";

        let reasoning = self.call_api(prompt).await?;
        println!("{}", reasoning);
        Ok(reasoning)
    }

    async fn create_tool_calls_with_context(&mut self, reasoning: &str) -> Result<Vec<ToolCall>> {
        self.output.tool_header("Planning Tool Calls");

        let prompt = format!(
            "Reasoning: {}\n\n\
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
            Return ONLY the JSON array, nothing else. If no tools needed, return []",
            reasoning
        );

        let json_text = self.call_api(&prompt).await?;

        let cleaned_json = json_text
            .trim()
            .strip_prefix("```json")
            .or_else(|| json_text.trim().strip_prefix("```"))
            .unwrap_or(json_text.trim())
            .strip_suffix("```")
            .unwrap_or(json_text.trim())
            .trim();

        let tool_calls: Vec<ToolCall> = serde_json::from_str(cleaned_json)
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse tool calls: {}. Response was:\n{}", e, json_text);
                vec![]
            });

        for (i, call) in tool_calls.iter().enumerate() {
            self.output.list_item(i + 1, &format!("{} {}", call.tool, call.args.join(" ")));
        }

        Ok(tool_calls)
    }

    async fn call_api(&mut self, prompt: &str) -> Result<String> {
        // Add user message to history
        self.messages.push(Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let request = AnthropicRequest {
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 2000,
            messages: self.messages.clone(),
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

        let assistant_message = response_body.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_else(|| "No response".to_string());

        // Add assistant response to history
        self.messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_message.clone(),
        });

        Ok(assistant_message)
    }

    pub async fn reason(&mut self, user_input: &str) -> Result<String> {
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

    pub async fn respond(&mut self, user_input: &str, reasoning: &str) -> Result<String> {
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

    pub async fn create_tool_calls(&mut self, user_input: &str, reasoning: &str) -> Result<Vec<ToolCall>> {
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

        // Strip markdown code blocks if present
        let cleaned_json = json_text
            .trim()
            .strip_prefix("```json")
            .or_else(|| json_text.trim().strip_prefix("```"))
            .unwrap_or(json_text.trim())
            .strip_suffix("```")
            .unwrap_or(json_text.trim())
            .trim();

        let tool_calls: Vec<ToolCall> = serde_json::from_str(cleaned_json)
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse tool calls: {}. Response was:\n{}", e, json_text);
                vec![]
            });

        for (i, call) in tool_calls.iter().enumerate() {
            self.output.list_item(i + 1, &format!("{} {}", call.tool, call.args.join(" ")));
        }

        Ok(tool_calls)
    }

    pub async fn execute_tool_calls(&mut self, tool_calls: Vec<ToolCall>) -> Result<()> {
        self.execute_tool_calls_with_retry(tool_calls, 0).await
    }

    async fn execute_tool_calls_with_retry(&mut self, tool_calls: Vec<ToolCall>, depth: u32) -> Result<()> {
        const MAX_RETRIES: u32 = 2;

        self.output.tool_header("Executing");

        let mut results = Vec::new();
        let mut had_error = false;
        let mut failed_call: Option<ToolCall> = None;
        let mut error_msg = String::new();

        for (i, call) in tool_calls.iter().enumerate() {
            self.output.info(&format!("\n[{}/{}] {}", i + 1, tool_calls.len(), call.description));

            match self.execute_single_tool(&call).await {
                Ok(result) => {
                    if !result.is_empty() {
                        println!("{}", result);
                    }
                    self.output.success("✓ Done");
                    results.push(format!("{}: {}", call.description, result));
                }
                Err(e) => {
                    let err_str = format!("{}", e);
                    self.output.error(&format!("✗ Error: {}", err_str));
                    results.push(format!("{}: Error - {}", call.description, err_str));

                    if depth < MAX_RETRIES {
                        had_error = true;
                        failed_call = Some(call.clone());
                        error_msg = err_str;
                        break;
                    }
                }
            }
        }

        println!();

        // Add tool execution results to conversation history
        if !results.is_empty() {
            let results_summary = format!("Tool execution results:\n{}", results.join("\n"));
            self.messages.push(Message {
                role: "user".to_string(),
                content: results_summary,
            });
        }

        // If error occurred and we haven't hit max retries, try error recovery
        if had_error {
            if let Some(call) = failed_call {
                println!();
                self.output.info(&format!("Retry attempt {}/{}", depth + 1, MAX_RETRIES));
                self.handle_error(&error_msg, &call, depth).await?;
            }
        } else {
            self.output.success("All tasks completed!");
        }

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
