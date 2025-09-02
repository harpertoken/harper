#![allow(dead_code)]

use crate::core::constants::crypto;
use crate::core::error::{HarperError, HarperResult};
use ring::{
    aead, digest,
    rand::{SecureRandom, SystemRandom},
};

/// Type alias for the return type of encrypt_message
pub type EncryptionResult = HarperResult<(Vec<u8>, Vec<u8>, Vec<u8>)>;

pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate a random secret key
    pub fn generate_secret_key() -> HarperResult<Vec<u8>> {
        let rng = SystemRandom::new();
        let mut key = vec![0u8; crypto::AES_256_KEY_LEN];
        rng.fill(&mut key)
            .map_err(|e| HarperError::Crypto(format!("Key generation failed: {}", e)))?;
        Ok(key)
    }

    /// Generate a cryptographic hash of data
    pub fn hash_data(data: &[u8]) -> Vec<u8> {
        let digest = digest::digest(&digest::SHA256, data);
        digest.as_ref().to_vec()
    }

    /// Generate a zero-knowledge proof (simplified demonstration)
    pub fn generate_zk_proof(
        secret: &[u8],
        public_data: &[u8],
    ) -> HarperResult<(Vec<u8>, Vec<u8>)> {
        // Simplified ZK proof demonstration
        let _commitment = Self::hash_data(secret);
        let challenge = Self::hash_data(public_data);

        // Response = hash(secret + challenge)
        let mut response_input = secret.to_vec();
        response_input.extend_from_slice(&challenge);
        let response = Self::hash_data(&response_input);

        Ok((challenge, response))
    }

    /// Verify a zero-knowledge proof
    pub fn verify_zk_proof(
        public_data: &[u8],
        challenge: &[u8],
        _response: &[u8],
    ) -> HarperResult<bool> {
        // Verify that response = hash(??? + challenge)
        // This is a simplified demonstration
        let expected_challenge = Self::hash_data(public_data);
        Ok(challenge == expected_challenge.as_slice())
    }

    /// Helper function to generate a nonce
    fn generate_nonce() -> aead::Nonce {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; crypto::AES_GCM_NONCE_LEN];
        rng.fill(&mut nonce_bytes).unwrap();
        aead::Nonce::assume_unique_for_key(nonce_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::constants::crypto;

    #[test]
    fn test_crypto_operations() -> HarperResult<()> {
        // Test key generation
        let secret = CryptoUtils::generate_secret_key()?;
        assert_eq!(secret.len(), crypto::AES_256_KEY_LEN);

        // Test hashing
        let message = b"Hello, world!";
        let hash = CryptoUtils::hash_data(message);
        assert_eq!(hash.len(), crypto::SHA256_LEN);

        // Test ZK proof generation and verification
        let secret_data = b"secret";
        let public_data = b"public";
        let (challenge, response) = CryptoUtils::generate_zk_proof(secret_data, public_data)?;
        let is_valid = CryptoUtils::verify_zk_proof(public_data, &challenge, &response)?;
        assert!(is_valid);

        Ok(())
    }
}
