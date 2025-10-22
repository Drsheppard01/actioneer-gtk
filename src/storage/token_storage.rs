// Allow dead code for now - these are stubs that will be implemented
#![allow(dead_code)]

use keyring::Entry;
use thiserror::Error;
use tracing::{debug, info, warn};

const SERVICE_NAME: &str = "me.spaceinbox.actioneer";
const TOKEN_KEY: &str = "github_token";

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Token not found")]
    TokenNotFound,

    #[error("Secure keyring storage is not available")]
    KeyringUnavailable,
}

pub struct TokenStorage {
    entry: Entry,
}

impl TokenStorage {
    pub fn new() -> Result<Self, StorageError> {
        info!(
            "Creating TokenStorage with service: {}, key: {}",
            SERVICE_NAME, TOKEN_KEY
        );
        let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;

        // Check if there's an existing token first - don't disturb it!
        let has_existing_token = match entry.get_password() {
            Ok(_) => true,
            Err(keyring::Error::NoEntry) => false,
            Err(err) => {
                warn!("Failed to check existing token: {:?}", err);
                false
            }
        };

        if has_existing_token {
            info!("✅ Found existing token in keyring, skipping test");
            return Ok(Self { entry });
        }

        // Test if keyring is actually working using a DIFFERENT key (to avoid deleting real tokens)
        let test_key = "github_token_test";
        let test_entry = Entry::new(SERVICE_NAME, test_key)?;
        let test_value = format!(
            "test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        info!("No existing token, testing keyring with separate test key");

        let keyring_works = match test_entry.set_password(&test_value) {
            Ok(()) => {
                info!("Test write successful, attempting read...");
                // Try to read it back immediately
                match test_entry.get_password() {
                    Ok(retrieved) => {
                        let matches = retrieved == test_value;
                        info!("Test read successful, matches: {}", matches);
                        // Clean up test value (this won't affect the real token key)
                        let _ = test_entry.delete_credential();
                        matches
                    }
                    Err(e) => {
                        warn!("Test read failed: {:?}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("Test write failed: {:?}", e);
                false
            }
        };

        if !keyring_works {
            warn!("❌ Keyring test failed; secure storage unavailable");
            return Err(StorageError::KeyringUnavailable);
        }

        info!("✅ Keyring is available and working");
        info!("TokenStorage initialized");
        Ok(Self { entry })
    }

    /// Save the token to secure storage
    pub fn save_token(&self, token: &str) -> Result<(), StorageError> {
        // Use keyring
        info!(
            "Saving token to keyring storage (service: {}, key: {})",
            SERVICE_NAME, TOKEN_KEY
        );
        self.entry.set_password(token)?;
        info!("Token saved to keyring - verifying...");

        // Immediately verify we can read it back
        match self.entry.get_password() {
            Ok(retrieved) => {
                if retrieved == token {
                    info!("✅ Token verified in keyring - save successful");
                } else {
                    warn!("⚠️  Token mismatch after save!");
                }
            }
            Err(e) => {
                warn!("⚠️  Could not verify token after save: {}", e);
            }
        }

        Ok(())
    }

    /// Retrieve the token from secure storage
    pub fn get_token(&self) -> Result<String, StorageError> {
        // Use keyring
        info!(
            "Retrieving token from keyring storage (service: {}, key: {})",
            SERVICE_NAME, TOKEN_KEY
        );
        match self.entry.get_password() {
            Ok(token) => {
                info!(
                    "✅ Token retrieved from keyring successfully (length: {} chars)",
                    token.len()
                );
                Ok(token)
            }
            Err(keyring::Error::NoEntry) => {
                info!("❌ No token found in keyring storage");
                Err(StorageError::TokenNotFound)
            }
            Err(e) => {
                info!("❌ Keyring error: {:?}", e);
                Err(StorageError::KeyringError(e))
            }
        }
    }

    /// Delete the token from secure storage
    pub fn delete_token(&self) -> Result<(), StorageError> {
        info!("Deleting token from secure storage");
        // Delete from keyring
        match self.entry.delete_credential() {
            Ok(()) => {
                debug!("Token deleted from keyring successfully");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                debug!("No token in keyring to delete");
                Ok(())
            }
            Err(e) => Err(StorageError::KeyringError(e)),
        }
    }

    /// Check if a token exists
    pub fn has_token(&self) -> bool {
        self.entry.get_password().is_ok()
    }
}

impl Default for TokenStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create TokenStorage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_storage_lifecycle() {
        let storage = match TokenStorage::new() {
            Ok(storage) => storage,
            Err(StorageError::KeyringUnavailable) => {
                eprintln!("Skipping token storage lifecycle test: keyring unavailable");
                return;
            }
            Err(err) => panic!("Failed to create TokenStorage: {}", err),
        };

        // Clean up any existing token
        let _ = storage.delete_token();

        // Initially should have no token
        assert!(!storage.has_token());

        // Save a token
        let test_token = "ghp_test_token_123";
        storage.save_token(test_token).unwrap();

        // Should now have a token
        assert!(storage.has_token());

        // Retrieve should return the same token
        let retrieved = storage.get_token().unwrap();
        assert_eq!(retrieved, test_token);

        // Delete the token
        storage.delete_token().unwrap();

        // Should no longer have a token
        assert!(!storage.has_token());
    }
}
