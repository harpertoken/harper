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

//! Image processing tool
//!
//! This module provides functionality for image information and processing.

use crate::core::error::HarperError;
use crate::tools::parsing;
use colored::*;
use image::GenericImageView;

/// Get image information
pub fn get_image_info(response: &str) -> crate::core::error::HarperResult<String> {
    let path = parsing::extract_tool_arg(response, "[IMAGE_INFO")?;

    println!(
        "{} Get info for image: {}",
        "System:".bold().magenta(),
        path.magenta()
    );

    let img = image::open(&path)
        .map_err(|e| HarperError::Command(format!("Failed to open image {}: {}", path, e)))?;

    let (width, height) = img.dimensions();
    let color_type = img.color();

    Ok(format!(
        "Image: {}\nDimensions: {}x{}\nColor type: {:?}",
        path, width, height, color_type
    ))
}

/// Resize image
pub fn resize_image(response: &str) -> crate::core::error::HarperResult<String> {
    let args = parsing::extract_tool_args(response, "[IMAGE_RESIZE", 4)?;
    let input_path = &args[0];
    let output_path = &args[1];
    let width: u32 = args[2]
        .parse()
        .map_err(|_| HarperError::Command("Invalid width".to_string()))?;
    let height: u32 = args[3]
        .parse()
        .map_err(|_| HarperError::Command("Invalid height".to_string()))?;

    println!(
        "{} Resize image {} to {}x{} and save to {} ? (y/n): ",
        "System:".bold().magenta(),
        input_path.magenta(),
        width,
        height,
        output_path.magenta()
    );
    let mut approval = String::new();
    std::io::stdin().read_line(&mut approval)?;
    if !approval.trim().eq_ignore_ascii_case("y") {
        return Ok("Image resize cancelled by user".to_string());
    }

    println!(
        "{} Resizing image: {}",
        "System:".bold().magenta(),
        input_path.magenta()
    );

    let img = image::open(input_path)
        .map_err(|e| HarperError::Command(format!("Failed to open image {}: {}", input_path, e)))?;

    let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);

    resized.save(output_path).map_err(|e| {
        HarperError::Command(format!("Failed to save image {}: {}", output_path, e))
    })?;

    Ok(format!(
        "Image resized to {}x{} and saved to {}",
        width, height, output_path
    ))
}
