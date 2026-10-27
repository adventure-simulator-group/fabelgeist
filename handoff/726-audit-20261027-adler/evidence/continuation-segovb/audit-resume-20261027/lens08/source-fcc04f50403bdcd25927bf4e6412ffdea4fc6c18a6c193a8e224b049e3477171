//! Forged blade, axe and spear surface profiles.
use super::*;
use std::f64::consts::PI;

pub(super) fn blade(p: &BladeParameters, detail: Detail) -> Result<Solid, String> {
    generic_blade::blade(p, detail)
}

pub(super) fn axe(p: &AxeParameters, detail: Detail) -> Result<Solid, String> {
    let width = p.width.get();
    let height = p.height.get();
    let thickness = p.thickness.get();
    let side = p.side.map_or(1.0, Direction::sign);
    let root = p.root_width.map_or((width * 0.18).min(0.055), Metres::get);
    let flare = p.flare.map_or(0.0, Ratio::get);
    let curvature = p.curvature.map_or(0.12, Ratio::get);
    let beard = p.beard.map_or(0.35, Ratio::get);
    let upper = p.upper_shoulder.map_or(0.38, Ratio::get);
    let lower = p.lower_shoulder.map_or(0.26, Ratio::get);
    let mut outline = vec![
        [-root * side, height * upper],
        [
            width * 0.3 * side,
            height * (upper - p.upper_cusp.map_or(0.0, Ratio::get)),
        ],
        [
            width * (0.68 + flare * 0.12) * side,
            height * (0.48 + p.toe.map_or(0.0, Ratio::get)),
        ],
    ];
    outline.extend(adaptive_curve(
        |t| {
            [
                width * (0.82 + flare * (t - 0.5) + curvature * (PI * t).sin()) * side,
                height * (0.42 - t * 0.9),
            ]
        },
        CurveQuality {
            minimum_segments: 8,
            max_chord: width.max(height) / 18.0,
            max_deviation: width / 220.0,
        },
        detail,
    ));
    outline.extend([
        [
            width * beard * side,
            -height
                * (0.34
                    + p.beard_drop.map_or(beard * 0.45, Ratio::get)
                    + p.heel.map_or(0.0, Ratio::get)),
        ],
        [
            width * 0.18 * side,
            -height * (lower - p.lower_cusp.map_or(0.0, Ratio::get)),
        ],
        [-root * side, -height * lower],
    ]);
    if p.shoulder_construction == Some(AxeShoulderConstruction::Straight) {
        outline.remove(outline.len() - 2);
        outline.remove(1);
    }
    if side < 0.0 {
        outline.reverse();
    }
    let thickness_root = p.root_width.map_or(width * 0.18, Metres::get);
    Solid::shaped_plate(
        &outline,
        |x, y| {
            let t = ((0.42 - y / height) / 0.9).clamp(0.0, 1.0);
            let edge = width * (0.82 + flare * (t - 0.5) + curvature * (PI * t).sin());
            let remaining =
                1.0 - ((side * x + thickness_root) / (edge + thickness_root)).clamp(0.0, 1.0);
            0.0006 + (thickness - 0.0006) * remaining
        },
        detail,
    )
}

pub(super) fn section_blade(p: &SectionBladeParameters, detail: Detail) -> Result<Solid, String> {
    blade_sections::blade(
        BladeProfile::from(p),
        blade_sections::BladeSampling::Section,
        detail,
    )
}

pub(super) fn diamond_blade(p: &DiamondBladeParameters, detail: Detail) -> Solid {
    let length = p.length.get();
    let width = p.width.get();
    let thickness = p.thickness.get();
    let taper = p.taper.map_or(0.92, Ratio::get);
    let samples = detail.samples(14, 4);
    let ring = |t: f64| {
        let w = width * 0.5 * (1.0 - t * taper);
        let d = thickness * 0.5 * (1.0 - t * 0.7);
        [
            [-w, t * length, 0.0],
            [0.0, t * length, d],
            [w, t * length, 0.0],
            [0.0, t * length, -d],
        ]
    };
    let mut solid = Solid::default();
    for i in 0..samples {
        let a = ring(i as f64 / samples as f64);
        let b = ring((i + 1) as f64 / samples as f64);
        for side in 0..4 {
            let next = (side + 1) % 4;
            solid.quad(a[side], a[next], b[next], b[side], 0);
        }
    }
    // Both finite section ends close the solid; the old preview omitted these.
    for (t, reverse) in [(0.0, true), (1.0, false)] {
        let ring = ring(t);
        for side in 0..4 {
            let a = ring[side];
            let b = ring[(side + 1) % 4];
            if reverse {
                solid.triangle([0.0, t * length, 0.0], b, a, 0);
            } else {
                solid.triangle([0.0, t * length, 0.0], a, b, 0);
            }
        }
    }
    solid.positive()
}

pub(super) fn fork(p: &ForkParameters, detail: Detail) -> Result<Solid, String> {
    let length = p.length.get();
    let width = p.width.get();
    let half = width / 2.0;
    let root = p.base_width.get() / 2.0;
    let tine = p.working_tine_width().get();
    let taper = p.tine_taper.map_or(0.55, Ratio::get);
    let shoulder = p.shoulder_blend.map_or(0.2, Ratio::get);
    let crotch = p.crotch.map_or(0.34, Ratio::get);
    let round = p.crotch_round.map_or(0.05, Ratio::get);
    Solid::prism(
        &[
            [-root, 0.0],
            [-half, length * shoulder],
            [-half, length],
            [-half + tine * taper, length * 0.94],
            [-tine * 0.45, length * (crotch + round)],
            [0.0, length * crotch],
            [tine * 0.45, length * (crotch + round)],
            [half - tine * taper, length * 0.94],
            [half, length],
            [half, length * shoulder],
            [root, 0.0],
        ],
        p.thickness.get(),
        detail,
    )
}

pub(super) fn partisan(p: &PartisanParameters, detail: Detail) -> Result<Solid, String> {
    let length = p.length.get();
    let width = p.width.get();
    let lug = p.lug_width.get() / 2.0;
    let belly = p.belly_position.map_or(0.32, Ratio::get);
    let root = p.root_width.map_or(width * 0.18, Metres::get);
    let drop = p.lug_drop.map_or(0.08, Ratio::get);
    let sweep = p.lug_sweep.map_or(0.055, Ratio::get);
    let acuteness = p.acuteness.map_or(1.0, Ratio::get);
    let shoulder = length * belly.clamp(0.18, 0.48);
    let point = shoulder + (length - shoulder) * (1.0 - 1.0 / (1.0 + acuteness));
    Solid::prism(
        &[
            [0.0, length],
            [-width * 0.34, point],
            [-width / 2.0, shoulder],
            [-width * 0.34, length * 0.12],
            [-lug, length * drop],
            [-lug * 0.72, 0.0],
            [-root / 2.0, length * sweep],
            [root / 2.0, length * sweep],
            [lug * 0.72, 0.0],
            [lug, length * drop],
            [width * 0.34, length * 0.12],
            [width / 2.0, shoulder],
            [width * 0.34, point],
        ],
        p.thickness.get(),
        detail,
    )
}

pub(super) fn glaive(p: &GlaiveParameters, detail: Detail) -> Result<Solid, String> {
    let outline = glaive_outline(p, detail);
    let length = p.length.get();
    Solid::shaped_plate(
        &outline,
        |_x, y| p.thickness.get() * (1.0 - 0.6 * (y / length).clamp(0.0, 1.0)),
        detail,
    )
}

pub(super) fn glaive_outline(p: &GlaiveParameters, detail: Detail) -> Vec<PlanarPoint> {
    let length = p.length.get();
    let width = p.width.get();
    let root = p.root.map_or(0.035, Metres::get);
    let curvature = p.curvature.map_or(0.1, Metres::get);
    let belly = p.belly_position.map_or(0.42, Ratio::get);
    let point = p.point_length.map_or(0.24, Ratio::get);
    let root_length = p.root_length.map_or(0.08, Metres::get);
    let boundary = |side: f64| {
        adaptive_curve(
            |t| {
                let phase = if t < belly {
                    t / belly
                } else {
                    (1.0 - t) / (1.0 - belly)
                };
                let swelling = (PI * phase / 2.0).sin().powi(2);
                let tip = ((1.0 - t) / point).clamp(0.0, 1.0);
                let proportion = if side > 0.0 {
                    0.75 + p.edge_curvature.map_or(0.24, Ratio::get) * 0.2
                } else {
                    0.25 + p.spine_curvature.map_or(0.2, Ratio::get) * 0.2
                };
                let breadth = (root / 2.0 * (1.0 - swelling) + width * swelling * proportion) * tip;
                [curvature * t * t + side * breadth, length * t]
            },
            CurveQuality {
                minimum_segments: 18,
                max_chord: length / 28.0,
                max_deviation: width / 220.0,
            },
            detail,
        )
    };
    let mut outline = vec![[root / 2.0, -root_length]];
    outline.extend(boundary(1.0));
    let mut spine = boundary(-1.0);
    spine.pop();
    outline.extend(spine.into_iter().rev());
    outline.push([-root / 2.0, -root_length]);
    outline
}
