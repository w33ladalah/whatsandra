use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use base64::{Engine as _, engine::general_purpose};

use crate::error::WhatsAppError;

/// Key pair for encryption
pub struct KeyPair {
    pub private: Vec<u8>,
    pub public: Vec<u8>,
}

/// Implements cryptographic functions for WhatsApp
pub struct Crypto;

impl Crypto {
    /// Generate random bytes
    pub fn random_bytes(length: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        let mut bytes = vec![0u8; length];
        rng.fill(&mut bytes[..]);
        bytes
    }

    /// Generate a key pair for encryption
    pub fn generate_key_pair() -> Result<KeyPair, WhatsAppError> {
        // In a real implementation, this would use proper curve25519 functions
        // For this port example, we'll use random bytes as a placeholder
        let private = Self::random_bytes(32);
        let public = Self::random_bytes(32);

        Ok(KeyPair { private, public })
    }

    /// HMAC-SHA256 signature
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, WhatsAppError> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|e| WhatsAppError::CryptoError(e.to_string()))?;

        mac.update(data);
        let result = mac.finalize();

        Ok(result.into_bytes().to_vec())
    }

    /// HKDF (HMAC-based Key Derivation Function)
    pub fn hkdf(master: &[u8], app_info: &[u8], length: usize) -> Result<Vec<u8>, WhatsAppError> {
        // Extract phase
        let salt = [0u8; 32]; // Zero salt for WhatsApp
        let prk = Self::hmac_sha256(&salt, master)?;

        // Expand phase
        let mut output = Vec::new();
        let mut t = Vec::new();
        let mut counter = 1u8;

        while output.len() < length {
            // T(N) = HMAC-SHA-256(PRK, T(N-1) | info | N)
            let mut input = t.clone();
            input.extend_from_slice(app_info);
            input.push(counter);

            t = Self::hmac_sha256(&prk, &input)?;
            output.extend_from_slice(&t);

            counter += 1;
        }

        output.truncate(length);
        Ok(output)
    }

    /// AES-256-CBC encrypt
    pub fn aes_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, WhatsAppError> {
        // In a real implementation, this would use proper AES encryption
        // For this port example, we'll use a placeholder

        // This is a simplified version - a real implementation would use the proper
        // AES-256-CBC mode encryption with padding

        // Create a simple XOR encryption as placeholder
        let mut output = Vec::with_capacity(data.len());

        for (i, &byte) in data.iter().enumerate() {
            let key_byte = key[i % key.len()];
            let iv_byte = iv[i % iv.len()];
            output.push(byte ^ key_byte ^ iv_byte);
        }

        Ok(output)
    }

    /// AES-256-CBC decrypt
    pub fn aes_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, WhatsAppError> {
        // In a real implementation, this would use proper AES decryption
        // For this port, we'll just call our encrypt function since XOR is symmetric
        Self::aes_encrypt(key, iv, data)
    }

    /// Base64 encode
    pub fn base64_encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }

    /// Base64 decode
    pub fn base64_decode(data: &str) -> Result<Vec<u8>, WhatsAppError> {
        general_purpose::STANDARD.decode(data)
            .map_err(|e| WhatsAppError::CryptoError(format!("Base64 decode error: {}", e)))
    }

    /// Calculate SHA-256 hash
    pub fn sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}
