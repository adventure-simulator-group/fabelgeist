//! Turning a sewing pattern into simulated cloth.
//!
//! `fabelgeist-garment-code` produces a [`PatternSpec`]: flat panels, each with a
//! placement around the body, and a list of which edge is sewn to which.
//! `fabelgeist-cloth` wants panels as plain outlines and seams as edge pairs. This
//! is the adapter between them, plus the driver that steps the result against
//! an MHR body.
//!
//! The two libraries know nothing about each other, and neither depends on the
//! other -- which is the point. `fabelgeist-cloth` sits in the prism workspace and
//! would be just as happy with a pattern from anywhere else.
//!
//! [`pose`] is the third piece: the pattern's placement is drawn for a set of
//! measurements rather than for the body in front of it, so the garment can
//! be moved onto that body before the solver is allowed to start.

use fabelgeist_cloth::garment::{Panel, Placement, Seam, SeamSide};
use fabelgeist_cloth::{Cloth, Fabric, GarmentMesh};
use fabelgeist_compute::prelude::KernelCache;
use fabelgeist_garment_code::pattern::PatternSpec;
use fabelgeist_gpu::prelude::WgpuContext;
use fabelgeist_math::{Vec2, Vec3};
use fabelgeist_physics::{Collider, Collisions, MeshCollider, MeshSurface};
use fabelgeist_xpbd::{Solver, SolverSettings, SubstepCount};

pub mod pose;
mod step;

pub use pose::{GarmentPose, Pivots};

/// A sewing pattern is measured in centimetres; the solver works in metres,
/// because that is what gravity is in.
pub const CM_TO_M: f32 = 0.01;

/// How finely a panel outline's curves are sampled before meshing. The mesher
/// resamples anyway, so this only has to be fine enough not to lose the shape
/// of a curved edge -- a neckline, an armhole.
const CURVE_SAMPLES_PER_CM: f64 = 0.5;

/// Simulation settings the tab exposes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitSettings {
    /// Target mesh edge, in centimetres. The resolution knob: it decides the
    /// particle count and the finest fold the fabric can make.
    pub resolution_cm: f32,
    pub substeps: SubstepCount,
    /// How far a garment particle is held off the body surface.
    pub body_offset_cm: f32,
    pub self_collision: bool,
    pub gravity: bool,
    /// The height, in centimetres, of the body the pattern was drawn for.
    ///
    /// A body model and a sewing pattern agree on nothing -- not the unit, not
    /// where the origin is -- so the one thing they can be matched on is how
    /// tall the person is. The pattern's own measurements say; the model is
    /// scaled to it.
    pub body_height_cm: f32,
    /// Substeps between the host-side swept-contact passes `fabelgeist-shell` runs
    /// for self-collision, or 0 to leave contact to the GPU kernels.
    ///
    /// Each pass reads the garment back and waits for the GPU, so this is what
    /// an interleaved step costs. Measured at 12 substeps: every substep
    /// (prism's default) drapes the MHR test body in 542 ms a frame, once a
    /// frame in 57 ms, off in 9.3 ms, and all three drape it the same. A
    /// garment crumpling onto the floor is where they part: every substep
    /// keeps its edges within 50% of rest, off leaves one at 50%, and once a
    /// frame stretches one by 235% -- a sparse pass is worse than none.
    pub host_contact_interval: SubstepCount,
}

impl Default for FitSettings {
    fn default() -> Self {
        Self {
            // Measured: about 1200 particles for a full outfit, which steps in
            // roughly 60 ms. Finer is available on the slider and costs
            // roughly the square.
            resolution_cm: 2.5,
            substeps: 12.into(),
            body_offset_cm: 0.6,
            self_collision: true,
            gravity: true,
            // The bundled GarmentCode bodies are around this tall; the tab
            // overwrites it from whichever body is selected.
            body_height_cm: 164.0,
            host_contact_interval: SubstepCount::ONE,
        }
    }
}

impl FitSettings {
    pub fn body_height_m(&self) -> f32 {
        self.body_height_cm * CM_TO_M
    }
}

/// How many outline edges one pattern edge needs to keep its own shape.
///
/// A straight edge needs no interior samples at all; a curve needs enough that
/// the mesher sees a curve rather than a chord. This is the edge's own demand:
/// [`stitched_segments`] may raise it to match whatever it is sewn to.
fn curve_segments(panel: &fabelgeist_garment_code::pattern::spec::PanelSpec) -> Vec<usize> {
    panel
        .edges
        .iter()
        .map(|edge| {
            if edge.curvature.is_none() {
                1
            } else {
                let curve = fabelgeist_garment_code::pattern::core::edge_as_curve(
                    &panel.vertices,
                    edge,
                    false,
                );
                ((curve.length() * CURVE_SAMPLES_PER_CM).ceil() as usize).clamp(1, 64)
            }
        })
        .collect()
}

/// Raise the two sides of every stitch to a common number of outline edges.
///
/// A seam joins one outline edge to one outline edge, all the way along both.
/// So two pattern edges sewn together have to be cut into the *same* number of
/// pieces: if a curved edge became eleven and the straight edge it is sewn to
/// stayed one, the whole of that straight edge is sewn to each of the eleven in
/// turn, and the constraint gathers it into a bunch a tenth of its width. That
/// is what crushed a circle skirt's waistband: a 46 cm band pulled down to 9,
/// with the skirt bundled up around it.
///
/// Only the demand is shared, never the sampling -- each edge is still sampled
/// along its own curve. The extra points a straight edge gains are collinear,
/// so they cost a few particles and change no shape.
fn stitched_segments(spec: &PatternSpec) -> Vec<Vec<usize>> {
    let mut segments: Vec<Vec<usize>> = spec
        .panels
        .iter()
        .map(|(_, panel)| curve_segments(panel))
        .collect();
    let index: std::collections::HashMap<&str, usize> = spec
        .panels
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (name.as_str(), i))
        .collect();

    // A pass raises both sides of each stitch to their maximum. One pass is
    // not enough on its own: an edge raised here may itself be the smaller
    // side of a stitch already visited -- a chain of edges around a tiered
    // skirt does exactly that -- so it repeats until nothing moves.
    let mut settled = false;
    while !settled {
        settled = true;
        for stitch in &spec.stitches {
            let (Some(&a), Some(&b)) = (
                index.get(stitch.sides[0].panel.as_str()),
                index.get(stitch.sides[1].panel.as_str()),
            ) else {
                continue;
            };
            let (a_edge, b_edge) = (stitch.sides[0].edge, stitch.sides[1].edge);
            if a_edge >= segments[a].len() || b_edge >= segments[b].len() {
                continue;
            }
            let want = segments[a][a_edge].max(segments[b][b_edge]);
            if segments[a][a_edge] != want || segments[b][b_edge] != want {
                segments[a][a_edge] = want;
                segments[b][b_edge] = want;
                settled = false;
            }
        }
    }

    segments
}

/// Sample one panel's outline into a polygon, and remember where each pattern
/// edge starts.
///
/// The returned outline has one point per sample, and `edge_starts[i]` is the
/// index at which pattern edge `i` begins -- which is what lets a stitch
/// declared between two *pattern* edges become a seam between two *outline*
/// edges. Every pattern edge becomes a run of outline edges, so the two
/// numberings have to be translated between rather than assumed equal.
fn sample_outline(
    panel: &fabelgeist_garment_code::pattern::spec::PanelSpec,
    segments: &[usize],
) -> (Vec<Vec2>, Vec<usize>) {
    let mut outline = Vec::new();
    let mut edge_starts = Vec::with_capacity(panel.edges.len());

    for (index, edge) in panel.edges.iter().enumerate() {
        edge_starts.push(outline.len());
        let curve =
            fabelgeist_garment_code::pattern::core::edge_as_curve(&panel.vertices, edge, false);
        let segments = segments.get(index).copied().unwrap_or(1).max(1);
        // The end point is the next edge's start, so it is left for that edge
        // to add -- and the last edge's end is outline[0], closing the loop.
        for step in 0..segments {
            let t = step as f64 / segments as f64;
            let point = curve.point(t);
            outline.push(Vec2::new(point[0] as f32, point[1] as f32));
        }
    }

    (outline, edge_starts)
}

/// The outline edges a pattern edge covers, as a range.
fn outline_edges(edge_starts: &[usize], total: usize, edge: usize) -> std::ops::Range<usize> {
    let start = edge_starts[edge];
    let end = if edge + 1 < edge_starts.len() {
        edge_starts[edge + 1]
    } else {
        total
    };
    start..end
}

/// Everything a fit needs, derived from a pattern.
pub struct GarmentBuild {
    pub mesh: GarmentMesh,
    /// Panels that produced no geometry -- degenerate outlines from a pattern
    /// that did not close.
    pub skipped: Vec<String>,
}

/// Convert a pattern into a meshed garment.
pub fn build_garment(
    spec: &PatternSpec,
    settings: &FitSettings,
    fabric: &Fabric,
) -> anyhow::Result<GarmentBuild> {
    let mut panels = Vec::new();
    let mut panel_index = std::collections::HashMap::new();
    let mut edge_maps = Vec::new();
    let mut skipped = Vec::new();

    let segments = stitched_segments(spec);
    for (index, (name, spec_panel)) in spec.panels.iter().enumerate() {
        let (outline, edge_starts) = sample_outline(spec_panel, &segments[index]);
        if outline.len() < 3 {
            skipped.push(name.clone());
            continue;
        }

        let outline: Vec<Vec2> = outline.into_iter().map(|p| p * CM_TO_M).collect();
        let placement = Placement {
            translation: Vec3::new(
                spec_panel.translation[0] as f32 * CM_TO_M,
                spec_panel.translation[1] as f32 * CM_TO_M,
                spec_panel.translation[2] as f32 * CM_TO_M,
            ),
            rotation: Vec3::new(
                spec_panel.rotation[0] as f32,
                spec_panel.rotation[1] as f32,
                spec_panel.rotation[2] as f32,
            ),
        };

        panel_index.insert(name.clone(), panels.len());
        edge_maps.push((edge_starts, outline.len()));
        panels.push(Panel::new(name.clone(), outline).placed(placement));
    }

    if panels.is_empty() {
        anyhow::bail!("the pattern has no usable panels");
    }

    // A stitch joins two pattern edges. Each of those covers a run of outline
    // edges, so one stitch becomes several seams -- paired up along the run so
    // that the fabric is sewn all the way along rather than only at the ends.
    let mut seams = Vec::new();
    for stitch in &spec.stitches {
        let (Some(&a_panel), Some(&b_panel)) = (
            panel_index.get(&stitch.sides[0].panel),
            panel_index.get(&stitch.sides[1].panel),
        ) else {
            continue;
        };

        let (a_starts, a_total) = &edge_maps[a_panel];
        let (b_starts, b_total) = &edge_maps[b_panel];
        if stitch.sides[0].edge >= a_starts.len() || stitch.sides[1].edge >= b_starts.len() {
            continue;
        }

        let a_range = outline_edges(a_starts, *a_total, stitch.sides[0].edge);
        let b_range = outline_edges(b_starts, *b_total, stitch.sides[1].edge);

        // Which way round the two runs meet is decided geometrically, by
        // trying both and keeping the shorter total. A pattern format's own
        // convention is easy to get backwards, and getting it backwards
        // produces a garment with a twist in it that still looks plausible
        // until you turn it round.
        let reversed = should_reverse(&panels, a_panel, &a_range, b_panel, &b_range);

        let steps = a_range.len().max(b_range.len());
        for step in 0..steps {
            let t = if steps > 1 {
                step as f32 / (steps - 1) as f32
            } else {
                0.0
            };
            let a_edge = a_range.start + pick(t, a_range.len());
            let b_step = if reversed { 1.0 - t } else { t };
            let b_edge = b_range.start + pick(b_step, b_range.len());
            seams.push(
                Seam::new(
                    SeamSide {
                        panel: a_panel,
                        edge: a_edge,
                    },
                    SeamSide {
                        panel: b_panel,
                        edge: b_edge,
                    },
                )
                .reversed(reversed),
            );
        }
    }

    let mesh = fabelgeist_cloth::build(
        &panels,
        &seams,
        settings.resolution_cm * CM_TO_M,
        fabric.density,
    )?;

    Ok(GarmentBuild { mesh, skipped })
}

fn pick(t: f32, count: usize) -> usize {
    ((t * (count.saturating_sub(1)) as f32).round() as usize).min(count.saturating_sub(1))
}

/// Whether pattern edge `b` runs the opposite way to `a` along the seam.
fn should_reverse(
    panels: &[Panel],
    a_panel: usize,
    a_range: &std::ops::Range<usize>,
    b_panel: usize,
    b_range: &std::ops::Range<usize>,
) -> bool {
    let point = |panel: usize, vertex: usize| {
        let panel = &panels[panel];
        panel
            .placement
            .apply(panel.outline[vertex % panel.outline.len()])
    };

    let a_start = point(a_panel, a_range.start);
    let a_end = point(a_panel, a_range.end);
    let b_start = point(b_panel, b_range.start);
    let b_end = point(b_panel, b_range.end);

    let aligned = (a_start - b_start).length() + (a_end - b_end).length();
    let crossed = (a_start - b_end).length() + (a_end - b_start).length();
    crossed < aligned
}

/// A garment being fitted: the cloth, the solver, and what it collides with.
///
/// The solver runs on a device of its **own**, never the one the application
/// presents its window from. Sharing the two couples the simulation to the
/// compositor: a long dispatch starves it, an error scope swallows its errors,
/// and losing the device takes the window down. Isolating them costs a copy of
/// the positions per frame -- nineteen kilobytes for a garment -- and removes
/// the whole class of failure.
pub struct Fit {
    pub context: WgpuContext,
    pub cloth: Cloth,
    pub collisions: Collisions,
    pub solver: Solver,
    cache: KernelCache,
    pub frames: u64,
    /// The body on the host, kept for the one-off recovery that lifts garment
    /// particles out of it. See [`Fit::clear_body`].
    body: Option<fabelgeist_bvh::TriangleBvh>,
    /// Where the pattern put the panels, kept unposed.
    ///
    /// Every pose is composed against these rather than against wherever the
    /// garment currently is, so that moving it and moving it back is exact
    /// and a pose means a place rather than a journey.
    base_positions: Vec<Vec3>,
    pivots: Pivots,
    pose: GarmentPose,
}

impl Fit {
    /// Build a fit on the application's own device.
    pub fn new(
        context: WgpuContext,
        build: &GarmentBuild,
        fabric: Fabric,
        settings: &FitSettings,
    ) -> anyhow::Result<Self> {
        check_device(&context)?;

        let cache = KernelCache::new();
        let mut cloth = Cloth::new(&context, &cache, &build.mesh, fabric)?;
        cloth.self_collision.enabled = settings.self_collision;
        cloth.shell.host_contacts.interval_substeps = settings.host_contact_interval;

        let mut collisions = Collisions::new(&context, &cache)?;
        // A floor well below the body, so a garment that slips off lands
        // somewhere visible instead of falling forever.
        collisions.set_colliders(&context, vec![Collider::ground(-1.0).with_friction(0.5)])?;

        let solver = Solver::with_cache(
            &context,
            &cache,
            SolverSettings {
                substeps: settings.substeps,
                gravity: if settings.gravity {
                    Vec3::new(0.0, -9.81, 0.0)
                } else {
                    Vec3::default()
                }
                .into(),
                damping: fabric.damping.into(),
                ..Default::default()
            },
        )?;

        Ok(Self {
            context,
            cloth,
            collisions,
            solver,
            cache,
            frames: 0,
            body: None,
            base_positions: build.mesh.positions.clone(),
            pivots: Pivots::of(&build.mesh),
            pose: GarmentPose::for_mesh(&build.mesh),
        })
    }

    /// Give the fit a body to drape over.
    pub async fn set_body(
        &mut self,
        vertices: &[Vec3],
        triangles: &[[u32; 3]],
        settings: &FitSettings,
        fabric: &Fabric,
    ) -> anyhow::Result<()> {
        self.set_collision_mesh(vertices, triangles, settings, fabric)?;
        self.body = Some(fabelgeist_bvh::TriangleBvh::new(
            vertices.to_vec(),
            triangles.to_vec(),
        ));

        self.clear_body(settings).await
    }

    pub fn set_collision_mesh(
        &mut self,
        vertices: &[Vec3],
        triangles: &[[u32; 3]],
        settings: &FitSettings,
        fabric: &Fabric,
    ) -> anyhow::Result<()> {
        let surface = MeshSurface {
            thickness: settings.body_offset_cm * CM_TO_M,
            friction: fabric.friction,
        };
        let mesh = MeshCollider::new(
            &self.context,
            &self.cache,
            fabelgeist_bvh::gpu::BvhKernels::with_cache(&self.context, &self.cache)?,
            vertices,
            triangles,
            surface,
        )?;
        self.collisions.set_mesh(Some(mesh));
        Ok(())
    }

    /// The posed positions on the host, before any body clearance.
    ///
    /// What a renderer needs to show the garment as it has been placed,
    /// without waiting on a readback from the solver.
    pub fn placed_positions(&self) -> Vec<Vec3> {
        self.pose.place(&self.base_positions, &self.pivots)
    }

    /// Put the garment somewhere else, at rest.
    ///
    /// No rebuild: a pose is rigid, so every rest length, bending weight and
    /// mass the constraints were built with still holds, and only the
    /// positions move. Velocities go back to zero with them -- this is a
    /// placement, not a shove -- and the frame count restarts, because a
    /// garment that has been moved is no longer the drape that was running.
    pub fn set_pose(&mut self, pose: GarmentPose) -> anyhow::Result<()> {
        let positions = pose.place(&self.base_positions, &self.pivots);
        self.cloth
            .particles
            .write_positions(&self.context, &positions)?;
        self.pose = pose;
        self.frames = 0;
        Ok(())
    }

    /// Lift every garment particle that sits inside the body out to its
    /// surface, once, before the first step.
    ///
    /// A garment's initial layout puts panels straight through the body, and a
    /// substep cannot fix that: it looks only as far as the collision shell,
    /// and from inside a torso there is no triangle in that range.
    ///
    /// This runs on the **host**, against `fabelgeist-bvh`'s own tree, and that is
    /// deliberate. The GPU version of the same idea has to be told how far to
    /// look, and the honest answer for "somewhere inside a body" is about a
    /// quarter of a metre -- which turns every particle's query into a box
    /// that matches a large fraction of the body's triangles. Thousands of
    /// particles against thousands of triangles in one dispatch runs for
    /// seconds, and a compute dispatch that runs for seconds is what Windows'
    /// display watchdog exists to kill: it resets the GPU, which takes the
    /// device, the compositor and the window with it.
    ///
    /// On the host the same query is exact rather than radius-limited, prunes
    /// properly, costs a few milliseconds for a whole garment, and cannot
    /// stall a display driver. It happens once, so nothing is lost by it.
    pub async fn clear_body(&mut self, settings: &FitSettings) -> anyhow::Result<()> {
        let Some(body) = &self.body else {
            return Ok(());
        };

        let offset = settings.body_offset_cm * CM_TO_M + self.cloth.fabric.particle_radius();
        let mut positions = self.cloth.read_positions(&self.context).await?;

        // Generous, but a *distance* bound rather than a box: the tree prunes
        // on it, so a particle already outside the body stops almost at once.
        let reach = 0.5f32;
        let mut lifted = 0usize;

        for position in &mut positions {
            let Some((triangle, closest, distance)) = body.closest_point(*position, reach) else {
                continue;
            };
            let (a, b, c) = body.triangle(triangle);
            let normal = (b - a).cross(c - a);
            let length = normal.length();
            if length < 1e-12 {
                continue;
            }
            let normal = normal / length;

            // Which side of the surface the particle is on. The face normal
            // decides, because the direction to the closest point says nothing
            // once the particle is on the surface.
            let outward = (*position - closest).dot(normal);
            if outward < 0.0 {
                *position = closest + normal * offset;
                lifted += 1;
            } else if distance < offset {
                let direction = if distance > 1e-6 {
                    (*position - closest) / distance
                } else {
                    normal
                };
                *position = closest + direction * offset;
                lifted += 1;
            }
        }

        if lifted > 0 {
            self.cloth
                .particles
                .write_positions(&self.context, &positions)?;
        }
        Ok(())
    }

    /// The garment's current positions on the host.
    #[cfg_attr(not(test), allow(dead_code))]
    /// What a caller needs to read the positions back *without* holding a
    /// lock on the fit.
    ///
    /// Both are cheap handles. The read is asynchronous, and holding a
    /// blocking lock across it would stall every other handler on the thread.
    pub fn readback_handles(&self) -> (WgpuContext, fabelgeist_gpu::prelude::Buffer) {
        (self.context.clone(), self.cloth.particles.positions.clone())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn positions(&self) -> anyhow::Result<Vec<[f32; 3]>> {
        Ok(self
            .cloth
            .read_positions(&self.context)
            .await?
            .into_iter()
            .map(|p| [p.x, p.y, p.z])
            .collect())
    }

    /// Put the garment back where it was placed, at rest.
    ///
    /// `set_pose` writes every particle from the pattern's own positions, so
    /// it subsumes `Cloth::reset` -- and unlike it, keeps the garment where
    /// it was put. Reset undoes the simulation, not the positioning.
    pub async fn reset(&mut self, settings: &FitSettings) -> anyhow::Result<()> {
        let pose = self.pose.clone();
        self.set_pose(pose)?;
        self.clear_body(settings).await
    }
}

/// Refuse a device that cannot run the solver, with an explanation.
///
/// The solver is all storage buffers and compute, and a device created with
/// the WebGL2 baseline limits allows neither -- every buffer comes back
/// invalid and the first write panics inside wgpu's validation, far from
/// anything that names the cause. Checking up front turns that into a line of
/// text in the tab.
fn check_device(context: &WgpuContext) -> anyhow::Result<()> {
    // The widest kernel here binds seven storage buffers (mesh collision:
    // positions, previous, mesh positions, mesh triangles, and three for the
    // hierarchy) plus a uniform.
    const STORAGE_BUFFERS: u32 = 8;
    const WORKGROUP_SIZE: u32 = 256;

    let limits = context.device.limits();
    if limits.max_storage_buffers_per_shader_stage < STORAGE_BUFFERS {
        anyhow::bail!(
            "this GPU device allows {} storage buffers per stage and the solver needs {STORAGE_BUFFERS};              it was probably created with the WebGL2 baseline limits",
            limits.max_storage_buffers_per_shader_stage
        );
    }
    if limits.max_compute_invocations_per_workgroup < WORKGROUP_SIZE {
        anyhow::bail!(
            "this GPU device allows {} compute invocations per workgroup and the solver needs {WORKGROUP_SIZE}",
            limits.max_compute_invocations_per_workgroup
        );
    }
    Ok(())
}

/// Scale and centre a body mesh so that it stands the given height, in metres,
/// with its feet at the origin.
///
/// The body model and the pattern come from different worlds and agree on
/// nothing -- not the unit, not where the origin is. The pattern's own body
/// measurements give a height in centimetres, and matching the model to it is
/// what puts the garment on the body rather than beside it.
pub fn normalize_body(vertices: &[[f32; 3]], target_height_m: f32) -> Vec<Vec3> {
    if vertices.is_empty() {
        return Vec::new();
    }

    let mut lowest = [f32::INFINITY; 3];
    let mut highest = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        for axis in 0..3 {
            lowest[axis] = lowest[axis].min(vertex[axis]);
            highest[axis] = highest[axis].max(vertex[axis]);
        }
    }

    let height = highest[1] - lowest[1];
    let scale = if height > 1e-6 {
        target_height_m / height
    } else {
        1.0
    };
    let center_x = (lowest[0] + highest[0]) * 0.5;
    let center_z = (lowest[2] + highest[2]) * 0.5;

    vertices
        .iter()
        .map(|vertex| {
            Vec3::new(
                (vertex[0] - center_x) * scale,
                (vertex[1] - lowest[1]) * scale,
                (vertex[2] - center_z) * scale,
            )
        })
        .collect()
}

#[cfg(test)]
mod pose_tests;
#[cfg(test)]
mod tests;
