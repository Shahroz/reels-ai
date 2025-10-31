//! Provides a function to decrypt data encrypted with AES-256-GCM.
//!
//! This function assumes the input data is a concatenation of the 12-byte
//! nonce followed by the actual ciphertext, as produced by the `encrypt` function.

/// Decrypts the given data using AES-256-GCM.
///
/// It extracts the nonce from the beginning of the `encrypted_data` slice
/// and uses it to decrypt the remaining ciphertext.
///
/// # Arguments
///
/// * `encrypted_data` - The data to decrypt, including the prepended nonce.
/// * `key` - A reference to the `Aes256Gcm` key that was used for encryption.
///
/// # Returns
///
/// A `Result` containing the decrypted plaintext data (`Vec<u8>`) on success,
/// or a `String` error message if decryption fails.
pub fn decrypt(
    encrypted_data: &[u8],
    key: &aes_gcm::Key<aes_gcm::Aes256Gcm>,
) -> std::result::Result<std::vec::Vec<u8>, std::string::String> {
    use aes_gcm::aead::{generic_array::GenericArray, Aead};
    use aes_gcm::KeyInit;

    if encrypted_data.len() < 12 {
        return std::result::Result::Err("Encrypted data is too short to contain a nonce".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = GenericArray::from_slice(nonce_bytes);

    let cipher = aes_gcm::Aes256Gcm::new(key);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {e}"))
} 