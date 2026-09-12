//! Circular armpit defenses suspended independently in front of the shoulder.
use crate::{
    ArmorComponentRole, BoundaryNormals, DesignError, GenerateError, Millimeters, Milliradians,
    PartMesh, PlateGauge, RadialFluting, ShellExtrusion,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BesagewDesign {
    pub radius: Millimeters,
    pub boss_height: Millimeters,
    pub fluting: Option<RadialFluting>,
    pub shoulder_drop: Millimeters,
    pub medial_offset: Millimeters,
    pub plate_clearance: Millimeters,
    pub outward_tilt: Milliradians,
}

impl Default for BesagewDesign {
    fn default() -> Self {
        Self {
            radius: Millimeters(58),
            boss_height: Millimeters(14),
            fluting: None,
            shoulder_drop: Millimeters(65),
            medial_offset: Millimeters(30),
            plate_clearance: Millimeters(5),
            outward_tilt: Milliradians(100),
        }
    }
}

impl BesagewDesign {
    pub fn validate(&self) -> Result<(), DesignError> {
        if !(35..=85).contains(&self.radius.0)
            || self.boss_height.0 > 30
            || !(35..=120).contains(&self.shoulder_drop.0)
            || self.medial_offset.0 > 70
            || !(3..=15).contains(&self.plate_clearance.0)
            || self.outward_tilt.0 > 350
        {
            return Err(DesignError::ParametricParameters);
        }
        if let Some(fluting) = &self.fluting {
            fluting.validate()?;
        }
        Ok(())
    }
}

/// A closed disc in its own X/Y plane, with the outward face toward +Z.
pub fn generate_besagew(d: &BesagewDesign, gauge: PlateGauge) -> Result<PartMesh, GenerateError> {
    d.validate()?;
    gauge.validate()?;
    const RADIAL_ROWS: usize = 24;
    const MINIMUM_COLUMNS: usize = 96;
    const BOSS_RADIUS_FRACTION: f32 = 0.28;
    let count = d
        .fluting
        .map_or(MINIMUM_COLUMNS, |f| f.columns().max(MINIMUM_COLUMNS));
    let columns = (0..count)
        .map(|i| i as f32 / count as f32)
        .collect::<Vec<_>>();
    let mut positions = vec![[0.0, 0.0, d.boss_height.metres()]];
    let mut relief = vec![0.0];
    let mut indices = Vec::new();
    let mut previous = Vec::new();
    for row in 1..=RADIAL_ROWS {
        let radial = row as f32 / RADIAL_ROWS as f32;
        let boss = (radial / BOSS_RADIUS_FRACTION).min(1.0);
        let height = d.boss_height.metres() * (1.0 - boss * boss * (3.0 - 2.0 * boss));
        let mut ring = Vec::new();
        for u in &columns {
            let angle = std::f32::consts::TAU * u;
            ring.push(positions.len() as u32);
            positions.push([
                d.radius.metres() * radial * angle.cos(),
                d.radius.metres() * radial * angle.sin(),
                height,
            ]);
            relief.push(d.fluting.as_ref().map_or(0.0, |f| f.relief(*u, radial)));
        }
        for i in 0..ring.len() {
            let next = (i + 1) % ring.len();
            if previous.is_empty() {
                indices.extend([0, ring[i], ring[next]]);
            } else {
                indices.extend([
                    previous[i],
                    ring[i],
                    ring[next],
                    previous[i],
                    ring[next],
                    previous[next],
                ]);
            }
        }
        previous = ring;
    }
    Ok(PartMesh::from_relief_surface(
        positions,
        indices,
        gauge.thickness.metres(),
        BoundaryNormals::Smooth,
        ShellExtrusion::Along {
            direction: [0.0, 0.0, 1.0],
        },
        Some(crate::SurfaceRelief::ShellHeights(relief)),
    )?
    .with_component(ArmorComponentRole::Besagew, None))
}
