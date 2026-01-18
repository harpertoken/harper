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

//! Web search tool
//!
//! This module provides functionality for performing web searches.

use crate::runtime::utils::web_search;
use colored::*;

/// Perform web search
pub async fn perform_web_search(response: &str) -> crate::core::error::HarperResult<String> {
    let query_part = response
        .split_once(':')
        .map(|x| x.1)
        .unwrap_or("")
        .trim_end_matches(']');

    println!(
        "{} Searching the web for: {}",
        "System:".bold().magenta(),
        query_part.magenta()
    );

    web_search(query_part).await
}
