//! Utility functions for web search and cryptography
//!
//! This module provides helper functions for web searching and cryptographic operations.

use crate::core::constants::timeouts;
use crate::core::error::HarperResult;

pub mod crypto;

/// Perform a web search using DuckDuckGo API
///
/// Searches the web for the given query and returns the results.
/// This is used by the AI assistant to gather information when needed.
///
/// # Arguments
/// * `query` - The search query string
///
/// # Returns
/// Search results as a string, or an error message if the search fails
///
/// # Errors
/// Returns `HarperError::WebSearch` if the API request fails
pub async fn web_search(query: &str) -> HarperResult<String> {
    let client = reqwest::Client::builder()
        .timeout(timeouts::WEB_SEARCH)
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

    Ok(response.text().await?)
}
