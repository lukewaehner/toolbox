//! Password Manager Module
//!
//! This module provides secure password storage and management functionality using AES-256 encryption.
//!
//! # Security
//!
//! - Passwords are encrypted using AES-256 in CBC mode
//! - Each encryption operation uses a random initialization vector (IV)
//! - The encryption key is loaded from the `ENCRYPTION_KEY` environment variable
//! - The key must be exactly 32 bytes (256 bits) long
//! - Encrypted data is stored in `passwords.json`
//!
//! # Example
//!
//! ```no_run
//! use password_manager::{PasswordEntry, save_password, retrieve_password};
//!
//! let entry = PasswordEntry {
//!     service: "example.com".to_string(),
//!     username: "user@example.com".to_string(),
//!     password: "secret123".to_string(),
//! };
//!
//! save_password(&entry).expect("Failed to save password");
//! let entries = retrieve_password().expect("Failed to retrieve passwords");
//! ```

use aes::Aes256;
use base64::{engine::general_purpose, Engine as _};
use cbc::{Decryptor, Encryptor};
use cipher::{block_padding::Pkcs7, KeyIvInit};
use cipher::{BlockDecryptMut, BlockEncryptMut};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::{fs, io};
use dotenv::dotenv;
use std::env;

/// Path to the encrypted password storage file
const FILE_PATH: &str = "passwords.json";

/// Represents a single password entry
///
/// Contains the service name, username, and password for a login credential.
#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
    /// The name of the service (e.g., "gmail.com", "GitHub")
    pub service: String,
    /// The username or email address
    pub username: String,
    /// The password for this service
    pub password: String,
}

/// Retrieves all stored password entries
///
/// Decrypts and deserializes all password entries from the encrypted storage file.
///
/// # Returns
///
/// Returns a vector of `PasswordEntry` objects on success.
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - The file cannot be read
/// - Decryption fails (e.g., wrong encryption key)
/// - JSON deserialization fails
pub fn retrieve_password() -> io::Result<Vec<PasswordEntry>> {
    let entries = load_passwords()?;
    Ok(entries)
}

/// Saves a password entry to encrypted storage
///
/// Adds a new password entry to the existing list and saves it to the encrypted file.
///
/// # Arguments
///
/// * `entry` - The password entry to save
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - Loading existing passwords fails
/// - JSON serialization fails
/// - Encryption fails
/// - Writing to the file fails
pub fn save_password(entry: &PasswordEntry) -> io::Result<()> {
    let mut entries = load_passwords().or_else(|e| {
        println!("Error loading existing passwords: {}", e);
        // Explicitly specify the type of Vec::new() as Vec<PasswordEntry>
        Ok::<Vec<PasswordEntry>, io::Error>(Vec::new())
    })?;

    entries.push(entry.clone());
    let json = serde_json::to_string(&entries).map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "Failed to serialize passwords to JSON",
        )
    })?;

    let encrypted = encrypt(json.as_bytes())?;
    fs::write(FILE_PATH, &encrypted)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to write to passwords.json"))?;

    Ok(())
}

/// Loads and decrypts password entries from storage
///
/// Internal function to load the encrypted password file and decrypt its contents.
///
/// # Returns
///
/// Returns a vector of `PasswordEntry` objects, or an empty vector if the file doesn't exist.
///
/// # Errors
///
/// Returns an `io::Error` if decryption or deserialization fails.
fn load_passwords() -> io::Result<Vec<PasswordEntry>> {
    if let Ok(encrypted) = fs::read(FILE_PATH) {
        if encrypted.is_empty() {
            // Treat an empty file as having no passwords
            return Ok(Vec::<PasswordEntry>::new());
        }
        match decrypt(&encrypted) {
            Ok(decrypted) => {
                let entries: Vec<PasswordEntry> =
                    serde_json::from_slice(&decrypted).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "JSON deserialization failed")
                    })?;
                Ok(entries)
            }
            Err(_) => {
                // Handle decryption error
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Failed to decrypt existing passwords. The data might be corrupted.",
                ))
            }
        }
    } else {
        Ok(Vec::<PasswordEntry>::new())
    }
}

/// Encrypts data using AES-256-CBC encryption
///
/// # Arguments
///
/// * `data` - The plaintext data to encrypt
///
/// # Returns
///
/// Returns a vector containing the IV (first 16 bytes) followed by the ciphertext.
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - The encryption key cannot be retrieved
/// - The key is not 32 bytes long
/// - Encryption fails
fn encrypt(data: &[u8]) -> io::Result<Vec<u8>> {
    let key = get_encryption_key().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "Failed to get encryption key")
    })?;
    if key.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Key must be 32 bytes long",
        ));
    }

    let mut iv = [0u8; 16];
    thread_rng().fill_bytes(&mut iv);

    let cipher = Encryptor::<Aes256>::new_from_slices(&key, &iv)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid key or IV length"))?;

    // Prepare output buffer with sufficient capacity
    let mut encrypted = vec![0u8; data.len() + 16]; // Adjust size as needed

    // Encrypt data and apply padding
    let ciphertext = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut encrypted) // Pass `data` directly
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Encryption failed: {:?}", e)))?;

    // Prepend IV to the ciphertext for decryption
    let mut result = iv.to_vec();
    result.extend_from_slice(ciphertext);

    // Encode the result in base64
    let encoded = general_purpose::STANDARD.encode(&result);
    Ok(encoded.into_bytes())
}

fn decrypt(data: &[u8]) -> io::Result<Vec<u8>> {
    let key = get_encryption_key().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "Failed to get encryption key")
    })?;
    // Decode base64
    let decoded = general_purpose::STANDARD.decode(data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Base64 decoding failed: {:?}", e),
        )
    })?;

    if decoded.len() < 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid data length",
        ));
    }

    // Split IV and ciphertext
    let (iv, ciphertext) = decoded.split_at(16);

    let cipher = Decryptor::<Aes256>::new_from_slices(&key, iv).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Cipher creation failed: {:?}", e),
        )
    })?;

    // Prepare buffer for decrypted data
    let mut decrypted = vec![0u8; ciphertext.len()];
    let decrypted_data = cipher
        .decrypt_padded_b2b_mut::<Pkcs7>(ciphertext, &mut decrypted)
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Decryption failed: {:?}", e),
            )
        })?;

    // Capture the decrypted length before mutable borrow
    let decrypted_len = decrypted_data.len();

    // Truncate buffer to actual decrypted data length
    decrypted.truncate(decrypted_len);

    Ok(decrypted)
}

fn get_encryption_key() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from .env file
    let key = env::var("ENCRYPTION_KEY")?;
    if key.len() != 32 {
        return Err("Key must be 32 bytes long".into());
    }
    Ok(key.into_bytes())
}

// Use this function to get the key where needed

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original_data = b"Test data for encryption";

        // Encrypt the data
        let encrypted = match encrypt(original_data) { // Remove the `key` argument
            Ok(data) => data,
            Err(e) => panic!("Encryption failed with error: {}", e),
        };

        // Decrypt the data
        let decrypted = match decrypt(&encrypted) { // Remove the `key` argument
            Ok(data) => data,
            Err(e) => panic!("Decryption failed with error: {}", e),
        };

        assert_eq!(original_data.to_vec(), decrypted);
    }
}
