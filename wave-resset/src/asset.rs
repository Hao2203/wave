use crate::file::FileRef;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assets {
    name: String,
    file_list: HashMap<PathBuf, FileRef>,
}

impl Assets {
    pub fn new(name: String) -> Self {
        let file_list = HashMap::new();
        Self { name, file_list }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn push_file(&mut self, file: FileRef) {
        self.file_list.insert(file.path().to_path_buf(), file);
    }

    pub fn remove_file(&mut self, path: &PathBuf) {
        self.file_list.remove(path);
    }

    pub fn file_iter(&self) -> impl Iterator<Item = &FileRef> {
        self.file_list.values()
    }

    /// Validate and return the list of files to be removed.
    /// return an empty vector if all files are valid.
    pub fn validate(&mut self) -> Vec<PathBuf> {
        let path_to_remove: Vec<_> = self.file_list.keys().cloned().collect();

        for path in &path_to_remove {
            self.remove_file(path);
        }

        path_to_remove
    }
}
