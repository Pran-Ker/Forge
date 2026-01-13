use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn read(path: &str, offset: Option<usize>, limit: Option<usize>) -> Result<String> {
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", path));
    }

    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let start = offset.unwrap_or(0);
    let end = limit
        .map(|l| (start + l).min(lines.len()))
        .unwrap_or(lines.len());

    let formatted_lines: Vec<String> = lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{:>6}\t{}", start + i + 1, line))
        .collect();

    Ok(formatted_lines.join("\n"))
}
