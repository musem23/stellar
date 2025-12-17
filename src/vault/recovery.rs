use rand::Rng;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::vault::crypto::{decrypt_with_key, encrypt_with_key, KEY_SIZE};
use crate::vault::{VaultError, VaultResult};

const CODE_SEGMENT_LENGTH: usize = 4;
const CODE_SEGMENTS: usize = 3;
const CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct RecoveryCodes {
    pub code1: String,
    pub code2: String,
}

impl RecoveryCodes {
    pub fn generate() -> Self {
        Self {
            code1: generate_code(),
            code2: generate_code(),
        }
    }

    pub fn encrypt_key(&self, key: &[u8; KEY_SIZE]) -> VaultResult<Vec<u8>> {
        let combined = self.combined_key();
        encrypt_with_key(key, &combined)
    }

    pub fn decrypt_key(code1: &str, code2: &str, encrypted: &[u8]) -> VaultResult<[u8; KEY_SIZE]> {
        let combined = combine_codes(code1, code2);
        let decrypted =
            decrypt_with_key(encrypted, &combined).map_err(|_| VaultError::InvalidRecoveryCode)?;

        decrypted.try_into().map_err(|_| VaultError::CorruptedData)
    }

    fn combined_key(&self) -> [u8; KEY_SIZE] {
        combine_codes(&self.code1, &self.code2)
    }
}

fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    (0..CODE_SEGMENTS)
        .map(|_| {
            (0..CODE_SEGMENT_LENGTH)
                .map(|_| CODE_CHARS[rng.gen_range(0..CODE_CHARS.len())] as char)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn normalize(code: &str) -> String {
    code.to_uppercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn combine_codes(code1: &str, code2: &str) -> [u8; KEY_SIZE] {
    use sha2::{Digest, Sha256};
    let combined = format!("{}{}", normalize(code1), normalize(code2));
    let hash = Sha256::digest(combined.as_bytes());
    hash.into()
}
