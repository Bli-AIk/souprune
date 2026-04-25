//! Stable serialization helpers for unordered map-backed schema fields.
//!
//! 为基于无序 map 的 Schema 字段提供稳定序列化辅助。

use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

struct OrderedMapRef<'a, K, V>(&'a HashMap<K, V>);

impl<K, V> Serialize for OrderedMapRef<'_, K, V>
where
    K: Eq + Hash + Ord + Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ordered = self.0.iter().collect::<BTreeMap<_, _>>();
        ordered.serialize(serializer)
    }
}

struct OrderedNestedMapRef<'a, K, InnerK, V>(&'a HashMap<K, HashMap<InnerK, V>>);

impl<K, InnerK, V> Serialize for OrderedNestedMapRef<'_, K, InnerK, V>
where
    K: Eq + Hash + Ord + Serialize,
    InnerK: Eq + Hash + Ord + Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ordered = self
            .0
            .iter()
            .map(|(key, value)| (key, value.iter().collect::<BTreeMap<_, _>>()))
            .collect::<BTreeMap<_, _>>();
        ordered.serialize(serializer)
    }
}

/// Serialize a `HashMap` with a stable key order.
///
/// 以稳定键顺序序列化 `HashMap`。
pub fn serialize_ordered_map<S, K, V>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Eq + Hash + Ord + Serialize,
    V: Serialize,
{
    OrderedMapRef(map).serialize(serializer)
}

/// Serialize an optional `HashMap` with a stable key order.
///
/// 以稳定键顺序序列化可选的 `HashMap`。
pub fn serialize_optional_ordered_map<S, K, V>(
    map: &Option<HashMap<K, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Eq + Hash + Ord + Serialize,
    V: Serialize,
{
    match map {
        Some(value) => serializer.serialize_some(&OrderedMapRef(value)),
        None => serializer.serialize_none(),
    }
}

/// Serialize a nested `HashMap<String, HashMap<...>>` shape with stable key order.
///
/// 以稳定键顺序序列化嵌套 `HashMap<String, HashMap<...>>` 结构。
pub fn serialize_ordered_nested_map<S, K, InnerK, V>(
    map: &HashMap<K, HashMap<InnerK, V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Eq + Hash + Ord + Serialize,
    InnerK: Eq + Hash + Ord + Serialize,
    V: Serialize,
{
    OrderedNestedMapRef(map).serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::{
        serialize_optional_ordered_map, serialize_ordered_map, serialize_ordered_nested_map,
    };
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize)]
    struct SimpleMapWrapper {
        #[serde(serialize_with = "serialize_ordered_map")]
        values: HashMap<String, i32>,
    }

    #[derive(Serialize)]
    struct OptionalMapWrapper {
        #[serde(serialize_with = "serialize_optional_ordered_map")]
        values: Option<HashMap<String, i32>>,
    }

    #[derive(Serialize)]
    struct NestedMapWrapper {
        #[serde(serialize_with = "serialize_ordered_nested_map")]
        values: HashMap<String, HashMap<String, i32>>,
    }

    #[test]
    fn orders_simple_hash_map_keys() {
        let values = HashMap::from([
            ("z".to_owned(), 3),
            ("a".to_owned(), 1),
            ("m".to_owned(), 2),
        ]);
        let ron =
            ron::ser::to_string(&SimpleMapWrapper { values }).expect("wrapper should serialize");
        let a_idx = ron.find("\"a\"").expect("a key should exist");
        let m_idx = ron.find("\"m\"").expect("m key should exist");
        let z_idx = ron.find("\"z\"").expect("z key should exist");
        assert!(a_idx < m_idx && m_idx < z_idx, "unexpected order: {ron}");
    }

    #[test]
    fn orders_nested_hash_map_keys() {
        let inner = HashMap::from([("z".to_owned(), 2), ("a".to_owned(), 1)]);
        let values = HashMap::from([
            ("outer_z".to_owned(), HashMap::from([("b".to_owned(), 2)])),
            ("outer_a".to_owned(), inner),
        ]);
        let ron = ron::ser::to_string(&NestedMapWrapper { values })
            .expect("nested wrapper should serialize");
        let outer_a = ron.find("\"outer_a\"").expect("outer_a key should exist");
        let outer_z = ron.find("\"outer_z\"").expect("outer_z key should exist");
        let inner_a = ron.find("\"a\"").expect("inner a key should exist");
        let inner_z = ron.rfind("\"z\"").expect("inner z key should exist");
        assert!(outer_a < outer_z, "unexpected outer order: {ron}");
        assert!(inner_a < inner_z, "unexpected inner order: {ron}");
    }

    #[test]
    fn keeps_none_for_optional_maps() {
        let ron = ron::ser::to_string(&OptionalMapWrapper { values: None })
            .expect("optional wrapper should serialize");
        assert_eq!(ron, "(values:None)");
    }
}
