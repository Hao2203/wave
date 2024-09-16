use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileRef {
    path: Path,
    filename: String,
}

impl FileRef {
    pub fn new(path: &Path) -> Option<Self> {
        path.file_name().map(|filename| Self {
            path: path.clone(),
            filename: filename.to_string_lossy().to_string(),
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn filename(&self) -> &str {
        &self.filename
    }
}
