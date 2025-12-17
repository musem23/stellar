use std::fs;
use std::path::{Path, PathBuf};

use crate::vault::crypto::{decrypt, encrypt};
use crate::vault::{VaultError, VaultResult};

const VAULT_EXTENSION: &str = "stlr";

fn get_vault_path(path: &Path) -> PathBuf {
    let mut vault_path = path.to_path_buf();
    let new_extension = match path.extension() {
        Some(ext) => format!("{}.{}", ext.to_string_lossy(), VAULT_EXTENSION),
        None => VAULT_EXTENSION.to_string(),
    };
    vault_path.set_extension(new_extension);
    vault_path
}

fn get_original_path(vault_path: &Path) -> VaultResult<PathBuf> {
    let path_str = vault_path.to_string_lossy();
    let suffix = format!(".{}", VAULT_EXTENSION);

    if !path_str.ends_with(&suffix) {
        return Err(VaultError::NotVaultFile(vault_path.to_path_buf()));
    }

    Ok(PathBuf::from(&path_str[..path_str.len() - suffix.len()]))
}

pub fn lock_file(path: &Path, password: &str, keep_original: bool) -> VaultResult<PathBuf> {
    if !path.exists() {
        return Err(VaultError::FileNotFound(path.to_path_buf()));
    }

    if path
        .extension()
        .map(|e| e == VAULT_EXTENSION)
        .unwrap_or(false)
    {
        return Err(VaultError::AlreadyExists(path.display().to_string()));
    }

    let data = fs::read(path)?;
    let encrypted = encrypt(&data, password)?;
    let vault_path = get_vault_path(path);

    fs::write(&vault_path, encrypted)?;

    if !keep_original {
        fs::remove_file(path)?;
    }

    Ok(vault_path)
}

pub fn unlock_file(vault_path: &Path, password: &str) -> VaultResult<PathBuf> {
    if !vault_path.exists() {
        return Err(VaultError::FileNotFound(vault_path.to_path_buf()));
    }

    let original_path = get_original_path(vault_path)?;
    let encrypted = fs::read(vault_path)?;
    let data = decrypt(&encrypted, password)?;

    fs::write(&original_path, data)?;
    fs::remove_file(vault_path)?;

    Ok(original_path)
}
