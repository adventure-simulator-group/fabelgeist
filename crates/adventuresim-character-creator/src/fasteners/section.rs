//! Taut strap support from exact triangle cross-sections.
use adventuresim_armor_model::PartFrame;
use anyhow::{Context, Result, ensure};

pub(super) fn local_points(points: &[[f32; 3]], frame: &PartFrame) -> Vec<[f32; 3]> {
    points
        .iter()
        .map(|p| {
            frame
                .axes
                .map(|axis| (0..3).map(|i| (p[i] - frame.origin[i]) * axis[i]).sum())
        })
        .collect()
}

pub(super) struct ClosureSection {
    hull: Vec<[f32; 2]>,
}

pub(super) struct SupportSurfaces<'a> {
    pub body: &'a [[f32; 3]],
    pub body_faces: &'a [[u32; 3]],
    pub plate: &'a [[f32; 3]],
    pub plate_faces: &'a [[u32; 3]],
}

impl ClosureSection {
    pub fn new(
        body: &[[f32; 3]],
        body_faces: &[[u32; 3]],
        plate: &[[f32; 3]],
        plate_faces: &[[u32; 3]],
        height: f32,
        width: f32,
        lining: f32,
    ) -> Result<Self> {
        let mut points = Vec::new();
        let mut skin = Vec::new();
        let plate_vertices = plate_faces
            .iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        points.extend(
            plate_vertices
                .into_iter()
                .map(|i| &plate[i as usize])
                .filter(|p| (p[1] - height).abs() <= width * 0.5)
                .map(|p| [p[0], p[2]]),
        );
        // The entire width shares an enclosing section, as a taut belt does.
        for offset in [-0.5, 0.0, 0.5] {
            let y = height + width * offset;
            crossings(plate, plate_faces, y, &mut points);
            crossings(body, body_faces, y, &mut skin);
        }
        let owned = body_faces
            .iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        skin.extend(
            owned
                .into_iter()
                .map(|i| body[i as usize])
                .filter(|p| (p[1] - height).abs() <= width * 0.5)
                .map(|p| [p[0], p[2]]),
        );
        const ALLOWANCE_DIRECTIONS: usize = 16;
        for point in skin {
            for i in 0..ALLOWANCE_DIRECTIONS {
                let angle = std::f32::consts::TAU * i as f32 / ALLOWANCE_DIRECTIONS as f32;
                points.push([
                    point[0] + lining * angle.sin(),
                    point[1] + lining * angle.cos(),
                ]);
            }
        }
        let hull = hull(points);
        ensure!(
            hull.len() >= 3,
            "strap plane does not cut its supporting plate"
        );
        Ok(Self { hull })
    }

    /// Follow the upper arm at each height along a descending shoulder band.
    /// A single hull of the whole vertical sweep would retain deltoid width
    /// below the deltoid and leave the strap visibly slack.
    pub fn follow_underarm(
        &mut self,
        surfaces: SupportSurfaces<'_>,
        height: f32,
        design: &super::StrapDesign,
    ) -> Result<()> {
        const UNDERARM_SECTION_COUNT: usize = 33;
        let mut sections = Vec::new();
        for i in 0..UNDERARM_SECTION_COUNT {
            let y = height
                - design.underarm_drop.metres() * i as f32 / (UNDERARM_SECTION_COUNT - 1) as f32;
            sections.push(Self::new(
                surfaces.body,
                surfaces.body_faces,
                surfaces.plate,
                surfaces.plate_faces,
                y,
                design.width.metres() * 1.4,
                design.lining_clearance.metres(),
            )?);
        }
        // Tighten the projected path around the sampled constraints. This
        // bridges steps in the plate silhouette with straight leather spans.
        const TENSION_PATH_SAMPLES: usize = 193;
        let start = design.start_angle.radians();
        let span = design.end_angle.radians() - start;
        let mut points = vec![[0.0; 2]];
        for i in 0..TENSION_PATH_SAMPLES {
            let fraction = i as f32 / (TENSION_PATH_SAMPLES - 1) as f32;
            let angle = start + span * fraction;
            let row =
                (fraction * std::f32::consts::PI).sin().max(0.0) * (sections.len() - 1) as f32;
            let low = (row.floor() as usize).min(sections.len() - 1);
            let high = (low + 1).min(sections.len() - 1);
            let blend = row - low as f32;
            let radius = sections[low]
                .radius(angle)
                .with_context(|| format!("underarm section {low}, angle {angle}"))?
                * (1.0 - blend)
                + sections[high]
                    .radius(angle)
                    .with_context(|| format!("underarm section {high}, angle {angle}"))?
                    * blend;
            points.push([radius * angle.sin(), radius * angle.cos()]);
        }
        self.hull = hull(points);
        Ok(())
    }

    pub fn include_support(
        &mut self,
        vertices: &[[f32; 3]],
        faces: &[[u32; 3]],
        height: f32,
        width: f32,
    ) {
        let mut points = self.hull.clone();
        points.extend(
            vertices
                .iter()
                .filter(|p| (p[1] - height).abs() <= width * 0.5)
                .map(|p| [p[0], p[2]]),
        );
        for offset in [-0.5, 0.0, 0.5] {
            crossings(vertices, faces, height + width * offset, &mut points);
        }
        self.hull = hull(points);
    }

    pub fn radius(&self, angle: f32) -> Result<f32> {
        let ray = [angle.sin(), angle.cos()];
        let mut radius = 0.0_f32;
        for (&a, &b) in self.hull.iter().zip(self.hull.iter().cycle().skip(1)) {
            let edge = [b[0] - a[0], b[1] - a[1]];
            let determinant = cross(ray, edge);
            if determinant.abs() < 1e-8 {
                continue;
            }
            let distance = cross(a, edge) / determinant;
            let fraction = cross(a, ray) / determinant;
            // Rays through a hull vertex can round to just outside either
            // adjoining edge; include that floating-point endpoint tolerance.
            if (-1e-5..=1.0 + 1e-5).contains(&fraction) {
                radius = radius.max(distance);
            }
        }
        ensure!(radius > 0.0, "strap axis lies outside its support section");
        Ok(radius)
    }
}

fn crossings(vertices: &[[f32; 3]], faces: &[[u32; 3]], y: f32, output: &mut Vec<[f32; 2]>) {
    for face in faces {
        let points = face.map(|i| vertices[i as usize]);
        for i in 0..3 {
            let a = points[i];
            let b = points[(i + 1) % 3];
            if (a[1] <= y && b[1] > y) || (b[1] <= y && a[1] > y) {
                let t = (y - a[1]) / (b[1] - a[1]);
                output.push([a[0] + t * (b[0] - a[0]), a[2] + t * (b[2] - a[2])]);
            }
        }
    }
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

fn hull(mut points: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for &p in &points {
        push_hull(&mut lower, p);
    }
    for &p in points.iter().rev() {
        push_hull(&mut upper, p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn push_hull(hull: &mut Vec<[f32; 2]>, p: [f32; 2]) {
    while hull.len() >= 2 {
        let a = hull[hull.len() - 2];
        let b = hull[hull.len() - 1];
        if cross([b[0] - a[0], b[1] - a[1]], [p[0] - b[0], p[1] - b[1]]) > 0.0 {
            break;
        }
        hull.pop();
    }
    hull.push(p);
}
