//! Solid blade lofts preserve the full ricasso and selected cross-section.
use super::*;
use std::f64::consts::PI;
pub(super) fn blade(p: &LoftedBladeParameters, detail: Detail) -> Result<Solid, String> {
    let samples = detail.samples(p.samples.0 as usize, 4);
    let ricasso = p.ricasso.get() / p.length.get();
    let ring = |t: f64| {
        let edge_t = ((t - ricasso) / (1.0 - ricasso)).clamp(0.0, 1.0);
        let sine = (PI * edge_t).sin();
        let profile = match p.plan {
            BladePlan::Straight => 1.0,
            BladePlan::Leaf => 0.78 + 0.22 * sine,
            BladePlan::Cleaver => 0.9 + 0.24 * sine,
        };
        let taper = 0.025 + 0.975 * (1.0 - edge_t).powf(p.taper.get());
        let half = p.width.get() / 2.0 * profile * taper * (1.0 + p.belly.get() * sine);
        let center = p.curvature.get() * edge_t * edge_t + half * p.single_edge.get() * 0.35;
        let depth = p.thickness.get() / 2.0 * (1.0 - edge_t * 0.72);
        let section = match p.section {
            BladeCrossSection::Diamond => {
                vec![[-half, 0.0], [0.0, depth], [half, 0.0], [0.0, -depth]]
            }
            BladeCrossSection::Fullered => vec![
                [-half, 0.0],
                [-half * 0.72, depth],
                [-half * 0.28, depth * 0.32],
                [0.0, depth * 0.22],
                [half * 0.28, depth * 0.32],
                [half * 0.72, depth],
                [half, 0.0],
                [0.0, -depth],
            ],
        };
        section
            .into_iter()
            .map(|[x, z]| [center + x, t * p.length.get(), z])
            .collect::<Vec<Point>>()
    };
    let mut stations: Vec<_> = (0..=samples).map(|i| i as f64 / samples as f64).collect();
    if ricasso > 0.0 {
        stations.push(ricasso);
        stations.sort_by(f64::total_cmp);
        stations.dedup();
    }
    let mut solid = Solid::default();
    let rings: Vec<_> = stations.into_iter().map(ring).collect();
    for rows in rings.windows(2) {
        for side in 0..rows[0].len() {
            let next = (side + 1) % rows[0].len();
            solid.quad(
                rows[0][side],
                rows[0][next],
                rows[1][next],
                rows[1][side],
                1,
            );
        }
    }
    for (row, reverse) in [(0, true), (rings.len() - 1, false)] {
        let outline: Vec<_> = rings[row].iter().map(|p| [p[0], p[2]]).collect();
        let region = Region::triangulate(&outline, false)?;
        for [a, b, c] in region.triangles {
            let point = |index: usize| {
                let [x, z] = region.points[index];
                [x, rings[row][0][1], z]
            };
            if reverse {
                solid.triangle(point(a), point(b), point(c), 0);
            } else {
                solid.triangle(point(a), point(c), point(b), 0);
            }
        }
    }
    Ok(solid.positive())
}
