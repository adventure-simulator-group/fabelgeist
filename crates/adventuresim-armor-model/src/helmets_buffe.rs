//! A removable face defense under a burgonet's peak, separate from its skull.
use crate::{
    ArmorComponentRole, DesignError, GenerateError, Millimeters, Milliradians, PartMesh, Permille,
    VisorBreaths,
};
use serde::{Deserialize, Serialize};
#[path = "helmets_buffe_courses.rs"]
mod courses;
pub use courses::BuffeCourses;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffeDesign {
    pub breaths: Option<VisorBreaths>,
    pub courses: Option<BuffeCourses>,
    pub sight_gap: Millimeters,
    pub face_projection: Millimeters,
    pub chin_width: Permille,
    pub throat_depth: Permille,
    pub neck_drop: Millimeters,
    pub side_wrap: Milliradians,
    pub medial_ridge: Millimeters,
    pub ridge_sharpness: Permille,
    pub chin_point: Millimeters,
}

impl Default for BuffeDesign {
    fn default() -> Self {
        Self {
            breaths: None,
            courses: None,
            sight_gap: Millimeters(8),
            face_projection: Millimeters(18),
            chin_width: Permille(850),
            throat_depth: Permille(650),
            neck_drop: Millimeters(12),
            side_wrap: Milliradians(1800),
            medial_ridge: Millimeters(10),
            ridge_sharpness: Permille(0),
            chin_point: Millimeters(35),
        }
    }
}

impl BuffeDesign {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(5..=20).contains(&self.sight_gap.0)
            || self.face_projection.0 > 40
            || !(500..=1100).contains(&self.chin_width.0)
            || !(550..=1000).contains(&self.throat_depth.0)
            || self.neck_drop.0 > 35
            || !(1500..=1950).contains(&self.side_wrap.0)
            || self.medial_ridge.0 > 25
            || self.chin_point.0 > 60
            || self.ridge_sharpness.0 > 1000
        {
            return Err(DesignError::ParametricParameters);
        }
        if let Some(courses) = &self.courses {
            courses.validate()?;
        }
        if let Some(breaths) = &self.breaths {
            breaths.validate(50..=950)?;
        }
        Ok(())
    }
}

pub(super) fn generate(
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    helmet: &super::BurgonetDesign,
    design: &BuffeDesign,
) -> Result<PartMesh, GenerateError> {
    let carrier = Carrier {
        radii,
        brow,
        half_height,
        helmet,
        design,
    };
    let gauge = helmet.fit.wall_thickness.metres();
    let mesh = if let Some(courses) = &design.courses {
        courses.generate(&carrier, gauge)?
    } else if let Some(breaths) = design.breaths.filter(|b| b.count_per_row > 0) {
        carrier.pierced(&breaths, gauge)?
    } else {
        crate::plate_patch::fluted_patch(
            FACE_ROWS,
            false,
            gauge,
            None,
            [0.0, 1.0],
            |_, _| 0.0,
            |u, v| carrier.point(u, v),
        )?
    };
    Ok(mesh.with_component(ArmorComponentRole::Buffe, None))
}

const FACE_ROWS: usize = 32;
const CHEEK_LAP_GAUGES: f32 = 3.0;
const CHART_HALF_WIDTH_MM: f64 = 160.0;
const CHART_HEIGHT_MM: f64 = 100.0;

struct Carrier<'a> {
    radii: [f32; 3],
    brow: f32,
    half_height: f32,
    helmet: &'a super::BurgonetDesign,
    design: &'a BuffeDesign,
}
impl Carrier<'_> {
    fn point(&self, u: f32, v: f32) -> [f32; 3] {
        let Self {
            radii,
            brow,
            half_height,
            helmet,
            design,
        } = *self;
        let gauge = helmet.fit.wall_thickness.metres();
        let angle = (2.0 * u - 1.0) * f32::from(design.side_wrap.0) / 1000.0;
        let front = angle.cos().max(0.0);
        let top = brow - (design.sight_gap.metres() + helmet.peak_drop.metres()) * front
            + super::burgonet::peak_rise(helmet, angle);
        let hem = -half_height - design.neck_drop.metres()
            + design.chin_point.metres() * angle.sin().abs();
        let mut width = radii[0]
            * (design.chin_width.unit() + (1.0 - design.chin_width.unit()) * v)
            + gauge * CHEEK_LAP_GAUGES;
        let depth =
            radii[2] * (design.throat_depth.unit() + (1.0 - design.throat_depth.unit()) * v);
        let angular_ridge = if front > 0.0 {
            1.0 - angle.sin().abs()
        } else {
            0.0
        };
        let ridge_height = design.medial_ridge.metres() * (std::f32::consts::PI * v).sin();
        let ridge = if design.courses.is_some() {
            courses::formed_ridge(angle, depth, ridge_height, design.ridge_sharpness.unit())
        } else {
            ridge_height
                * (front.powi(4) * (1.0 - design.ridge_sharpness.unit())
                    + angular_ridge * design.ridge_sharpness.unit())
        };
        let y = hem + (top - hem) * v;
        let z = depth * angle.cos() + design.face_projection.metres() * v * front.powi(4) + ridge;
        // Broad horizontal sections enclose the cheek plates. A smooth
        // transition below their lower edge preserves the chin taper
        // without imprinting each cheek plate into the outer face guard.
        let lower = ((brow - y) / half_height).clamp(0.0, 1.0);
        let cheek_bottom = brow
            - half_height * (0.40 + 0.46 * helmet.cheek_depth.unit())
            - helmet.chin_tab.metres();
        const JAW_TRANSITION_M: f32 = 0.040;
        let blend = ((y - cheek_bottom + JAW_TRANSITION_M) / JAW_TRANSITION_M).clamp(0.0, 1.0);
        let blend = blend * blend * (3.0 - 2.0 * blend);
        let cheek_width =
            radii[0] * (1.0 - (1.0 - helmet.cheek_taper.unit()) * lower) + gauge * 5.0;
        width += (cheek_width - width).max(0.0) * blend;
        [width * angle.sin(), y, z]
    }

    fn pierced(&self, breaths: &VisorBreaths, gauge: f32) -> Result<PartMesh, GenerateError> {
        pierced_patch(breaths, [0.0, CHART_HEIGHT_MM], gauge, |u, v| {
            self.point(u, v)
        })
    }
}

fn pierced_patch(
    breaths: &VisorBreaths,
    [top, bottom]: [f64; 2],
    gauge: f32,
    point: impl Fn(f32, f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    let outer = [
        [-CHART_HALF_WIDTH_MM, top],
        [CHART_HALF_WIDTH_MM, top],
        [CHART_HALF_WIDTH_MM, bottom],
        [-CHART_HALF_WIDTH_MM, bottom],
    ];
    const HALF_COLUMNS: i32 = 40;
    let interior = (1..FACE_ROWS).flat_map(|row| {
        (-HALF_COLUMNS + 1..HALF_COLUMNS).map(move |column| {
            [
                f64::from(column) * CHART_HALF_WIDTH_MM / f64::from(HALF_COLUMNS),
                top + row as f64 * (bottom - top) / FACE_ROWS as f64,
            ]
        })
    });
    let holes = breaths.openings(CHART_HEIGHT_MM);
    let domain = crate::pierced_plate_domain::PiercedDomain::new(&outer, &holes, interior)?;
    let positions = domain
        .points
        .iter()
        .map(|p| {
            point(
                ((p[0] / CHART_HALF_WIDTH_MM + 1.0) * 0.5) as f32,
                (1.0 - (p[1] - top) / (bottom - top)) as f32,
            )
        })
        .collect();
    PartMesh::from_relief_surface(
        positions,
        domain.indices,
        gauge,
        crate::BoundaryNormals::Separate,
        crate::ShellExtrusion::Normal,
        None,
    )
}
