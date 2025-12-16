// Copyright 2025 harpertoken
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
