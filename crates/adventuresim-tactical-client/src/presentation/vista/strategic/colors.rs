//! Join coarse foreground pigment and sward coverage to the first scenic ring.
use super::*;
use bevy::mesh::VertexAttributeValues;

struct EdgeColor {
    pigment: Vec4,
    vista_weight: f32,
}

impl StrategicBackdrop {
    pub(super) fn stitch_colors(&self, mesh: &mut Mesh, foreground: &[[f32; 4]], spacing: f32) {
        let Some(positions) = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
        else {
            return;
        };
        let half = Vec2::new(self.terrain.width(), self.terrain.depth()) * 0.5;
        let edges = positions
            .iter()
            .map(|position| {
                let point = Vec2::new(position[0], position[2]);
                let outside = (point.abs() - half).max(Vec2::ZERO).max_element();
                EdgeColor {
                    pigment: self.boundary_color(point.clamp(-half, half), foreground),
                    vista_weight: (outside / (spacing * pigment::PLAYABLE_COLOR_TRANSITION_CELLS))
                        .clamp(0.0, 1.0),
                }
            })
            .collect::<Vec<_>>();
        let Some(VertexAttributeValues::Float32x4(colors)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        else {
            return;
        };
        for (color, edge) in colors.iter_mut().zip(edges) {
            *color = edge
                .pigment
                .lerp(Vec4::from_array(*color), edge.vista_weight)
                .to_array();
        }
    }

    fn boundary_color(&self, point: Vec2, colors: &[[f32; 4]]) -> Vec4 {
        // The admitted sampled mesh is row-major with the same anti-diagonal
        // triangles as SceneTerrain. Clamp at the final cell so edge vertices
        // still interpolate their actual foreground RGB and grass coverage.
        let width = self.terrain.grid_width();
        let depth = self.terrain.grid_depth();
        let size = Vec2::new((width - 1) as f32, (depth - 1) as f32);
        let coordinate = (point / self.terrain.grid_scale() + size * 0.5).clamp(Vec2::ZERO, size);
        let lower = coordinate
            .floor()
            .as_uvec2()
            .min(UVec2::new((width - 2) as u32, (depth - 2) as u32));
        let fraction = coordinate - lower.as_vec2();
        let at = |x, z| Vec4::from_array(colors[z as usize * width + x as usize]);
        let a = at(lower.x, lower.y);
        let b = at(lower.x + 1, lower.y);
        let c = at(lower.x, lower.y + 1);
        let d = at(lower.x + 1, lower.y + 1);
        if fraction.x + fraction.y <= 1.0 {
            a * (1.0 - fraction.x - fraction.y) + b * fraction.x + c * fraction.y
        } else {
            b * (1.0 - fraction.y) + c * (1.0 - fraction.x) + d * (fraction.x + fraction.y - 1.0)
        }
    }
}
