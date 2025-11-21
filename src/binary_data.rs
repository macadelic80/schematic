use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

/// Represents a loaded binary file with its data and metadata
#[derive(Default)]
pub struct BinaryData {
    /// The raw bytes of the file
    data: Vec<u8>,
    /// Path to the loaded file
    file_path: Option<PathBuf>,
    /// Whether the data has been modified
    modified: bool,
}

impl BinaryData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a binary file from the given path
    pub fn load_from_file(&mut self, path: PathBuf) -> io::Result<()> {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        self.data = data;
        self.file_path = Some(path);
        self.modified = false;

        Ok(())
    }

    /// Get a reference to the raw bytes
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the file path if a file is loaded
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Check if the file is loaded
    pub fn is_loaded(&self) -> bool {
        !self.data.is_empty()
    }

    /// Get the size of the loaded data
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if the data has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Clear the loaded data
    pub fn clear(&mut self) {
        self.data.clear();
        self.file_path = None;
        self.modified = false;
    }
}
