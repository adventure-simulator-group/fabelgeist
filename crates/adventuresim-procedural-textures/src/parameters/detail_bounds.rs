//! Sampling support bounds for finite stamps and cut-board anatomy.
pub(super) fn bounds(path: &str, name: &str) -> Option<(f64, f64)> {
    if path.starts_with("/rock/facets/") || path.starts_with("/surface/plates/") {
        return Some(match name {
            "cells" => (1.0, 64.0),
            _ => (0.0, 1.0),
        });
    }
    if matches!(
        name,
        "aggregate_angularity" | "litter_patch_strength" | "tool_smoothing"
    ) {
        return Some((0.0, 1.0));
    }
    if name == "edge_width_metres" {
        return Some((0.001, 0.04));
    }
    let stamp_roots = [
        "/handmade_brick/pores/",
        "/handmade_brick/mortar_grit/",
        "/rock/spalls/",
        "/rock/pores/",
        "/lime_plaster/trowel_strokes/",
        "/lime_plaster/float_tracks/",
        "/wattle_and_daub/application/",
        "/rubble_masonry/pores/",
        "/rubble_masonry/mortar_grit/",
        "/clay_roof_tile/pores/",
        "/clay_roof_tile/drag_marks/",
        "/lead_sheet/dents/",
    ];
    if stamp_roots.iter().any(|root| path.starts_with(root)) {
        return Some(match name {
            "cells" => (1.0, 256.0),
            "cluster_cells" => (1.0, 16.0),
            "radius" => (0.03, 0.9),
            "size_variation" | "jitter" => (0.0, 0.9),
            "edge_width" => (0.03, 1.0),
            "angle" => (-std::f64::consts::PI, std::f64::consts::PI),
            "angle_variation" => (0.0, std::f64::consts::TAU),
            "depth" => (0.0, 0.5),
            _ => (0.0, 1.0),
        });
    }
    if path.starts_with("/plank_floor/grain/") || path.starts_with("/timber_shingle/grain/") {
        return Some(match name {
            "ring_count" => (1.0, 48.0),
            "fiber_count" => (1.0, 128.0),
            "ring_width" => (0.04, 0.45),
            "knot_radius" => (0.01, 0.16),
            "knot_flow" => (0.0, 2.0),
            "knot_influence" => (1.0, 3.0),
            "knot_core" => (0.05, 1.0),
            "knot_margin" => (0.0, 0.45),
            "ring_width_variation" => (0.0, 0.3),
            "ring_shoulder" => (0.1, 0.5),
            "wander_cells" => (1.0, 24.0),
            "sawn_arch_depth" => (0.01, 1.0),
            "ring_wander" => (0.0, 0.2),
            "ring_depth" | "fiber_depth" => (0.0, 0.1),
            "light_srgb" | "dark_srgb" => (0.0, 255.0),
            _ => (0.0, 1.0),
        });
    }
    None
}
