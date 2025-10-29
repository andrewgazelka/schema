// Re-export derive macro
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};

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
    /// Set type with unique items
    Set {
        items: Box<SchemaType>,
        ordered: bool,
    },
    /// Map/dictionary type with key-value pairs
    Map {
        key: Box<SchemaType>,
        value: Box<SchemaType>,
        ordered: bool,
    },
    Enum {
        variants: Vec<String>,
    },
    /// Legacy flattened representation for backward compatibility
    TaggedUnion {
        tag_field: String,
        tag_variants: Vec<String>,
        data_fields: HashMap<String, SchemaType>,
    },
    /// Proper variant type that preserves per-case structure (for WIT/WASM)
    Variant {
        cases: Vec<VariantCase>,
    },
    /// Result type (for WIT/WASM)
    Result {
        ok: Box<SchemaType>,
        err: Box<SchemaType>,
    },
    /// Tuple type (for WIT/WASM)
    Tuple {
        fields: Vec<SchemaType>,
    },
    #[allow(dead_code)]
    Ref {
        name: String,
    },
}

/// A single case in a variant type
#[derive(Debug, Clone, PartialEq)]
pub struct VariantCase {
    pub name: String,
    pub data: Option<SchemaType>,
    pub description: Option<String>,
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

impl<K: Schema, V: Schema> Schema for HashMap<K, V> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Map {
                key: Box::new(K::schema()),
                value: Box::new(V::schema()),
                ordered: false,
            },
            description: Some("Unordered map/dictionary of key-value pairs".to_string()),
        }
    }
}

impl<T: Schema> Schema for HashSet<T> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Set {
                items: Box::new(T::schema()),
                ordered: false,
            },
            description: Some("Unordered set of unique values".to_string()),
        }
    }
}

impl<K: Schema, V: Schema> Schema for BTreeMap<K, V> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Map {
                key: Box::new(K::schema()),
                value: Box::new(V::schema()),
                ordered: true,
            },
            description: Some("Ordered map/dictionary of key-value pairs".to_string()),
        }
    }
}

impl<T: Schema> Schema for BTreeSet<T> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Set {
                items: Box::new(T::schema()),
                ordered: true,
            },
            description: Some("Ordered set of unique values".to_string()),
        }
    }
}

impl<T: Schema> Schema for LinkedList<T> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Array {
                items: Box::new(T::schema()),
            },
            description: Some("Doubly-linked list".to_string()),
        }
    }
}

impl<T: Schema, E: Schema> Schema for Result<T, E> {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Result {
                ok: Box::new(T::schema()),
                err: Box::new(E::schema()),
            },
            description: None,
        }
    }
}

// Tuple implementations for common sizes
impl<T1: Schema> Schema for (T1,) {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Tuple {
                fields: vec![T1::schema()],
            },
            description: None,
        }
    }
}

impl<T1: Schema, T2: Schema> Schema for (T1, T2) {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Tuple {
                fields: vec![T1::schema(), T2::schema()],
            },
            description: None,
        }
    }
}

impl<T1: Schema, T2: Schema, T3: Schema> Schema for (T1, T2, T3) {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Tuple {
                fields: vec![T1::schema(), T2::schema(), T3::schema()],
            },
            description: None,
        }
    }
}

impl<T1: Schema, T2: Schema, T3: Schema, T4: Schema> Schema for (T1, T2, T3, T4) {
    fn schema() -> SchemaType {
        SchemaType {
            kind: TypeKind::Tuple {
                fields: vec![T1::schema(), T2::schema(), T3::schema(), T4::schema()],
            },
            description: None,
        }
    }
}
