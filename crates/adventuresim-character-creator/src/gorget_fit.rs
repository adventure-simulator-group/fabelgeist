//! An authored neck collar and short shoulder bib fitted from sectional bounds.
//! Broad anatomical sections define the bib, with a coarse radial cage around
//! the shoulders and chest. Output tessellation is independent of body topology.

use adventuresim_armor_model::{
    GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape, PartFrame, PartMesh,
};
use anyhow::{Context, Result, ensure};

use crate::armor_frames::{FitRegion, Wearer};

#[path = "gorget_bib_clearance.rs"]
mod bib_clearance;
#[path = "gorget_bib_fit.rs"]
mod bib_fit;
#[path = "gorget_meridian.rs"]
mod meridian;
#[path = "gorget_plate_mesh.rs"]
mod plate_mesh;
use bib_fit::BibFit;
#[path = "gorget_sections.rs"]
mod sections;
use sections::{MINIMUM_SECTION_SAMPLES, Section};
const COLLAR_HEIGHT_NECK_RATIO: f32 = 0.20;
const COLLAR_BASE_NECK_RATIO: f32 = 0.58;
const SAGITTAL_SECTION_HALF_WIDTH_NECK_RATIO: f32 = 0.18;
const COLLAR_MAX_PITCH: f32 = 0.45;
const FRONT_BIB_DROP_NECK_RATIO: f32 = 0.48;
const REAR_BIB_DROP_NECK_RATIO: f32 = 0.30;
const SIDE_BIB_RISE_NECK_RATIO: f32 = 0.45;
const SHOULDER_WIDTH_SECTION_NECK_RATIO: f32 = 0.30;
const BIB_FLARE_WIDTH_GAIN: f32 = 0.25;
const POSTERIOR_SHOULDER_CROWN_NECK_RATIO: f32 = 0.20;
const SIDE_COLLAR_RISE_NECK_RATIO: f32 = 0.15;
const SIDE_COLLAR_RISE_HEIGHT_LIMIT: f32 = 0.60;

pub fn fit(design: &GarmentArmorDesign, wearer: &Wearer<'_>) -> Result<PartMesh> {
    design.validate()?;
    ensure!(
        design.kind == GarmentArmorKind::Gorget,
        "gorget fitter requires a gorget design"
    );
    let neck = joint(wearer, "c_neck")?;
    let head = joint(wearer, "c_head")?;
    let height = head[1] - neck[1];
    ensure!(height.is_finite() && height > 0.0, "invalid neck landmarks");
    let heading = wearer.frame(FitRegion::Head)?;
    let frame = PartFrame {
        origin: neck,
        axes: heading.axes,
        half_extents: [height; 3],
    };
    let samples = wearer
        .positions
        .iter()
        .map(|p| local(&frame, *p))
        .collect::<Vec<_>>();
    let mut cage = CollarCage::new(design, height, &samples, wearer.faces)?;
    cage.bib_fit = BibFit::measure(
        |t, angle| {
            cage.bib_point(
                t,
                adventuresim_armor_model::gorget_control_angle(angle, cage.rear_sweep),
            )
        },
        &samples,
        wearer.faces,
        design.clearance.metres() + design.wall_thickness.metres(),
    );
    Ok(cage
        .mesh(design, &samples, wearer.faces)?
        .transformed(&frame))
}

fn joint(wearer: &Wearer<'_>, name: &str) -> Result<[f32; 3]> {
    let index = wearer
        .joint_names
        .iter()
        .position(|n| n == name)
        .with_context(|| format!("missing gorget landmark {name}"))?;
    Ok(std::array::from_fn(|axis| wearer.joints[index][axis]))
}

fn local(frame: &PartFrame, p: [f32; 3]) -> [f32; 3] {
    frame
        .axes
        .map(|axis| (0..3).map(|i| axis[i] * (p[i] - frame.origin[i])).sum())
}

/// Quadratic through three broad horizontal section bounds.
struct DepthCurve([f32; 3]);

impl DepthCurve {
    fn at(&self, t: f32) -> f32 {
        self.0[0] * (1.0 - t) * (1.0 - 2.0 * t)
            + self.0[1] * 4.0 * t * (1.0 - t)
            + self.0[2] * t * (2.0 * t - 1.0)
    }
}

struct CollarCage {
    bib_fit: BibFit,
    center: [f32; 2],
    collar_radius: [f32; 2],
    base_radius: f32,
    outer_width: [f32; 2],
    top: CollarPlane,
    base: CollarPlane,
    hem: [f32; 3],
    front: DepthCurve,
    back: DepthCurve,
    posterior_shoulder_crown: f32,
    side_collar_rise: f32,
    hem_flatness: f32,
    rear_hem_flatness: f32,
    rear_sweep: adventuresim_armor_model::Permille,
}

impl CollarCage {
    fn new(
        design: &GarmentArmorDesign,
        height: f32,
        samples: &[[f32; 3]],
        faces: &[[u32; 3]],
    ) -> Result<Self> {
        let GarmentPlateShape::Gorget {
            neck_clearance,
            collar_slope,
            collar_height,
            hem_flatness,
            rear_hem_flatness,
            rear_sweep,
            front_depth,
            back_depth,
            front_width,
            back_width,
        } = design.plate_shape
        else {
            anyhow::bail!("gorget requires collar shape controls");
        };
        let clearance = design.clearance.metres() + design.wall_thickness.metres();
        let collar_padding = neck_clearance.metres() + design.wall_thickness.metres();
        // A planar oval rim follows the oblique neck section beneath the jaw.
        // Horizontal bounds at the low throat include shoulder tissue and cannot
        // define the lateral collar width.
        let top = CollarPlane {
            height: (COLLAR_BASE_NECK_RATIO + COLLAR_HEIGHT_NECK_RATIO * collar_height.unit())
                * height,
            pitch: COLLAR_MAX_PITCH * collar_slope.unit(),
        };
        let base = CollarPlane {
            height: COLLAR_BASE_NECK_RATIO * height,
            pitch: top.pitch,
        };
        let hem = [
            -FRONT_BIB_DROP_NECK_RATIO * design.length.unit() * front_depth.unit(),
            SIDE_BIB_RISE_NECK_RATIO,
            -REAR_BIB_DROP_NECK_RATIO * back_depth.unit(),
        ]
        .map(|v| v * height);
        let neck = top.section(samples, faces)?;
        let lower_neck = base.section(samples, faces)?;
        let center = [
            (neck.low[0] + neck.high[0]) * 0.5,
            (neck.low[2] + neck.high[2]) * 0.5,
        ];
        let collar_radius = [
            (neck.high[0] - neck.low[0]) * 0.5 + collar_padding,
            (neck.high[2] - neck.low[2]) * 0.5 + collar_padding,
        ];
        let base_radius = (lower_neck.high[0] - lower_neck.low[0]) * 0.5 + collar_padding;
        let shoulder = Section::measure(samples, height * SHOULDER_WIDTH_SECTION_NECK_RATIO)?;
        let outer_width = ((shoulder.high[0] - shoulder.low[0]) * 0.5 + clearance)
            * (1.0 + design.flare.unit() * BIB_FLARE_WIDTH_GAIN);
        let outer_width = [
            front_width.unit() * outer_width,
            back_width.unit() * outer_width,
        ];
        let mut front = [0.0; 3];
        let mut back = [0.0; 3];
        for (i, t) in [0.0, 0.5, 1.0].into_iter().enumerate() {
            front[i] = Section::sagittal_slice(
                samples,
                faces,
                base.at(lower_neck.high[2]) + (hem[0] - base.at(lower_neck.high[2])) * t,
                height * SAGITTAL_SECTION_HALF_WIDTH_NECK_RATIO,
            )?
            .high[2]
                - center[1]
                + clearance;
            back[i] = center[1]
                - Section::sagittal_slice(
                    samples,
                    faces,
                    base.at(lower_neck.low[2]) + (hem[2] - base.at(lower_neck.low[2])) * t,
                    height * SAGITTAL_SECTION_HALF_WIDTH_NECK_RATIO,
                )?
                .low[2]
                + clearance;
        }
        front[0] = lower_neck.high[2] - center[1] + collar_padding;
        back[0] = center[1] - lower_neck.low[2] + collar_padding;
        Ok(Self {
            bib_fit: BibFit::default(),
            center,
            collar_radius,
            base_radius,
            outer_width,
            top,
            base,
            hem,
            front: DepthCurve(front),
            back: DepthCurve(back),
            posterior_shoulder_crown: height * POSTERIOR_SHOULDER_CROWN_NECK_RATIO,
            side_collar_rise: (height * SIDE_COLLAR_RISE_NECK_RATIO)
                .min((top.height - base.height) * SIDE_COLLAR_RISE_HEIGHT_LIMIT),
            hem_flatness: hem_flatness.unit(),
            rear_hem_flatness: rear_hem_flatness.unit(),
            rear_sweep,
        })
    }

    fn collar_point(&self, t: f32, angle: f32) -> [f32; 3] {
        self.collar_boundary(t, angle, 0.0)
    }

    /// Exact radius/height derivative of the collar at its join to the bib.
    fn collar_tangent(&self, control_angle: f32) -> bevy::math::Vec2 {
        let angle = self.surface_angle(control_angle);
        let depth = if angle.cos() >= 0.0 {
            self.front.at(0.0)
        } else {
            self.back.at(0.0)
        };
        let power = collar_power(angle);
        let radius = polar_radius(self.base_radius, depth, power, angle);
        let width_weight = (radius * angle.sin() / self.base_radius).abs().powf(power);
        let depth_weight = (radius * angle.cos() / depth).abs().powf(power);
        let radial = radius
            * (width_weight * (self.base_radius - self.collar_radius[0]) / self.base_radius
                + depth_weight * (depth - self.collar_radius[1]) / depth);
        let z = self.center[1] + radius * angle.cos();
        bevy::math::Vec2::new(
            radial,
            self.base_height(angle, z) - self.top.at(z) - self.base.pitch * radial * angle.cos(),
        )
    }

    fn collar_clearance(&self, t: f32, angle: f32, padding: f32) -> bevy::math::Vec3 {
        bevy::math::Vec3::from_array(self.collar_point(t, angle))
            - bevy::math::Vec3::from_array(self.collar_boundary(t, angle, padding))
    }

    /// Insets the authored radial section before constructing its oblique plane.
    fn collar_boundary(&self, t: f32, angle: f32, inset: f32) -> [f32; 3] {
        let angle = self.surface_angle(angle);
        let start_depth = self.collar_radius[1];
        let end_depth = if angle.cos() >= 0.0 {
            self.front.at(0.0)
        } else {
            self.back.at(0.0)
        };
        let depth = start_depth + (end_depth - start_depth) * t;
        let width = self.collar_radius[0] + (self.base_radius - self.collar_radius[0]) * t;
        let radius = polar_radius(width - inset, depth - inset, collar_power(angle), angle);
        let z = self.center[1] + radius * angle.cos();
        [
            self.center[0] + radius * angle.sin(),
            self.top.at(z) + (self.base_height(angle, z) - self.top.at(z)) * t,
            z,
        ]
    }

    fn base_height(&self, angle: f32, z: f32) -> f32 {
        self.base.at(z) + self.side_collar_rise * angle.sin().powi(2)
    }

    fn hem_height(&self, angle: f32) -> f32 {
        let cosine = angle.cos();
        let end = if cosine >= 0.0 {
            self.hem[0]
        } else {
            self.hem[2]
        };
        let flatness = if cosine >= 0.0 {
            self.hem_flatness
        } else {
            self.rear_hem_flatness
        };
        self.hem[1]
            + (end - self.hem[1])
                * (cosine * cosine
                    / (cosine * cosine + (1.0 - 0.975 * flatness) * angle.sin().powi(2)))
    }

    fn surface_angle(&self, control: f32) -> f32 {
        adventuresim_armor_model::gorget_surface_angle(control, self.rear_sweep)
    }

    fn formed_bib_point(&self, t: f32, control_angle: f32) -> Result<[f32; 3]> {
        let angle = self.surface_angle(control_angle);
        let section = |point: [f32; 3]| {
            bevy::math::Vec2::new(
                (point[0] - self.center[0]).hypot(point[2] - self.center[1]),
                point[1],
            )
        };
        let curve = meridian::Meridian::new(
            section(self.bib_point(0.0, control_angle)),
            section(self.bib_point(0.5, control_angle)),
            section(self.bib_point(1.0, control_angle)),
            self.collar_tangent(control_angle),
        )
        .with_context(|| format!("gorget formed meridian at control angle {control_angle}"))?;
        let point = curve.point(t);
        Ok([
            self.center[0] + point.x * angle.sin(),
            point.y,
            self.center[1] + point.x * angle.cos(),
        ])
    }

    // These nominal anatomical samples seed the few formed-curve controls;
    // the dense measured field is never emitted as the final carrier.
    fn bib_point(&self, t: f32, control_angle: f32) -> [f32; 3] {
        let hem_height = self.hem_height(control_angle);
        let angle = self.surface_angle(control_angle);
        let outer = self.outer_width[if angle.cos() >= 0.0 { 0 } else { 1 }];
        let side =
            self.base_radius + (self.outer_width[0].min(self.outer_width[1]) - self.base_radius);
        let width = self.base_radius
            + (outer - self.base_radius + (side - outer) * angle.sin().powi(2)) * t;
        let cosine = angle.cos();
        let depth = if cosine >= 0.0 {
            self.front.at(t)
        } else {
            self.back.at(t)
        };
        let hem_power = if cosine >= 0.0 { 2.0 } else { 6.0 };
        let radius = polar_radius(
            width,
            depth,
            collar_power(angle) + (hem_power - collar_power(angle)) * t,
            angle,
        );
        // The authored chart descends smoothly before anatomical seating.
        // Limit its shoulder crown to the available collar-to-hem drop.
        let base_depth = if cosine >= 0.0 {
            self.front.at(0.0)
        } else {
            self.back.at(0.0)
        };
        let base_z = self.center[1]
            + polar_radius(self.base_radius, base_depth, collar_power(angle), angle) * cosine;
        let base_height = self.base_height(angle, base_z);
        let available_drop = base_height - hem_height;
        let shoulder_crown = self
            .posterior_shoulder_crown
            .min(available_drop * 0.8 / std::f32::consts::PI)
            * angle.sin().powi(2)
            * (std::f32::consts::PI * t).sin();
        let offset = self.bib_fit.offset(t, angle);
        [
            self.center[0] + radius * angle.sin(),
            base_height + (hem_height - base_height) * t + shoulder_crown + offset[1],
            self.center[1] + radius * angle.cos() + offset[2],
        ]
    }
}

/// The posterior neck flattens across the trapezius; the throat stays rounded.
fn collar_power(angle: f32) -> f32 {
    if angle.cos() >= 0.0 { 2.0 } else { 3.0 }
}

/// A true polar chart keeps anatomical angular boundaries fixed while its
/// cross-section changes from elliptical neck to rounded shoulder bib.
fn polar_radius(width: f32, depth: f32, power: f32, angle: f32) -> f32 {
    ((angle.sin() / width).abs().powf(power) + (angle.cos() / depth).abs().powf(power))
        .powf(-1.0 / power)
}

/// Plane height decreases toward the throat; all rim points remain coplanar.
#[derive(Clone, Copy)]
struct CollarPlane {
    height: f32,
    pitch: f32,
}

impl CollarPlane {
    fn at(self, z: f32) -> f32 {
        self.height - self.pitch * z
    }

    fn section(self, samples: &[[f32; 3]], faces: &[[u32; 3]]) -> Result<Section> {
        let mut section = Section {
            low: [f32::INFINITY; 3],
            high: [f32::NEG_INFINITY; 3],
        };
        let mut count = 0;
        for face in faces {
            for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                let a = samples[face[a] as usize];
                let b = samples[face[b] as usize];
                let da = a[1] - self.at(a[2]);
                let db = b[1] - self.at(b[2]);
                if (da > 0.0) == (db > 0.0) {
                    continue;
                }
                let t = da / (da - db);
                for axis in 0..3 {
                    let value = a[axis] + (b[axis] - a[axis]) * t;
                    section.low[axis] = section.low[axis].min(value);
                    section.high[axis] = section.high[axis].max(value);
                }
                count += 1;
            }
        }
        ensure!(
            count >= MINIMUM_SECTION_SAMPLES,
            "collar plane does not cross a supported neck section"
        );
        Ok(section)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn cage() -> CollarCage {
        CollarCage {
            bib_fit: BibFit::default(),
            center: [0.0, 0.02],
            collar_radius: [0.078, 0.075],
            base_radius: 0.09,
            outer_width: [0.15; 2],
            top: CollarPlane {
                height: 0.074,
                pitch: 0.30,
            },
            base: CollarPlane {
                height: 0.055,
                pitch: 0.30,
            },
            hem: [-0.046, 0.043, 0.011],
            front: DepthCurve([0.065, 0.071, 0.092]),
            back: DepthCurve([0.078, 0.109, 0.131]),
            posterior_shoulder_crown: 0.0095,
            side_collar_rise: 0.014,
            hem_flatness: 0.0,
            rear_hem_flatness: 0.0,
            rear_sweep: adventuresim_armor_model::Permille(0),
        }
    }

    #[test]
    fn collar_join_derivative_matches_the_oblique_anatomical_surface() {
        let mut cage = cage();
        for sweep in [0, 600] {
            cage.rear_sweep = adventuresim_armor_model::Permille(sweep);
            for index in 0..64 {
                let angle = index as f32 / 64.0 * std::f32::consts::TAU;
                let section = |t| {
                    let point = cage.collar_point(t, angle);
                    bevy::math::Vec2::new(
                        (point[0] - cage.center[0]).hypot(point[2] - cage.center[1]),
                        point[1],
                    )
                };
                let difference = (section(1.001) - section(0.999)) / 0.002;
                let exact = cage.collar_tangent(angle);
                assert!(
                    exact.distance(difference) < 0.00002,
                    "angle {angle}: {exact:?} / {difference:?}"
                );
            }
        }
    }

    #[test]
    fn collar_and_bib_form_one_closed_consistently_wound_material_shell() {
        let mesh = cage()
            .mesh(&GarmentArmorDesign::new(GarmentArmorKind::Gorget), &[], &[])
            .unwrap();
        mesh.normals().unwrap();
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        for &[a, b, c] in mesh.indices.as_chunks::<3>().0 {
            for (u, v) in [(a, b), (b, c), (c, a)] {
                edges.entry((u.min(v), u.max(v))).or_default().push((u, v));
            }
        }
        for incident in edges.values() {
            assert_eq!(incident.len(), 2);
            assert_eq!(incident[0], (incident[1].1, incident[1].0));
        }
    }

    #[test]
    fn oblique_section_interpolates_the_neck_instead_of_inflating_to_nearby_vertices() {
        let plane = CollarPlane {
            height: 0.075,
            pitch: 0.35,
        };
        let mut points = Vec::new();
        for offset in [-0.012, 0.012] {
            for i in 0..48 {
                let angle = i as f32 / 48.0 * std::f32::consts::TAU;
                let z = (0.075 + offset) * angle.cos();
                points.push([(0.060 + offset) * angle.sin(), plane.at(z) + offset, z]);
            }
        }
        let mut faces = Vec::new();
        for i in 0..48u32 {
            let next = (i + 1) % 48;
            faces.extend([[i, next, 48 + next], [i, 48 + next, 48 + i]]);
        }
        let neck = plane.section(&points, &faces).unwrap();
        assert!((neck.high[0] - 0.060).abs() < 1e-6);
        assert!((neck.high[2] - 0.075).abs() < 1e-6);
        assert!((neck.low[2] + 0.075).abs() < 1e-6);
        assert!(plane.section(&points, &[]).is_err());
    }
}
