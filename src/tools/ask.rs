use anyhow::Result;
use inquire::{Select, MultiSelect, Text};
use crate::types::AskResult;

pub fn ask(
    question: &str,
    options: Option<Vec<String>>,
    multi: bool,
) -> Result<AskResult> {
    if let Some(choices) = options {
        if multi {
            let selected = MultiSelect::new(question, choices).prompt()?;
            Ok(AskResult::Multi(selected))
        } else {
            let selected = Select::new(question, choices).prompt()?;
            Ok(AskResult::Single(selected))
        }
    } else {
        let answer = Text::new(question).prompt()?;
        Ok(AskResult::Text(answer))
    }
}
