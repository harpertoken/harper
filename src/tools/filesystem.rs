//! Filesystem operations tool
//!
//! This module provides functionality for reading, writing, and
//! searching files with user approval.

use crate::core::error::HarperError;
use crate::tools::parsing;
use colored::*;
use std::io;

/// Read a file
pub fn read_file(response: &str) -> crate::core::error::HarperResult<String> {
    let path = parsing::extract_tool_arg(response, "[READ_FILE")?;

    println!(
        "{} Reading file: {}",
        "System:".bold().magenta(),
        path.magenta()
    );

    std::fs::read_to_string(&path)
        .map_err(|e| HarperError::Command(format!("Failed to read file {}: {}", path, e)))
}

/// Write to a file
pub fn write_file(response: &str) -> crate::core::error::HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[WRITE_FILE", 2)?;
    let path = &args[0];
    let content = &args[1];

    println!(
        "{} Write to file {}? (y/n): ",
        "System:".bold().magenta(),
        path.magenta()
    );
    let mut approval = String::new();
    io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
        return Ok("File write cancelled by user".to_string());
    }

    println!(
        "{} Writing to file: {}",
        "System:".bold().magenta(),
        path.magenta()
    );

    std::fs::write(path, content)
        .map_err(|e| HarperError::Command(format!("Failed to write file {}: {}", path, e)))?;

    Ok(format!(
        "Successfully wrote {} bytes to {}",
        content.len(),
        path
    ))
}

/// Search and replace in a file
pub fn search_replace(response: &str) -> crate::core::error::HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[SEARCH_REPLACE", 3)?;
    let path = &args[0];
    let old_string = &args[1];
    let new_string = &args[2];

    println!(
        "{} Search and replace in file {}? (y/n): ",
        "System:".bold().magenta(),
        path.magenta()
    );
    let mut approval = String::new();
    io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
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

    std::fs::write(path, &new_content)
        .map_err(|e| HarperError::Command(format!("Failed to write file {}: {}", path, e)))?;

    Ok(format!("Replaced {} occurrences in {}", replacements, path))
}
/// Extract single argument from tool command
fn extract_tool_arg(response: &str, prefix: &str) -> crate::core::error::HarperResult<String> {
    let arg_str = response
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| HarperError::Command("Invalid tool format".to_string()))?
        .trim();

    if arg_str.is_empty() {
        return Err(HarperError::Command("No argument provided".to_string()));
    }

    Ok(arg_str.to_string())
}

/// Extract multiple arguments from tool command with proper quote handling
fn extract_tool_args(
    response: &str,
    prefix: &str,
    num_args: usize,
) -> crate::core::error::HarperResult<Vec<String>> {
    let arg_str = extract_tool_arg(response, prefix)?;
    let args = parse_quoted_args(&arg_str)?;

    if args.len() != num_args {
        return Err(HarperError::Command(format!(
            "Expected {} arguments, got {}",
            num_args,
            args.len()
        )));
    }

    Ok(args)
}

/// Parse space-separated arguments with quote support
fn parse_quoted_args(input: &str) -> crate::core::error::HarperResult<Vec<String>> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => {
                if in_quotes {
                    // End of quoted string
                    in_quotes = false;
                } else {
                    // Start of quoted string
                    in_quotes = true;
                }
            }
            ' ' => {
                if in_quotes {
                    // Space inside quotes is part of the argument
                    current_arg.push(ch);
                } else if !current_arg.is_empty() {
                    // End of unquoted argument
                    args.push(current_arg);
                    current_arg = String::new();
                }
                // Skip multiple spaces
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    // Add the last argument if any
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    // Check for unclosed quotes
    if in_quotes {
        return Err(HarperError::Command(
            "Unclosed quote in arguments".to_string(),
        ));
    }

    Ok(args)
}
