use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::vault::crypto::{
    decrypt_with_key, derive_key, encrypt_with_key, generate_salt, KEY_SIZE,
};
use crate::vault::recovery::RecoveryCodes;
use crate::vault::{VaultError, VaultResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Standard,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub added_at: DateTime<Utc>,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct VaultIndex {
    security_level: SecurityLevel,
    entries: HashMap<String, VaultEntry>,
}

impl VaultIndex {
    fn new(security_level: SecurityLevel) -> Self {
        Self {
            security_level,
            entries: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct VaultMeta {
    salt: Vec<u8>,
    security_level: SecurityLevel,
}

pub struct Vault {
    path: PathBuf,
}

impl Vault {
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("stellar")
            .join("vault")
    }

    fn meta_path(&self) -> PathBuf {
        self.path.join("meta.json")
    }

    fn index_path(&self) -> PathBuf {
        self.path.join("index.stlr")
    }

    fn recovery_path(&self) -> PathBuf {
        self.path.join("recovery.stlr")
    }

    fn data_path(&self) -> PathBuf {
        self.path.join("data")
    }

    fn entry_path(&self, id: &str) -> PathBuf {
        self.data_path().join(format!("{}.stlr", id))
    }

    fn ensure_dirs(&self) -> VaultResult<()> {
        fs::create_dir_all(&self.path)?;
        fs::create_dir_all(self.data_path())?;
        Ok(())
    }

    fn read_meta(&self) -> VaultResult<VaultMeta> {
        let data = fs::read(self.meta_path())?;
        serde_json::from_slice(&data).map_err(|e| VaultError::CryptoError(e.to_string()))
    }

    fn write_meta(&self, meta: &VaultMeta) -> VaultResult<()> {
        let data = serde_json::to_vec(meta).map_err(|e| VaultError::CryptoError(e.to_string()))?;
        fs::write(self.meta_path(), data)?;
        Ok(())
    }

    fn derive_master_key(&self, password: &str) -> VaultResult<[u8; KEY_SIZE]> {
        let meta = self.read_meta()?;
        derive_key(password, &meta.salt)
    }

    fn read_index(&self, key: &[u8; KEY_SIZE]) -> VaultResult<VaultIndex> {
        let encrypted = fs::read(self.index_path())?;
        let data = decrypt_with_key(&encrypted, key)?;
        serde_json::from_slice(&data).map_err(|e| VaultError::CryptoError(e.to_string()))
    }

    fn write_index(&self, index: &VaultIndex, key: &[u8; KEY_SIZE]) -> VaultResult<()> {
        let data = serde_json::to_vec(index).map_err(|e| VaultError::CryptoError(e.to_string()))?;
        let encrypted = encrypt_with_key(&data, key)?;
        fs::write(self.index_path(), encrypted)?;
        Ok(())
    }

    fn generate_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..12)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'a' + idx - 10) as char
                }
            })
            .collect()
    }

    pub fn open(path: Option<PathBuf>) -> Self {
        Self {
            path: path.unwrap_or_else(Self::default_path),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.meta_path().exists()
    }

    pub fn init(
        &self,
        password: &str,
        security_level: SecurityLevel,
    ) -> VaultResult<Option<RecoveryCodes>> {
        if self.is_initialized() {
            return Err(VaultError::AlreadyExists("Vault".to_string()));
        }

        self.ensure_dirs()?;

        let salt = generate_salt();
        let key = derive_key(password, &salt)?;

        let meta = VaultMeta {
            salt: salt.to_vec(),
            security_level,
        };
        self.write_meta(&meta)?;

        let index = VaultIndex::new(security_level);
        self.write_index(&index, &key)?;

        if security_level == SecurityLevel::Standard {
            let codes = RecoveryCodes::generate();
            let encrypted_key = codes.encrypt_key(&key)?;
            fs::write(self.recovery_path(), encrypted_key)?;
            Ok(Some(codes))
        } else {
            Ok(None)
        }
    }

    pub fn add(&self, path: &Path, password: &str) -> VaultResult<VaultEntry> {
        if !path.exists() {
            return Err(VaultError::FileNotFound(path.to_path_buf()));
        }

        let key = self.derive_master_key(password)?;
        let mut index = self.read_index(&key)?;

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        if index.entries.values().any(|e| e.name == name) {
            return Err(VaultError::AlreadyExists(name));
        }

        let is_directory = path.is_dir();
        let data = if is_directory {
            self.compress_directory(path)?
        } else {
            fs::read(path)?
        };

        let size = data.len() as u64;
        let id = Self::generate_id();

        let encrypted = encrypt_with_key(&data, &key)?;
        fs::write(self.entry_path(&id), encrypted)?;

        let entry = VaultEntry {
            id: id.clone(),
            name,
            size,
            added_at: Utc::now(),
            is_directory,
        };

        index.entries.insert(id, entry.clone());
        self.write_index(&index, &key)?;

        if is_directory {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }

        Ok(entry)
    }

    pub fn list(&self, password: &str) -> VaultResult<Vec<VaultEntry>> {
        let key = self.derive_master_key(password)?;
        let index = self.read_index(&key)?;
        Ok(index.entries.values().cloned().collect())
    }

    pub fn extract(&self, name: &str, password: &str, dest: &Path) -> VaultResult<PathBuf> {
        let key = self.derive_master_key(password)?;
        let index = self.read_index(&key)?;

        let entry = index
            .entries
            .values()
            .find(|e| e.name == name)
            .ok_or_else(|| VaultError::FileNotFound(PathBuf::from(name)))?;

        let encrypted = fs::read(self.entry_path(&entry.id))?;
        let data = decrypt_with_key(&encrypted, &key)?;

        let output_path = dest.join(&entry.name);

        if entry.is_directory {
            self.extract_directory(&data, &output_path)?;
        } else {
            fs::write(&output_path, data)?;
        }

        Ok(output_path)
    }

    pub fn destroy(&self, name: &str, password: &str) -> VaultResult<()> {
        let key = self.derive_master_key(password)?;
        let mut index = self.read_index(&key)?;

        let id = index
            .entries
            .values()
            .find(|e| e.name == name)
            .map(|e| e.id.clone())
            .ok_or_else(|| VaultError::FileNotFound(PathBuf::from(name)))?;

        let entry_path = self.entry_path(&id);
        if entry_path.exists() {
            fs::remove_file(entry_path)?;
        }

        index.entries.remove(&id);
        self.write_index(&index, &key)?;

        Ok(())
    }

    pub fn recover(
        &self,
        code1: &str,
        code2: &str,
        new_password: &str,
    ) -> VaultResult<RecoveryCodes> {
        let meta = self.read_meta()?;

        if meta.security_level == SecurityLevel::Maximum {
            return Err(VaultError::RecoveryNotAvailable);
        }

        let recovery_path = self.recovery_path();
        if !recovery_path.exists() {
            return Err(VaultError::RecoveryNotAvailable);
        }

        let encrypted_key = fs::read(&recovery_path)?;
        let old_key = RecoveryCodes::decrypt_key(code1, code2, &encrypted_key)?;

        let index = self.read_index(&old_key)?;

        let new_salt = generate_salt();
        let new_key = derive_key(new_password, &new_salt)?;

        let new_meta = VaultMeta {
            salt: new_salt.to_vec(),
            security_level: meta.security_level,
        };
        self.write_meta(&new_meta)?;
        self.write_index(&index, &new_key)?;

        for entry in index.entries.values() {
            let entry_path = self.entry_path(&entry.id);
            let encrypted = fs::read(&entry_path)?;
            let data = decrypt_with_key(&encrypted, &old_key)?;
            let re_encrypted = encrypt_with_key(&data, &new_key)?;
            fs::write(&entry_path, re_encrypted)?;
        }

        let new_codes = RecoveryCodes::generate();
        let encrypted_new_key = new_codes.encrypt_key(&new_key)?;
        fs::write(&recovery_path, encrypted_new_key)?;

        Ok(new_codes)
    }

    fn compress_directory(&self, path: &Path) -> VaultResult<Vec<u8>> {
        use tar::Builder;
        let mut archive = Builder::new(Vec::new());
        archive
            .append_dir_all(".", path)
            .map_err(VaultError::IoError)?;
        archive.into_inner().map_err(VaultError::IoError)
    }

    fn extract_directory(&self, data: &[u8], dest: &Path) -> VaultResult<()> {
        use tar::Archive;
        fs::create_dir_all(dest)?;
        let mut archive = Archive::new(data);
        archive.unpack(dest).map_err(VaultError::IoError)?;
        Ok(())
    }
}
