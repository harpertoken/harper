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

//! Filesystem operations tool
//!
//! This module provides functionality for reading, writing, and
//! searching files with user approval.

use crate::core::error::{HarperError, HarperResult};
use crate::memory::cache::CacheAlignedBuffer;
use crate::tools::parsing;
use colored::*;
use std::io::{self, Write};

use crate::core::io_traits::UserApproval;
use std::sync::Arc;

/// Read a file
pub async fn read_file(
    response: &str,
    approver: Option<Arc<dyn UserApproval>>,
) -> HarperResult<String> {
    let path = parsing::extract_tool_arg(response, "[READ_FILE")?;

    if let Some(appr) = approver {
        if !appr.approve("Read file?", &path).await? {
            return Ok("File read cancelled by user".to_string());
        }
    } else {
        println!(
            "{} Reading file: {}",
            "System:".bold().magenta(),
            path.magenta()
        );
    }

    std::fs::read_to_string(&path)
        .map_err(|e| HarperError::Command(format!("Failed to read file {}: {}", path, e)))
}

/// Write to a file
pub async fn write_file(
    response: &str,
    approver: Option<Arc<dyn UserApproval>>,
) -> HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[WRITE_FILE", 2)?;
    let path = &args[0];
    let content = &args[1];

    let is_approved = if let Some(appr) = approver {
        appr.approve("Write to file?", path).await?
    } else {
        let p = path.clone();
        tokio::task::spawn_blocking(move || {
            println!(
                "{} Write to file {}? (y/n): ",
                "System:".bold().magenta(),
                p.magenta()
            );
            io::stdout()
                .flush()
                .map_err(|e| HarperError::Io(e.to_string()))?;
            let mut approval = String::new();
            io::stdin()
                .read_line(&mut approval)
                .map_err(|e| HarperError::Io(e.to_string()))?;
            Ok::<bool, HarperError>(approval.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| HarperError::Command(format!("Task failed: {}", e)))??
    };

    if !is_approved {
        return Ok("File write cancelled by user".to_string());
    }

    println!(
        "{} Writing to file: {}",
        "System:".bold().magenta(),
        path.magenta()
    );

    write_cache_aligned(path, content.as_bytes())
        .map_err(|e| HarperError::Command(format!("Failed to write file {}: {}", path, e)))?;

    Ok(format!(
        "Successfully wrote {} bytes to {}",
        content.len(),
        path
    ))
}

/// Search and replace in a file
pub async fn search_replace(
    response: &str,
    approver: Option<Arc<dyn UserApproval>>,
) -> HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[SEARCH_REPLACE", 3)?;
    let path = &args[0];
    let old_string = &args[1];
    let new_string = &args[2];

    let is_approved = if let Some(appr) = approver {
        appr.approve("Search and replace in file?", path).await?
    } else {
        let p = path.clone();
        tokio::task::spawn_blocking(move || {
            println!(
                "{} Search and replace in file {}? (y/n): ",
                "System:".bold().magenta(),
                p.magenta()
            );
            io::stdout()
                .flush()
                .map_err(|e| HarperError::Io(e.to_string()))?;
            let mut approval = String::new();
            io::stdin()
                .read_line(&mut approval)
                .map_err(|e| HarperError::Io(e.to_string()))?;
            Ok::<bool, HarperError>(approval.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .map_err(|e| HarperError::Command(format!("Task failed: {}", e)))??
    };

    if !is_approved {
        return Ok("Search and replace cancelled by user".to_string());
    }

    println!(
        "{} Searching and replacing in file: {}",
        "System:".bold().magenta(),
        path.magenta()
    );

    let content = std::fs::read_to_string(path)
        .map_err(|e| HarperError::Command(format!("Failed to read file {}: {}", path, e)))?;

    let new_content = content.replace(old_string, new_string);
    let replacements = content.matches(old_string).count();

    write_cache_aligned(path, new_content.as_bytes())
        .map_err(|e| HarperError::Command(format!("Failed to write file {}: {}", path, e)))?;

    Ok(format!("Replaced {} occurrences in {}", replacements, path))
}

fn write_cache_aligned(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut buffer = CacheAlignedBuffer::with_capacity(bytes.len());
    buffer.write_bytes(bytes);
    std::fs::write(path, buffer.as_slice())
}
