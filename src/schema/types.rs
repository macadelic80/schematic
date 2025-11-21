use serde::{Deserialize, Serialize};

/// Byte ordering for multi-byte values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Endianness {
    Little,
    Big,
}

impl Default for Endianness {
    fn default() -> Self {
        Self::Little
    }
}

/// Primitive data types supported by the schema system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    // Signed integers
    I8,
    I16,
    I32,
    I64,
    // Floating point
    F32,
    F64,
}

impl DataType {
    /// Get the size of this type in bytes
    pub fn size(&self) -> usize {
        match self {
            DataType::U8 | DataType::I8 => 1,
            DataType::U16 | DataType::I16 => 2,
            DataType::U32 | DataType::I32 | DataType::F32 => 4,
            DataType::U64 | DataType::I64 | DataType::F64 => 8,
        }
    }

    /// Get the name of this type as a string
    pub fn name(&self) -> &'static str {
        match self {
            DataType::U8 => "u8",
            DataType::U16 => "u16",
            DataType::U32 => "u32",
            DataType::U64 => "u64",
            DataType::I8 => "i8",
            DataType::I16 => "i16",
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
        }
    }

    /// Read a value of this type from bytes at the given offset
    pub fn read_value(&self, data: &[u8], offset: usize, endianness: Endianness) -> Option<String> {
        if offset + self.size() > data.len() {
            return None;
        }

        let bytes = &data[offset..offset + self.size()];

        Some(match self {
            DataType::U8 => bytes[0].to_string(),
            DataType::I8 => (bytes[0] as i8).to_string(),

            DataType::U16 => {
                let value = match endianness {
                    Endianness::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
                    Endianness::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
                };
                value.to_string()
            }
            DataType::I16 => {
                let value = match endianness {
                    Endianness::Little => i16::from_le_bytes([bytes[0], bytes[1]]),
                    Endianness::Big => i16::from_be_bytes([bytes[0], bytes[1]]),
                };
                value.to_string()
            }

            DataType::U32 => {
                let value = match endianness {
                    Endianness::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    Endianness::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                value.to_string()
            }
            DataType::I32 => {
                let value = match endianness {
                    Endianness::Little => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    Endianness::Big => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                value.to_string()
            }

            DataType::U64 => {
                let value = match endianness {
                    Endianness::Little => u64::from_le_bytes(bytes.try_into().unwrap()),
                    Endianness::Big => u64::from_be_bytes(bytes.try_into().unwrap()),
                };
                value.to_string()
            }
            DataType::I64 => {
                let value = match endianness {
                    Endianness::Little => i64::from_le_bytes(bytes.try_into().unwrap()),
                    Endianness::Big => i64::from_be_bytes(bytes.try_into().unwrap()),
                };
                value.to_string()
            }

            DataType::F32 => {
                let value = match endianness {
                    Endianness::Little => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    Endianness::Big => f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                value.to_string()
            }
            DataType::F64 => {
                let value = match endianness {
                    Endianness::Little => f64::from_le_bytes(bytes.try_into().unwrap()),
                    Endianness::Big => f64::from_be_bytes(bytes.try_into().unwrap()),
                };
                value.to_string()
            }
        })
    }

    /// Get all available data types
    pub fn all() -> &'static [DataType] {
        &[
            DataType::U8,
            DataType::U16,
            DataType::U32,
            DataType::U64,
            DataType::I8,
            DataType::I16,
            DataType::I32,
            DataType::I64,
            DataType::F32,
            DataType::F64,
        ]
    }
}
