use anyhow::Result;
use crate::types::SearchResult;

pub async fn websearch(query: &str) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;

    let response = client.get(&url).send().await?;
    let html = response.text().await?;

    let mut results = Vec::new();

    for line in html.lines() {
        if line.contains("result__title") {
            results.push(SearchResult {
                title: "DuckDuckGo Search".to_string(),
                url: "https://duckduckgo.com".to_string(),
                snippet: query.to_string(),
            });
            if results.len() >= 5 {
                break;
            }
        }
    }

    if results.is_empty() {
        results.push(SearchResult {
            title: format!("Search: {}", query),
            url: format!("https://duckduckgo.com/?q={}", urlencoding::encode(query)),
            snippet: "No detailed results available".to_string(),
        });
    }

    Ok(results)
}
