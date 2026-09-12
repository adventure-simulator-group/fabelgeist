//! Continuous anatomical sections for the chin-to-neck transition.
use super::{CloseHelmetDesign, SUBMENTAL_NECK_FRACTION};
use crate::{GenerateError, PartFrame};
use serde::{Deserialize, Serialize};

const NECK_ROWS: usize = super::super::close::NECK_ROWS;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub(super) struct NeckSectionReserve {
    pub half_width: f32,
    pub back_depth: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub(super) struct NeckEnclosure {
    sections: [NeckSectionReserve; NECK_ROWS + 1],
}

impl NeckEnclosure {
    pub fn is_valid(self) -> bool {
        self.sections.iter().all(|section| {
            [section.half_width, section.back_depth]
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0)
        })
    }
    pub fn fit(
        profile: &super::CloseHelmetProfile,
        design: &CloseHelmetDesign,
        frame: &PartFrame,
        positions: &[[f32; 3]],
        faces: &[[u32; 3]],
        margin: crate::Millimeters,
    ) -> Result<Self, GenerateError> {
        if positions.is_empty()
            || faces.is_empty()
            || positions.iter().flatten().any(|v| !v.is_finite())
            || faces
                .iter()
                .flatten()
                .any(|i| *i as usize >= positions.len())
        {
            return Err(GenerateError::InvalidSurface);
        }
        let points = super::local_samples(frame, positions);
        let jaw = -frame.half_extents[1] * super::super::close::CHIN_HEAD_RATIO
            + design.back_edge_lift.metres();
        let hem = -frame.half_extents[1] * super::super::close::NECK_HEM_HEAD_RATIO
            - design.neck_length.metres()
            + design.back_edge_lift.metres();
        let jaw_width = profile
            .jaw_half_width
            .max(profile.temple_half_width * design.jaw_width.unit());
        let neck_width = profile
            .neck_half_width
            .max(profile.temple_half_width * design.neck_width.unit());
        let mut enclosure = Self::default();
        for (row, reserve) in enclosure.sections.iter_mut().enumerate() {
            let t = row as f32 / NECK_ROWS as f32;
            let blend = t * t * (3.0 - 2.0 * t);
            let width = jaw_width + (neck_width - jaw_width) * blend;
            let back = profile.nape_waist + (profile.nape_back - profile.nape_waist) * blend;
            let front = profile.section(design, 0.0, 1.0, t, 1.0)[1];
            if front <= back {
                return Err(GenerateError::InvalidSurface);
            }
            let lip = super::super::close::neck_lip_radius(design.back_flare.metres(), blend);
            let scale = posterior_section_scale(
                &points,
                faces,
                jaw + (hem - jaw) * t,
                margin.metres(),
                width + lip,
                front + lip,
                back - lip,
            );
            reserve.half_width = (width + lip) * (scale - 1.0);
            reserve.back_depth = (front - back + 2.0 * lip) * (scale - 1.0);
        }
        Ok(enclosure)
    }

    pub fn at(self, fraction: f32) -> NeckSectionReserve {
        let coordinate = fraction.clamp(0.0, 1.0) * NECK_ROWS as f32;
        let index = (coordinate.floor() as usize).min(NECK_ROWS - 1);
        let t = coordinate - index as f32;
        let a = self.sections[index];
        let b = self.sections[index + 1];
        NeckSectionReserve {
            half_width: a.half_width + (b.half_width - a.half_width) * t,
            back_depth: a.back_depth + (b.back_depth - a.back_depth) * t,
        }
    }
}

/// Smallest ellipse dilation anchored at the unchanged front. The ellipse
/// inequality gives an exact scale bound for each clipped support vertex.
/// Convexity makes polygon vertices sufficient to enclose their interiors.
fn posterior_section_scale(
    points: &[[f32; 3]],
    faces: &[[u32; 3]],
    height: f32,
    margin: f32,
    width: f32,
    front: f32,
    back: f32,
) -> f32 {
    let depth = front - back;
    let center = (front + back) * 0.5;
    let mut scale = 1.0_f32;
    let mut polygon = Vec::with_capacity(8);
    let mut clipped = Vec::with_capacity(8);
    for face in faces {
        polygon.clear();
        polygon.extend(face.map(|index| points[index as usize]));
        for (axis, bound, below) in [
            (1, height - margin, false),
            (1, height + margin, true),
            (2, center, true),
        ] {
            clip_polygon(&polygon, &mut clipped, axis, bound, below);
            std::mem::swap(&mut polygon, &mut clipped);
            if polygon.is_empty() {
                break;
            }
        }
        for point in &polygon {
            let x = point[0].abs() + margin;
            let behind = front - (point[2] - margin);
            scale = scale.max(behind / depth + x * x * depth / (4.0 * behind * width * width));
        }
    }
    scale
}

fn clip_polygon(
    input: &[[f32; 3]],
    output: &mut Vec<[f32; 3]>,
    axis: usize,
    bound: f32,
    below: bool,
) {
    output.clear();
    for (i, a) in input.iter().enumerate() {
        let b = input[(i + 1) % input.len()];
        let inside = |p: [f32; 3]| {
            if below {
                p[axis] <= bound
            } else {
                p[axis] >= bound
            }
        };
        if inside(*a) {
            output.push(*a);
        }
        if inside(*a) != inside(b) {
            let t = (bound - a[axis]) / (b[axis] - a[axis]);
            output.push(std::array::from_fn(|axis| {
                a[axis] + (b[axis] - a[axis]) * t
            }));
        }
    }
}

pub(super) struct NeckProfile {
    pub half_width: f32,
    pub submental_front: f32,
    pub throat_front: f32,
    pub nape_back: f32,
    pub nape_waist: f32,
}

pub(super) fn from_samples(
    design: &CloseHelmetDesign,
    frame: &PartFrame,
    points: &[[f32; 3]],
) -> Result<NeckProfile, GenerateError> {
    const SECTION_HALF_BAND_M: f32 = 0.006;
    let chin = -frame.half_extents[1];
    let jaw = chin * crate::helmets::close::CHIN_HEAD_RATIO;
    let hem = chin * crate::helmets::close::NECK_HEM_HEAD_RATIO - design.neck_length.metres();
    let measure = |height: f32| {
        super::Bounds::measure(
            points,
            height - SECTION_HALF_BAND_M,
            height + SECTION_HALF_BAND_M,
        )
    };
    let neck = measure(hem + design.back_edge_lift.metres())?;
    let nape = measure(chin + design.back_edge_lift.metres())?;
    let throat = measure(hem)?;
    let submental = measure(jaw + (hem - jaw) * SUBMENTAL_NECK_FRACTION)?;
    let gap = design.face_clearance.metres() + design.fit.wall_thickness.metres();
    Ok(NeckProfile {
        half_width: neck.half_width() + gap,
        submental_front: submental.high[2] + gap,
        throat_front: throat.high[2] + gap,
        nape_back: neck.low[2] - gap,
        nape_waist: nape.low[2] - gap,
    })
}

pub(super) fn measure(
    design: &CloseHelmetDesign,
    frame: &PartFrame,
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    section_margin: crate::Millimeters,
) -> Result<NeckProfile, GenerateError> {
    if positions.iter().flatten().any(|value| !value.is_finite())
        || faces
            .iter()
            .flatten()
            .any(|index| *index as usize >= positions.len())
    {
        return Err(GenerateError::InvalidSurface);
    }
    let points = positions
        .iter()
        .map(|point| {
            frame
                .axes
                .map(|axis| (0..3).map(|i| axis[i] * (point[i] - frame.origin[i])).sum())
        })
        .collect::<Vec<[f32; 3]>>();
    let chin = -frame.half_extents[1];
    let jaw = chin * crate::helmets::close::CHIN_HEAD_RATIO;
    let hem = chin * crate::helmets::close::NECK_HEM_HEAD_RATIO - design.neck_length.metres();
    let measure = |height| section(&points, faces, height, section_margin.metres());
    let throat = measure(hem)?;
    let neck = measure(hem + design.back_edge_lift.metres())?;
    let nape = measure(chin + design.back_edge_lift.metres())?;
    let submental = measure(jaw + (hem - jaw) * SUBMENTAL_NECK_FRACTION)?;
    let gap = design.face_clearance.metres() + design.fit.wall_thickness.metres();
    Ok(NeckProfile {
        half_width: neck.0[0].abs().max(neck.1[0].abs()) + gap,
        submental_front: submental.1[2] + gap,
        throat_front: throat.1[2] + gap,
        nape_back: neck.0[2] - gap,
        nape_waist: nape.0[2] - gap,
    })
}

fn section(
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    height: f32,
    margin: f32,
) -> Result<([f32; 3], [f32; 3]), GenerateError> {
    let mut low = [f32::INFINITY; 3];
    let mut high = [f32::NEG_INFINITY; 3];
    for face in faces {
        let points = face.map(|index| positions[index as usize]);
        for (a, b) in [
            (points[0], points[1]),
            (points[1], points[2]),
            (points[2], points[0]),
        ] {
            let bottom = height - margin;
            let top = height + margin;
            if a[1].max(b[1]) < bottom || a[1].min(b[1]) > top {
                continue;
            }
            let interval = if a[1] == b[1] {
                [0.0, 1.0]
            } else {
                [
                    (bottom - a[1]) / (b[1] - a[1]),
                    (top - a[1]) / (b[1] - a[1]),
                ]
            };
            for t in interval.map(|value| value.clamp(0.0, 1.0)) {
                for axis in 0..3 {
                    let value = a[axis] + (b[axis] - a[axis]) * t;
                    low[axis] = low[axis].min(value);
                    high[axis] = high[axis].max(value);
                }
            }
        }
    }
    if !low.iter().chain(&high).all(|value| value.is_finite()) {
        return Err(GenerateError::InvalidSurface);
    }
    Ok((low, high))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_hem_flare_already_enclosing_support_needs_no_additional_reserve() {
        let mut design = CloseHelmetDesign {
            neck_length: crate::Millimeters(20),
            back_edge_lift: crate::Millimeters(0),
            back_flare: crate::Millimeters(40),
            ..Default::default()
        };
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.10, 0.10, 0.11],
        };
        let profile = super::super::CloseHelmetProfile::authored([0.10, 0.13, 0.11], &design);
        let hem = -frame.half_extents[1] * super::super::super::close::NECK_HEM_HEAD_RATIO
            - design.neck_length.metres();
        let side = profile.section(&design, std::f32::consts::FRAC_PI_2, 1.0, 1.0, 1.0);
        let back = profile.section(&design, std::f32::consts::PI, 1.0, 1.0, 1.0)[1];
        let points = [
            [-side[0] - 0.01, hem, side[1]],
            [0.0, hem, back - 0.01],
            [side[0] + 0.01, hem, side[1]],
        ];
        let faces = [[0, 1, 2]];
        let fitted = NeckEnclosure::fit(
            &profile,
            &design,
            &frame,
            &points,
            &faces,
            crate::Millimeters(0),
        )
        .unwrap();
        assert_eq!(fitted.at(1.0).half_width, 0.0);
        assert_eq!(fitted.at(1.0).back_depth, 0.0);
        design.back_flare = crate::Millimeters(0);
        let unflared = NeckEnclosure::fit(
            &profile,
            &design,
            &frame,
            &points,
            &faces,
            crate::Millimeters(0),
        )
        .unwrap();
        assert!(unflared.at(1.0).half_width > 0.0);
        assert!(unflared.at(1.0).back_depth > 0.0);
    }

    #[test]
    fn intermediate_support_adds_local_reserve_without_inflating_terminal_sections() {
        let design = CloseHelmetDesign {
            neck_length: crate::Millimeters(20),
            back_edge_lift: crate::Millimeters(0),
            back_flare: crate::Millimeters(0),
            ..Default::default()
        };
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.10, 0.10, 0.11],
        };
        let profile = super::super::CloseHelmetProfile::authored([0.10, 0.13, 0.11], &design);
        let jaw = -frame.half_extents[1] * super::super::super::close::CHIN_HEAD_RATIO;
        let hem = -frame.half_extents[1] * super::super::super::close::NECK_HEM_HEAD_RATIO
            - design.neck_length.metres();
        let mut points = Vec::new();
        let mut faces = Vec::new();
        for row in 0..=NECK_ROWS {
            let y = jaw + (hem - jaw) * row as f32 / NECK_ROWS as f32;
            let (width, back) = if row == NECK_ROWS / 2 {
                (0.12, -0.15)
            } else {
                (0.02, -0.02)
            };
            let side_depth = if row == NECK_ROWS / 2 { -0.04 } else { 0.0 };
            points.extend([
                [-width, y, side_depth],
                [0.0, y, 0.02],
                [width, y, side_depth],
                [0.0, y, back],
            ]);
            if row > 0 {
                for column in 0..4 {
                    let a = ((row - 1) * 4 + column) as u32;
                    let b = ((row - 1) * 4 + (column + 1) % 4) as u32;
                    faces.extend([[a, b, b + 4], [a, b + 4, a + 4]]);
                }
            }
        }
        let enclosure = NeckEnclosure::fit(
            &profile,
            &design,
            &frame,
            &points,
            &faces,
            crate::Millimeters(0),
        )
        .unwrap();
        assert_eq!(enclosure.at(0.0).half_width, 0.0);
        assert_eq!(enclosure.at(1.0).half_width, 0.0);
        assert_eq!(enclosure.at(0.0).back_depth, 0.0);
        assert_eq!(enclosure.at(1.0).back_depth, 0.0);
        assert!(enclosure.at(0.5).half_width > 0.01);
        assert!(enclosure.at(0.5).back_depth > 0.01);
        let before = profile.section(&design, std::f32::consts::FRAC_PI_2, 1.0, 0.5, 1.0);
        let mut fitted = profile;
        fitted.neck_enclosure = enclosure;
        let after = fitted.section(&design, std::f32::consts::FRAC_PI_2, 1.0, 0.5, 1.0);
        assert!(after[0] > before[0]);
        let front = fitted.section(&design, 0.0, 1.0, 0.5, 1.0)[1];
        let back = fitted.section(&design, std::f32::consts::PI, 1.0, 0.5, 1.0)[1];
        let center = (front + back) * 0.5;
        let depth = (front - back) * 0.5;
        for point in [[-0.12_f32, -0.04], [0.12, -0.04], [0.0, -0.15]] {
            let ellipse = (point[0] / after[0]).powi(2) + ((point[1] - center) / depth).powi(2);
            assert!(
                ellipse <= 1.00001,
                "local support escaped the fitted ellipse"
            );
        }
        assert_eq!(
            profile.section(&design, 1.0, 0.0, 0.0, 0.0),
            fitted.section(&design, 1.0, 0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn section_tracks_a_sloping_surface_between_sparse_vertex_rows() {
        let positions = [
            [-0.1, -0.2, 0.02],
            [0.1, -0.2, 0.02],
            [0.1, 0.0, 0.10],
            [-0.1, 0.0, 0.10],
        ];
        let faces = [[0, 1, 2], [0, 2, 3]];
        for height in [-0.15, -0.10, -0.05] {
            let (low, high) = section(&positions, &faces, height, 0.0).unwrap();
            let expected = 0.02 + (height + 0.2) * 0.4;
            assert!((high[2] - expected).abs() < 1e-6);
            assert!((low[2] - expected).abs() < 1e-6);
            assert_eq!((low[0], high[0]), (-0.1, 0.1));
        }
        assert!(section(&positions, &faces, 0.1, 0.0).is_err());
        let (low, high) = section(&positions, &faces, -0.10, 0.003).unwrap();
        assert!((low[2] - 0.0588).abs() < 1e-6);
        assert!((high[2] - 0.0612).abs() < 1e-6);
    }
}
