use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn write(path: &str, content: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, content)?;
    Ok(())
}
