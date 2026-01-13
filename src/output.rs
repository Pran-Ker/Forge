use owo_colors::OwoColorize;
use console::Term;

pub struct Output {
    pub term: Term,
}

impl Output {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    pub fn success(&self, msg: &str) {
        println!("{}", msg.green().bold());
    }

    pub fn error(&self, msg: &str) {
        eprintln!("{}", msg.red().bold());
    }

    pub fn info(&self, msg: &str) {
        println!("{}", msg.blue());
    }

    pub fn tool_header(&self, tool: &str) {
        println!("{}", format!("→ {}", tool).cyan().bold());
    }

    pub fn tool_output(&self, content: &str) {
        println!("{}", content);
    }

    pub fn diff(&self, diff_text: &str) {
        for line in diff_text.lines() {
            match line.chars().next() {
                Some('+') => println!("{}", line.green()),
                Some('-') => println!("{}", line.red()),
                Some('@') => println!("{}", line.cyan()),
                _ => println!("{}", line),
            }
        }
    }

    pub fn list_item(&self, index: usize, content: &str) {
        println!("{} {}", format!("{}.", index).cyan().bold(), content);
    }
}
