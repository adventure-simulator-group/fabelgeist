//! Hammer faces, piercing beaks and bill hooks.
use super::*;

pub(super) fn beak(p: &BeakParameters, detail: Detail) -> Result<Solid, String> {
    let length = p.length.get();
    let radius = p.radius.get();
    let direction = p.direction.map_or(-1.0, Direction::sign);
    let curvature = p.curvature.map_or(0.16, Metres::get);
    let root = p.working_root().get();
    let tip = p.tip_section.map_or(radius * 0.12, Metres::get);
    let bend = p.bend_position.map_or(0.55, Ratio::get);
    let droop = p.droop.map_or(curvature * 0.35, Metres::get);
    let exponent = 0.5_f64.ln() / bend.clamp(0.15, 0.85).ln();
    let edge = |sign: f64| {
        adaptive_curve(
            |t| {
                [
                    direction * length * t,
                    curvature
                        * if p.bend_profile == Some(BeakBendProfile::SineArch) {
                            (std::f64::consts::PI * t.powf(exponent)).sin()
                        } else {
                            t.powf(exponent)
                        }
                        + droop * t
                        + sign * (root * (1.0 - t) + tip * t) / 2.0,
                ]
            },
            CurveQuality {
                minimum_segments: 12,
                max_chord: length / 22.0,
                max_deviation: curvature.max(root) / 180.0,
            },
            detail,
        )
    };
    let mut outline = edge(1.0);
    outline.extend(edge(-1.0).into_iter().rev());
    Solid::prism(&outline, p.working_thickness().get(), detail)
}

pub(super) fn hammer(p: &HammerParameters, detail: Detail) -> Result<Solid, String> {
    let length = p.length.get();
    let face = p.face.get();
    let neck = p.neck.get();
    let direction = p.direction.map_or(1.0, Direction::sign);
    let ratio = p.neck_ratio.map_or(0.72, Ratio::get).clamp(0.05, 0.88);
    let neck_length = length * ratio;
    let face_height = face * (1.0 + p.face_flare.map_or(0.0, Ratio::get));
    let crown = p
        .crown_length
        .map_or(length * p.crown.map_or(0.12, Ratio::get), Metres::get)
        .max(0.0);
    Solid::prism(
        &[
            [0.0, -neck / 2.0],
            [neck_length * direction, -neck / 2.0],
            [length * direction, -face_height / 2.0],
            [(length + crown) * direction, 0.0],
            [length * direction, face_height / 2.0],
            [neck_length * direction, neck / 2.0],
            [0.0, neck / 2.0],
        ],
        p.face_thickness.map_or(p.thickness.get(), Metres::get),
        detail,
    )
}

pub(super) fn faceted_beak(p: &FacetedBeakParameters, detail: Detail) -> Result<Solid, String> {
    let length = p.length.get();
    let root = p.root.get();
    let tip = p.tip.map_or(root * 0.16, Metres::get);
    let direction = p.direction.map_or(-1.0, Direction::sign);
    let set = p.set.map_or(0.0, Metres::get);
    let bend = p.bend_position.map_or(0.22, Ratio::get).clamp(0.12, 0.7);
    Solid::prism(
        &[
            [0.0, -root / 2.0],
            [length * bend * direction, set * bend - root * 0.42],
            [length * direction, set - tip / 2.0],
            [length * direction, set + tip / 2.0],
            [length * bend * direction, set * bend + root * 0.42],
            [0.0, root / 2.0],
        ],
        p.tip_thickness.map_or(p.thickness.get(), Metres::get),
        detail,
    )
}

pub(super) fn bill(p: &BillParameters, detail: Detail) -> Result<Solid, String> {
    Solid::prism(&bill_outline(p, detail), p.thickness.get(), detail)
}

pub(super) fn bill_spans(p: &BillParameters) -> [[PlanarPoint; 4]; 6] {
    let length = p.length.get();
    let width = p.width.get();
    let hook = p.hook.get();
    let root = p.root.map_or(0.032, Metres::get);
    let root_length = p.root_length.map_or(0.06, Metres::get);
    let belly = p.belly_position.map_or(0.48, Ratio::get);
    let depth = p.hook_depth.map_or(0.19, Ratio::get);
    let curve = p.hook_curvature.map_or(0.22, Ratio::get);
    let point = p.point_length.map_or(0.24, Ratio::get);
    let root_left = [-root, -root_length];
    let apex = [0.0, length];
    let shoulder = [width, length * 0.68];
    let upper = [width + hook * 0.72, length * (0.68 + curve * 0.55)];
    let tip = [width + hook, length * (0.68 - depth)];
    let inner = [width + hook * 0.46, length * (0.62 + curve * 0.2)];
    let root_right = [root, -root_length];
    let shoulder_handle = width.min(hook) * 0.16;
    let crown_handle = hook * 0.18;
    let inner_handle = hook * 0.16;
    [
        [
            root_left,
            [-root * 0.92, length * (1.0 - belly) * 0.42],
            [-root * 0.52, length * 0.84],
            apex,
        ],
        [
            apex,
            [width * 0.1, length * (0.97 - point * 0.08)],
            [shoulder[0] - shoulder_handle, shoulder[1]],
            shoulder,
        ],
        [
            shoulder,
            [shoulder[0] + shoulder_handle, shoulder[1]],
            [upper[0] - crown_handle, upper[1]],
            upper,
        ],
        [
            upper,
            [upper[0] + crown_handle, upper[1]],
            [tip[0], length * (0.62 - depth * 0.25)],
            tip,
        ],
        [
            tip,
            [tip[0], length * (0.56 - depth)],
            [inner[0] + inner_handle, inner[1]],
            inner,
        ],
        [
            inner,
            [inner[0] - inner_handle, inner[1]],
            [width * 0.62, length * belly * 0.28],
            root_right,
        ],
    ]
}

pub(super) fn bill_outline(p: &BillParameters, detail: Detail) -> Vec<PlanarPoint> {
    let length = p.length.get();
    let width = p.width.get();
    let hook = p.hook.get();
    let mut outline = Vec::new();
    for span in bill_spans(p) {
        append_curve(
            &mut outline,
            cubic_bezier(
                span,
                CurveQuality {
                    minimum_segments: 3,
                    max_chord: length / 28.0,
                    max_deviation: width.min(hook) / 90.0,
                },
                detail,
            ),
        );
    }
    outline
}
