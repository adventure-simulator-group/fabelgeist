//! Horizontal overlapping courses sampled from the completed fitted torso.
use fabelgeist_compute::{KernelBatch, NormalWeighting, VertexNormals};
use fabelgeist_gpu::prelude::Buffer;

use super::kernels::{Params, dispatch};
use super::plates::{Plates, Shell};
use super::skin_wgsl::SKIN_WORDS;
use super::topology::{MidTopology, OUTER_BIT, SolidTopology};
use crate::gpu::{ArmorGpu, device_error};
use crate::{AnimeDesign, ArmorComponent, ArmorComponentRole, GenerateError};

const COURSE_ROWS: usize = 8;
const UPPER_PLATE_ROWS: usize = 24;
const LINK_WORDS: u64 = 3;

/// Original solid vertices retained only for morph correspondence. Runtime
/// wearer fits have no morphs; they still use the same construction pipeline.
pub(super) struct Articulation {
    pub source: Shell,
    links: Buffer,
    pub components: Vec<ArmorComponent>,
}

pub(super) struct CourseInputs<'a> {
    pub plates: &'a Plates,
    pub frame: &'a Buffer,
    pub skin: &'a Buffer,
    pub status: &'a Buffer,
}

pub(super) struct Articulated {
    pub shell: Shell,
    pub skin: Buffer,
    pub correspondence: Articulation,
}

struct Layout {
    topology: SolidTopology,
    coordinates: Vec<[u32; 4]>,
    references: Vec<[u32; 2]>,
    components: Vec<ArmorComponent>,
}

impl Layout {
    fn new(plates: &Plates, source: &Shell, count: usize) -> Result<Self, GenerateError> {
        let mut layout = Self {
            components: Vec::new(),
            topology: SolidTopology::default(),
            coordinates: Vec::new(),
            references: vec![[u32::MAX; 2]; (plates.front.count() + plates.back.count()) as usize],
        };
        for (vertex, &mid) in source.topology.sources.iter().enumerate() {
            layout.references[(mid & !OUTER_BIT) as usize][usize::from(mid & OUTER_BIT != 0)] =
                vertex as u32;
        }
        let mut mid_offset = 0;
        for (side, plate) in [&plates.front, &plates.back].into_iter().enumerate() {
            for course in 0..=count {
                let rows = if course == count {
                    UPPER_PLATE_ROWS
                } else {
                    COURSE_ROWS
                };
                let mid = MidTopology::course(
                    side != 0,
                    plate.topology.columns.clone(),
                    rows + 1,
                    !plate.topology.medial_crease.is_empty(),
                );
                let solid = SolidTopology::new(&mid)?;
                layout.components.push(ArmorComponent {
                    role: ArmorComponentRole::Plate,
                    vertices: layout.topology.sources.len()
                        ..layout.topology.sources.len() + solid.sources.len(),
                    indices: layout.topology.indices.len()
                        ..layout.topology.indices.len() + solid.indices.len(),
                    hinge: None,
                    material: None,
                });
                for &source in &solid.sources {
                    let vertex = (source & !OUTER_BIT) as usize;
                    layout.coordinates.push([
                        (vertex % mid.width()) as u32,
                        ((vertex / mid.width()) as f32 / rows as f32).to_bits(),
                        course as u32,
                        side as u32 | (source & OUTER_BIT),
                    ]);
                }
                layout.topology.extend(solid, mid_offset);
                mid_offset += mid.vertex_count() as u32;
            }
        }
        Ok(layout)
    }
}

impl Articulation {
    pub(super) fn record(
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        design: &AnimeDesign,
        source: Shell,
        input: CourseInputs<'_>,
    ) -> Result<Articulated, GenerateError> {
        let layout = Layout::new(input.plates, &source, usize::from(design.lame_count))?;
        let count = layout.coordinates.len() as u32;
        let triangles = (layout.topology.indices.len() / 3) as u32;
        let mut shell = Shell {
            sources: gpu.upload(&layout.topology.sources)?,
            indices: gpu.upload(&layout.topology.indices)?,
            positions: gpu.scratch(count as u64 * 12, "anime positions")?,
            normals: VertexNormals::new(gpu.context(), count, triangles).map_err(device_error)?,
            topology: layout.topology,
        };
        let links = gpu.scratch(count as u64 * LINK_WORDS * 4, "anime correspondence")?;
        let skin = gpu.scratch(count as u64 * SKIN_WORDS as u64 * 4, "anime skin")?;
        let design = gpu.upload(&design_words(design))?;
        let bounds = gpu.scratch(16, "anime course bounds")?;
        let params = Params {
            count,
            width: input.plates.front.width(),
            extra: input.plates.back.width(),
            front_count: input.plates.front.count(),
            ..Params::default()
        };
        dispatch(
            gpu,
            batch,
            include_str!("anime_bounds.wgsl"),
            &[("original", false)],
            params,
            &[
                ("plate", input.frame),
                ("original", &source.positions),
                ("references", &gpu.upload(&layout.references)?),
                ("design", &design),
                ("bounds", &bounds),
                ("status", input.status),
            ],
            2,
        )?;
        dispatch(
            gpu,
            batch,
            include_str!("anime_resample.wgsl"),
            &[("original", false), ("positions", true)],
            params,
            &[
                ("plate", input.frame),
                ("original", &source.positions),
                ("references", &gpu.upload(&layout.references)?),
                ("design", &design),
                ("bounds", &bounds),
                ("coordinates", &gpu.upload(&layout.coordinates)?),
                ("positions", &shell.positions),
                ("links", &links),
            ],
            count,
        )?;
        dispatch(
            gpu,
            batch,
            include_str!("anime_skin.wgsl"),
            &[],
            params,
            &[
                ("links", &links),
                ("original_skin", input.skin),
                ("skin", &skin),
            ],
            count,
        )?;
        gpu.normals(NormalWeighting::Area)
            .record(
                batch,
                &shell.positions,
                &shell.indices,
                count,
                triangles,
                &mut shell.normals,
            )
            .map_err(device_error)?;
        Ok(Articulated {
            shell,
            skin,
            correspondence: Self {
                source,
                links,
                components: layout.components,
            },
        })
    }

    /// Apply the original carrier's morph displacement to each new vertex;
    /// the course's lift and wall thickness remain part of its fitted shape.
    pub(super) fn record_morph(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        base: &Shell,
        target: &Buffer,
    ) -> Result<Buffer, GenerateError> {
        let positions = gpu.scratch(base.count() as u64 * 12, "anime morph")?;
        dispatch(
            gpu,
            batch,
            include_str!("anime_morph.wgsl"),
            &[
                ("original", false),
                ("target_body", false),
                ("base", false),
                ("positions", true),
            ],
            Params::counted(base.count()),
            &[
                ("original", &self.source.positions),
                ("target_body", target),
                ("base", &base.positions),
                ("links", &self.links),
                ("positions", &positions),
            ],
            base.count(),
        )?;
        Ok(positions)
    }
}

fn design_words(design: &AnimeDesign) -> Vec<f32> {
    vec![
        f32::from(design.lame_count),
        design.articulated_height.unit(),
        design.overlap.metres(),
        design.chevron_slope.unit(),
        design.rear_chevron_slope.unit(),
        design.lap_lift.metres(),
    ]
}
