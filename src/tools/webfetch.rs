use anyhow::Result;
use std::time::Duration;

pub struct FetchResult {
    pub url: String,
    pub content: String,
}

pub async fn webfetch(url: &str) -> Result<FetchResult> {
    let client = reqwest::Client::builder()
        .user_agent("Forge/1.0")
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;
    let final_url = response.url().to_string();
    let html = response.text().await?;

    let markdown = html2md::parse_html(&html);

    Ok(FetchResult {
        url: final_url,
        content: markdown,
    })
}
