pub mod commands;
pub mod crypto;
pub mod locker;
pub mod recovery;
pub mod storage;

use std::io;
use std::path::PathBuf;

pub use locker::{lock_file, unlock_file};
pub use storage::Vault;

#[derive(Debug)]
pub enum VaultError {
    InvalidPassword,
    FileNotFound(PathBuf),
    AlreadyExists(String),
    CorruptedData,
    IoError(io::Error),
    CryptoError(String),
    RecoveryNotAvailable,
    InvalidRecoveryCode,
    WeakPassword(String),
    NotVaultFile(PathBuf),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::InvalidPassword => write!(f, "Invalid password"),
            VaultError::FileNotFound(p) => write!(f, "File not found: {}", p.display()),
            VaultError::AlreadyExists(n) => write!(f, "Already exists: {}", n),
            VaultError::CorruptedData => write!(f, "Data corrupted or tampered"),
            VaultError::IoError(e) => write!(f, "IO error: {}", e),
            VaultError::CryptoError(e) => write!(f, "Crypto error: {}", e),
            VaultError::RecoveryNotAvailable => write!(f, "Recovery not available"),
            VaultError::InvalidRecoveryCode => write!(f, "Invalid recovery code"),
            VaultError::WeakPassword(msg) => write!(f, "Password too weak: {}", msg),
            VaultError::NotVaultFile(p) => write!(f, "Not a .stlr file: {}", p.display()),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<io::Error> for VaultError {
    fn from(err: io::Error) -> Self {
        VaultError::IoError(err)
    }
}

pub type VaultResult<T> = Result<T, VaultError>;

pub const MIN_PASSWORD_LENGTH: usize = 12;

/// Validate password strength for government-level security
/// Requirements:
/// - Minimum 12 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
pub fn validate_password(password: &str) -> VaultResult<()> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(VaultError::WeakPassword(format!(
            "Minimum {} characters required",
            MIN_PASSWORD_LENGTH
        )));
    }

    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_upper {
        return Err(VaultError::WeakPassword(
            "Must contain at least one uppercase letter".to_string(),
        ));
    }

    if !has_lower {
        return Err(VaultError::WeakPassword(
            "Must contain at least one lowercase letter".to_string(),
        ));
    }

    if !has_digit {
        return Err(VaultError::WeakPassword(
            "Must contain at least one digit".to_string(),
        ));
    }

    if !has_special {
        return Err(VaultError::WeakPassword(
            "Must contain at least one special character".to_string(),
        ));
    }

    // Check for common weak patterns
    let lower = password.to_lowercase();
    const WEAK_PATTERNS: &[&str] = &[
        "password", "123456", "qwerty", "admin", "letmein", "welcome",
        "monkey", "dragon", "master", "111111", "abc123", "654321",
    ];

    for pattern in WEAK_PATTERNS {
        if lower.contains(pattern) {
            return Err(VaultError::WeakPassword(
                "Password contains a common weak pattern".to_string(),
            ));
        }
    }

    Ok(())
}
