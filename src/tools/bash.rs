use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use std::process::Stdio;

pub struct BashOutput {
    pub output: String,
    pub exit_code: Option<i32>,
}

pub async fn bash(command: &str) -> Result<BashOutput> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output = String::new();

    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(l) => {
                        println!("{}", l);
                        output.push_str(&l);
                        output.push('\n');
                    }
                    None => break,
                }
            }
            line = stderr_reader.next_line() => {
                match line? {
                    Some(l) => {
                        eprintln!("{}", l);
                        output.push_str(&l);
                        output.push('\n');
                    }
                    None => {}
                }
            }
        }
    }

    let status = child.wait().await?;

    Ok(BashOutput {
        output,
        exit_code: status.code(),
    })
}
