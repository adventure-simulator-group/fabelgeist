//! Open-sided shoulder panels retain frozen body-triangle correspondence.
use crate::armor_frames::Wearer;
use crate::cloth_torso_frame::ClothTorsoFrame;
use crate::surface_cut::{
    CUT_VERTEX_SNAP_METRES, ConvexRegion, Plane, SurfaceCut, dot, interpolate, weld_patch,
};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{PartFrame, PartletDesign};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct PartletCut {
    pub surface: SurfaceCut,
    pub reference: Vec<[f32; 3]>,
    pub neck: Vec<u32>,
}

impl PartletCut {
    pub fn new(design: &PartletDesign, body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        design.validate()?;
        let frame = ClothTorsoFrame::new(body, head)?;
        let hem = frame.waist[1] + design.hem_rise.metres();
        let neck = frame.neck[1] + design.neck_rise.metres();
        let top_normal: [f32; 3] = std::array::from_fn(|axis| {
            (if axis == 1 { 1. } else { 0. }) + frame.anterior[axis] * design.neckline_slope.unit()
        });
        let top_length = dot(top_normal, top_normal).sqrt();
        let top = Plane {
            normal: top_normal.map(|v| v / top_length),
            offset: (dot(frame.neck, top_normal) + design.neck_rise.metres()) / top_length,
        };
        let bridge = frame
            .shoulders
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min)
            - design.shoulder_bridge.metres();
        ensure!(
            hem < bridge && bridge < neck && neck - design.back_slit.metres() > hem,
            "partlet needs open side spans and a rear slit ending above its hem"
        );
        let mut torso = vec![
            Plane {
                normal: [0., -1., 0.],
                offset: -hem,
            },
            top,
            Plane {
                normal: [0., 1., 0.],
                offset: neck,
            },
        ];
        for (side, shoulder) in [-1., 1.].into_iter().zip(frame.shoulders) {
            torso.push(Plane {
                normal: frame.lateral.map(|v| v * side),
                offset: side * dot(shoulder, frame.lateral),
            });
        }
        let mut subtract = side_openings(&frame, design, bridge);
        let mut slit = vec![
            frame.depth_half(-1.),
            Plane {
                normal: [0., -1., 0.],
                offset: -(neck - design.back_slit.metres()),
            },
        ];
        for sign in [-1., 1.] {
            slit.push(Plane {
                normal: frame.lateral.map(|v| v * sign),
                offset: sign * dot(frame.neck, frame.lateral) + design.back_gap.metres() * 0.5,
            });
        }
        subtract.push(slit);
        let surface = SurfaceCut::new(body.positions, body.faces, body.faces, &[torso], &subtract);
        let reference = positions(&surface, body);
        let surface = weld_patch(surface, &reference).cut;
        let reference = positions(&surface, body);
        let surface = surface.collapse_cut_roundoff(&reference)?;
        surface.validate_connected()?;
        let reference = positions(&surface, body);
        let boundaries = surface.boundary_loops(&reference)?;
        ensure!(
            boundaries.len() == 2,
            "partlet must retain shoulder bridges, open sides and a neckline"
        );
        let neck = neckline_chain(
            &surface,
            &reference,
            &[
                top,
                Plane {
                    normal: [0., 1., 0.],
                    offset: neck,
                },
            ],
        )?;
        Ok(Self {
            surface,
            reference,
            neck,
        })
    }
}

fn side_openings(frame: &ClothTorsoFrame, design: &PartletDesign, top: f32) -> Vec<ConvexRegion> {
    let slope = (design.side_opening.unit() * std::f32::consts::FRAC_PI_2).tan();
    [-1., 1.]
        .into_iter()
        .map(|side| {
            let mut region = vec![
                Plane {
                    normal: [0., 1., 0.],
                    offset: top,
                },
                Plane {
                    normal: frame.lateral.map(|v| -side * v),
                    offset: -side * dot(frame.neck, frame.lateral),
                },
            ];
            for direction in [-1., 1.] {
                let normal = std::array::from_fn(|i| {
                    direction * frame.anterior[i] - slope * side * frame.lateral[i]
                });
                let length = dot(normal, normal).sqrt();
                region.push(Plane {
                    normal: normal.map(|v| v / length),
                    offset: dot(frame.neck, normal) / length,
                });
            }
            region
        })
        .collect()
}

fn positions(surface: &SurfaceCut, body: &Wearer<'_>) -> Vec<[f32; 3]> {
    surface
        .points
        .iter()
        .map(|point| {
            interpolate(
                body.faces[point.triangle].map(|v| body.positions[v as usize]),
                point.weights,
            )
        })
        .collect()
}

/// Only the actual top cut belongs to the collar. Rear slit edges remain free.
fn neckline_chain(surface: &SurfaceCut, points: &[[f32; 3]], top: &[Plane]) -> Result<Vec<u32>> {
    let mut edges = BTreeMap::new();
    let mut incoming = BTreeSet::new();
    for [a, b] in surface.borders(points) {
        if top.iter().any(|plane| {
            [a, b].iter().all(|&v| {
                (dot(points[v as usize], plane.normal) - plane.offset).abs()
                    <= 2. * CUT_VERTEX_SNAP_METRES
            })
        }) {
            ensure!(edges.insert(a, b).is_none(), "partlet neckline branches");
            incoming.insert(b);
        }
    }
    let starts = edges
        .keys()
        .filter(|v| !incoming.contains(v))
        .copied()
        .collect::<Vec<_>>();
    ensure!(
        starts.len() == 1,
        "partlet collar needs one open neck chain"
    );
    let mut at = *starts.first().context("partlet neckline is empty")?;
    let mut chain = vec![at];
    while let Some(next) = edges.remove(&at) {
        chain.push(next);
        at = next;
    }
    ensure!(
        edges.is_empty() && chain.len() >= 3,
        "partlet neckline is disconnected"
    );
    Ok(chain)
}
