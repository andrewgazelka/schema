// Re-export derive macro
use std::collections::HashMap;

pub use schema_derive::Schema;

/// Core schema representation for types (not values)
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaType {
    pub kind: TypeKind,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    String,
    Integer(IntegerKind),
    Number(NumberKind),
    Boolean,
    Null,
    Object {
        properties: HashMap<String, SchemaType>,
        required: Vec<String>,
    },
    Array {
        items: Box<SchemaType>,
    },
    Enum {
        variants: Vec<String>,
    },
    TaggedUnion {
        tag_field: String,
        tag_variants: Vec<String>,
        data_fields: HashMap<String, SchemaType>,
    },
    #[allow(dead_code)]
    Ref {
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerKind {
    I32,
    I64,
    U8,
    U32,
    U64,
    Usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    F32,
    F64,
}

/// Trait for types that can be represented as schemas
pub trait Schema {
    /// Get the schema for this type
    fn schema() -> SchemaType;

    /// Optional: Get the type name for references
    fn type_name() -> Option<&'static str> {
        None
    }
}

// Implement for primitive types
impl Schema for String {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::String,
            description: None,
        }
    }
}

impl Schema for i32 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Integer(IntegerKind::I32),
            description: None,
        }
    }
}

impl Schema for i64 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Integer(IntegerKind::I64),
            description: None,
        }
    }
}

impl Schema for u8 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Integer(IntegerKind::U8),
            description: None,
        }
    }
}

impl Schema for u32 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Integer(IntegerKind::U32),
            description: None,
        }
    }
}

impl Schema for u64 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Integer(IntegerKind::U64),
            description: None,
        }
    }
}

impl Schema for usize {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Integer(IntegerKind::Usize),
            description: None,
        }
    }
}

impl Schema for f32 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Number(NumberKind::F32),
            description: None,
        }
    }
}

impl Schema for f64 {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Number(NumberKind::F64),
            description: None,
        }
    }
}

impl Schema for bool {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Boolean,
            description: None,
        }
    }
}

impl Schema for () {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Object {
                properties: HashMap::new(),
                required: Vec::new(),
            },
            description: None,
        }
    }
}

impl Schema for std::path::PathBuf {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::String,
            description: Some("File system path".to_string()),
        }
    }
}

impl Schema for serde_json::Value {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Object {
                properties: HashMap::new(),
                required: Vec::new(),
            },
            description: Some("Dynamic JSON value".to_string()),
        }
    }
}

impl<T: Schema> Schema for Option<T> {
    fn schema() -> SchemaType {
        T::schema()
    }
}

impl<T: Schema> Schema for Vec<T> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Array {
                items: Box::new(T::schema()),
            },
            description: None,
        }
    }
}
