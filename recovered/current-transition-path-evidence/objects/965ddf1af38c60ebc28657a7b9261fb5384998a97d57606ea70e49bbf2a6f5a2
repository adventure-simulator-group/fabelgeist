//! Encode each tuft's animated bound for the custom GPU visibility pass.
use bevy::{mesh::VertexAttributeValues, prelude::*};

// Bounds for the deformation terms in tactical_grass_instanced.wgsl.
const MAX_BLADE_VIGOR: f32 = 1.06;
const MAX_NATURAL_AND_AUTHORED_LEAN_METRES: f32 = 0.157;
const MAX_WIND_ENVELOPE: f32 = 1.18;
const MAX_INTERACTION_DISPLACEMENT: f32 = 1.24;

pub(super) fn metadata(mesh: &Mesh, width: f32, wind: f32, maximum_push: f32) -> LinearRgba {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("grass tuft positions are float3");
    };
    let Some(VertexAttributeValues::Float32x2(roots)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1) else {
        panic!("grass tufts retain their blade roots");
    };
    let geometry_radius = positions
        .iter()
        .zip(roots)
        .map(|(position, root)| {
            let root = Vec2::from_array(*root);
            root.length()
                + position[1].abs() * MAX_BLADE_VIGOR
                + (Vec2::new(position[0], position[2]) - root).length() * width.max(1.0)
        })
        .fold(0.0, f32::max);
    let radius = geometry_radius
        + MAX_NATURAL_AND_AUTHORED_LEAN_METRES
        + wind.abs() * MAX_WIND_ENVELOPE
        + maximum_push.abs() * MAX_INTERACTION_DISPLACEMENT;
    // The grass shader has its own coverage alpha. Its unused batch alpha lane
    // carries a root-centred radius, while RGB retains the white pigment tint.
    LinearRgba::new(1.0, 1.0, 1.0, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animated_bound_encloses_rotated_width_and_maximum_push() {
        let mut mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.9, 1.4, -0.2]]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.6, -0.2]]);
        let bound = metadata(&mesh, 2.0, 0.5, 1.35);
        // Worst-direction wind, player push and lean, with camera-facing width.
        let extreme = Vec3::new(0.6 + 0.6 + 0.59 + 1.215 + 0.157, 1.484 + 0.459, -0.2);
        assert!(bound.alpha >= extreme.length());
        assert_eq!([bound.red, bound.green, bound.blue], [1.0; 3]);
        assert!(metadata(&mesh, 2.0, 1.0, 2.0).alpha > bound.alpha);
    }
}
