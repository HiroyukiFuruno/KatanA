use std::fmt;

use serde::de::{Deserializer, Error, MapAccess, SeqAccess, Visitor};

use super::cancellable_deserializer::{CheckedDeserializer, CheckedSeed};
use super::cancellation::Cancellation;

pub(super) struct CheckedVisitor<'a, V, F> {
    inner: V,
    cancellation: &'a Cancellation<F>,
}

impl<'a, V, F> CheckedVisitor<'a, V, F> {
    pub(super) fn new(inner: V, cancellation: &'a Cancellation<F>) -> Self {
        Self {
            inner,
            cancellation,
        }
    }
}

macro_rules! scalar {
    ($($method:ident($value:ident: $kind:ty)),* $(,)?) => {
        $(fn $method<E: Error>(self, $value: $kind) -> Result<Self::Value, E> {
            self.inner.$method($value)
        })*
    };
}

impl<'de, V: Visitor<'de>, F: Fn() -> bool> Visitor<'de> for CheckedVisitor<'_, V, F> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(formatter)
    }

    scalar!(
        visit_bool(value: bool),
        visit_i8(value: i8),
        visit_i16(value: i16),
        visit_i32(value: i32),
        visit_i64(value: i64),
        visit_i128(value: i128),
        visit_u8(value: u8),
        visit_u16(value: u16),
        visit_u32(value: u32),
        visit_u64(value: u64),
        visit_u128(value: u128),
        visit_f32(value: f32),
        visit_f64(value: f64),
        visit_char(value: char),
        visit_str(value: &str),
        visit_borrowed_str(value: &'de str),
        visit_string(value: String),
        visit_bytes(value: &[u8]),
        visit_borrowed_bytes(value: &'de [u8]),
        visit_byte_buf(value: Vec<u8>),
    );

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        self.inner.visit_unit()
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        self.inner.visit_none()
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        self.inner
            .visit_some(CheckedDeserializer::new(deserializer, self.cancellation))
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        self.inner
            .visit_newtype_struct(CheckedDeserializer::new(deserializer, self.cancellation))
    }

    fn visit_seq<S: SeqAccess<'de>>(self, sequence: S) -> Result<Self::Value, S::Error> {
        self.inner.visit_seq(CheckedSequence {
            inner: sequence,
            cancellation: self.cancellation,
        })
    }

    fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
        self.inner.visit_map(CheckedMap {
            inner: map,
            cancellation: self.cancellation,
        })
    }
}

struct CheckedSequence<'a, S, F> {
    inner: S,
    cancellation: &'a Cancellation<F>,
}

impl<'de, S: SeqAccess<'de>, F: Fn() -> bool> SeqAccess<'de> for CheckedSequence<'_, S, F> {
    type Error = S::Error;

    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.cancellation.check_json()?;
        self.inner
            .next_element_seed(CheckedSeed::new(seed, self.cancellation))
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner.size_hint()
    }
}

struct CheckedMap<'a, M, F> {
    inner: M,
    cancellation: &'a Cancellation<F>,
}

impl<'de, M: MapAccess<'de>, F: Fn() -> bool> MapAccess<'de> for CheckedMap<'_, M, F> {
    type Error = M::Error;

    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        self.cancellation.check_json()?;
        self.inner
            .next_key_seed(CheckedSeed::new(seed, self.cancellation))
    }

    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        self.cancellation.check_json()?;
        self.inner
            .next_value_seed(CheckedSeed::new(seed, self.cancellation))
    }

    fn size_hint(&self) -> Option<usize> {
        self.inner.size_hint()
    }
}
