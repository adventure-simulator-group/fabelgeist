//! Separate lower joint plates, including the long terminal plate of a poleyn.
use crate::{
    ArmorComponentRole, DesignError, GenerateError, JointCupDesign, Millimeters, Milliradians,
    PartMesh, Permille,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointExtension {
    pub length: Millimeters,
    pub lame_count: u8,
    /// Share of the extension occupied by its lowest plate.
    pub terminal_share: Permille,
    /// Distal girth relative to the cup edge, shared across all lower plates.
    pub distal_taper: Permille,
    pub wrap: Milliradians,
    pub hem_rounding: Millimeters,
}

impl Default for JointExtension {
    fn default() -> Self {
        Self {
            length: Millimeters(130),
            lame_count: 3,
            terminal_share: Permille(650),
            distal_taper: Permille(1000),
            wrap: Milliradians(1350),
            hem_rounding: Millimeters(12),
        }
    }
}

impl JointExtension {
    pub fn validate(&self) -> Result<(), DesignError> {
        let terminal_length = f32::from(self.length.0)
            * if self.lame_count == 1 {
                1.0
            } else {
                self.terminal_share.unit()
            };
        if !(40..=180).contains(&self.length.0)
            || !(1..=4).contains(&self.lame_count)
            || !(350..=850).contains(&self.terminal_share.0)
            || !(700..=1100).contains(&self.distal_taper.0)
            || !(900..=1550).contains(&self.wrap.0)
            || self.hem_rounding.0 > 25
            || 4.0 * f32::from(self.hem_rounding.0) >= terminal_length
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }
}

pub(crate) fn append(
    mut mesh: PartMesh,
    d: &JointCupDesign,
    edge: impl Fn(f32) -> [f32; 3],
) -> Result<PartMesh, GenerateError> {
    let Some(extension) = &d.distal_extension else {
        return Ok(mesh);
    };
    let gauge = d.gauge.thickness.metres();
    const LAME_ROWS: usize = 12;
    const LAP_SPACING_GAUGES: f32 = 2.5;
    const AXIAL_OVERLAP_M: f32 = 0.002;
    const CUP_EDGE_OVERLAP_M: f32 = 0.0005;
    let mut lower = PartMesh::new();
    let mut start = 0.0;
    for i in 0..extension.lame_count {
        let last = i + 1 == extension.lame_count;
        let share = if extension.lame_count == 1 {
            1.0
        } else if last {
            extension.terminal_share.unit()
        } else {
            (1.0 - extension.terminal_share.unit()) / f32::from(extension.lame_count - 1)
        };
        let end = start + share;
        lower.append(crate::plate_patch::fluted_patch(
            LAME_ROWS,
            false,
            gauge,
            d.fluting.as_ref(),
            [start, end],
            |_, _| 0.0,
            |u, v| {
                let down = end - (end - start) * v;
                let angle = extension.wrap.radians() * (2.0 * u - 1.0);
                // Every plate follows the same actual cup edge. The lap offset
                // is independent of sampling within a plate, so adjacent lames
                // cannot cross as their axial domains overlap.
                let point = edge(angle);
                let relief = d.fluting.as_ref().map_or(0.0, |f| f.depth.metres());
                let lap = (gauge + relief) * LAP_SPACING_GAUGES * f32::from(i + 1);
                let taper = 1.0 + (extension.distal_taper.unit() - 1.0) * down;
                [
                    point[0] * taper + lap * angle.sin(),
                    point[1] - extension.length.metres() * down
                        + if i == 0 {
                            CUP_EDGE_OVERLAP_M * v
                        } else {
                            AXIAL_OVERLAP_M * v
                        }
                        + if last {
                            extension.hem_rounding.metres()
                                * (2.0 * u - 1.0).powi(2)
                                * (1.0 - v).powi(4)
                        } else {
                            0.0
                        },
                    point[2] * taper + lap * angle.cos(),
                ]
            },
        )?);
        start = end;
    }
    mesh.append(lower.with_component(ArmorComponentRole::JointExtension, None));
    Ok(mesh)
}
