use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = filename.to_string();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid path: {0:?}. Expected a file path not a directory")]
    InvalidPath(PathBuf),
}
