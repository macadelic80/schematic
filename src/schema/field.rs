use super::types::{DataType, Endianness};
use serde::{Deserialize, Serialize};

/// Represents a field in a binary schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Name of the field
    pub name: String,
    /// Offset in bytes from the start of the file
    pub offset: usize,
    /// Data type of the field
    pub data_type: DataType,
    /// Optional comment/description
    pub comment: String,
    /// Endianness for this field
    pub endianness: Endianness,
}

impl Field {
    pub fn new(name: String, offset: usize, data_type: DataType) -> Self {
        Self {
            name,
            offset,
            data_type,
            comment: String::new(),
            endianness: Endianness::default(),
        }
    }

    /// Get the size of this field in bytes
    pub fn size(&self) -> usize {
        self.data_type.size()
    }

    /// Read the value of this field from the given binary data
    pub fn read_value(&self, data: &[u8]) -> Option<String> {
        self.data_type.read_value(data, self.offset, self.endianness)
    }
}
