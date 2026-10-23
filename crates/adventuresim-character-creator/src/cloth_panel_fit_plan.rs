//! Fit a bounded continuation around the cloth before physical material cuts.
//! Its coherent source field supports fine cut edges without sliver controls.
use super::{PanelSurface, anatomical::AnatomicalCarrier, domain::PanelDomain};
use crate::armor_frames::Wearer;
use crate::cloth_torso_frame::ClothTorsoFrame;
use crate::skin_transfer::SkinWeights;
use crate::surface_cut::{ConvexRegion, Plane};
use anyhow::{Result, ensure};
use fabelgeist_armor::{BodiceDesign, FittedSleeves, Millimeters, PartFrame};
use fabelgeist_rig::RigJointName;

#[derive(Clone)]
pub struct PanelPlan {
    surface: PanelDomain,
    support: PanelDomain,
    skin: SkinWeights,
    banks: super::banks::ContactBanks,
    clearance: Millimeters,
    body_faces: Vec<[u32; 3]>,
    body_vertices: usize,
}

impl PanelPlan {
    pub fn new(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        head: &PartFrame,
        sleeves: Option<&FittedSleeves>,
    ) -> Result<(Self, PanelSurface)> {
        design.validate()?;
        let carrier = AnatomicalCarrier::new(body, design.hem_rise, sleeves)?;
        let boundary = FittingBoundary::new(design, body, head, sleeves)?;
        let support = PanelDomain::new(
            carrier.source,
            body,
            &boundary.include,
            &boundary.subtract,
            boundary.center,
        )?;
        let sampled = support.clone().sample(body)?;
        let surface = sampled.surface;
        let banks = super::banks::ContactBanks::new(&surface, &support, &sampled.ancestry)?;
        let carrier_skin = surface.carrier_skin(body)?;
        let skin = crate::skin_transfer::interpolate_sources(
            &surface
                .cut
                .source_table(&surface.carrier.faces, &surface.carrier.faces),
            &carrier_skin.indices,
            &carrier_skin.weights,
        )?;
        let plan = Self {
            surface,
            support,
            skin,
            banks,
            clearance: design.clearance,
            body_faces: body.faces.to_vec(),
            body_vertices: body.positions.len(),
        };
        let surface = plan.realize(body)?;
        Ok((plan, surface))
    }

    pub(crate) fn skin(&self) -> &SkinWeights {
        &self.skin
    }

    pub fn realize(&self, body: &Wearer<'_>) -> Result<PanelSurface> {
        ensure!(
            body.faces == self.body_faces && body.positions.len() == self.body_vertices,
            "cloth refit requires matching canonical body topology"
        );
        self.surface
            .enclosed_surface(&self.support, &self.skin, &self.banks, self.clearance, body)
    }
}

/// Physical design planes extended only for numerical fitting support.
struct FittingBoundary {
    include: Vec<ConvexRegion>,
    subtract: Vec<ConvexRegion>,
    center: [f32; 3],
}

impl FittingBoundary {
    fn new(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        head: &PartFrame,
        sleeves: Option<&FittedSleeves>,
    ) -> Result<Self> {
        let marks = ClothTorsoFrame::new(body, head)?;
        let include = if sleeves.is_some() {
            vec![vec![]]
        } else {
            vec![vec![Plane {
                normal: [0., -1., 0.],
                offset: -marks.waist[1] - design.hem_rise.metres(),
            }]]
        };
        let subtract = marks.carrier_openings(design, body, sleeves)?;
        let anchor = RigJointName::C_SPINE3.require_in(body.joint_names)?;
        let center = [
            body.joints[anchor][0],
            body.joints[anchor][1],
            body.joints[anchor][2],
        ];
        // Retain only the offset reserve needed by the final cut. A large
        // unused continuation can drag a coarse neck edge away from the body.
        let margin = design.clearance.metres();
        let mut fit_include = include.clone();
        if let Some(plane) = fit_include[0].first_mut() {
            plane.offset += margin;
        }
        let mut fit_subtract = subtract.clone();
        for (region, planes) in fit_subtract.iter_mut().enumerate() {
            // The front/back neck half-space is a chart partition, not a seam.
            let first = usize::from(region < 2);
            for plane in planes.iter_mut().skip(first) {
                plane.offset -= margin;
            }
        }
        Ok(Self {
            include: fit_include,
            subtract: fit_subtract,
            center,
        })
    }
}
