use std::fmt;

use serde::Deserialize;
use serde::de::{Deserializer, SeqAccess, Visitor};

use super::cancellable_deserializer::CheckedDeserializer;
use super::cancellation::Cancellation;

#[derive(Debug, Deserialize, PartialEq)]
struct DerivedNewtype(u32);

#[derive(Debug, PartialEq)]
struct ObservedSequence {
    size_hint: Option<usize>,
    values: Vec<u32>,
}

impl<'de> Deserialize<'de> for ObservedSequence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(SequenceObserver)
    }
}

struct SequenceObserver;

impl<'de> Visitor<'de> for SequenceObserver {
    type Value = ObservedSequence;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an integer sequence")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<Self::Value, S::Error> {
        let size_hint = sequence.size_hint();
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element()? {
            values.push(value);
        }
        Ok(ObservedSequence { size_hint, values })
    }
}

#[test]
fn checked_deserializer_preserves_human_readable_flag() {
    let cancellation = Cancellation::new(|| false);
    let mut deserializer = serde_json::Deserializer::from_str("null");
    let checked = CheckedDeserializer::new(&mut deserializer, &cancellation);
    assert!(checked.is_human_readable());
}

#[test]
fn checked_visitor_delegates_unit_and_none() {
    let cancellation = Cancellation::new(|| false);
    let mut unit_deserializer = serde_json::Deserializer::from_str("null");
    let unit = <()>::deserialize(CheckedDeserializer::new(
        &mut unit_deserializer,
        &cancellation,
    ));
    assert!(unit.is_ok());

    let mut deserializer = serde_json::Deserializer::from_str("null");
    let value =
        Option::<()>::deserialize(CheckedDeserializer::new(&mut deserializer, &cancellation));
    assert_eq!(value.unwrap(), None);
}

#[test]
fn checked_visitor_delegates_some_and_newtype_struct() {
    let cancellation = Cancellation::new(|| false);
    let mut option_deserializer = serde_json::Deserializer::from_str("7");
    let option = Option::<u32>::deserialize(CheckedDeserializer::new(
        &mut option_deserializer,
        &cancellation,
    ));
    assert_eq!(option.unwrap(), Some(7));

    let mut newtype_deserializer = serde_json::Deserializer::from_str("9");
    let newtype = DerivedNewtype::deserialize(CheckedDeserializer::new(
        &mut newtype_deserializer,
        &cancellation,
    ));
    assert_eq!(newtype.unwrap(), DerivedNewtype(9));
}

#[test]
fn checked_visitor_delegates_expecting_for_invalid_json_type() {
    let cancellation = Cancellation::new(|| false);
    let mut deserializer = serde_json::Deserializer::from_str("true");
    let error =
        u64::deserialize(CheckedDeserializer::new(&mut deserializer, &cancellation)).unwrap_err();
    assert!(error.to_string().contains("expected u64"));
}

#[test]
fn checked_sequence_preserves_size_hint_for_real_json_array() {
    let cancellation = Cancellation::new(|| false);
    let mut deserializer = serde_json::Deserializer::from_str("[1,2]");
    let result =
        ObservedSequence::deserialize(CheckedDeserializer::new(&mut deserializer, &cancellation));
    assert_eq!(
        result.unwrap(),
        ObservedSequence {
            size_hint: None,
            values: vec![1, 2],
        }
    );
}

#[test]
fn checked_sequence_preserves_known_size_hint_and_elements_from_json_value() {
    let cancellation = Cancellation::new(|| false);
    let input = serde_json::json!([1, 2]);
    let result = ObservedSequence::deserialize(CheckedDeserializer::new(input, &cancellation));
    assert_eq!(
        result.unwrap(),
        ObservedSequence {
            size_hint: Some(2),
            values: vec![1, 2],
        }
    );
}

#[test]
fn checked_deserializer_reports_cancellation_as_serde_error() {
    let cancellation = Cancellation::new(|| true);
    let mut deserializer = serde_json::Deserializer::from_str("null");
    let error =
        Option::<u32>::deserialize(CheckedDeserializer::new(&mut deserializer, &cancellation))
            .unwrap_err();
    assert_eq!(error.to_string(), "Canvas parsing was cancelled");
}
