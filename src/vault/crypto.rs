use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::Zeroize;

use crate::vault::{VaultError, VaultResult};

pub const SALT_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;
pub const KEY_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const ARGON2_M_COST: u32 = 65536;
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

pub fn derive_key(password: &str, salt: &[u8]) -> VaultResult<[u8; KEY_SIZE]> {
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_SIZE))
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_SIZE];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    Ok(key)
}

pub fn encrypt_with_key(data: &[u8], key: &[u8; KEY_SIZE]) -> VaultResult<Vec<u8>> {
    let nonce = generate_nonce();
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), data)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_with_key(encrypted: &[u8], key: &[u8; KEY_SIZE]) -> VaultResult<Vec<u8>> {
    if encrypted.len() < NONCE_SIZE + TAG_SIZE {
        return Err(VaultError::CorruptedData);
    }

    let nonce: [u8; NONCE_SIZE] = encrypted[..NONCE_SIZE]
        .try_into()
        .map_err(|_| VaultError::CorruptedData)?;
    let ciphertext = &encrypted[NONCE_SIZE..];

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| VaultError::CryptoError(e.to_string()))?;

    cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext)
        .map_err(|_| VaultError::InvalidPassword)
}

pub fn encrypt(data: &[u8], password: &str) -> VaultResult<Vec<u8>> {
    let salt = generate_salt();
    let mut key = derive_key(password, &salt)?;
    let encrypted = encrypt_with_key(data, &key)?;
    key.zeroize();

    let mut output = Vec::with_capacity(SALT_SIZE + encrypted.len());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&encrypted);
    Ok(output)
}

pub fn decrypt(encrypted: &[u8], password: &str) -> VaultResult<Vec<u8>> {
    if encrypted.len() < SALT_SIZE + NONCE_SIZE + TAG_SIZE {
        return Err(VaultError::CorruptedData);
    }

    let salt = &encrypted[..SALT_SIZE];
    let ciphertext = &encrypted[SALT_SIZE..];

    let mut key = derive_key(password, salt)?;
    let plaintext = decrypt_with_key(ciphertext, &key)?;
    key.zeroize();

    Ok(plaintext)
}
