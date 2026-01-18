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

//! UI components for Harper

pub mod interfaces;
pub mod plugins;

#[cfg(test)]
mod tests {
    use arboard::ImageData;
    use std::borrow::Cow;
    use std::error::Error;
    use std::fs;

    #[test]
    fn test_clipboard_image_processing() -> Result<(), Box<dyn Error>> {
        // Test the core image processing logic used in clipboard functionality
        // Create mock RGBA image data (2x2 red square)
        let mock_bytes = vec![
            255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
        ];
        let image_data = ImageData {
            width: 2,
            height: 2,
            bytes: Cow::from(mock_bytes),
        };

        // Call the actual function
        let result = crate::interfaces::ui::events::save_image_to_temp(&image_data);
        assert!(result.is_ok(), "save_image_to_temp should succeed");
        let file_path = result.unwrap();

        // Verify the file was created
        assert!(file_path.exists(), "Image file should be created");
        assert!(file_path.to_string_lossy().contains("harper_images"));
        assert!(file_path.to_string_lossy().ends_with(".png"));

        let result = (|| {
            let loaded_img = image::open(&file_path)?;
            assert_eq!(loaded_img.width(), 2);
            assert_eq!(loaded_img.height(), 2);
            Ok(())
        })();

        // Clean up is now guaranteed to run.
        // We ignore the result of remove_file, as the test result is more important.
        let _ = fs::remove_file(&file_path);

        result
        // Optional: clean up directory if empty, but be careful in tests
        // fs::remove_dir(file_path.parent().unwrap()).ok();
    }
}
