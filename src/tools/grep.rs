use anyhow::Result;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use std::path::Path;
use walkdir::WalkDir;

use crate::types::GrepMatch;

pub fn grep(
    pattern: &str,
    path: &str,
    case_insensitive: bool,
) -> Result<Vec<GrepMatch>> {
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(case_insensitive)
        .build(pattern)?;

    let mut matches = Vec::new();
    let mut searcher = Searcher::new();

    if Path::new(path).is_dir() {
        for entry in WalkDir::new(path).follow_links(true) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path().display().to_string();
                search_file(&matcher, &mut searcher, &file_path, &mut matches)?;
            }
        }
    } else {
        search_file(&matcher, &mut searcher, path, &mut matches)?;
    }

    Ok(matches)
}

fn search_file(
    matcher: &grep_regex::RegexMatcher,
    searcher: &mut Searcher,
    path: &str,
    matches: &mut Vec<GrepMatch>,
) -> Result<()> {
    let path_string = path.to_string();

    searcher.search_path(
        matcher,
        path,
        UTF8(|lnum, line| {
            matches.push(GrepMatch {
                file: path_string.clone(),
                line_num: lnum,
                content: line.trim_end().to_string(),
            });
            Ok(true)
        }),
    )?;

    Ok(())
}
