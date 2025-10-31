//! This module provides AES-256-GCM encryption and decryption services.
//!
//! It encapsulates the logic for securely handling data by encrypting it at rest
//! and decrypting it for use, using a key loaded from the environment.
 
pub mod decrypt;
pub mod encrypt;
pub mod key; 