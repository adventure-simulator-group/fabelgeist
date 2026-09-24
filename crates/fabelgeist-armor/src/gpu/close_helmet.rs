//! The close helmet on the device, placed by the head frame and shaped by
//! sections measured on the wearer.
//!
//! The helmet's topology -- the styled bowl, the jaw and neck rows below it,
//! the nape lames, the bevor grid and the visor's pierced domain -- depends
//! on the design alone. It is built once per design on the host and cached,
//! the visor's constrained triangulation above all. Every position is
//! evaluated on the device from the fit buffer: the head frame's
//! [`FIT_PROFILE_WORD`] words, then the wearer's [`super::CloseHelmetProfile`].

use std::f32::consts::{FRAC_PI_2, PI};
use std::sync::{Arc, Mutex};

use fabelgeist_compute::{KernelBatch, host_float};
use fabelgeist_gpu::prelude::Buffer;

use super::close_helmet_dome::{AROUND, DOME, flute_words, styled_dome};
use super::close_helmet_shape::{DESIGN, SHAPE};
use super::coord::{CoordExtrusion, CoordKernel, CoordShell};
use super::coord_topology::CoordTopology;
use super::recipe::PartRecipe;
use super::{ArmorGpu, DevicePart};
use crate::helmets::visor_domain::{HALF_WIDTH_MM, HEIGHT_MM, VisorDomain};
use crate::{ArmorComponentRole, BoundaryNormals, CloseHelmetDesign, GenerateError, HelmetDesign};

/// Where a fit buffer's wearer profile starts: after the device frame's
/// origin, axes, half extents and landmark span.
pub const FIT_PROFILE_WORD: usize = 16;
/// Floats of a measured [`super::CloseHelmetProfile`] in a fit buffer.
pub const CLOSE_HELMET_PROFILE_WORDS: usize = 11;

const JAW_ROWS: usize = 12;
const NECK_ROWS: usize = 6;
const BEVOR_COLUMNS: usize = 32;
const SIDE_WRAP_RADIANS: f32 = PI * 0.55;
const NAPE_LAMES: usize = 3;
const NAPE_LAME_ROWS: usize = 4;
const NAPE_LAME_COLUMNS: usize = 24;
const NAPE_LAME_OVERLAP: f32 = 0.06;
/// The skull's jaw and neck rows, in a coordinate's fourth float.
const KIND_JAW_ROW: f32 = 4.0;
/// Each shell's role in the shape, in its first shell value.
const SHELL_SKULL: f32 = 0.0;
const SHELL_NAPE: f32 = 1.0;
const SHELL_BEVOR: f32 = 2.0;
const SHELL_VISOR: f32 = 3.0;
/// Floats per vertex in a shell's turn table.
const TURN_WORDS: usize = 8;
const NORMAL_SAMPLE_ANGLE: f32 = 0.001;
const NAPE_HALF_WRAP: f32 = PI * 7.0 / 18.0;
const NAPE_SAMPLE: f32 = 0.001;
/// Designs whose layouts are kept; a fit and its morph samples share one.
const CACHED_LAYOUTS: usize = 8;

/// One shell of the helmet: its topology, its role in the shape, and where
/// its turn table starts among the design floats.
struct HelmetShell {
    surface: CoordTopology,
    role: Role,
    table: usize,
}

/// Which part of the helmet a shell is, as the shape tells them apart.
#[derive(Clone, Copy, PartialEq)]
enum Role {
    Skull,
    Nape,
    Bevor,
    Visor,
}

impl Role {
    fn value(self) -> f32 {
        match self {
            Self::Skull => SHELL_SKULL,
            Self::Nape => SHELL_NAPE,
            Self::Bevor => SHELL_BEVOR,
            Self::Visor => SHELL_VISOR,
        }
    }

    /// Each vertex's turns: the sine and cosine of its design angle and of a
    /// finite-difference sample either side, then one more design value and
    /// a visor vertex's bellows relief.
    fn turns(self, coord: [f32; 4], d: &CloseHelmetDesign) -> [f32; TURN_WORDS] {
        let around = |angle: f32| {
            [
                angle,
                angle + NORMAL_SAMPLE_ANGLE,
                angle - NORMAL_SAMPLE_ANGLE,
            ]
        };
        let bellows = match self {
            Self::Visor => d.bellows.map_or(0.0, |bellows| bellows.relief(coord[1])),
            _ => 0.0,
        };
        let (angles, extra) = match self {
            Self::Skull if coord[3] == KIND_JAW_ROW => ([coord[0]; 3], 0.0),
            Self::Skull => return [0.0; TURN_WORDS],
            Self::Nape => {
                let u = coord[0];
                (
                    [u, u + NAPE_SAMPLE, u - NAPE_SAMPLE].map(|u| PI + NAPE_HALF_WRAP * u),
                    0.0,
                )
            }
            Self::Bevor => (around(coord[0]), 0.0),
            Self::Visor => (around(coord[0]), visor_ridge_rounding(d, coord[1])),
        };
        let mut words = [0.0; TURN_WORDS];
        for (k, angle) in angles.iter().enumerate() {
            words[2 * k] = angle.sin();
            words[2 * k + 1] = angle.cos();
        }
        words[TURN_WORDS - 2] = extra;
        words[TURN_WORDS - 1] = bellows;
        words
    }
}

/// The visor ridge's rounded profile `t` of the way down the face.
fn visor_ridge_rounding(d: &CloseHelmetDesign, t: f32) -> f32 {
    let ridge = d.ridge_height.unit();
    let ramp = if t <= ridge {
        t / ridge
    } else {
        (1.0 - t) / (1.0 - ridge)
    };
    (ramp.clamp(0.0, 1.0) * FRAC_PI_2).sin()
}

/// A close helmet's shells and design floats, decided by its design.
struct CloseHelmetLayout {
    shells: Vec<HelmetShell>,
    design: Vec<f32>,
    wall: f32,
}

impl CloseHelmetLayout {
    fn new(d: &CloseHelmetDesign) -> Result<Self, GenerateError> {
        HelmetDesign::CloseHelmet(*d).validate()?;
        let mut surfaces = vec![(skull(d), Role::Skull)];
        if d.nape_length.0 > 0 {
            surfaces.extend(napes().into_iter().map(|nape| (nape, Role::Nape)));
        }
        surfaces.push((bevor(), Role::Bevor));
        surfaces.push((visor(d)?, Role::Visor));
        let mut design = design_words(d);
        let shells = surfaces
            .into_iter()
            .map(|(surface, role)| {
                let table = design.len();
                for coord in &surface.coords {
                    design.extend(role.turns(*coord, d));
                }
                HelmetShell {
                    surface,
                    role,
                    table,
                }
            })
            .collect();
        Ok(Self {
            shells,
            design,
            wall: d.fit.wall_thickness.metres(),
        })
    }

    /// The layout of a design, built once.
    fn cached(d: &CloseHelmetDesign) -> Result<Arc<Self>, GenerateError> {
        crate::device_support::on_device(
            crate::HelmetDesign::CloseHelmet(d.clone()).device_unsupported(),
        )?;
        static LAYOUTS: Mutex<Vec<(CloseHelmetDesign, Arc<CloseHelmetLayout>)>> =
            Mutex::new(Vec::new());
        let cached = LAYOUTS
            .lock()
            .expect("close helmet layouts")
            .iter()
            .find(|(design, _)| design == d)
            .map(|(_, layout)| layout.clone());
        if let Some(layout) = cached {
            return Ok(layout);
        }
        let layout = Arc::new(Self::new(d)?);
        let mut layouts = LAYOUTS.lock().expect("close helmet layouts");
        if layouts.len() == CACHED_LAYOUTS {
            layouts.remove(0);
        }
        layouts.push((*d, layout.clone()));
        Ok(layout)
    }

    fn recipe(&self, gpu: &ArmorGpu) -> Result<PartRecipe, GenerateError> {
        let source = format!(
            "{DESIGN}{}{}{DOME}{SHAPE}",
            host_float::zero_hook("bitcast<u32>(design[ZERO])"),
            host_float::wgsl()
        );
        let kernel = CoordKernel::new(gpu, &source)?;
        let mut recipe = PartRecipe::new();
        let hinge = recipe.hinge();
        for shell in &self.shells {
            // Each component closes where the next begins.
            match shell.role {
                Role::Bevor => recipe.component(ArmorComponentRole::Skull, None),
                Role::Visor => recipe.component(ArmorComponentRole::Bevor, Some(hinge)),
                Role::Skull | Role::Nape => {}
            }
            let role = shell.role;
            let visor = role == Role::Visor;
            recipe.push_coord(
                CoordShell {
                    coords: shell.surface.coords.clone(),
                    indices: shell.surface.indices.clone(),
                    boundary_normals: if visor {
                        BoundaryNormals::Separate
                    } else {
                        BoundaryNormals::Smooth
                    },
                    thickness: self.wall,
                    // The visor thickens radially about the skull's axis.
                    extrusion: if visor {
                        CoordExtrusion::Radial {
                            origin: [0.0; 3],
                            axis: [0.0, 1.0, 0.0],
                        }
                    } else {
                        CoordExtrusion::Normal
                    },
                    values: [role.value(), shell.table as f32, 0.0, 0.0],
                    mirrored: false,
                    frame: 0,
                    passes: 1,
                    hinge: (role == Role::Bevor).then_some(hinge),
                },
                kernel.clone(),
            )?;
        }
        recipe.component(ArmorComponentRole::Visor, Some(hinge));
        Ok(recipe)
    }
}

/// The design floats the shape reads, in `close_helmet_shape::DESIGN`'s order.
fn design_words(d: &CloseHelmetDesign) -> Vec<f32> {
    let mut words = vec![
        d.fit.wall_thickness.metres(),
        d.fit.clearance.metres() + d.fit.wall_thickness.metres(),
        d.fit.crown_height.unit(),
        d.back_edge_lift.metres(),
        d.neck_length.metres(),
        d.jaw_width.unit(),
        d.neck_width.unit(),
        d.throat_flare.metres(),
        d.back_flare.metres(),
        d.visor_projection.metres(),
        d.chin_projection.metres(),
        d.nape_length.metres(),
        d.nape_flare.metres(),
        d.ridge_height.unit(),
        d.ridge_sharpness.unit(),
        d.sight_ledge.metres(),
        d.crown.fullness.unit(),
        d.crown.ridge_height.metres(),
        d.comb_height.metres(),
        f32::from(u8::from(d.crown.fluting.is_some())),
        SIDE_WRAP_RADIANS.cos(),
        // A zero the device reads at run time, for exact arithmetic.
        0.0,
    ];
    words.extend(flute_words(d.crown.fluting.as_ref()));
    words.push(d.bellows.map_or(0.0, |bellows| bellows.cheek_rise.metres()));
    words
}

/// The styled bowl, then the jaw and neck rows hung from its rim's face half.
fn skull(d: &CloseHelmetDesign) -> CoordTopology {
    let mut surface = CoordTopology::default();
    let rim = styled_dome(&mut surface, &d.crown, d.comb_height.metres());
    let first = AROUND / 4;
    let last = AROUND - first;
    let mut previous = rim[first..=last].to_vec();
    for row in 1..=JAW_ROWS + NECK_ROWS {
        let ring = (first..=last)
            .map(|i| {
                let angle = i as f32 / AROUND as f32 * PI * 2.0;
                surface.vertex([angle, row as f32, 0.0, KIND_JAW_ROW])
            })
            .collect::<Vec<_>>();
        surface.connect(&previous, &ring, false);
        previous = ring;
    }
    surface
}

/// Three overlapping nape lames, each a layer further out than the next.
fn napes() -> Vec<CoordTopology> {
    (0..NAPE_LAMES)
        .map(|lame| {
            let start = if lame == 0 {
                0.0
            } else {
                lame as f32 / NAPE_LAMES as f32 - NAPE_LAME_OVERLAP
            };
            let end = (lame + 1) as f32 / NAPE_LAMES as f32;
            let layer = (NAPE_LAMES - lame) as f32;
            let mut surface = CoordTopology::default();
            let mut previous = Vec::new();
            for row in 0..=NAPE_LAME_ROWS {
                let t = start + (end - start) * row as f32 / NAPE_LAME_ROWS as f32;
                let ring = (0..=NAPE_LAME_COLUMNS)
                    .map(|column| {
                        let u = column as f32 / NAPE_LAME_COLUMNS as f32 * 2.0 - 1.0;
                        surface.vertex([u, t, layer, 0.0])
                    })
                    .collect::<Vec<_>>();
                if !previous.is_empty() {
                    surface.connect(&previous, &ring, false);
                }
                previous = ring;
            }
            surface
        })
        .collect()
}

fn bevor() -> CoordTopology {
    let mut surface = CoordTopology::default();
    let mut previous = Vec::new();
    for row in 0..=JAW_ROWS + NECK_ROWS {
        let ring = (0..=BEVOR_COLUMNS)
            .map(|column| {
                let angle = (column as f32 / BEVOR_COLUMNS as f32 * 2.0 - 1.0) * SIDE_WRAP_RADIANS;
                surface.vertex([angle, row as f32, 0.0, 0.0])
            })
            .collect::<Vec<_>>();
        if !previous.is_empty() {
            surface.connect(&previous, &ring, false);
        }
        previous = ring;
    }
    surface
}

/// The visor's triangulated domain, as angles around the head and fractions
/// down the face, with the flute relief it carries when fluted.
fn visor(d: &CloseHelmetDesign) -> Result<CoordTopology, GenerateError> {
    let domain = VisorDomain::new(d, crate::ArmorDetail::BakeSource)?;
    let fluted = d.visor_fluting.is_some();
    Ok(CoordTopology {
        coords: domain
            .points
            .iter()
            .zip(&domain.relief)
            .map(|(p, relief)| {
                [
                    p[0] as f32 / HALF_WIDTH_MM * SIDE_WRAP_RADIANS,
                    p[1] as f32 / HEIGHT_MM,
                    if fluted { *relief } else { 0.0 },
                    0.0,
                ]
            })
            .collect(),
        indices: domain.indices,
    })
}

/// Record a close helmet into a new device part.
///
/// The helmet is evaluated in its own frame and placed by `frame`, the head
/// frame at the start of that buffer. `fit` is the helmet's own frame --
/// identity axes at the origin, with the head frame's half extents -- in
/// [`FIT_PROFILE_WORD`] floats, then the wearer's
/// [`CLOSE_HELMET_PROFILE_WORDS`] profile floats in field order.
pub fn record_close_helmet(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    design: &CloseHelmetDesign,
    fit: &Buffer,
    frame: &Buffer,
) -> Result<DevicePart, GenerateError> {
    let layout = CloseHelmetLayout::cached(design)?;
    let mut part = layout
        .recipe(gpu)?
        .record(gpu, batch, &layout.design, &[fit])?;
    part.place_by(frame);
    Ok(part)
}
