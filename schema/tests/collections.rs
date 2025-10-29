use schema::{Schema, TypeKind};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList};

#[test]
fn test_hashset_schema() {
    let schema = HashSet::<String>::schema();

    match schema.kind {
        TypeKind::Set { items, ordered } => {
            assert_eq!(items.kind, TypeKind::String);
            assert_eq!(ordered, false);
            assert_eq!(
                schema.description,
                Some("Unordered set of unique values".to_string())
            );
        }
        _ => panic!("Expected Set schema for HashSet"),
    }
}

#[test]
fn test_btreemap_schema() {
    let schema = BTreeMap::<String, i32>::schema();

    match schema.kind {
        TypeKind::Map {
            key,
            value,
            ordered,
        } => {
            assert_eq!(key.kind, TypeKind::String);
            assert!(matches!(value.kind, TypeKind::Integer(_)));
            assert_eq!(ordered, true);
            assert_eq!(
                schema.description,
                Some("Ordered map/dictionary of key-value pairs".to_string())
            );
        }
        _ => panic!("Expected Map schema for BTreeMap"),
    }
}

#[test]
fn test_btreeset_schema() {
    let schema = BTreeSet::<u64>::schema();

    match schema.kind {
        TypeKind::Set { items, ordered } => {
            assert!(matches!(items.kind, TypeKind::Integer(_)));
            assert_eq!(ordered, true);
            assert_eq!(
                schema.description,
                Some("Ordered set of unique values".to_string())
            );
        }
        _ => panic!("Expected Set schema for BTreeSet"),
    }
}

#[test]
fn test_linkedlist_schema() {
    let schema = LinkedList::<bool>::schema();

    match schema.kind {
        TypeKind::Array { items } => {
            assert_eq!(items.kind, TypeKind::Boolean);
            assert_eq!(schema.description, Some("Doubly-linked list".to_string()));
        }
        _ => panic!("Expected Array schema for LinkedList"),
    }
}

#[test]
fn test_hashmap_schema() {
    let schema = HashMap::<String, i32>::schema();

    match schema.kind {
        TypeKind::Map {
            key,
            value,
            ordered,
        } => {
            assert_eq!(key.kind, TypeKind::String);
            assert!(matches!(value.kind, TypeKind::Integer(_)));
            assert_eq!(ordered, false);
            assert_eq!(
                schema.description,
                Some("Unordered map/dictionary of key-value pairs".to_string())
            );
        }
        _ => panic!("Expected Map schema for HashMap"),
    }
}
