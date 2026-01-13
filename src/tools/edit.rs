use anyhow::Result;
use similar::TextDiff;
use std::fs;
use std::path::Path;

pub fn edit(path: &str, search: &str, replace: &str, replace_all: bool) -> Result<String> {
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", path));
    }

    let original = fs::read_to_string(path)?;

    let modified = if replace_all {
        original.replace(search, replace)
    } else {
        original.replacen(search, replace, 1)
    };

    if original == modified {
        return Err(anyhow::anyhow!("Pattern not found in file"));
    }

    let diff = TextDiff::from_lines(&original, &modified);
    let mut diff_output = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        diff_output.push_str(&format!("{}{}", sign, change));
    }

    fs::write(path, &modified)?;

    Ok(diff_output)
}
