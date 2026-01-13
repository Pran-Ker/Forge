use anyhow::Result;
use globset::Glob;
use walkdir::WalkDir;
use std::time::SystemTime;

pub fn glob(pattern: &str, base_path: Option<&str>) -> Result<Vec<String>> {
    let glob = Glob::new(pattern)?.compile_matcher();
    let path = base_path.unwrap_or(".");

    let mut matches: Vec<(String, SystemTime)> = Vec::new();

    for entry in WalkDir::new(path).follow_links(true) {
        let entry = entry?;
        if glob.is_match(entry.path()) {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    matches.push((entry.path().display().to_string(), modified));
                }
            }
        }
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(matches.into_iter().map(|(path, _)| path).collect())
}
