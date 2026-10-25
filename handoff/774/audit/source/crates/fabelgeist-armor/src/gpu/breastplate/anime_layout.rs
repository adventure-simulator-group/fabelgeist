//! Course connectivity cut directly from the evaluated carrier facets.
use super::course_clip::CourseClip;
use super::cut_frame::CourseFrame;
use super::plates::Plates;
use super::topology::{OUTER_BIT, SolidTopology};
use crate::{
    AnimeDesign, ArmorComponent, ArmorComponentRole, GenerateError, Permille, PlateCourse,
    PlateGridEnd, PlateJointMotion, PlateMount, PlateParent,
};

/// Packed course queries reserve the two high bits for wall and side.
pub(super) const REAR_BIT: u32 = 1 << 30;

pub(super) struct Layout {
    pub topology: SolidTopology,
    pub coordinates: Vec<[u32; 4]>,
    pub components: Vec<ArmorComponent>,
    pub bounds: Vec<[f32; 4]>,
}

impl Layout {
    pub fn new(
        plates: &Plates,
        design: &AnimeDesign,
        points: &[[f32; 3]],
        references: &[[u32; 2]],
        frame: &CourseFrame,
        bounds: &[[f32; 4]],
    ) -> Result<Self, GenerateError> {
        let mut layout = Self {
            components: Vec::new(),
            topology: SolidTopology::default(),
            coordinates: Vec::new(),
            bounds: bounds.to_vec(),
        };
        let count = usize::from(design.lame_count);
        let mut offset = 0;
        let mut mid_offset = 0;
        for (side, plate) in [&plates.front, &plates.back].into_iter().enumerate() {
            let original = (0..plate.count() as usize)
                .map(|i| points[references[offset + i][0] as usize])
                .collect::<Vec<_>>();
            let outer = (0..plate.count() as usize)
                .map(|i| points[references[offset + i][1] as usize])
                .collect::<Vec<_>>();
            let flare_side = if side == 0 {
                super::course_flare::CourseSide::Front
            } else {
                super::course_flare::CourseSide::Rear
            };
            layout.bounds[side][3] = super::course_flare::CourseFlare::fit(
                &original,
                &outer,
                &plate.topology.faces,
                frame,
                design,
                flare_side,
                bounds[side],
            )?
            .scale();
            let slope = if side == 0 {
                design.chevron_slope
            } else {
                design.rear_chevron_slope
            }
            .unit();
            let [floor, pitch, _, _] = bounds[side].map(f64::from);
            for course in 0..=count {
                let low = (floor + pitch * course as f64
                    - if course == 0 {
                        0.0
                    } else {
                        f64::from(design.overlap.metres())
                    })
                .max(floor);
                let high = (course < count).then_some(floor + pitch * (course + 1) as f64);
                let clipped =
                    CourseClip::new(&plate.topology, original.clone(), frame, slope, low, high)?;
                let mut solid = SolidTopology::new(&clipped.mid, &clipped.mid)?;
                solid.compact();
                layout.components.push(ArmorComponent {
                    role: ArmorComponentRole::Plate,
                    vertices: layout.topology.sources.len()
                        ..layout.topology.sources.len() + solid.sources.len(),
                    indices: layout.topology.indices.len()
                        ..layout.topology.indices.len() + solid.indices.len(),
                    hinge: None,
                    mount: Some(mount(side, course, count)?),
                    material: None,
                });
                layout
                    .coordinates
                    .extend(solid.sources.iter().map(|&source| {
                        let sample = clipped.samples[(source & !OUTER_BIT) as usize];
                        [
                            sample.edge[0] + offset as u32,
                            sample.edge[1] + offset as u32,
                            sample.blend.to_bits(),
                            course as u32
                                | if side != 0 { REAR_BIT } else { 0 }
                                | (source & OUTER_BIT),
                        ]
                    }));
                layout.topology.extend(solid, mid_offset);
                mid_offset += clipped.mid.vertex_count() as u32;
            }
            offset += plate.count() as usize;
        }
        Ok(layout)
    }
}

fn mount(side: usize, course: usize, count: usize) -> Result<PlateMount, GenerateError> {
    Ok(PlateMount {
        course: PlateCourse(
            u16::try_from(side * (count + 1) + count - course)
                .map_err(|_| GenerateError::InvalidSurface)?,
        ),
        parent: (course < count).then(|| PlateParent {
            course: PlateCourse((side * (count + 1) + count - course - 1) as u16),
            edge: PlateGridEnd::Last,
        }),
        incoming: PlateGridEnd::First,
        motion: PlateJointMotion::Flexible,
        follow: Permille((usize::from(Permille::ONE.0) * (count - course) / count) as u16),
    })
}
