use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Pattern error: {0}")]
    PatternError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct GrepMatch {
    pub file: String,
    pub line_num: u64,
    pub content: String,
}

pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub enum AskResult {
    Single(String),
    Multi(Vec<String>),
    Text(String),
}
