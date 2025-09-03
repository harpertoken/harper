//! AI provider integrations and cryptographic utilities
//!
//! This module handles communication with different AI providers (OpenAI, Sambanova, Gemini)
//! and provides cryptographic functions for secure data handling.

use crate::core::constants::crypto::*;
use crate::core::error::{HarperError, HarperResult};
use crate::core::{ApiConfig, ApiProvider, Message};
// use mcp_client::{transport::sse::SseTransportHandle, McpClient, McpClientTrait, McpService}; // Temporarily disabled
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use ring::{
    aead::{self},
    rand::{SecureRandom, SystemRandom},
};
use serde_json::json;
// use tower::timeout::Timeout; // Temporarily disabled

/// Call the configured LLM API with conversation history
///
/// Sends a request to the configured AI provider with the conversation history
/// and returns the AI's response.
///
/// # Arguments
/// * `client` - HTTP client for making API requests
/// * `config` - API configuration including provider, key, and model
/// * `history` - Conversation history as a slice of messages
///
/// # Returns
/// The AI's response as a string
///
/// # Errors
/// Returns `HarperError` if the API call fails or response parsing fails
pub async fn call_llm(
    client: &reqwest::Client,
    config: &ApiConfig,
    history: &[Message],
) -> HarperResult<String> {
    let res = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => {
            let messages_json: Vec<_> = history
                .iter()
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect();

            let extra_query = String::new();
            // MCP functionality temporarily disabled due to dependency conflicts
            // This resolves CodeQL duplicate dependency warnings and improves security analysis
            // MCP can be re-enabled when a compatible client version becomes available

            let body = json!({
                "model": config.model_name,
                "messages": messages_json,
                "temperature": 0.1,
                "top_p": 0.1,
                "extra_query": extra_query,
            });
            client
                .post(&config.base_url)
                .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
        }
        ApiProvider::Gemini => {
            let mut gemini_contents = Vec::new();
            if let Some(first_message) = history.first() {
                if first_message.role == "system" {
                    gemini_contents.push(json!({
                        "role": "user",
                        "parts": [{"text": first_message.content}]
                    }));
                    gemini_contents.push(json!({
                        "role": "model",
                        "parts": [{"text": "Understood."}]
                    }));
                }
            }

            for msg in history.iter().skip(1) {
                let role = if msg.role == "assistant" {
                    "model"
                } else {
                    "user"
                };
                gemini_contents.push(json!({
                    "role": role,
                    "parts": [{"text": msg.content}]
                }));
            }

            let body = json!({
                "contents": gemini_contents
            });
            client
                .post(&config.base_url)
                .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
        }
    };

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(HarperError::Api(format!(
            "API Error: {} ({})",
            error_text, status
        )));
    }

    let resp_json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| HarperError::Api(e.to_string()))?;

    let assistant_reply = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => resp_json["choices"][0]["message"]
            ["content"]
            .as_str()
            .unwrap_or("[No response]")
            .to_string(),
        ApiProvider::Gemini => resp_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("[No response]")
            .to_string(),
    };

    Ok(assistant_reply)
}

/// Encrypts data using AES-GCM with a randomly generated key
///
/// # Arguments
/// * `data` - The data to encrypt
///
/// # Returns
/// A tuple containing (encrypted_data, key) on success
///
/// # Errors
/// Returns `HarperError` if any cryptographic operation fails
///
/// Encrypts data using AES-GCM with a randomly generated key
///
/// # Arguments
/// * `data` - The data to encrypt
///
/// # Returns
/// A tuple containing (encrypted_data, key) on success
///
/// # Errors
/// Returns `HarperError` if any cryptographic operation fails
#[allow(dead_code)]
pub fn encrypt_data(data: &[u8]) -> HarperResult<(Vec<u8>, Vec<u8>)> {
    if data.is_empty() {
        return Err(HarperError::Crypto("empty data".to_string()));
    }

    let rng = SystemRandom::new();

    // Generate a random 256-bit key
    let mut key_bytes = [0u8; AES_256_KEY_LEN];
    rng.fill(&mut key_bytes)
        .map_err(|e| HarperError::Crypto(format!("Key generation failed: {}", e)))?;

    // Generate a random 96-bit nonce (12 bytes)
    let mut nonce_bytes = [0u8; AES_GCM_NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| HarperError::Crypto(format!("Nonce generation failed: {}", e)))?;

    let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|e| HarperError::Crypto(format!("Invalid key: {}", e)))?;

    let sealing_key = aead::LessSafeKey::new(key);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    // Create a mutable copy of the data to encrypt
    let mut in_out = data.to_vec();

    // Encrypt the data in-place
    let tag = sealing_key
        .seal_in_place_separate_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|e| HarperError::Crypto(format!("Encryption failed: {}", e)))?;

    // Combine nonce (12) + ciphertext (same as input) + tag (16)
    let mut encrypted = Vec::with_capacity(AES_GCM_NONCE_LEN + in_out.len() + AES_GCM_TAG_LEN);
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&in_out);
    encrypted.extend_from_slice(tag.as_ref());

    Ok((encrypted, key_bytes.to_vec()))
}

/// Decrypts data using AES-GCM
///
/// # Arguments
/// * `encrypted_data` - The encrypted data (nonce + ciphertext + tag)
/// * `key` - The 256-bit key used for decryption
///
/// # Returns
/// The decrypted data on success
///
/// # Errors
/// Returns `HarperError` if decryption fails or input is invalid
#[allow(dead_code)]
pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> HarperResult<Vec<u8>> {
    // Minimum size is nonce (12) + tag (16) = 28 bytes
    if encrypted_data.is_empty() {
        return Err(HarperError::Crypto("empty data".to_string()));
    }
    if encrypted_data.len() < MIN_ENCRYPTED_LEN {
        return Err(HarperError::Crypto("data too short".to_string()));
    }

    // Split the encrypted data into nonce, ciphertext, and tag
    let (nonce_bytes, rest) = encrypted_data.split_at(AES_GCM_NONCE_LEN);
    let (ciphertext, tag) = rest.split_at(rest.len() - AES_GCM_TAG_LEN);

    // Validate key length (256 bits = 32 bytes)
    if key.len() != AES_256_KEY_LEN {
        return Err(HarperError::Crypto("invalid key length".to_string()));
    }

    // Create the key and nonce
    let key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
        .map_err(|e| HarperError::Crypto(format!("Invalid key: {}", e)))?;

    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| HarperError::Crypto("invalid nonce format".to_string()))?;

    // Combine ciphertext and tag for decryption
    let mut in_out = ciphertext.to_vec();
    in_out.extend_from_slice(tag);

    // Decrypt the data
    let opening_key = aead::LessSafeKey::new(key);
    let decrypted = opening_key
        .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|e| HarperError::Crypto(format!("Decryption failed: {}", e)))?;

    Ok(decrypted.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::test_data;
    use hex_literal::hex;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let message = b"Hello, World! This is a test message.";

        // Test with empty data (should fail)
        assert!(
            matches!(encrypt_data(&[]), Err(HarperError::Crypto(_))),
            "Encryption with empty data should fail with Crypto error"
        );

        // Test with valid data
        let (encrypted, key) = encrypt_data(message).expect("Encryption should succeed");

        // Verify the encrypted data has the correct structure
        assert!(encrypted.len() > AES_GCM_NONCE_LEN + AES_GCM_TAG_LEN);

        // Decrypt the message
        let decrypted = decrypt_data(&encrypted, &key).expect("Decryption should succeed");

        // Verify the decrypted message matches the original
        assert_eq!(message, decrypted.as_slice());

        // Test with maximum length data
        let large_message = vec![0xAA; test_data::LARGE_MESSAGE_SIZE];
        let (encrypted_large, key_large) =
            encrypt_data(&large_message).expect("Should encrypt large data");
        let decrypted_large =
            decrypt_data(&encrypted_large, &key_large).expect("Should decrypt large data");
        assert_eq!(large_message, decrypted_large);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let message = b"Test message";

        // Encrypt with one key
        let (encrypted, _) = encrypt_data(message).expect("Encryption should succeed");

        // Try to decrypt with a different key
        let wrong_key = [0u8; AES_256_KEY_LEN]; // All zeros key
        let result = decrypt_data(&encrypted, &wrong_key);

        // Should fail with Crypto error
        assert!(
            matches!(result, Err(HarperError::Crypto(_))),
            "Decryption with wrong key should fail with Crypto error"
        );

        // Try with invalid key length
        let invalid_key = [0u8; test_data::INVALID_KEY_LEN]; // 128-bit key (too short)
        let result = decrypt_data(&encrypted, &invalid_key);

        // Should fail with Crypto error
        assert!(
            matches!(result, Err(HarperError::Crypto(_))),
            "Decryption with invalid key length should fail with Crypto error"
        );
    }

    #[test]
    fn test_decrypt_invalid_data_fails() {
        // Test with empty data
        assert!(
            matches!(decrypt_data(&[], &[0u8; 32]), Err(HarperError::Crypto(_))),
            "Decryption with empty data should fail with Crypto error"
        );

        // Test with data that's too short (less than nonce + tag)
        let short_data = [0u8; test_data::SHORT_DATA_LEN];
        assert!(
            matches!(
                decrypt_data(&short_data, &[0u8; 32]),
                Err(HarperError::Crypto(_))
            ),
            "Decryption with short data should fail with Crypto error"
        );

        // Test with valid length but invalid format
        let mut invalid_data = vec![0u8; AES_GCM_NONCE_LEN + AES_GCM_TAG_LEN + AES_GCM_TAG_LEN];
        invalid_data[..AES_GCM_NONCE_LEN].copy_from_slice(&hex!("000000000000000000000000")); // Valid nonce

        // This should fail during decryption
        let result = decrypt_data(&invalid_data, &[0u8; 32]);
        assert!(
            matches!(result, Err(HarperError::Crypto(_))),
            "Decryption of invalid data should fail with Crypto error"
        );
    }

    #[test]
    fn test_error_messages() {
        // Test error message formatting
        let key_error = HarperError::Crypto("Invalid key: test".to_string());
        assert_eq!(
            format!("{}", key_error),
            "Cryptography error: Invalid key: test"
        );

        let nonce_error = HarperError::Crypto("Invalid nonce: test".to_string());
        assert_eq!(
            format!("{}", nonce_error),
            "Cryptography error: Invalid nonce: test"
        );

        let decryption_error = HarperError::Crypto("Decryption failed: test".to_string());
        assert_eq!(
            format!("{}", decryption_error),
            "Cryptography error: Decryption failed: test"
        );

        let input_error = HarperError::Crypto("Invalid input: test".to_string());
        assert_eq!(
            format!("{}", input_error),
            "Cryptography error: Invalid input: test"
        );
    }
}
