//! Decoding retains the physical member identity before admitting its leaves.
use super::*;

#[derive(Deserialize)]
struct SerializedCuboid {
    source: ResolvedItemId,
    centre: Vec3,
    size: Vec3,
    yaw_radians: f32,
    crossfall_radians: f32,
    longfall_radians: f32,
}

impl<'de, F: GeometryFrame> Deserialize<'de> for CollisionCuboid<F> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = SerializedCuboid::deserialize(deserializer)?;
        Self::from_metres(
            value.source,
            value.centre,
            value.size,
            value.yaw_radians,
            value.crossfall_radians,
            value.longfall_radians,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoding_names_the_member_and_rejects_invalid_leaves() {
        let admitted = CollisionCuboid::<Architectural>::from_metres(
            ResolvedItemId(741),
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 2.0),
            0.3,
            0.0,
            0.0,
        )
        .unwrap();
        let wire = serde_json::to_value(admitted).unwrap();
        assert_eq!(
            serde_json::from_value::<CollisionCuboid<Architectural>>(wire.clone()).unwrap(),
            admitted
        );
        let mut reversed = wire;
        reversed["size"][1] = serde_json::json!(-1.0);
        let error = serde_json::from_value::<CollisionCuboid<Architectural>>(reversed).unwrap_err();
        let diagnostic = error.to_string();
        assert!(diagnostic.contains("741"));
        assert!(diagnostic.contains("CuboidDimensions"));
        assert!(diagnostic.contains("negative"));
        let input = CollisionCuboid::<Architectural>::from_metres(
            ResolvedItemId(742),
            Vec3::splat(f32::NAN),
            Vec3::ONE,
            0.0,
            0.0,
            0.0,
        )
        .unwrap_err();
        assert_eq!(input.source_id, ResolvedItemId(742));
        assert!(matches!(
            input.cause,
            GeometryError::NonFinite {
                role: crate::spatial_geometry::GeometryRole::Position,
                ..
            }
        ));
    }

    #[test]
    fn packed_cuboids_keep_the_native_wire_representation() {
        #[derive(Serialize)]
        struct NativeCuboid {
            source: ResolvedItemId,
            centre: Vec3,
            size: Vec3,
            yaw_radians: f32,
            crossfall_radians: f32,
            longfall_radians: f32,
        }
        let native = NativeCuboid {
            source: ResolvedItemId(72),
            centre: Vec3::new(-4.0, 0.0, 3.0),
            size: Vec3::new(1.0, 0.0, 2.0),
            yaw_radians: 0.7,
            crossfall_radians: -0.2,
            longfall_radians: 0.3,
        };
        let encoded = postcard::to_allocvec(&native).unwrap();
        let admitted: CollisionCuboid<Architectural> = postcard::from_bytes(&encoded).unwrap();
        assert_eq!(postcard::to_allocvec(&admitted).unwrap(), encoded);
    }
}
