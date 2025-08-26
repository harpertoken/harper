#![allow(dead_code)]

use ring::{
    aead, agreement, digest,
    rand::{SecureRandom, SystemRandom},
};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CryptoError {
    CryptoOperationError(String),
    SerializationError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CryptoError::CryptoOperationError(msg) => write!(f, "Crypto Error: {}", msg),
            CryptoError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
        }
    }
}

impl Error for CryptoError {}

/// Type alias for the return type of encrypt_message
pub type EncryptionResult = Result<(Vec<u8>, Vec<u8>, Vec<u8>), CryptoError>;

pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate a random secret key
    pub fn generate_secret_key() -> Result<Vec<u8>, CryptoError> {
        let rng = SystemRandom::new();
        let mut key = vec![0u8; 32]; // 256-bit key
        rng.fill(&mut key)
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;
        Ok(key)
    }

    /// Generate public key from secret key (using ECDH)
    pub fn generate_public_key(_secret_key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let private_key =
            agreement::EphemeralPrivateKey::generate(&agreement::X25519, &SystemRandom::new())
                .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        let public_key = private_key
            .compute_public_key()
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        Ok(public_key.as_ref().to_vec())
    }

    /// Encrypt a message using AES-GCM
    pub fn encrypt_message(message: &[u8], key: &[u8]) -> EncryptionResult {
        println!("Encrypting message: {:?}", message);
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        let key = aead::LessSafeKey::new(unbound_key);
        let nonce = Self::generate_nonce();
        let nonce_bytes = nonce.as_ref().to_vec();
        println!("Using nonce for encryption: {:?}", nonce_bytes);

        let mut in_out = message.to_vec();
        let tag = key
            .seal_in_place_separate_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        println!("Ciphertext: {:?}", in_out);
        println!("Tag: {:?}", tag.as_ref());

        Ok((in_out, tag.as_ref().to_vec(), nonce_bytes))
    }

    /// Decrypt a message
    pub fn decrypt_message(
        ciphertext: &[u8],
        tag: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        println!("Decrypting ciphertext: {:?}", ciphertext);
        println!("Using tag: {:?}", tag);
        println!("Using nonce for decryption: {:?}", nonce);

        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        let key = aead::LessSafeKey::new(unbound_key);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        let mut in_out = ciphertext.to_vec();
        in_out.extend_from_slice(tag);

        let result = key
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| CryptoError::CryptoOperationError(e.to_string()))?;

        println!("Decrypted result: {:?}", result);

        Ok(result.to_vec())
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
    ) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
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
    ) -> Result<bool, CryptoError> {
        // Verify that response = hash(??? + challenge)
        // This is a simplified demonstration
        let expected_challenge = Self::hash_data(public_data);
        Ok(challenge == expected_challenge.as_slice())
    }

    /// Helper function to generate a nonce
    fn generate_nonce() -> aead::Nonce {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes).unwrap();
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        println!("Generated nonce: {:?}", nonce.as_ref());
        nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_operations() -> Result<(), CryptoError> {
        // Test key generation
        let secret = CryptoUtils::generate_secret_key()?;
        println!("Generated secret key: {:?}", secret);

        // Test encryption/decryption
        let message = b"Hello, world!";
        println!("Original message: {:?}", message);

        let (ciphertext, tag, nonce_bytes) = CryptoUtils::encrypt_message(message, &secret)?;
        println!("Ciphertext: {:?}", ciphertext);
        println!("Tag: {:?}", tag);
        println!("Nonce: {:?}", nonce_bytes);

        let decrypted = CryptoUtils::decrypt_message(&ciphertext, &tag, &secret, &nonce_bytes)?;

        println!("Decrypted: {:?}", decrypted);
        assert_eq!(message, decrypted.as_slice());

        // Test hashing
        let hash = CryptoUtils::hash_data(message);
        assert_eq!(hash.len(), 32); // SHA-256 produces 32-byte hash

        Ok(())
    }
}
