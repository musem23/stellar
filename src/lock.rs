// Stellar - Folder Lock Module
// @musem23
//
// Prevents multiple Stellar instances from operating on the same folder.
// Creates a .stellar.lock file with an exclusive lock.
// Lock is automatically released when FolderLock is dropped.

use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::path::PathBuf;

pub struct FolderLock {
    _file: File,
    path: PathBuf,
}

impl FolderLock {
    /// Try to acquire an exclusive lock on a folder
    pub fn acquire(folder_path: &str) -> Result<Self, String> {
        let path = PathBuf::from(folder_path).join(".stellar.lock");

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("Cannot create lock file: {}", e))?;

        file.try_lock_exclusive().map_err(|e| {
            if e.kind() == ErrorKind::WouldBlock {
                "Another Stellar instance is already operating on this folder.".to_string()
            } else {
                format!("Failed to acquire lock: {}", e)
            }
        })?;

        Ok(FolderLock { _file: file, path })
    }
}

impl Drop for FolderLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
