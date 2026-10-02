//! Joint-following tubes attached to a shared curved sewing boundary.
use super::*;

// Signed identity blends can pull the fitted exterior upper sleeve inward.
// Reserve cloth ease there while leaving its shared armscye and cuff fixed.
const QUILTED_UPPER_ARM_BLEND_RESERVE_M: f32 = 0.011;
const UPPER_ARM_RESERVE_CENTER: f32 = 0.18;
const UPPER_ARM_RESERVE_HALF_SPAN: f32 = 0.18;

pub(super) fn fit(
    design: &GarmentArmorDesign,
    placement: &str,
    wearer: &Wearer<'_>,
) -> Result<PartMesh> {
    let side = Side::from_placement(placement)?;
    let prefix = if matches!(side, Side::Left) { "l" } else { "r" };
    let arm = matches!(design.kind, Kind::MailSleeve | Kind::QuiltedSleeve);
    let names = if arm {
        ["uparm", "lowarm", "wrist"]
    } else {
        ["upleg", "lowleg", "foot"]
    };
    let anchors = names
        .map(|name| joint(wearer, &format!("{prefix}_{name}")))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let region = if arm {
        FitRegion::WholeArm(side)
    } else {
        FitRegion::WholeLeg(side)
    };
    let frame = wearer.frame(region)?;
    let support = wearer.support_indices(region)?;
    let lengths = [
        length(subtract(anchors[1], anchors[0])),
        length(subtract(anchors[2], anchors[1])),
    ];
    let bend = lengths[0] / (lengths[0] + lengths[1]);
    let gap = design.clearance.metres() + design.wall_thickness.metres() + GARMENT_FIT_MARGIN_M;
    let mesh = generate_garment_armor(design, &frame)?;
    let attachment = if arm {
        let mut torso_design = GarmentArmorDesign::new(if design.kind == Kind::QuiltedSleeve {
            Kind::ArmingDoublet
        } else {
            Kind::MailShirt
        });
        torso_design.clearance = design.clearance;
        torso_design.wall_thickness = design.wall_thickness;
        Some(AttachmentRing::armhole(
            &torso::fit(&torso_design, wearer)?,
            side,
            &frame,
        ))
    } else {
        None
    };
    let cage = LimbCage {
        frame,
        support,
        anchors,
        bend,
        gap,
        arm,
        attachment,
    };
    Ok(mesh.refit_surfaces(|positions, _| {
        for point in positions {
            let local = local_point(&cage.frame, *point);
            *point = cage.point(design, wearer, local);
        }
    })?)
}

struct LimbCage {
    frame: PartFrame,
    support: Vec<usize>,
    anchors: Vec<[f32; 3]>,
    bend: f32,
    gap: f32,
    arm: bool,
    attachment: Option<AttachmentRing>,
}

impl LimbCage {
    fn point(&self, design: &GarmentArmorDesign, wearer: &Wearer<'_>, local: [f32; 3]) -> [f32; 3] {
        let frame = &self.frame;
        let anchors = &self.anchors;
        let support = &self.support;
        let attachment = &self.attachment;
        let (bend, gap, arm) = (self.bend, self.gap, self.arm);
        let raw_t = ((frame.half_extents[1] - local[1]) / (2.0 * frame.half_extents[1])).max(0.0);
        let angle = (local[0] / frame.half_extents[0]).atan2(local[2] / frame.half_extents[2]);
        let radial_direction = std::array::from_fn(|axis| {
            frame.axes[0][axis] * angle.sin() + frame.axes[2][axis] * angle.cos()
        });
        let toward_trunk = subtract([0.0, anchors[0][1], anchors[0][2]], anchors[0]);
        let toward_trunk = normalized(subtract(
            toward_trunk,
            scale(frame.axes[1], dot(toward_trunk, frame.axes[1])),
        ));
        let medial = dot(radial_direction, toward_trunk).max(0.0);
        // The proximal opening climbs over the outer shoulder/hip and drops
        // beneath the armpit/groin. A planar bone-centered ring cuts the trunk.
        let setback = if arm {
            0.025 + 0.22 * medial.powi(2)
        } else {
            0.015 + 0.20 * medial.powi(2)
        };
        let t = raw_t * if arm { 0.96 } else { 0.985 } + setback * (1.0 - raw_t).max(0.0).powi(3);
        let (segment, fraction) = if t < bend {
            (0, t / bend)
        } else {
            (1, (t - bend) / (1.0 - bend))
        };
        let (center, tangent) = limb_station(anchors, segment, fraction);
        let front = normalized(subtract(
            frame.axes[2],
            scale(tangent, dot(frame.axes[2], tangent)),
        ));
        let across = cross(tangent, front);
        let axes = [across, tangent, front];
        let (lo, hi) = enclosing_section(wearer, support, center, axes);
        let radius = [(hi[0] - lo[0]) * 0.5, (hi[2] - lo[2]) * 0.5];
        let quilting = if matches!(design.kind, Kind::QuiltedSleeve | Kind::PaddedChausses) {
            0.0015 * (angle * 6.0).cos().powi(2)
        } else {
            0.0
        };
        let blend_reserve = if design.kind == Kind::QuiltedSleeve {
            let axial = (raw_t - UPPER_ARM_RESERVE_CENTER) / UPPER_ARM_RESERVE_HALF_SPAN;
            QUILTED_UPPER_ARM_BLEND_RESERVE_M
                * (1.0 - axial * axial).max(0.0).powi(2)
                * (-angle.cos()).max(0.0).max(1.0 - medial)
        } else {
            0.0
        };
        let offset = [
            (hi[0] + lo[0]) * 0.5 + (radius[0] + gap + quilting + blend_reserve) * angle.sin(),
            0.0,
            (hi[2] + lo[2]) * 0.5 + (radius[1] + gap + quilting + blend_reserve) * angle.cos(),
        ];
        let mut point = std::array::from_fn(|axis| {
            center[axis] + (0..3).map(|i| axes[i][axis] * offset[i]).sum::<f32>()
        });
        if let Some(attachment) = &attachment {
            let transition = (raw_t / 0.24).clamp(0.0, 1.0);
            let blend = 1.0 - transition * transition * (3.0 - 2.0 * transition);
            point = lerp(point, attachment.at(angle), blend);
        }
        point
    }
}

fn limb_station(anchors: &[[f32; 3]], segment: usize, t: f32) -> ([f32; 3], [f32; 3]) {
    let a = anchors[segment];
    let b = anchors[segment + 1];
    if t > 1.0 {
        return (
            std::array::from_fn(|axis| b[axis] + (b[axis] - a[axis]) * (t - 1.0)),
            normalized(subtract(a, b)),
        );
    }
    let bend_tangent = scale(subtract(anchors[2], anchors[0]), 0.5);
    let m0 = if segment == 0 {
        subtract(b, a)
    } else {
        bend_tangent
    };
    let m1 = if segment == 0 {
        bend_tangent
    } else {
        subtract(b, a)
    };
    let position = std::array::from_fn(|axis| {
        (2.0 * t.powi(3) - 3.0 * t * t + 1.0) * a[axis]
            + (t.powi(3) - 2.0 * t * t + t) * m0[axis]
            + (-2.0 * t.powi(3) + 3.0 * t * t) * b[axis]
            + (t.powi(3) - t * t) * m1[axis]
    });
    let derivative = std::array::from_fn(|axis| {
        (6.0 * t * t - 6.0 * t) * a[axis]
            + (3.0 * t * t - 4.0 * t + 1.0) * m0[axis]
            + (-6.0 * t * t + 6.0 * t) * b[axis]
            + (3.0 * t * t - 2.0 * t) * m1[axis]
    });
    (position, normalized(scale(derivative, -1.0)))
}
