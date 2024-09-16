use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileRef {
    path: PathBuf,
    filename: String,
}

impl FileRef {
    pub fn new(path: &Path, filename: String) -> Self {
        Self {
            path: path.to_path_buf(),
            filename,
        }
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
