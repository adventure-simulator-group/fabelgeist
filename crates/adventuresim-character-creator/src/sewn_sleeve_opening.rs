//! Actual periodic cut edges supply the cuff's closure anchors.
use crate::cloth_opening::{ClothEdgeStation, ClothOpening};
use crate::surface_cut::{CORRESPONDENCE_PRECISION, CUT_VERTEX_SNAP_METRES};
use anyhow::{Result, ensure};
use fabelgeist_armor::SewnSleeves;
use std::collections::{BTreeMap, BTreeSet};

impl ClothOpening {
    /// Native patch ordinals, normalized sleeve UVs, reference reach in metres.
    pub fn cuff(
        first: u32,
        chart: &[[f32; 2]],
        indices: &[u32],
        design: &SewnSleeves,
        reach: f32,
    ) -> Result<Option<Self>> {
        let Some(slit) = &design.cuff_slit else {
            return Ok(None);
        };
        let tip = 1. - slit.length.metres() / reach;
        let circumference = design.cuff_circumference.metres()
            + std::f32::consts::TAU * design.frill.as_ref().map_or(0., |f| f.flare.metres());
        // Boolean cuts can snap to a source corner within their physical
        // tolerance. Match that same edge rather than an ideal unsnapped line;
        // angle, axial chart and opening progress have different scales.
        let cut_tolerance = CUT_VERTEX_SNAP_METRES + 1. / CORRESPONDENCE_PRECISION;
        let slope = slit.gap.metres() / (2. * slit.length.metres());
        let angular_tolerance = cut_tolerance * (1. + slope * slope).sqrt() / circumference;
        let axial_tolerance = cut_tolerance / reach;
        let station_tolerance = 1. / (CORRESPONDENCE_PRECISION * reach);
        let half_gap = slit.gap.metres() / (2. * circumference);
        let mut edges = BTreeMap::<[u32; 2], usize>::new();
        for face in indices.as_chunks::<3>().0 {
            for [mut a, mut b] in [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]] {
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }
                *edges.entry([a, b]).or_default() += 1;
            }
        }
        let uv = |vertex: u32| {
            vertex
                .checked_sub(first)
                .and_then(|i| chart.get(i as usize))
                .copied()
        };
        let phase = |u: f32| (u - slit.rotation.unit() + 0.5).rem_euclid(1.) - 0.5;
        let mut sides = [BTreeSet::new(), BTreeSet::new()];
        for ([a, b], count) in edges {
            if count != 1 {
                continue;
            }
            let (Some(uv_a), Some(uv_b)) = (uv(a), uv(b)) else {
                continue;
            };
            if uv_a[1].min(uv_b[1]) < tip - axial_tolerance
                || (uv_a[1] - uv_b[1]).abs() <= station_tolerance
            {
                continue;
            }
            let angles = [phase(uv_a[0]), phase(uv_b[0])];
            if angles
                .iter()
                .any(|v| v.abs() > half_gap + angular_tolerance)
            {
                continue;
            }
            let side = usize::from(angles[0] + angles[1] > 0.);
            sides[side].extend([a, b]);
        }
        let stations = |vertices: &BTreeSet<u32>| -> Result<Vec<ClothEdgeStation>> {
            let mut stations = vertices
                .iter()
                .map(|&vertex| ClothEdgeStation {
                    vertex: vertex as usize,
                    progress: chart[(vertex - first) as usize][1],
                })
                .collect::<Vec<_>>();
            stations.sort_by(|a, b| a.progress.total_cmp(&b.progress));
            // Keep both physical endpoints, including a snapped tip. A
            // near-duplicate interior station must not shorten the opening.
            let end = stations.last().copied();
            stations.dedup_by(|a, b| (a.progress - b.progress).abs() < station_tolerance);
            if let (Some(end), Some(last)) = (end, stations.last_mut()) {
                *last = end;
            }
            ensure!(
                stations.len() >= 2,
                "cuff slit has no physical closure edge"
            );
            let start = stations[0].progress;
            let span = stations[stations.len() - 1].progress - start;
            ensure!(span > 0., "cuff opening stations must advance");
            for station in &mut stations {
                station.progress = (station.progress - start) / span;
            }
            Ok(stations)
        };
        Ok(Some(Self {
            left: stations(&sides[0])?,
            right: stations(&sides[1])?,
        }))
    }
}
