//! Checked decoding for single-field numeric database products.
//!
//! The product name, field name, and primitive wire type match the domain
//! type's schema. Decoding uses its `TryFrom` implementation, including SATS
//! validation and BSATN decoding; Serde validation alone is insufficient.

macro_rules! checked_numeric_product {
    ($name:ident, $field:ident: $raw:ty) => {
        impl spacetimedb_lib::SpacetimeType for $name {
            fn make_type<S: spacetimedb_lib::sats::typespace::TypespaceBuilder>(
                typespace: &mut S,
            ) -> spacetimedb_lib::AlgebraicType {
                typespace.add(
                    std::any::TypeId::of::<Self>(),
                    Some(stringify!($name)),
                    |space| {
                        spacetimedb_lib::AlgebraicType::product([(
                            Some(stringify!($field)),
                            <$raw as spacetimedb_lib::SpacetimeType>::make_type(space),
                        )])
                    },
                )
            }
        }

        impl spacetimedb_lib::ser::Serialize for $name {
            fn serialize<S: spacetimedb_lib::ser::Serializer>(
                &self,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                use spacetimedb_lib::ser::SerializeNamedProduct;
                let mut product = serializer.serialize_named_product(1)?;
                product.serialize_element(Some(stringify!($field)), &self.get())?;
                product.end()
            }
        }

        impl<'de> spacetimedb_lib::de::Deserialize<'de> for $name {
            fn deserialize<D: spacetimedb_lib::de::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Self, D::Error> {
                #[derive(spacetimedb_lib::de::Deserialize)]
                #[sats(crate = spacetimedb_lib)]
                struct Wire {
                    $field: $raw,
                }
                let wire = <Wire as spacetimedb_lib::de::Deserialize>::deserialize(deserializer)?;
                Self::try_from(wire.$field).map_err(spacetimedb_lib::de::Error::custom)
            }
        }
    };
}

pub(crate) use checked_numeric_product;
