//! Native patch-local metre mesh construction for renderer-owned litter.
use super::geometry::{BentTwig, CamberedLeaf, RosetteLeaf, TwigBase};
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
};
use bevy::{color::LinearRgba, math::Vec3Swizzles};
use fabelgeist_determinism::Seed;

impl GroundLitterMeshData {
    pub(super) fn append_rosette_leaf(&mut self, leaf: RosetteLeaf, color: Color) {
        let base = self.positions.len() as u32;
        // Native mesh kernel: patch-local X/Z metres and normalized plan axes.
        let root = leaf.root.metres().xz();
        let direction = leaf.direction.vector();
        let length = leaf.length.metres();
        let width = leaf.width.metres();
        let rise = leaf.rise.metres();
        let side = Vec2::new(-direction.y, direction.x);
        let centre = |along: f32, lateral: f32, height: f32| {
            let point = root + direction * (length * along) + side * (width * lateral);
            Vec3::new(point.x, height, point.y)
        };
        let positions = [
            centre(0.0, -0.18, 0.002),
            centre(0.0, 0.18, 0.002),
            centre(0.38, -1.0, rise * 0.72),
            centre(0.38, 1.0, rise * 0.72),
            centre(0.76, -0.62, rise),
            centre(0.76, 0.62, rise),
            centre(1.0, 0.0, rise * 0.82),
        ];
        let normal = Vec3::new(-direction.x * 0.24, 0.94, -direction.y * 0.24).normalize();
        let linear_color = color.to_linear();
        for (index, position) in positions.into_iter().enumerate() {
            self.positions.push(position.to_array());
            self.normals.push(normal.to_array());
            self.uvs.push([
                if index % 2 == 0 { 0.0 } else { 1.0 },
                [0.0, 0.0, 0.38, 0.38, 0.76, 0.76, 1.0][index],
            ]);
            self.roots.push(root.to_array());
            self.colors.push(linear_color);
        }
        self.indices.extend_from_slice(&[
            base,
            base + 2,
            base + 1,
            base + 1,
            base + 2,
            base + 3,
            base + 2,
            base + 4,
            base + 3,
            base + 3,
            base + 4,
            base + 5,
            base + 4,
            base + 6,
            base + 5,
        ]);
    }

    pub(super) fn append_bent_twig(&mut self, twig: BentTwig, color: Color) {
        let base = self.positions.len() as u32;
        // Native mesh kernel: checked patch-local metre stations and radii.
        let [start_station, middle_station, end_station] = twig.stations;
        let start = start_station.centre.metres();
        let middle = middle_station.centre.metres();
        let end = end_station.centre.metres();
        let start_radius = start_station.radius.metres();
        let middle_radius = middle_station.radius.metres();
        let end_radius = end_station.radius.metres();
        let sides = twig.cross_section.sides();
        let root = twig.root.metres().xz();
        let direction = (end - start).normalize();
        let reference = if direction.y.abs() < 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };
        let right = direction.cross(reference).normalize();
        let forward = right.cross(direction).normalize();
        let linear_color = color.to_linear();
        let near_tip = middle.lerp(end, 0.86);
        let late_tip = middle.lerp(end, 0.97);
        for (ring, (centre, radius)) in [
            (start, start_radius),
            (middle, middle_radius),
            (near_tip, middle_radius.lerp(end_radius, 0.72)),
            (late_tip, end_radius),
        ]
        .into_iter()
        .enumerate()
        {
            for side_index in 0..sides {
                let phase = side_index as f32 * core::f32::consts::TAU / sides as f32;
                let normal = right * phase.cos() + forward * phase.sin();
                self.positions.push((centre + normal * radius).to_array());
                self.normals.push(normal.to_array());
                self.uvs
                    .push([side_index as f32 / sides as f32, ring as f32 / 3.0]);
                self.roots.push(root.to_array());
                self.colors.push(linear_color);
            }
        }
        for ring in 0..3_u32 {
            let from = base + ring * sides;
            let to = from + sides;
            for side_index in 0..sides {
                let next = (side_index + 1) % sides;
                self.indices.extend_from_slice(&[
                    from + side_index,
                    to + side_index,
                    to + next,
                    from + side_index,
                    to + next,
                    from + next,
                ]);
            }
        }
        if matches!(twig.base, TwigBase::Closed) {
            let cap = self.positions.len() as u32;
            self.positions.push(start.to_array());
            self.normals.push((-direction).to_array());
            self.uvs.push([0.5, 0.0]);
            self.roots.push(root.to_array());
            self.colors.push(linear_color);
            for side_index in 0..sides {
                let next = (side_index + 1) % sides;
                self.indices
                    .extend_from_slice(&[cap, base + side_index, base + next]);
            }
        }
        let apex = self.positions.len() as u32;
        self.positions.push(end.to_array());
        self.normals.push(direction.to_array());
        self.uvs.push([0.5, 1.0]);
        self.roots.push(root.to_array());
        self.colors.push(linear_color);
        let tip_ring = base + sides * 3;
        for side_index in 0..sides {
            let next = (side_index + 1) % sides;
            self.indices
                .extend_from_slice(&[tip_ring + side_index, apex, tip_ring + next]);
        }
    }

    pub(super) fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.roots);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            self.colors
                .into_iter()
                .map(|color| color.to_f32_array())
                .collect::<Vec<_>>(),
        );
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

#[derive(Default)]
pub(super) struct GroundLitterMeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    roots: Vec<[f32; 2]>,
    colors: Vec<LinearRgba>,
    indices: Vec<u32>,
}

impl GroundLitterMeshData {
    pub(super) fn append_cambered_leaf(&mut self, leaf: CamberedLeaf, seed: Seed, color: Color) {
        let base = self.positions.len() as u32;
        // Fallen leaves should curl without becoming little tents. Build the
        // varied plate first, then seat its lowest vertex just below the local
        // patch ground plane so every instance visibly makes contact.
        // Native mesh kernel: checked patch-local metre point and spans.
        let centre = leaf.centre.metres().xz();
        let long = leaf.long.metres().xz();
        let side = leaf.side.metres().xz();
        let height = leaf.height.metres();
        let elevated = height >= 0.004;
        let long_slope = (streams::LONG_SLOPE.rng(seed, &[]).inclusive_unit_f32() - 0.5)
            * if elevated { 0.12 } else { 0.035 };
        let side_slope = (streams::SIDE_SLOPE.rng(seed, &[]).inclusive_unit_f32() - 0.5)
            * if elevated { 0.08 } else { 0.025 };
        let camber = if elevated {
            0.004 + streams::CAMBER.rng(seed, &[]).inclusive_unit_f32() * 0.007
        } else {
            0.0012 + streams::CAMBER.rng(seed, &[]).inclusive_unit_f32() * 0.0022
        };
        let curl = (streams::CURL.rng(seed, &[]).inclusive_unit_f32() - 0.5)
            * if elevated { 0.007 } else { 0.002 };
        let burial = 0.0007
            + height.min(0.006) * 0.15
            + streams::BURIAL.rng(seed, &[]).inclusive_unit_f32() * 0.001;
        let long3 = Vec3::new(long.x, long_slope * long.length(), long.y);
        let side3 = Vec3::new(side.x, side_slope * side.length(), side.y);
        let centre3 = Vec3::new(centre.x, 0.0, centre.y);
        let outline = [
            (0.0, -1.0),
            (0.82, -0.55),
            (1.0, 0.0),
            (0.74, 0.58),
            (0.0, 1.0),
            (-0.74, 0.58),
            (-1.0, 0.0),
            (-0.82, -0.55),
        ];
        let mut leaf_positions = Vec::with_capacity(9);
        leaf_positions.push(centre3 + Vec3::Y * camber);
        for (u, v) in outline {
            let lift = camber * (1.0 - u * u) * (1.0 - v * v) + curl * v * v;
            leaf_positions.push(centre3 + long3 * v + side3 * u + Vec3::Y * lift);
        }
        let minimum_y = leaf_positions
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min);
        for point in &mut leaf_positions {
            point.y += height - minimum_y - burial;
        }
        let mut leaf_normals = [Vec3::ZERO; 9];
        for outline_index in 0..8_usize {
            let left = 1 + outline_index;
            let right = 1 + (outline_index + 1) % 8;
            let face = (leaf_positions[right] - leaf_positions[0])
                .cross(leaf_positions[left] - leaf_positions[0]);
            leaf_normals[0] += face;
            leaf_normals[left] += face;
            leaf_normals[right] += face;
            self.indices
                .extend_from_slice(&[base, base + right as u32, base + left as u32]);
        }
        for (index, point) in leaf_positions.into_iter().enumerate() {
            self.positions.push(point.to_array());
            self.normals
                .push(leaf_normals[index].normalize().to_array());
            self.roots.push(centre.to_array());
        }
        self.uvs.push([0.5, 0.5]);
        self.uvs
            .extend(outline.map(|(u, v)| [0.5 + u * 0.5, 0.5 + v * 0.5]));
        let color = color.to_linear();
        self.colors.extend_from_slice(&[color; 9]);
    }
}
