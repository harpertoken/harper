use crate::core::{ApiConfig, ApiProvider, Message};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use mcp_client::{transport::sse::SseTransportHandle, McpClient, McpClientTrait, McpService};
use ring::{
    aead::{self, NONCE_LEN},
    error::Unspecified,
    rand::{SecureRandom, SystemRandom},
};
use tower::timeout::Timeout;

// Constants for AES-256-GCM
const AES_256_GCM_TAG_LEN: usize = 16;
use std::error::Error;
use std::fmt;

pub async fn call_llm(
    client: &reqwest::Client,
    config: &ApiConfig,
    history: &[Message],
    mcp_client: Option<&McpClient<Timeout<McpService<SseTransportHandle>>>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let res = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => {
            let messages_json: Vec<_> = history
                .iter()
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect();

            let mut extra_query = String::new();
            if let Some(mcp) = mcp_client {
                match mcp
                    .call_tool("llm_query", json!({ "query": history }))
                    .await
                {
                    Ok(result) => {
                        if let Some(content) = result.content.first() {
                            if let Some(text) = content.as_text() {
                                extra_query = text.to_string();
                            }
                        }
                    }
                    Err(e) => {
                        // Log the error but continue without the extra query
                        eprintln!("MCP tool call failed: {}", e);
                    }
                }
            }

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
            let url = format!("{}?key={}", config.base_url, config.api_key);
            client
                .post(&url)
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
        return Err(format!("API Error: {} ({})", error_text, status).into());
    }

    let resp_json: serde_json::Value = res.json().await.unwrap_or_else(|_| json!({}));

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

/// Error type for cryptographic operations
#[derive(Debug)]
pub enum CryptoError {
    KeyGenerationFailed(String),
    NonceGenerationFailed(String),
    InvalidKey(String),
    InvalidNonce(String),
    DecryptionFailed(String),
    InvalidInput(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyGenerationFailed(e) => write!(f, "Key generation failed: {}", e),
            CryptoError::NonceGenerationFailed(e) => write!(f, "Nonce generation failed: {}", e),
            CryptoError::InvalidKey(e) => write!(f, "Invalid key: {}", e),
            CryptoError::InvalidNonce(e) => write!(f, "Invalid nonce: {}", e),
            CryptoError::DecryptionFailed(e) => write!(f, "Decryption failed: {}", e),
            CryptoError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

impl Error for CryptoError {}

impl From<Unspecified> for CryptoError {
    fn from(err: Unspecified) -> Self {
        CryptoError::DecryptionFailed(err.to_string())
    }
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
/// Returns `CryptoError` if any cryptographic operation fails
#[allow(dead_code)]
pub fn encrypt_data(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    if data.is_empty() {
        return Err(CryptoError::InvalidInput("empty data".to_string()));
    }

    let rng = SystemRandom::new();

    // Generate a random 256-bit key
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes)
        .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?;

    // Generate a random 96-bit nonce (12 bytes)
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| CryptoError::NonceGenerationFailed(e.to_string()))?;

    let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;

    let sealing_key = aead::LessSafeKey::new(key);
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    // Create a mutable copy of the data to encrypt
    let mut in_out = data.to_vec();

    // Encrypt the data in-place
    let tag = sealing_key
        .seal_in_place_separate_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    // Combine nonce (12) + ciphertext (same as input) + tag (16)
    let mut encrypted = Vec::with_capacity(NONCE_LEN + in_out.len() + AES_256_GCM_TAG_LEN);
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
/// Returns `CryptoError` if decryption fails or input is invalid
#[allow(dead_code)]
pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // Minimum size is nonce (12) + tag (16) = 28 bytes
    if encrypted_data.is_empty() {
        return Err(CryptoError::InvalidInput("empty data".to_string()));
    }
    if encrypted_data.len() < NONCE_LEN + AES_256_GCM_TAG_LEN {
        return Err(CryptoError::InvalidInput("data too short".to_string()));
    }

    // Split the encrypted data into nonce, ciphertext, and tag
    let (nonce_bytes, rest) = encrypted_data.split_at(NONCE_LEN);
    let (ciphertext, tag) = rest.split_at(rest.len() - AES_256_GCM_TAG_LEN);

    // Validate key length (256 bits = 32 bytes)
    if key.len() != 32 {
        return Err(CryptoError::InvalidKey("invalid length".to_string()));
    }

    // Create the key and nonce
    let key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
        .map_err(|e| CryptoError::InvalidKey(format!("Invalid key: {}", e)))?;

    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| CryptoError::InvalidNonce("invalid format".to_string()))?;

    // Combine ciphertext and tag for decryption
    let mut in_out = ciphertext.to_vec();
    in_out.extend_from_slice(tag);

    // Decrypt the data
    let opening_key = aead::LessSafeKey::new(key);
    let decrypted = opening_key
        .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    Ok(decrypted.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let message = b"Hello, World! This is a test message.";

        // Test with empty data (should fail)
        assert!(
            matches!(encrypt_data(&[]), Err(CryptoError::InvalidInput(_))),
            "Encryption with empty data should fail with InvalidInput"
        );

        // Test with valid data
        let (encrypted, key) = encrypt_data(message).expect("Encryption should succeed");

        // Verify the encrypted data has the correct structure
        assert!(encrypted.len() > NONCE_LEN + AES_256_GCM_TAG_LEN);

        // Decrypt the message
        let decrypted = decrypt_data(&encrypted, &key).expect("Decryption should succeed");

        // Verify the decrypted message matches the original
        assert_eq!(message, decrypted.as_slice());

        // Test with maximum length data
        let large_message = vec![0xAA; 1024 * 1024]; // 1MB of data
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
        let wrong_key = [0u8; 32]; // All zeros key
        let result = decrypt_data(&encrypted, &wrong_key);

        // Should fail with DecryptionFailed error
        assert!(
            matches!(result, Err(CryptoError::DecryptionFailed(_))),
            "Decryption with wrong key should fail with DecryptionFailed"
        );

        // Try with invalid key length
        let invalid_key = [0u8; 16]; // 128-bit key (too short)
        let result = decrypt_data(&encrypted, &invalid_key);

        // Should fail with InvalidKey error
        assert!(
            matches!(result, Err(CryptoError::InvalidKey(_))),
            "Decryption with invalid key length should fail with InvalidKey"
        );
    }

    #[test]
    fn test_decrypt_invalid_data_fails() {
        // Test with empty data
        assert!(
            matches!(
                decrypt_data(&[], &[0u8; 32]),
                Err(CryptoError::InvalidInput(_))
            ),
            "Decryption with empty data should fail with InvalidInput"
        );

        // Test with data that's too short (less than nonce + tag)
        let short_data = [0u8; 20]; // 20 < 12 + 16
        assert!(
            matches!(
                decrypt_data(&short_data, &[0u8; 32]),
                Err(CryptoError::InvalidInput(_))
            ),
            "Decryption with short data should fail with InvalidInput"
        );

        // Test with valid length but invalid format
        let mut invalid_data = vec![0u8; 44]; // 12 (nonce) + 16 (data) + 16 (tag)
        invalid_data[..12].copy_from_slice(&hex!("000000000000000000000000")); // Valid nonce

        // This should fail during decryption
        let result = decrypt_data(&invalid_data, &[0u8; 32]);
        assert!(
            matches!(result, Err(CryptoError::DecryptionFailed(_))),
            "Decryption of invalid data should fail with DecryptionFailed"
        );
    }

    #[test]
    fn test_error_messages() {
        // Test error message formatting
        let key_error = CryptoError::InvalidKey("test".to_string());
        assert_eq!(format!("{}", key_error), "Invalid key: test");

        let nonce_error = CryptoError::InvalidNonce("test".to_string());
        assert_eq!(format!("{}", nonce_error), "Invalid nonce: test");

        let decryption_error = CryptoError::DecryptionFailed("test".to_string());
        assert_eq!(format!("{}", decryption_error), "Decryption failed: test");

        let input_error = CryptoError::InvalidInput("test".to_string());
        assert_eq!(format!("{}", input_error), "Invalid input: test");
    }
}
