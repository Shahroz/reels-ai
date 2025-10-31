//! Provides a function to encrypt data using AES-256-GCM.
//!
//! This function takes plaintext data and an encryption key, and returns
//! the ciphertext prepended with the nonce used for encryption. This ensures
//! that the nonce is available for the decryption process.

/// Encrypts the given data using AES-256-GCM.
///
/// A random 12-byte nonce is generated for each encryption operation. The resulting
/// ciphertext is a concatenation of the nonce and the encrypted data.
///
/// # Arguments
///
/// * `data` - The plaintext data to encrypt.
/// * `key` - A reference to the `Aes256Gcm` key.
///
/// # Returns
///
/// A `Result` containing the encrypted data (`Vec<u8>`) on success, or a `String`
/// error message if encryption fails.
pub fn encrypt(
    data: &[u8],
    key: &aes_gcm::Key<aes_gcm::Aes256Gcm>,
) -> std::result::Result<std::vec::Vec<u8>, std::string::String> {
    use aes_gcm::aead::{generic_array::GenericArray, Aead};
    use aes_gcm::KeyInit;
    use rand::RngCore;

    let cipher = aes_gcm::Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    let mut result = std::vec::Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    std::result::Result::Ok(result)
} 