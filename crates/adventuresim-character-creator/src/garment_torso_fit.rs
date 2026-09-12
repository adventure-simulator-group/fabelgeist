//! Pattern-cut torso panels with independently sampled shoulder and flank seams.
use super::*;

pub(super) fn fit(design: &GarmentArmorDesign, wearer: &Wearer<'_>) -> Result<PartMesh> {
    let mut support = wearer.support_indices(FitRegion::Torso)?;
    support.extend(wearer.support_indices(FitRegion::Hips)?);
    support.sort_unstable();
    support.dedup();
    let raw = wearer.frame(FitRegion::Torso)?;
    let neck = joint(wearer, "c_neck")?;
    let waist = joint(wearer, "c_spine1")?[1];
    let bottom = joint(
        wearer,
        if matches!(design.kind, Kind::ArmingDoublet | Kind::MailShirt) {
            "c_spine1"
        } else {
            "c_spine0"
        },
    )?[1];
    let frame = upright(
        wearer,
        neck[1] + SHOULDER_LIFT_M,
        bottom,
        raw.half_extents[0],
        raw.half_extents[2],
        raw.origin,
    )?;
    let mesh = generate_garment_armor(design, &frame)?;
    let gap = design.clearance.metres() + design.wall_thickness.metres() + GARMENT_FIT_MARGIN_M;
    let cage = SectionCage::new(wearer, &support, &frame);
    let neck_y = neck[1] - frame.origin[1];
    let armpit_y = joint(wearer, "l_uparm")?[1] - (neck[1] - waist) * 0.30 - frame.origin[1];
    let bottom_y = -frame.half_extents[1] * design.length.unit();
    let cage = TorsoCage {
        frame,
        sections: cage,
        design,
        neck_y,
        bottom_y,
        armpit_y,
        gap,
        waist_y: waist - frame.origin[1],
    };
    Ok(mesh.refit_surfaces(|positions, _| cage.fit(positions))?)
}

struct TorsoCage<'a> {
    frame: PartFrame,
    sections: SectionCage,
    design: &'a GarmentArmorDesign,
    neck_y: f32,
    bottom_y: f32,
    armpit_y: f32,
    gap: f32,
    waist_y: f32,
}

impl TorsoCage<'_> {
    fn panel_point(&self, index: usize) -> [f32; 3] {
        let cage = &self.sections;
        let design = self.design;
        let (neck_y, bottom_y, armpit_y, gap) =
            (self.neck_y, self.bottom_y, self.armpit_y, self.gap);
        let stride = PANEL_ACROSS + 1;
        let panel_count = (PANEL_ALONG + 1) * stride;
        let panel_index = index % panel_count;
        let row = panel_index / stride;
        let col = panel_index % stride;
        let front = index < panel_count;
        let u = 2.0 * col as f32 / PANEL_ACROSS as f32 - 1.0;
        let v = row as f32 / PANEL_ALONG as f32;
        let neckline = (1.0 - (u.abs() / 0.5).min(1.0).powi(2)).max(0.0);
        let shoulder_top = neck_y + 0.018 + SHOULDER_SEAM_EASE_M - 0.035 * u.abs().powi(2);
        let top = shoulder_top - if front { 0.090 } else { 0.025 } * neckline;
        let y = if row <= ARMPIT_ROW {
            bottom_y + (armpit_y - bottom_y) * row as f32 / ARMPIT_ROW as f32
        } else {
            armpit_y
                + (top - armpit_y) * (row - ARMPIT_ROW) as f32 / (PANEL_ALONG - ARMPIT_ROW) as f32
        };
        // Upper width comes from the shoulder girdle, independently of
        // the narrowing neck section and neckline drop.
        let (lo, hi) = cage.at(y.min(neck_y - 0.035));
        let width = (hi[0] - lo[0]) * 0.5;
        let (chest_lo, chest_hi) = cage.at(0.0);
        let chest_width = (chest_hi[0] - chest_lo[0]) * 0.5;
        let seam_transition = (v / 0.65).clamp(0.0, 1.0);
        let seam_blend = seam_transition * seam_transition * (3.0 - 2.0 * seam_transition);
        let lower_ease = if design.kind == Kind::MailShirt {
            1.0 - seam_blend
        } else {
            (1.0 - v).powi(2)
        };
        let waist_ease = (chest_width * design.waist.unit() - width).max(0.0) * lower_ease;
        let center_x = (hi[0] + lo[0]) * 0.5;
        let shoulder_inset = ((v - 0.6) / 0.4).max(0.0).powi(2);
        let hip_flare = if matches!(design.kind, Kind::Brigandine | Kind::JackOfPlates) {
            design.flare.unit() * ((self.waist_y - y) / (self.waist_y - bottom_y)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let x = center_x
            + u * (width * (1.0 + hip_flare) + gap + waist_ease) * (1.0 - 0.13 * shoulder_inset);
        let (lo, hi) = cage.at(y);
        let mut center_z = (hi[2] + lo[2]) * 0.5;
        let mut radius = (hi[2] - lo[2]) * 0.5;
        if design.kind == Kind::MailShirt {
            let chest_radius = (chest_hi[2] - chest_lo[2]) * 0.5;
            radius += (chest_radius * design.waist.unit() - radius).max(0.0) * lower_ease;
            let (hem_lo, hem_hi) = cage.at(bottom_y);
            let hem_center = (hem_lo[2] + hem_hi[2]) * 0.5;
            center_z = hem_center * (1.0 - seam_blend) + center_z * seam_blend;
        }
        radius *= 1.0 + hip_flare * 0.5;
        let contour = (1.0 - 0.78 * u * u).powf(0.25);
        let quilting = if design.kind == Kind::ArmingDoublet {
            0.0015 * (u * std::f32::consts::PI * 6.0).cos().powi(2)
        } else {
            0.0
        };
        let z = center_z + if front { 1.0 } else { -1.0 } * (radius * contour + gap + quilting);
        [x, y, z]
    }

    fn fit(&self, positions: &mut [[f32; 3]]) {
        let frame = &self.frame;
        let stride = PANEL_ACROSS + 1;
        let panel_count = (PANEL_ALONG + 1) * stride;
        let top_start = PANEL_ALONG * stride;
        for index in 0..positions.len() {
            let local = if index < 2 * panel_count {
                self.panel_point(index)
            } else if index < 2 * panel_count + 2 * (SHOULDER_DEPTH - 1) * (PANEL_ACROSS / 4 + 1) {
                let bridge_index = index - 2 * panel_count;
                let band_width = PANEL_ACROSS / 4 + 1;
                let band_vertices = (SHOULDER_DEPTH - 1) * band_width;
                let band = bridge_index / band_vertices;
                let depth_row = (bridge_index % band_vertices) / band_width + 1;
                let col =
                    bridge_index % band_width + if band == 0 { 0 } else { 3 * PANEL_ACROSS / 4 };
                let front = local_point(frame, positions[top_start + col]);
                let rear = local_point(frame, positions[panel_count + top_start + col]);
                let t = depth_row as f32 / SHOULDER_DEPTH as f32;
                let mut local = lerp(front, rear, t);
                local[1] += 0.045 * (std::f32::consts::PI * t).sin();
                local
            } else {
                let start = 2 * panel_count + 2 * (SHOULDER_DEPTH - 1) * (PANEL_ACROSS / 4 + 1);
                let side_count = (ARMPIT_ROW + 1) * (SHOULDER_DEPTH - 1);
                let side = (index - start) / side_count;
                let vertex = (index - start) % side_count;
                let row = vertex / (SHOULDER_DEPTH - 1);
                let depth = vertex % (SHOULDER_DEPTH - 1) + 1;
                let col = if side == 0 { 0 } else { PANEL_ACROSS };
                let front = local_point(frame, positions[row * stride + col]);
                let rear = local_point(frame, positions[panel_count + row * stride + col]);
                let t = depth as f32 / SHOULDER_DEPTH as f32;
                let mut local = lerp(front, rear, t);
                local[1] -= ARMPIT_SEAM_DROP_M
                    * (row as f32 / ARMPIT_ROW as f32).powi(4)
                    * (std::f32::consts::PI * t).sin();
                local
            };
            positions[index] = frame.point(local);
        }
    }
}
