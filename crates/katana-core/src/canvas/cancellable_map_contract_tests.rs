use std::fmt;

use serde::Deserialize;
use serde::de::{Deserializer, MapAccess, Visitor};

use super::cancellable_deserializer::CheckedDeserializer;
use super::cancellation::Cancellation;

#[derive(Debug, PartialEq)]
struct ObservedMap {
    size_hint: Option<usize>,
    entries: Vec<(String, u32)>,
}

impl<'de> Deserialize<'de> for ObservedMap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(MapObserver)
    }
}

struct MapObserver;

impl<'de> Visitor<'de> for MapObserver {
    type Value = ObservedMap;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map of counters")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let size_hint = map.size_hint();
        let mut entries = Vec::new();
        while let Some(entry) = map.next_entry()? {
            entries.push(entry);
        }
        Ok(ObservedMap { size_hint, entries })
    }
}

#[test]
fn checked_map_preserves_unknown_hint_and_all_json_entries() {
    let cancellation = Cancellation::new(|| false);
    let mut parser = serde_json::Deserializer::from_str(r#"{"first":7,"second":9}"#);
    let result = ObservedMap::deserialize(CheckedDeserializer::new(&mut parser, &cancellation));
    assert_eq!(
        result.unwrap(),
        ObservedMap {
            size_hint: None,
            entries: vec![("first".into(), 7), ("second".into(), 9)],
        }
    );
}

#[test]
fn checked_map_preserves_known_hint_and_all_json_value_entries() {
    let cancellation = Cancellation::new(|| false);
    let input = serde_json::json!({"first":7,"second":9});
    let result = ObservedMap::deserialize(CheckedDeserializer::new(input, &cancellation));
    assert_eq!(
        result.unwrap(),
        ObservedMap {
            size_hint: Some(2),
            entries: vec![("first".into(), 7), ("second".into(), 9)],
        }
    );
}
