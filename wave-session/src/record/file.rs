use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    path: PathBuf,
    filename: String,
}

impl FileRef {
    pub fn new(path: &Path, filename: Option<String>) -> Result<Self, Error> {
        let filename = filename
            .or_else(|| path.file_name().map(|f| f.to_string_lossy().to_string()))
            .ok_or(Error::InvalidPath(path.to_path_buf()))?;

        Ok(Self {
            path: path.to_path_buf(),
            filename,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid path: {0:?}. Expected a file path not a directory")]
    InvalidPath(PathBuf),
}
