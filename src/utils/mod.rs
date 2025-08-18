use std::time::Duration;

pub async fn web_search(query: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;
    let url = format!("https://api.duckduckgo.com/?q={}&format=json", query);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let error_text = format!(
            "Search API returned a non-success status: {}. Body: {}",
            response.status(),
            response
                .text()
                .await
                .unwrap_or_else(|_| "Could not read body".to_string())
        );
        return Ok(error_text);
    }

    response.text().await
}
