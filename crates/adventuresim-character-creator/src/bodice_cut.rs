//! Boolean boundaries for fitted cloth in canonical, unposed Y-up metres.
use crate::armor_frames::Wearer;
use crate::surface_cut::{ConvexRegion, Plane, SurfaceCut, connected_patch, dot, weld_patch};
use anyhow::Context;
use anyhow::{Result, ensure};
use fabelgeist_armor::{
    BodiceDesign, BodiceOpening, InterliningCoverage, Millimeters, NecklineShape, PartFrame,
};
use fabelgeist_rig::RigJointName;

const CUT_ARC_SEGMENTS: usize = 16;
const NECK_TOP_MARGIN_M: f32 = 0.08;
const NECKLINE_JUNCTION_TOLERANCE_M: f32 = 0.000001;

pub(crate) struct BodiceSurface {
    pub cut: SurfaceCut,
    pub facing: Option<SurfaceCut>,
}

/// Native cut-plane parameters in canonical lateral/upright metres.
struct OpeningCut {
    side: f32,
    slope: f32,
    lower_half_width: f32,
    hem: f32,
}

/// Numerical construction boundary: joints are metres; x/z are the fitted
/// head frame's lateral/anterior axes, not animation ownership.
struct Landmarks {
    waist: [f32; 3],
    neck: [f32; 3],
    shoulders: [[f32; 3]; 2],
    lateral: [f32; 3],
    anterior: [f32; 3],
}

impl BodiceSurface {
    pub fn new(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        head: &PartFrame,
        band: Millimeters,
    ) -> Result<Self> {
        design.validate()?;
        let marks = Landmarks::new(body, head)?;
        ensure!(
            marks.breadth() > 0. && marks.neck[1] > marks.waist[1] + design.hem_rise.metres(),
            "invalid bodice anatomical span"
        );
        let include = normalized_regions(marks.torso(design));
        let subtract = normalized_regions(marks.openings(design, band.metres())?);
        let cut = SurfaceCut::new(body.positions, body.faces, body.faces, &include, &subtract);
        let positions = cut_positions(body, &cut);
        let cut = connected_patch(cut, &positions, marks.waist)?;
        let reference = cut_positions(body, &cut);
        let cut = cut.collapse_cut_roundoff(&reference)?;
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
        let borders = cut.boundary_loops(&reference)?;
        ensure!(
            borders.len() == expected_boundaries,
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
                Some(weld_patch(facing, &positions).cut)
            }
        };

        Ok(Self { cut, facing })
    }
}

impl Landmarks {
    fn new(body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        let joint = |name: RigJointName| -> Result<[f32; 3]> {
            let index = usize::from(name.require_in(body.joint_names)?);
            let state = body
                .joints
                .get(index)
                .context("missing bodice joint state")?;
            Ok([state[0], state[1], state[2]])
        };
        let shoulders = [joint(RigJointName::R_UPARM)?, joint(RigJointName::L_UPARM)?];
        let span = std::array::from_fn(|axis| shoulders[1][axis] - shoulders[0][axis]);
        let lateral = if dot(span, head.axes[0]) < 0. {
            head.axes[0].map(|v| -v)
        } else {
            head.axes[0]
        };
        Ok(Self {
            waist: joint(RigJointName::ROOT)?,
            neck: joint(RigJointName::C_NECK)?,
            shoulders,
            lateral,
            anterior: head.axes[2],
        })
    }

    fn breadth(&self) -> f32 {
        dot(self.shoulders[1], self.lateral) - dot(self.shoulders[0], self.lateral)
    }

    fn torso(&self, design: &BodiceDesign) -> Vec<ConvexRegion> {
        let mut region = vec![
            Plane {
                normal: [0., -1., 0.],
                offset: -(self.waist[1] + design.hem_rise.metres()),
            },
            Plane {
                normal: [0., 1., 0.],
                offset: self.neck[1] + NECK_TOP_MARGIN_M,
            },
        ];
        for (side, shoulder) in [-1., 1.].into_iter().zip(self.shoulders) {
            region.push(Plane {
                normal: self.lateral.map(|v| v * side),
                offset: side * dot(shoulder, self.lateral),
            });
        }
        vec![region]
    }

    fn openings(&self, design: &BodiceDesign, band: f32) -> Result<Vec<ConvexRegion>> {
        let mut cuts = self.armholes(design);
        let radius = self.breadth() * design.neckline.width.unit() * 0.5;
        for (side, drop) in [
            (1., design.neckline.front_drop),
            (-1., design.neckline.back_drop),
        ] {
            let mut neck = vec![self.depth_half(side)];
            for sign in [-1., 1.] {
                neck.push(Plane {
                    normal: self.lateral.map(|v| v * sign),
                    offset: sign * dot(self.neck, self.lateral) + radius,
                });
            }
            if design.neckline.shape == NecklineShape::Square {
                neck.push(Plane {
                    normal: [0., -1., 0.],
                    offset: -self.neck[1] + drop.metres(),
                });
            } else {
                for segment in 1..CUT_ARC_SEGMENTS {
                    let angle = std::f32::consts::PI * segment as f32 / CUT_ARC_SEGMENTS as f32;
                    let mut normal = self.lateral.map(|v| v * angle.cos() / radius);
                    normal[1] -= angle.sin() / drop.metres().max(f32::EPSILON);
                    neck.push(Plane {
                        normal,
                        offset: dot(self.neck, normal) + 1.,
                    });
                }
            }
            cuts.push(neck);
        }
        let Some(opening) = self.opening_cut(design, band)? else {
            return Ok(cuts);
        };
        let mut region = vec![self.depth_half(opening.side)];
        if band > 0. {
            region.push(Plane {
                normal: [0., -1., 0.],
                offset: -opening.hem,
            });
        }
        for sign in [-1., 1.] {
            let mut normal = self.lateral.map(|v| v * sign);
            normal[1] -= opening.slope;
            region.push(Plane {
                normal,
                offset: sign * dot(self.waist, self.lateral) + opening.lower_half_width
                    - opening.slope * opening.hem,
            });
        }
        cuts.push(region);
        Ok(cuts)
    }

    fn opening_cut(&self, design: &BodiceDesign, band: f32) -> Result<Option<OpeningCut>> {
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
            self.round_opening_top(radius, drop.metres(), waist.metres() * 0.5, half_width, hem)?
        };
        ensure!(top > hem, "bodice neckline must remain above its hem");
        Ok(Some(OpeningCut {
            side,
            slope: (half_width - waist.metres() * 0.5) / (top - hem),
            lower_half_width: waist.metres() * 0.5,
            hem,
        }))
    }

    /// Native polygon/segment construction in lateral/upright metres. A
    /// flared edge must FIRST meet the neckline at the requested upper width;
    /// reject a combination whose segment already entered it further down.
    fn round_opening_top(
        &self,
        radius: f32,
        drop: f32,
        lower_half_width: f32,
        upper_half_width: f32,
        hem: f32,
    ) -> Result<f32> {
        let height = |angle: f32, across: f32| {
            self.neck[1] + drop * (angle.cos() * across / radius - 1.) / angle.sin()
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

    fn armholes(&self, design: &BodiceDesign) -> Vec<ConvexRegion> {
        // A cylindrical armhole cut with an elliptical section in the lateral/
        // upright plane. Its depth and medial reach are independent of the neck.
        self.shoulders
            .iter()
            .map(|shoulder| {
                (0..CUT_ARC_SEGMENTS)
                    .map(|segment| {
                        let angle =
                            std::f32::consts::TAU * segment as f32 / CUT_ARC_SEGMENTS as f32;
                        let mut normal = self
                            .lateral
                            .map(|v| v * angle.cos() / design.armhole_width.metres());
                        normal[1] += angle.sin() / design.armhole_depth.metres();
                        Plane {
                            normal,
                            offset: dot(*shoulder, normal) + 1.,
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn depth_half(&self, side: f32) -> Plane {
        Plane {
            normal: self.anterior.map(|v| -side * v),
            offset: -side * dot(self.neck, self.anterior),
        }
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

fn cut_positions(body: &Wearer<'_>, cut: &SurfaceCut) -> Vec<[f32; 3]> {
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
