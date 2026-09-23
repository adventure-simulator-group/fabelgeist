//! Grass: Fabelgeist's four-tier instanced sward over the patch disc, on
//! either bevy_eidolon (`eidolon.rs`, the reference) or the hand-rolled
//! chunked path (`simple.rs`, the thing under test).
//!
//! This file holds what both paths share: the tier table, the tuft mesh
//! builder and the placement generator (ports of `ground_scatter/grass.rs`
//! and `instanced_grass.rs`, reduced to one species and one community), and
//! the respawn-on-settings-change system.

mod cards;
pub mod culled;
pub mod eidolon;
mod mesh_chunks;
pub mod simple;

use std::f32::consts::TAU;

use bevy::asset::{RenderAssetUsages, load_internal_asset, uuid_handle};
use bevy::camera::primitives::Aabb;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy_eidolon::components::InstanceData;

use crate::scene::{PATCH_RADIUS, hash01, terrain_height, terrain_normal};
use crate::settings::{BenchSettings, InstancingMode};

pub const GRASS_COMMON_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("7e0a2c9b-5f31-4c6e-9d0a-3b8f2a1c4d10");
pub const GRASS_EIDOLON_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("7e0a2c9b-5f31-4c6e-9d0a-3b8f2a1c4d11");
pub const GRASS_SIMPLE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("7e0a2c9b-5f31-4c6e-9d0a-3b8f2a1c4d12");

// ---------------------------------------------------------------------------
// Tier table (tactical-graphics.yaml, grass section)
// ---------------------------------------------------------------------------

pub const TIER_COUNT: usize = 4;

/// Placement cell lattice spacing, metres; the vista tier walks a coarser lattice.
const CELL_SPACING: f32 = 1.81;
const VISTA_CELL_SPACING: f32 = 3.62;
/// Fraction of a cell the gate sample point is jittered by.
const JITTER_FRACTION: f32 = 0.09;
/// Grass rejects any site steeper than this surface normal tilt.
const MIN_SLOPE_NORMAL_Y: f32 = 0.72;
/// Authored blade ribbon size, metres.
const BLADE_WIDTH_M: f32 = 0.076;
const BLADE_HEIGHT_M: f32 = 0.82;
/// Maximum blade reach above a tuft root for the fitted bounds.
const TUFT_HEIGHT_MARGIN_METRES: f32 = 1.5;
/// Placement seeds (Fabelgeist's constants, minus the scene digest).
const BASE_SEED: u64 = 0x6772_6173_735f_6c6f;
const VISTA_SEED_SALT: u64 = 0x7669_7374_615f_6c6f;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierId {
    Near,
    NearEdge,
    Far,
    Vista,
}

#[derive(Clone)]
pub struct Tier {
    pub id: TierId,
    pub name: &'static str,
    /// Eidolon fade band: `fade_in` = xy, `fade_out` = zw.
    pub fade_in: [f32; 2],
    pub fade_out: [f32; 2],
    /// Paired ribbon rows (height fractions) plus a shared tip vertex.
    pub ribbon_rows: &'static [f32],
    pub tufts_per_cell_side: u32,
    pub blades_per_tuft_side: u32,
    pub cell_spacing: f32,
    pub casts_shadows: bool,
    /// `params.w` of the material: projected-cover compensation.
    pub width_compensation: f32,
    /// Sparse seed heads are near-only geometry.
    pub seed_heads: bool,
}

impl Tier {
    /// Whether the geometric range cuts this tier off entirely.
    pub fn dropped(&self, range: f32) -> bool {
        self.fade_in[0] >= range
    }

    pub fn footprint(&self) -> f32 {
        self.cell_spacing / self.tufts_per_cell_side as f32
    }

    pub fn visibility_range(&self) -> Vec4 {
        Vec4::new(
            self.fade_in[0],
            self.fade_in[1],
            self.fade_out[0],
            self.fade_out[1],
        )
    }
}

pub const TIERS: [Tier; TIER_COUNT] = [
    Tier {
        id: TierId::Near,
        name: "near",
        fade_in: [0.0, 0.001],
        fade_out: [6.0, 8.0],
        ribbon_rows: &[0.0, 0.22, 0.45, 0.68, 0.9],
        tufts_per_cell_side: 12,
        blades_per_tuft_side: 8,
        cell_spacing: CELL_SPACING,
        casts_shadows: true,
        width_compensation: 1.0,
        seed_heads: true,
    },
    Tier {
        id: TierId::NearEdge,
        name: "near_edge",
        fade_in: [6.0, 8.0],
        fade_out: [10.0, 20.0],
        ribbon_rows: &[0.0, 0.30, 0.58, 0.83],
        tufts_per_cell_side: 12,
        blades_per_tuft_side: 6,
        cell_spacing: CELL_SPACING,
        casts_shadows: true,
        width_compensation: 1.333_333_4,
        seed_heads: false,
    },
    Tier {
        id: TierId::Far,
        name: "far",
        fade_in: [10.0, 20.0],
        fade_out: [48.0, 60.0],
        ribbon_rows: &[0.0, 0.45, 0.82],
        tufts_per_cell_side: 4,
        blades_per_tuft_side: 10,
        cell_spacing: CELL_SPACING,
        casts_shadows: true,
        // sqrt(1024 near roots / 64 far strata), clamped to the configured 1.65 limit.
        width_compensation: 1.65,
        seed_heads: false,
    },
    Tier {
        id: TierId::Vista,
        name: "vista",
        fade_in: [46.0, 56.0],
        fade_out: [60.0, 72.0],
        ribbon_rows: &[0.0, 0.62],
        tufts_per_cell_side: 2,
        blades_per_tuft_side: 12,
        cell_spacing: VISTA_CELL_SPACING,
        casts_shadows: false,
        width_compensation: 1.8,
        seed_heads: false,
    },
];

/// The tier table clipped to the geometric grass range: tiers starting past
/// it are dropped (their batches stay empty), the last surviving tier fades
/// out over the final 3 m.
pub fn active_tiers(range: f32) -> [Tier; TIER_COUNT] {
    let mut tiers = TIERS.clone();
    for tier in &mut tiers {
        if tier.fade_out[1] > range {
            let start = (range - 3.0).max(tier.fade_in[1]);
            tier.fade_out = [start, range.max(start + 0.001)];
        }
    }
    tiers
}

// ---------------------------------------------------------------------------
// Material parameters (the five Vec4 uniforms of the Fabelgeist material)
// ---------------------------------------------------------------------------

/// Scene-wide pigment inputs: a moderately hydrated meadow, light breeze.
const GRASS_DRYNESS: f32 = 0.35;
const WIND_SCALE: f32 = 0.16 + 0.4 * 0.36;
const ROOT_OCCLUSION: f32 = 0.52;
const AMBIENT_SCALE: f32 = 0.72;
const AUTHORED_LEAN: f32 = 0.09;
const INTERACTION_RADIUS_M: f32 = 1.35;
const INTERACTION_MINIMUM_PUSH: f32 = 0.7;

#[derive(Clone, Copy, Debug)]
pub struct GrassParams {
    /// Wind direction xy, strength, time scale.
    pub wind: Vec4,
    /// Interactor position (xyz) and interaction radius (w).
    pub interaction: Vec4,
    /// Interactor smoothed velocity (xyz) and push strength.
    pub interaction_motion: Vec4,
    /// Root occlusion, dryness lane, authored lean, width compensation.
    pub params: Vec4,
    /// y scales the flat ambient term; x/z/w reserved.
    pub shading: Vec4,
}

/// The material inputs for one tier. The interactor sits far outside the
/// patch with the game's radius, so the shader runs the interaction branch
/// exactly as it does with a player present, at zero displacement.
pub fn tier_params(tier: &Tier) -> GrassParams {
    GrassParams {
        wind: Vec4::new(0.74, 0.67, WIND_SCALE, 1.35),
        interaction: Vec4::new(1.0e5, 0.0, 1.0e5, INTERACTION_RADIUS_M),
        interaction_motion: Vec4::new(0.0, 0.0, 0.0, INTERACTION_MINIMUM_PUSH),
        params: Vec4::new(
            ROOT_OCCLUSION,
            GRASS_DRYNESS,
            AUTHORED_LEAN,
            tier.width_compensation,
        ),
        shading: Vec4::new(1.0, AMBIENT_SCALE, 0.0, 0.0),
    }
}

// ---------------------------------------------------------------------------
// Hashes
// ---------------------------------------------------------------------------

pub fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn unit_hash(value: u64) -> f32 {
    (value >> 40) as f32 / (1u64 << 24) as f32
}

// ---------------------------------------------------------------------------
// Tuft mesh (port of grass_tuft_mesh / grass_ribbon_patch_mesh_with_rows)
// ---------------------------------------------------------------------------

/// The one species the bench grows: false oat-grass, the mesic meadow's
/// dominant (68 %) species in Fabelgeist.
const SPECIES_HEIGHT_SCALE: f32 = 1.24;
const SPECIES_WIDTH_SCALE: f32 = 0.88;
const SPECIES_PIGMENT_SCALE: [f32; 3] = [1.02, 1.0, 0.84];
const SPECIES_INFLORESCENCE_BRANCHES: usize = 2;
const SPECIES_SPIKELETS_PER_BRANCH: usize = 2;

/// Fabelgeist's grass pigment: hydrated chlorophyll blended toward a dry
/// olive sward by `GRASS_DRYNESS`, authored in sRGB.
pub(super) fn grass_color() -> Color {
    let hydrated = Vec3::new(82.0, 119.0, 45.0);
    let senescent = Vec3::new(150.0, 126.0, 52.0);
    let pigment = hydrated.lerp(senescent, GRASS_DRYNESS * 0.78);
    Color::srgb_u8(pigment.x as u8, pigment.y as u8, pigment.z as u8)
}

#[derive(Clone, Copy)]
struct Blade {
    offset_x: f32,
    offset_z: f32,
    height_scale: f32,
    width_scale: f32,
    seed: u64,
}

#[derive(Clone, Copy)]
struct Inflorescence {
    root: Vec3,
    angle: f32,
    total_height: f32,
    normal: [f32; 3],
    color: [f32; 4],
    blade_root: [f32; 2],
}

/// Mesh vertex contract (identical to the game's grass meshes):
///   uv    = (side 0|1, height fraction; tip uses 0.5)
///   uv_b  = blade root offset in tuft-local xz
///   color = species pigment, alpha = per-blade threshold/variation hash
#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    blade_roots: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuffers {
    #[expect(clippy::too_many_arguments, reason = "port of the Fabelgeist helper")]
    fn attached_quad(
        &mut self,
        lower_left: Vec3,
        lower_right: Vec3,
        upper_left: Vec3,
        upper_right: Vec3,
        normal: [f32; 3],
        color: [f32; 4],
        blade_root: [f32; 2],
        attachment_height: f32,
        attachment_fraction: f32,
    ) {
        let base = self.positions.len() as u32;
        self.positions.extend_from_slice(&[
            lower_left.to_array(),
            lower_right.to_array(),
            upper_left.to_array(),
            upper_right.to_array(),
        ]);
        self.normals.extend_from_slice(&[normal; 4]);
        // Negative V identifies rigid seed-head geometry. U carries its authored
        // stalk attachment height so the shader can preserve the mesh while
        // inheriting the parent shoot's deformation.
        self.uvs
            .extend_from_slice(&[[attachment_height, -attachment_fraction]; 4]);
        self.blade_roots.extend_from_slice(&[blade_root; 4]);
        self.colors.extend_from_slice(&[color; 4]);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 3, base, base + 3, base + 2]);
    }

    #[expect(clippy::too_many_arguments, reason = "port of the Fabelgeist helper")]
    fn crossed_spikelet(
        &mut self,
        centre: Vec3,
        branch_direction: Vec3,
        half_width: f32,
        half_height: f32,
        color: [f32; 4],
        blade_root: [f32; 2],
        attachment_height: f32,
        attachment_fraction: f32,
    ) {
        let horizontal = Vec3::new(-branch_direction.z, 0.0, branch_direction.x)
            .normalize_or_zero()
            * half_width;
        let along = branch_direction.normalize_or_zero() * half_width;
        for side in [horizontal, along] {
            let normal = Vec3::Y.cross(side).normalize_or_zero().to_array();
            self.attached_quad(
                centre - Vec3::Y * half_height,
                centre - side,
                centre + side,
                centre + Vec3::Y * half_height,
                normal,
                color,
                blade_root,
                attachment_height,
                attachment_fraction,
            );
        }
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.blade_roots);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// One instanced grass tuft: a jittered blade grid of `blades_per_tuft_side`²
/// ribbons over the tier's footprint (Fabelgeist's `grass_tuft_mesh` at full
/// blade density).
pub fn tuft_mesh(tier: &Tier, seed: u64) -> Mesh {
    let grid_side = tier.blades_per_tuft_side as usize;
    let centre = (grid_side - 1) as f32 * 0.5;
    let blade_spacing = tier.footprint() / grid_side as f32;
    let blades = (0..grid_side * grid_side)
        .map(|index| {
            let row = index / grid_side;
            let column = index % grid_side;
            let hash = splitmix64(index as u64 ^ seed ^ 0x8d12_6f4a_0bc3_7791);
            let jitter_x = (unit_hash(hash) - 0.5) * blade_spacing * 0.46;
            let jitter_z = (unit_hash(splitmix64(hash)) - 0.5) * blade_spacing * 0.46;
            let clump_vigor = 0.5 + 0.5 * (row as f32 * 0.31 + column as f32 * 0.17 + 0.8).sin();
            let height_scale =
                (0.50 + unit_hash(splitmix64(hash ^ 0x52a9_f131)) * 0.62 + clump_vigor * 0.20)
                    .clamp(0.50, 1.30);
            let width_scale = 0.62 + unit_hash(splitmix64(hash ^ 0x91e2_57a4)) * 0.76;
            Blade {
                offset_x: (column as f32 - centre) * blade_spacing + jitter_x,
                offset_z: (row as f32 - centre) * blade_spacing + jitter_z,
                height_scale,
                width_scale,
                seed: splitmix64(index as u64 ^ seed),
            }
        })
        .collect::<Vec<_>>();
    ribbon_mesh(
        BLADE_WIDTH_M * (0.026 / 0.076),
        BLADE_HEIGHT_M,
        grass_color(),
        tier,
        &blades,
    )
}

fn ribbon_mesh(width: f32, height: f32, color: Color, tier: &Tier, blades: &[Blade]) -> Mesh {
    let rows = tier.ribbon_rows;
    let vertices_per_blade = rows.len() * 2 + 1;
    let triangles_per_blade = (rows.len() - 1) * 2 + 1;
    let mut buffers = MeshBuffers {
        positions: Vec::with_capacity(blades.len() * vertices_per_blade),
        normals: Vec::with_capacity(blades.len() * vertices_per_blade),
        uvs: Vec::with_capacity(blades.len() * vertices_per_blade),
        blade_roots: Vec::with_capacity(blades.len() * vertices_per_blade),
        colors: Vec::with_capacity(blades.len() * vertices_per_blade),
        indices: Vec::with_capacity(blades.len() * triangles_per_blade * 3),
    };
    let mut inflorescences = Vec::new();
    let linear = color.to_linear().to_f32_array();

    for &Blade {
        offset_x,
        offset_z,
        height_scale,
        width_scale,
        seed: blade_seed,
    } in blades
    {
        let root = Vec3::new(offset_x, 0.0, offset_z);
        let hash = splitmix64(blade_seed ^ 0x6c8e_9cf5_701a_d30b);
        let angle = unit_hash(hash) * TAU;
        let half_width = Vec3::new(angle.cos(), 0.0, angle.sin())
            * width
            * width_scale
            * SPECIES_WIDTH_SCALE
            * 0.5;
        let normal = Vec3::Y.cross(half_width).normalize_or_zero().to_array();
        let blade_threshold = unit_hash(splitmix64(hash ^ 0x3d91_02ea_61b8_7c45));
        let age = unit_hash(splitmix64(hash ^ 0x1b47_c95a_622d_41e3));
        let lean_direction = Vec3::new(-angle.sin(), 0.0, angle.cos());
        let total_height = height * height_scale * SPECIES_HEIGHT_SCALE;
        let lean_metres =
            total_height * (0.008 + unit_hash(splitmix64(hash ^ 0x626c_6164_655f_6c65)) * 0.027);
        // Healthy blades share their species pigment. Senescent tips retain a
        // hard straw region.
        let blade_color = [
            (linear[0] * SPECIES_PIGMENT_SCALE[0]).clamp(0.0, 1.0),
            (linear[1] * SPECIES_PIGMENT_SCALE[1]).clamp(0.0, 1.0),
            (linear[2] * SPECIES_PIGMENT_SCALE[2]).clamp(0.0, 1.0),
            blade_threshold,
        ];
        let luminance = blade_color[0] * 0.2126 + blade_color[1] * 0.7152 + blade_color[2] * 0.0722;
        let straw_color = [
            luminance * 1.12,
            luminance * 0.88,
            luminance * 0.42,
            blade_threshold,
        ];
        let senescent = age > 0.82;
        let base = buffers.positions.len() as u32;

        for &height_fraction in rows {
            let taper = (1.0 - height_fraction).powf(0.72);
            let side = half_width * taper;
            let centre = root
                + Vec3::Y * total_height * height_fraction
                + lean_direction * lean_metres * height_fraction.powf(1.65);
            buffers
                .positions
                .extend_from_slice(&[(centre - side).to_array(), (centre + side).to_array()]);
            buffers.normals.extend_from_slice(&[normal; 2]);
            buffers
                .uvs
                .extend_from_slice(&[[0.0, height_fraction], [1.0, height_fraction]]);
            buffers
                .blade_roots
                .extend_from_slice(&[[offset_x, offset_z]; 2]);
            let row_color = if senescent && height_fraction >= 0.72 {
                straw_color
            } else {
                blade_color
            };
            buffers.colors.extend_from_slice(&[row_color; 2]);
        }
        buffers
            .positions
            .push((root + Vec3::Y * total_height + lean_direction * lean_metres).to_array());
        buffers.normals.push(normal);
        buffers.uvs.push([0.5, 1.0]);
        buffers.blade_roots.push([offset_x, offset_z]);
        buffers
            .colors
            .push(if senescent { straw_color } else { blade_color });

        for row in 0..rows.len() - 1 {
            let lower = base + (row * 2) as u32;
            let upper = lower + 2;
            buffers.indices.extend_from_slice(&[
                lower,
                lower + 1,
                upper + 1,
                lower,
                upper + 1,
                upper,
            ]);
        }
        let shoulder = base + ((rows.len() - 1) * 2) as u32;
        let tip = base + (vertices_per_blade - 1) as u32;
        buffers
            .indices
            .extend_from_slice(&[shoulder, shoulder + 1, tip]);

        // Seed heads: about one shoot in eight, near tier only.
        if tier.seed_heads && unit_hash(splitmix64(hash ^ 0x0070_616e_6963_6c65)) < 0.125 {
            inflorescences.push(Inflorescence {
                root,
                angle,
                total_height,
                normal,
                color: blade_color,
                blade_root: [offset_x, offset_z],
            });
        }
    }

    for Inflorescence {
        root,
        angle,
        total_height,
        normal,
        color,
        blade_root,
    } in inflorescences
    {
        // False oat-grass: open panicle (not the compact cocksfoot cluster).
        let stem_side = Vec3::new(angle.cos(), 0.0, angle.sin()) * 0.0025;
        for (start_fraction, end_fraction) in [(0.62, 0.82), (0.82, 1.03)] {
            let start = root + Vec3::Y * total_height * start_fraction;
            let end = root + Vec3::Y * total_height * end_fraction;
            buffers.attached_quad(
                start - stem_side,
                start + stem_side,
                end - stem_side * 0.62,
                end + stem_side * 0.62,
                normal,
                color,
                blade_root,
                start.y,
                start_fraction,
            );
        }

        for branch in 0..SPECIES_INFLORESCENCE_BRANCHES {
            let fraction = branch as f32 / SPECIES_INFLORESCENCE_BRANCHES as f32;
            let side_sign = if branch % 2 == 0 { 1.0 } else { -1.0 };
            let branch_angle = angle + side_sign * (0.72 + fraction * 0.38);
            let direction = Vec3::new(branch_angle.cos(), 0.0, branch_angle.sin());
            let length = 0.075 + fraction * 0.055;
            let thickness = 0.005;
            let start = root + Vec3::Y * total_height * (0.70 + fraction * 0.055);
            let end = start + direction * length + Vec3::Y * total_height * 0.045;
            let side = Vec3::new(-direction.z, 0.0, direction.x) * thickness;
            let attachment_fraction = start.y / total_height;
            buffers.attached_quad(
                start - side,
                start + side,
                end - side * 0.55,
                end + side * 0.55,
                normal,
                color,
                blade_root,
                start.y,
                attachment_fraction,
            );

            for spikelet in 0..SPECIES_SPIKELETS_PER_BRANCH {
                let along = 0.64 + spikelet as f32 * 0.27;
                let centre = start.lerp(end, along.min(1.0));
                buffers.crossed_spikelet(
                    centre,
                    direction,
                    0.005,
                    0.010,
                    color,
                    blade_root,
                    start.y,
                    attachment_fraction,
                );
            }
        }
    }

    buffers.into_mesh()
}

/// One tier's shared tuft mesh and its size.
pub struct TierMesh {
    pub handle: Handle<Mesh>,
    pub vertices: usize,
    pub triangles: usize,
}

#[derive(Resource, Default)]
struct GrassMeshes(Vec<TierMesh>);

// ---------------------------------------------------------------------------
// Placement (port of scatter_cell_tufts over the patch disc)
// ---------------------------------------------------------------------------

/// Jittered-lattice cell walk over the patch disc: a cell carries tufts when
/// its jittered gate point lies inside the disc on a gentle enough slope; each
/// accepted cell is then subdivided into `tufts_per_cell_side`² jittered tufts,
/// each thinned by `density`, each gated by slope again. Coverage is the
/// constant 255 (no cover mask here) packed into the seed's low byte exactly
/// as the game does.
pub fn scatter_tier(tier: &Tier, density: f32, base_seed: u64) -> Vec<InstanceData> {
    let spacing = tier.cell_spacing;
    let side = tier.tufts_per_cell_side as i32;
    let footprint = tier.footprint();
    let reach = (PATCH_RADIUS / spacing).ceil() as i32 + 1;
    let mut instances = Vec::new();
    for z in -reach..=reach {
        for x in -reach..=reach {
            let cell = ((x as u32 as u64) << 32) | z as u32 as u64;
            let cell_hash = splitmix64(base_seed ^ cell);
            let gate = Vec2::new(
                (x as f32 + (hash01(cell_hash, 0x39bd_7f21) - 0.5) * JITTER_FRACTION) * spacing,
                (z as f32 + (hash01(cell_hash, 0xe651_34aa) - 0.5) * JITTER_FRACTION) * spacing,
            );
            if gate.length() > PATCH_RADIUS || terrain_normal(gate.x, gate.y).y < MIN_SLOPE_NORMAL_Y
            {
                continue;
            }
            let cell_origin = Vec2::new(x as f32, z as f32) * spacing
                - Vec2::splat((side - 1) as f32 * 0.5 * footprint);
            for tuft_z in 0..side {
                for tuft_x in 0..side {
                    let tuft_hash =
                        splitmix64(cell_hash ^ (((tuft_x as u64) << 17) | ((tuft_z as u64) << 3)));
                    if density < 1.0 && hash01(tuft_hash, 0x6465_6e73) >= density {
                        continue;
                    }
                    let jitter = Vec2::new(hash01(tuft_hash, 1) - 0.5, hash01(tuft_hash, 2) - 0.5)
                        * footprint
                        * 0.35;
                    let centre =
                        cell_origin + Vec2::new(tuft_x as f32, tuft_z as f32) * footprint + jitter;
                    if centre.length() > PATCH_RADIUS
                        || terrain_normal(centre.x, centre.y).y < MIN_SLOPE_NORMAL_Y
                    {
                        continue;
                    }
                    let coverage = 255u8;
                    instances.push(InstanceData {
                        position: Vec3::new(centre.x, terrain_height(centre.x, centre.y), centre.y),
                        scale: 1.0,
                        rotation: hash01(tuft_hash, 0x0079_6177) * TAU,
                        index: instances.len() as u32,
                        batch_id: 0,
                        seed: u32::from(coverage)
                            | ((splitmix64(tuft_hash ^ 0x7365_6564) as u32) << 8),
                    });
                }
            }
        }
    }
    instances
}

/// Tight bounds over a batch's instances with blade headroom
/// (Fabelgeist's `fitted_batch_aabb`).
pub fn fitted_aabb(instances: &[InstanceData], footprint: f32) -> Aabb {
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);
    for instance in instances {
        minimum = minimum.min(instance.position);
        maximum = maximum.max(instance.position);
    }
    let margin = Vec3::new(footprint, TUFT_HEIGHT_MARGIN_METRES, footprint);
    let minimum = minimum - margin * Vec3::new(1.0, 0.2, 1.0);
    let maximum = maximum + margin;
    Aabb {
        center: ((minimum + maximum) * 0.5).into(),
        half_extents: ((maximum - minimum) * 0.5).into(),
    }
}

// ---------------------------------------------------------------------------
// Plugin and respawn
// ---------------------------------------------------------------------------

/// Every entity either grass path spawns; despawned wholesale on respawn.
#[derive(Component)]
pub struct GrassEntity;

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            GRASS_COMMON_SHADER_HANDLE,
            "shaders/grass_common.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            GRASS_EIDOLON_SHADER_HANDLE,
            "shaders/grass_eidolon.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            GRASS_SIMPLE_SHADER_HANDLE,
            "shaders/grass_simple.wgsl",
            Shader::from_wgsl
        );
        app.init_asset::<eidolon::GrassMaterial>()
            .init_resource::<GrassMeshes>()
            .add_plugins((
                #[cfg(not(feature = "downlevel"))]
                eidolon::GrassEidolonPlugin,
                simple::GrassSimplePlugin,
                culled::GrassCulledPlugin,
                mesh_chunks::GrassMeshChunksPlugin,
            ))
            .add_systems(Update, respawn_grass);
        if matches!(
            crate::settings::demo_mode(),
            None | Some(InstancingMode::Cards | InstancingMode::CardsCurved)
        ) {
            app.add_plugins(cards::GrassCardsPlugin);
        }
    }
}

fn respawn_grass(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tier_meshes: ResMut<GrassMeshes>,
    mut materials: ResMut<Assets<eidolon::GrassMaterial>>,
    existing: Query<Entity, With<GrassEntity>>,
    mut applied: Local<Option<(InstancingMode, u32, u32, bool)>>,
) {
    let wanted = (
        settings.instancing,
        settings.grass_density.to_bits(),
        settings.grass_range.to_bits(),
        settings.grass_shadows,
    );
    if *applied == Some(wanted) {
        return;
    }
    *applied = Some(wanted);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if settings.instancing == InstancingMode::None {
        info!("grass: off");
        return;
    }
    if matches!(
        settings.instancing,
        InstancingMode::MeshChunks
            | InstancingMode::MeshChunksMap
            | InstancingMode::Cards
            | InstancingMode::CardsCurved
    ) {
        // Streams its own chunks around the camera; see mesh_chunks.rs / cards.rs.
        info!(
            "grass: {}, range {:.0} m",
            settings.instancing.label(),
            settings.grass_range
        );
        return;
    }

    if tier_meshes.0.is_empty() {
        for (tier_index, tier) in TIERS.iter().enumerate() {
            let mesh = tuft_mesh(tier, splitmix64(BASE_SEED ^ tier_index as u64));
            let vertices = mesh.count_vertices();
            let triangles = mesh.indices().map_or(0, |indices| indices.len() / 3);
            tier_meshes.0.push(TierMesh {
                handle: meshes.add(mesh),
                vertices,
                triangles,
            });
        }
    }

    let started = Instant::now();
    let density = settings.grass_density.clamp(0.05, 1.0);
    let range = settings.grass_range.clamp(8.0, 72.0);
    let tiers = active_tiers(range);
    let mut batches: [Vec<InstanceData>; TIER_COUNT] = Default::default();
    for (tier_index, tier) in tiers.iter().enumerate() {
        if tier.dropped(range) {
            info!(
                "grass {}: dropped (starts past the {range:.0} m range)",
                tier.name
            );
            continue;
        }
        batches[tier_index] = match tier.id {
            // Reuse near placements so the near-edge crossfade does not move tufts.
            TierId::NearEdge => batches[0].clone(),
            TierId::Vista => scatter_tier(tier, density, BASE_SEED ^ VISTA_SEED_SALT),
            TierId::Near | TierId::Far => scatter_tier(tier, density, BASE_SEED),
        };
        let mesh = &tier_meshes.0[tier_index];
        info!(
            "grass {}: {} tufts x ({} verts, {} tris) = {} tris; band {:?}..{:?} m",
            tier.name,
            batches[tier_index].len(),
            mesh.vertices,
            mesh.triangles,
            batches[tier_index].len() * mesh.triangles,
            tier.fade_in,
            tier.fade_out,
        );
    }
    commands.insert_resource(simple::GrassSimpleParams::from_tiers(&tiers));
    match settings.instancing {
        InstancingMode::Simple => {
            simple::spawn(
                &mut commands,
                &tiers,
                &tier_meshes.0,
                batches,
                settings.grass_shadows,
            );
        }
        InstancingMode::SimpleCulled => {
            culled::spawn(&mut commands, &tiers, &tier_meshes.0, batches);
        }
        InstancingMode::Eidolon => eidolon::spawn(
            &mut commands,
            &mut materials,
            &tiers,
            &tier_meshes.0,
            batches,
            settings.grass_shadows,
        ),
        InstancingMode::None
        | InstancingMode::MeshChunks
        | InstancingMode::MeshChunksMap
        | InstancingMode::Cards
        | InstancingMode::CardsCurved => {}
    }
    info!(
        "grass: {} spawned at density {density:.2} in {} ms",
        settings.instancing.label(),
        started.elapsed().as_millis()
    );
}
