pub mod types;
pub mod field;

pub use types::{DataType, Endianness};
pub use field::Field;

use serde::{Deserialize, Serialize};

/// A complete schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<Field>,
}
