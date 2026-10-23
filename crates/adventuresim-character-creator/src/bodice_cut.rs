//! Boolean boundaries for fitted cloth in canonical, unposed Y-up metres.
use crate::armor_frames::Wearer;
use crate::cloth_torso_frame::ClothTorsoFrame;
use crate::surface_cut::{
    ClothAperture, ConvexRegion, Plane, SurfaceCut, connected_patch, dot, weld_patch,
};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{
    BodiceDesign, BodiceOpening, FittedSleeves, InterliningCoverage, Millimeters, NecklineShape,
    PartFrame,
};

#[path = "bodice_carrier_trim.rs"]
mod carrier_trim;
#[path = "bodice_sampling.rs"]
mod sampling;

pub(super) const CUT_ARC_SEGMENTS: usize = 16;
const NECKLINE_JUNCTION_TOLERANCE_M: f32 = 0.000001;

pub(crate) struct BodiceSurface {
    pub cut: SurfaceCut,
    pub inner: SurfaceCut,
    pub facing: Option<SurfaceCut>,
}

/// Native cut-plane parameters in canonical lateral/upright metres.
pub(crate) struct OpeningCut {
    pub side: f32,
    pub slope: f32,
    pub lower_half_width: f32,
    pub hem: f32,
    pub top: f32,
}

impl BodiceSurface {
    pub fn new(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        anatomy: &Wearer<'_>,
        head: &PartFrame,
        band: Millimeters,
    ) -> Result<Self> {
        Self::build(design, body, anatomy, head, band, &[], None)
    }
    pub fn for_gown(
        gown: &fabelgeist_armor::GownDesign,
        body: &Wearer<'_>,
        anatomy: &Wearer<'_>,
        head: &PartFrame,
    ) -> Result<Self> {
        let plain = Self::build(
            &gown.bodice,
            body,
            anatomy,
            head,
            gown.waist_band,
            &[],
            gown.sleeves.as_ref(),
        )?;
        let holes = crate::cloth_eyelet_holes::regions(&plain.cut, body, head, gown)?;
        let mut surface = if holes.is_empty() {
            plain
        } else {
            Self::build(
                &gown.bodice,
                body,
                anatomy,
                head,
                gown.waist_band,
                &holes,
                gown.sleeves.as_ref(),
            )?
        };
        if gown.dressing_slit.is_some() {
            let open = Self::build(
                &gown.bodice,
                body,
                anatomy,
                head,
                Millimeters(0),
                &holes,
                gown.sleeves.as_ref(),
            )?;
            surface.inner = open.inner;
            surface.facing = open.facing;
        }
        Ok(surface)
    }
    fn build(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        anatomy: &Wearer<'_>,
        head: &PartFrame,
        band: Millimeters,
        holes: &[ClothAperture],
        sleeves: Option<&FittedSleeves>,
    ) -> Result<Self> {
        design.validate()?;
        let marks = ClothTorsoFrame::new(body, head)?;
        ensure!(
            marks.breadth() > 0. && marks.neck[1] > marks.waist[1] + design.hem_rise.metres(),
            "invalid bodice anatomical span"
        );
        let include = if sleeves.is_some() {
            vec![vec![]]
        } else {
            vec![vec![Plane {
                normal: [0., -1., 0.],
                offset: -marks.waist[1] - design.hem_rise.metres(),
            }]]
        };
        let mut subtract = marks.carrier_openings(design, anatomy, sleeves)?;
        subtract.extend(normalized_regions(marks.openings(design, band.metres())?));
        subtract.extend(holes.iter().map(|a| a.region.clone()));
        let cut = SurfaceCut::new(body.positions, body.faces, body.faces, &include, &subtract);
        let positions = cut_positions(body, &cut);
        let cut = connected_patch(cut, &positions, marks.waist)?;
        let reference = cut_positions(body, &cut);
        let mut cut = cut
            .collapse_cut_roundoff(&reference)
            .context("conforming tailored opening corners")?;
        cut.apertures = holes.to_vec();
        cut.aperture_loops(&cut_positions(body, &cut))
            .context("checking tailored eyelet topology")?;
        ensure!(
            !cut.faces.is_empty(),
            "bodice cuts removed the whole garment"
        );
        // A sleeveless torso has neck, waist and two armholes. Opening its
        // front/back joins neck and waist boundaries; it must not erase a strap.
        let expected_boundaries = if band.0 > 0 || matches!(design.opening, BodiceOpening::Closed) {
            4
        } else {
            3
        };
        let reference = cut_positions(body, &cut);
        let borders = cut
            .boundary_loops(&reference)
            .context("checking tailored bodice boundaries")?;
        ensure!(
            borders.len() == expected_boundaries + holes.len(),
            "neckline and armholes must leave two complete shoulder straps"
        );
        let facing = match design.interlining.as_ref().map(|layer| layer.coverage) {
            None | Some(InterliningCoverage::Full) => None,
            Some(InterliningCoverage::Facings { width }) => {
                let facings =
                    normalized_regions(marks.facings(design, width.metres(), band.metres())?);
                let include = include
                    .iter()
                    .flat_map(|torso| {
                        facings.iter().map(move |facing| {
                            torso
                                .iter()
                                .chain(facing)
                                .copied()
                                .collect::<ConvexRegion>()
                        })
                    })
                    .collect::<Vec<_>>();
                let facing =
                    SurfaceCut::new(body.positions, body.faces, body.faces, &include, &subtract);
                let positions = cut_positions(body, &facing);
                let mut facing = weld_patch(facing, &positions).cut;
                // A partial facing can intersect only part of an aperture. Keep
                // every cut-boundary point in its authored prism exact too.
                facing.apertures = holes.to_vec();
                // Boolean unions subdivide the same planar parent facets.
                // Remove their redundant edges without coarsening curvature.
                Some(
                    facing
                        .sample_cloth(
                            body.positions,
                            body.faces,
                            crate::surface_cut::ClothSampling::CoplanarTrim,
                            &[],
                        )?
                        .surface,
                )
            }
        };

        Self::from_panels(cut, facing, body, expected_boundaries + holes.len())
    }
}

impl ClothTorsoFrame {
    fn openings(&self, design: &BodiceDesign, band: f32) -> Result<Vec<ConvexRegion>> {
        let mut cuts = Vec::new();
        let Some(opening) = self.opening_cut(design, band)? else {
            return Ok(cuts);
        };
        // Continue the lips into the authored neckline opening. A separately
        // approximated curve height would leave a closed sliver above them.
        let mut region = vec![self.depth_half(opening.side)];
        if band > 0. {
            region.push(Plane {
                normal: [0., -1., 0.],
                offset: -opening.hem,
            });
        }
        for sign in [-1., 1.] {
            region.push(opening.edge_plane(self, sign));
        }
        cuts.push(region);
        Ok(cuts)
    }

    pub(crate) fn opening_cut(
        &self,
        design: &BodiceDesign,
        band: f32,
    ) -> Result<Option<OpeningCut>> {
        let (side, waist, chest, drop) = match design.opening {
            BodiceOpening::Closed => return Ok(None),
            BodiceOpening::Front { waist, chest } => (1., waist, chest, design.neckline.front_drop),
            BodiceOpening::Back { waist, chest } => (-1., waist, chest, design.neckline.back_drop),
        };
        let half_width = chest.metres() * 0.5;
        let radius = self.breadth() * design.neckline.width.unit() * 0.5;
        let hem = self.waist[1] + design.hem_rise.metres() + band;
        ensure!(
            half_width < radius,
            "upper opening must be narrower than the neckline"
        );
        let top = if design.neckline.shape == NecklineShape::Square {
            self.neck[1] - drop.metres()
        } else {
            self.round_opening_top(
                design.neckline.shape,
                radius,
                drop.metres(),
                waist.metres() * 0.5,
                half_width,
                hem,
            )?
        };
        ensure!(top > hem, "bodice neckline must remain above its hem");
        let opening = OpeningCut {
            side,
            slope: (half_width - waist.metres() * 0.5) / (top - hem),
            lower_half_width: waist.metres() * 0.5,
            hem,
            top,
        };
        self.validate_opening_junction(design, &opening)?;
        Ok(Some(opening))
    }

    /// Native metre junction: both cap endpoints must lie inside every
    /// neckline plane. Reject lateral neck/waist drift that would open a gap.
    fn validate_opening_junction(&self, design: &BodiceDesign, opening: &OpeningCut) -> Result<()> {
        let radius = self.breadth() * design.neckline.width.unit() * 0.5;
        let half = opening.lower_half_width + opening.slope * (opening.top - opening.hem);
        let delta = dot(self.waist, self.lateral) - dot(self.neck, self.lateral);
        ensure!(
            delta.abs() + half <= radius + NECKLINE_JUNCTION_TOLERANCE_M,
            "opening junction extends outside the lateral neckline"
        );
        if design.neckline.shape == NecklineShape::Square {
            return Ok(());
        }
        let drop = if opening.side > 0. {
            design.neckline.front_drop
        } else {
            design.neckline.back_drop
        };
        for segment in 1..CUT_ARC_SEGMENTS {
            let angle = std::f32::consts::PI * segment as f32 / CUT_ARC_SEGMENTS as f32;
            let [across, down] = neckline_tangent(design.neckline.shape, angle);
            let mut normal = self.lateral.map(|v| v * across / radius);
            normal[1] -= down / drop.metres();
            let length = dot(normal, normal).sqrt();
            for sign in [-1., 1.] {
                let residual = across * (delta + sign * half) / radius
                    - down * (opening.top - self.neck[1]) / drop.metres()
                    - 1.;
                ensure!(
                    residual / length <= NECKLINE_JUNCTION_TOLERANCE_M,
                    "opening cap does not meet the curved neckline continuously"
                );
            }
        }
        Ok(())
    }

    /// Native polygon/segment construction in lateral/upright metres. A
    /// flared edge must FIRST meet the neckline at the requested upper width;
    /// reject a combination whose segment already entered it further down.
    fn round_opening_top(
        &self,
        shape: NecklineShape,
        radius: f32,
        drop: f32,
        lower_half_width: f32,
        upper_half_width: f32,
        hem: f32,
    ) -> Result<f32> {
        let height = |angle: f32, across: f32| {
            let [lateral, down] = neckline_tangent(shape, angle);
            self.neck[1] + drop * (lateral * across / radius - 1.) / down
        };
        let top = (1..CUT_ARC_SEGMENTS)
            .map(|segment| {
                height(
                    std::f32::consts::PI * segment as f32 / CUT_ARC_SEGMENTS as f32,
                    upper_half_width,
                )
            })
            .fold(f32::NEG_INFINITY, f32::max);
        let first_junction = (1..CUT_ARC_SEGMENTS).any(|segment| {
            let angle = std::f32::consts::PI * segment as f32 / CUT_ARC_SEGMENTS as f32;
            (height(angle, upper_half_width) - top).abs() <= NECKLINE_JUNCTION_TOLERANCE_M
                && height(angle, lower_half_width) > hem + NECKLINE_JUNCTION_TOLERANCE_M
        });
        ensure!(
            first_junction,
            "opening flare enters the round neckline below its requested upper width"
        );
        Ok(top)
    }

    fn facings(&self, design: &BodiceDesign, width: f32, band: f32) -> Result<Vec<ConvexRegion>> {
        let Some(opening) = self.opening_cut(design, band)? else {
            return Ok(Vec::new());
        };
        let mut regions = Vec::new();
        for sign in [-1., 1.] {
            let mut normal = self.lateral.map(|v| v * sign);
            normal[1] -= opening.slope;
            let inner = sign * dot(self.waist, self.lateral) + opening.lower_half_width
                - opening.slope * opening.hem;
            regions.push(vec![
                self.depth_half(opening.side),
                Plane {
                    normal,
                    offset: inner + width * dot(normal, normal).sqrt(),
                },
                Plane {
                    normal: normal.map(|v| -v),
                    offset: -inner,
                },
            ]);
        }
        let opposite_drop = if opening.side > 0. {
            design.neckline.back_drop
        } else {
            design.neckline.front_drop
        };
        let radius = self.breadth() * design.neckline.width.unit() * 0.5;
        let mut neck = vec![
            self.depth_half(-opening.side),
            Plane {
                normal: [0., -1., 0.],
                offset: -self.neck[1] + opposite_drop.metres() + width,
            },
        ];
        for sign in [-1., 1.] {
            neck.push(Plane {
                normal: self.lateral.map(|v| v * sign),
                offset: sign * dot(self.neck, self.lateral) + radius + width,
            });
        }
        regions.push(neck);
        Ok(regions)
    }
}

/// Unit superellipse support plane: ellipse exponent two, soft square four.
impl OpeningCut {
    /// Native unit-normal opening plane, shared by cutting and edge ownership.
    pub(crate) fn edge_plane(&self, frame: &ClothTorsoFrame, sign: f32) -> Plane {
        let mut normal = frame.lateral.map(|v| v * sign);
        normal[1] -= self.slope;
        let length = dot(normal, normal).sqrt();
        Plane {
            normal: normal.map(|v| v / length),
            offset: (sign * dot(frame.waist, frame.lateral) + self.lower_half_width
                - self.slope * self.hem)
                / length,
        }
    }
}

pub(super) fn neckline_tangent(shape: NecklineShape, angle: f32) -> [f32; 2] {
    if shape == NecklineShape::SoftSquare {
        let x = angle.cos();
        let y = angle.sin();
        [x.signum() * x.abs().powf(1.5), y.powf(1.5)]
    } else {
        [angle.cos(), angle.sin()]
    }
}

pub(super) fn cut_positions(body: &Wearer<'_>, cut: &SurfaceCut) -> Vec<[f32; 3]> {
    cut.points
        .iter()
        .map(|point| {
            crate::surface_cut::interpolate(
                body.faces[point.triangle].map(|v| body.positions[v as usize]),
                point.weights,
            )
        })
        .collect()
}

/// The Boolean cutter's snapping tolerances are metres. Normalize both sides
/// of every half-space so ellipse radii and opening slopes cannot rescale them.
fn normalized_regions(regions: Vec<ConvexRegion>) -> Vec<ConvexRegion> {
    regions
        .into_iter()
        .map(|region| {
            region
                .into_iter()
                .map(|plane| {
                    let length = dot(plane.normal, plane.normal).sqrt();
                    Plane {
                        normal: plane.normal.map(|v| v / length),
                        offset: plane.offset / length,
                    }
                })
                .collect()
        })
        .collect()
}
