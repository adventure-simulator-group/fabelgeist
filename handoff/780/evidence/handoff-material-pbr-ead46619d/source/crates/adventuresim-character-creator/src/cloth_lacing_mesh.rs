//! Coarse inner rings and crossing cords reuse the shared closed tube kernel.
use crate::cloth_fastener_mesh::{
    ClosureFrame, CordEnds, CordPath, FastenerFrame, FastenerMesh,
};
use crate::cloth_opening::ClosureSide;
use anyhow::Result;
use fabelgeist_armor::{SurfaceMaterial, ClothLacing};
use fabelgeist_math::Vec3;

const RING_ROUGHNESS: fabelgeist_math::units::Ratio = fabelgeist_math::units::Ratio::from_ratio(0.35);
impl FastenerMesh {
    pub fn lacing_rings(design: &ClothLacing, frame: FastenerFrame) -> Result<Self> {
        let frame = ClosureFrame::new(frame)?;
        let material = ring_material(design);
        let mut mesh = Self::empty();
        for side in [ClosureSide::Left, ClosureSide::Right] {
            mesh.add_ring(design, &frame, side, &material);
        }
        Ok(mesh)
    }
    pub(crate) fn single_lacing_ring(
        design: &ClothLacing,
        frame: FastenerFrame,
        side: ClosureSide,
    ) -> Result<Self> {
        let frame = ClosureFrame::new(frame)?;
        let mut mesh = Self::empty();
        mesh.add_ring(design, &frame, side, &ring_material(design));
        Ok(mesh)
    }
    fn add_ring(
        &mut self,
        design: &ClothLacing,
        frame: &ClosureFrame,
        side: ClosureSide,
        material: &fabelgeist_armor::ComponentAppearance,
    ) {
        let center = match side {
            ClosureSide::Left => -design.inset.metres(),
            ClosureSide::Right => frame.gap + design.inset.metres(),
        };
        let points = (0..ClothLacing::RING_CURVE_SEGMENTS)
            .map(|i| {
                let angle =
                    std::f32::consts::TAU * i as f32 / ClothLacing::RING_CURVE_SEGMENTS as f32;
                Vec3::new(
                    center + design.ring_radius.metres() * angle.cos(),
                    design.ring_radius.metres() * angle.sin(),
                    -design.ring_seat.metres(),
                )
            })
            .collect();
        self.cord(
            frame,
            &CordPath {
                points,
                normal: Vec3::forward(),
                ends: CordEnds::Joined,
            },
            design.eyelet.gauge().metres() * 0.5,
            side,
            material,
        );
    }
    pub fn lacing_cord(
        design: &ClothLacing,
        frame: FastenerFrame,
        backing: &fabelgeist_armor::BuiltPart,
        support: &crate::cloth_lacing_support::LacingSupport,
    ) -> Result<Self> {
        let frame = ClosureFrame::new(frame)?;
        let material = fabelgeist_armor::ComponentAppearance::Textile(design.fabric.clone());
        let height = support.height_metres(&frame, backing)?;
        let mut mesh = Self::empty();
        mesh.cord(
            &frame,
            &CordPath {
                points: vec![
                    Vec3::new(0., 0., 0.),
                    Vec3::new(
                        frame.gap * 0.5,
                        0.,
                        height.max(design.ring_seat.metres()) + design.cord_lift.metres(),
                    ),
                    Vec3::new(frame.gap, 0., 0.),
                ],
                normal: Vec3::up(),
                ends: CordEnds::Capped,
            },
            design.cord.metres() * 0.5,
            ClosureSide::Left,
            &material,
        );
        // The two endpoint rings inherit their respective cloth edge ownership.
        let half = mesh.owners.len() / 2;
        for owner in &mut mesh.owners[half..] {
            *owner = ClosureSide::Right;
        }
        Ok(mesh)
    }
}

/// Evaluate the same local seated center used by the ring tube geometry.
pub(crate) fn ring_center(
    design: &ClothLacing,
    frame: FastenerFrame,
    side: ClosureSide,
) -> Result<Vec3> {
    let frame = ClosureFrame::new(frame)?;
    let x = match side {
        ClosureSide::Left => -design.inset.metres(),
        ClosureSide::Right => frame.gap + design.inset.metres(),
    };
    let [x, y, z] = frame.point(Vec3::new(x, 0., -design.ring_seat.metres()));
    Ok(Vec3::new(x, y, z))
}

fn ring_material(design: &ClothLacing) -> fabelgeist_armor::ComponentAppearance {
    match &design.eyelet {
        fabelgeist_armor::LacingEyelet::Metal { color, .. } => fabelgeist_armor::ComponentAppearance::Surface(SurfaceMaterial {
                base_color: color.material().base_color,
                metallic: fabelgeist_math::units::Ratio::from_ratio(1.0),
                roughness: RING_ROUGHNESS,
            }),
        fabelgeist_armor::LacingEyelet::Sewn { fabric, .. } => fabelgeist_armor::ComponentAppearance::Textile(fabric.clone()),
    }
}
