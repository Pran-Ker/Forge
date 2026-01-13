use inquire::Text;
use shadow::{Output, Agent};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let output = Output::new();
    output.success("Forge v0.1.0 - AI Agent");
    output.info("Powered by Claude Sonnet 4.5\n");

    let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| {
        output.error("ANTHROPIC_API_KEY not found in environment");
        output.info("Set it with: export ANTHROPIC_API_KEY=your_key");
        output.info("Or create a .env file");
        std::process::exit(1);
    });

    let mut agent = Agent::new(api_key);

    output.info("Agent is ready. Ask me to do something with your files!\n");
    output.info("Type 'exit' or 'quit' to exit\n");

    loop {
        let input = match Text::new("you>").prompt() {
            Ok(i) => i,
            Err(_) => break,
        };

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            output.success("Goodbye!");
            break;
        }

        println!();

        match agent.reason(input).await {
            Ok(reasoning) => {
                println!();

                match agent.respond(input, &reasoning).await {
                    Ok(_response) => {
                        println!();

                        match agent.create_tool_calls(input, &reasoning).await {
                            Ok(tool_calls) => {
                                if tool_calls.is_empty() {
                                    output.info("No tool calls needed for this request");
                                    continue;
                                }

                                println!();

                                if let Err(e) = agent.execute_tool_calls(tool_calls).await {
                                    output.error(&format!("Execution error: {}", e));
                                }
                            }
                            Err(e) => output.error(&format!("Tool call planning error: {}", e)),
                        }
                    }
                    Err(e) => output.error(&format!("Response error: {}", e)),
                }
            }
            Err(e) => output.error(&format!("Reasoning error: {}", e)),
        }

        println!();
    }

    Ok(())
}
