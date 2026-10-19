//! Collection limits are enforced by Serde before reserving storage.
use serde::{
    Deserialize, Deserializer,
    de::{Error, SeqAccess, Visitor},
};
use std::{fmt, marker::PhantomData};

pub(in crate::presentation::obstacles::tree) fn sequence<'de, D, T, const LIMIT: usize>(
    decoder: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Bounded<T, const LIMIT: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> Visitor<'de> for Bounded<T, LIMIT> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a collection of at most {LIMIT} elements")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let count = sequence.size_hint().ok_or_else(|| {
                A::Error::custom("tree artifact requires explicit collection lengths")
            })?;
            if count > LIMIT {
                return Err(A::Error::custom("tree artifact collection limit"));
            }
            let mut result = Vec::with_capacity(count);
            for _ in 0..count {
                result.push(
                    sequence
                        .next_element()?
                        .ok_or_else(|| A::Error::custom("truncated tree collection"))?,
                );
            }
            if sequence.next_element::<T>()?.is_some() {
                return Err(A::Error::custom("tree artifact collection length"));
            }
            Ok(result)
        }
    }
    decoder.deserialize_seq(Bounded::<T, LIMIT>(PhantomData))
}
