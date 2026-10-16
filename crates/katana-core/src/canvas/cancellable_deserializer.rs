use serde::de::{DeserializeSeed, Deserializer, Visitor};

use super::cancellable_visitor::CheckedVisitor;
use super::cancellation::Cancellation;

pub(super) struct CheckedDeserializer<'a, D, F> {
    inner: D,
    cancellation: &'a Cancellation<F>,
}

impl<'a, D, F> CheckedDeserializer<'a, D, F> {
    pub(super) fn new(inner: D, cancellation: &'a Cancellation<F>) -> Self {
        Self {
            inner,
            cancellation,
        }
    }
}

macro_rules! delegate {
    ($($method:ident($($argument:ident: $kind:ty),*)),* $(,)?) => {
        $(fn $method<V: Visitor<'de>>(self, $($argument: $kind,)* visitor: V)
            -> Result<V::Value, Self::Error> {
            self.cancellation.check_json()?;
            let result = self.inner.$method($($argument,)*
                CheckedVisitor::new(visitor, self.cancellation))?;
            self.cancellation.check_json()?;
            Ok(result)
        })*
    };
}

impl<'de, D: Deserializer<'de>, F: Fn() -> bool> Deserializer<'de>
    for CheckedDeserializer<'_, D, F>
{
    type Error = D::Error;

    delegate!(
        deserialize_any(), deserialize_bool(), deserialize_i8(), deserialize_i16(),
        deserialize_i32(), deserialize_i64(), deserialize_i128(), deserialize_u8(),
        deserialize_u16(), deserialize_u32(), deserialize_u64(), deserialize_u128(),
        deserialize_f32(), deserialize_f64(), deserialize_char(), deserialize_str(),
        deserialize_string(), deserialize_bytes(), deserialize_byte_buf(),
        deserialize_option(), deserialize_unit(), deserialize_seq(), deserialize_map(),
        deserialize_identifier(), deserialize_ignored_any(),
        deserialize_unit_struct(name: &'static str),
        deserialize_newtype_struct(name: &'static str),
        deserialize_tuple(length: usize),
        deserialize_tuple_struct(name: &'static str, length: usize),
        deserialize_struct(name: &'static str, fields: &'static [&'static str]),
        deserialize_enum(name: &'static str, variants: &'static [&'static str]),
    );

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

pub(super) struct CheckedSeed<'a, S, F> {
    inner: S,
    cancellation: &'a Cancellation<F>,
}

impl<'a, S, F> CheckedSeed<'a, S, F> {
    pub(super) fn new(inner: S, cancellation: &'a Cancellation<F>) -> Self {
        Self {
            inner,
            cancellation,
        }
    }
}

impl<'de, S: DeserializeSeed<'de>, F: Fn() -> bool> DeserializeSeed<'de> for CheckedSeed<'_, S, F> {
    type Value = S::Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.inner
            .deserialize(CheckedDeserializer::new(deserializer, self.cancellation))
    }
}
