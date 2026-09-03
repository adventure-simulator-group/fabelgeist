//! Parity with the Python reference implementation.
//!
//! Every expected number here was produced by running the original
//! [GarmentCode](https://github.com/maria-korosteleva/GarmentCode) on the same
//! body and design files that ship in this crate's `assets/` directory, and
//! reading the resulting `*_specification.json`.
//!
//! Panel perimeters are the sharpest single check available: they fold in every
//! vertex position *and* every curve parameter, so a wrong control point or a
//! missing subdivision moves them.

use std::path::PathBuf;

use fabelgeist_garment_code::pattern::PatternSpec;
use fabelgeist_garment_code::programs::MetaGarment;
use fabelgeist_garment_code::{Body, Design};

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn build(design_file: &str, name: &str) -> PatternSpec {
    let body = Body::load(assets().join("bodies/mean_all.yaml")).expect("body");
    let design = Design::load(assets().join(design_file)).expect("design");
    let piece = MetaGarment::new(name, &body, &design);
    piece.assembly()
}

/// Total outline length of a panel, curves included.
fn perimeter(spec: &PatternSpec, panel: &str) -> f64 {
    let p = spec
        .panel(panel)
        .unwrap_or_else(|| panic!("no panel {panel}"));
    p.edges.iter().map(|e| p.edge_as_curve(e).length()).sum()
}

/// `(name, vertices, edges, perimeter, translation)` from the reference.
type PanelFacts = (&'static str, usize, usize, f64, [f64; 3]);

fn check_panels(spec: &PatternSpec, expected: &[PanelFacts], tol: f64, transl_tol: f64) {
    let mut names = spec.panel_names();
    names.sort();
    let mut want: Vec<&str> = expected.iter().map(|e| e.0).collect();
    want.sort();
    assert_eq!(names, want, "panel names");

    for (name, n_verts, n_edges, perim, translation) in expected {
        let p = spec.panel(name).unwrap();
        assert_eq!(p.vertices.len(), *n_verts, "{name}: vertex count");
        assert_eq!(p.edges.len(), *n_edges, "{name}: edge count");

        let got = perimeter(spec, name);
        assert!(
            (got - perim).abs() < tol,
            "{name}: perimeter {got} != {perim} (reference)"
        );

        for (i, (a, b)) in p.translation.iter().zip(translation).enumerate() {
            assert!(
                (a - b).abs() < transl_tol,
                "{name}: translation[{i}] {a} != {b} (reference)"
            );
        }
    }
}

/// The T-shirt preset: a straight shirt with short sleeves.
///
/// The perimeter tolerance is looser than for the other cases because the
/// sleeve's armhole curve comes out of `curve_match_tangents`, whose objective
/// mixes a `max`-over-samples curvature term and so has no unique minimum --
/// see the crate README.
#[test]
fn t_shirt_matches_reference() {
    let mut spec = build("design_params/t-shirt.yaml", "t-shirt");

    assert_eq!(spec.stitches.len(), 16, "stitch count");
    assert_eq!(
        spec.panel_order(false),
        vec![
            "right_btorso",
            "right_sleeve_b",
            "right_sleeve_f",
            "right_ftorso",
            "left_btorso",
            "left_sleeve_b",
            "left_sleeve_f",
            "left_ftorso",
        ],
        "panel order"
    );

    check_panels(
        &spec,
        &[
            ("left_btorso", 6, 6, 140.219209, [0.0, 91.444895, -25.0]),
            ("left_ftorso", 7, 7, 143.468535, [0.0, 91.400291, 30.0]),
            (
                "left_sleeve_b",
                5,
                5,
                79.831885,
                [37.647987, 126.059872, -12.5],
            ),
            (
                "left_sleeve_f",
                4,
                4,
                79.687430,
                [37.647987, 126.059872, 17.5],
            ),
            ("right_btorso", 6, 6, 140.219209, [0.0, 91.444895, -25.0]),
            ("right_ftorso", 7, 7, 143.468535, [0.0, 91.400291, 30.0]),
            (
                "right_sleeve_b",
                5,
                5,
                79.831885,
                [-37.647987, 126.059872, -12.5],
            ),
            (
                "right_sleeve_f",
                4,
                4,
                79.687430,
                [-37.647987, 126.059872, 17.5],
            ),
        ],
        0.2,
        0.05,
    );
}

/// The default preset with a straight shirt selected.
///
/// `default.yaml` leaves every `meta` slot empty -- it is the configurator's
/// starting point, not a garment -- so the upper is chosen here.
#[test]
fn default_shirt_matches_reference() {
    let design = Design::load(assets().join("design_params/default.yaml")).unwrap();
    design.set_v(
        "meta.upper",
        fabelgeist_garment_code::design::Value::Str("Shirt".into()),
    );
    let body = Body::load(assets().join("bodies/mean_all.yaml")).unwrap();
    let mut spec = MetaGarment::new("shirt", &body, &design).assembly();

    assert_eq!(spec.stitches.len(), 6, "stitch count");
    assert_eq!(
        spec.panel_order(false),
        vec!["right_btorso", "right_ftorso", "left_btorso", "left_ftorso"],
    );

    check_panels(
        &spec,
        &[
            ("left_btorso", 6, 6, 140.219209, [0.0, 91.444895, -25.0]),
            ("left_ftorso", 6, 6, 143.468535, [0.0, 91.400291, 30.0]),
            ("right_btorso", 6, 6, 140.219209, [0.0, 91.444895, -25.0]),
            ("right_ftorso", 6, 6, 143.468535, [0.0, 91.400291, 30.0]),
        ],
        1e-4,
        1e-4,
    );
}

/// The fitted bodice block: darts on both torso panels.
#[test]
fn fitted_shirt_matches_reference() {
    let design = Design::load(assets().join("design_params/default.yaml")).unwrap();
    design.set_v(
        "meta.upper",
        fabelgeist_garment_code::design::Value::Str("FittedShirt".into()),
    );

    let body = Body::load(assets().join("bodies/mean_all.yaml")).unwrap();
    let mut spec = MetaGarment::new("fitted_shirt", &body, &design).assembly();

    assert_eq!(spec.stitches.len(), 16, "stitch count");
    assert_eq!(
        spec.panel_order(false),
        vec!["right_btorso", "right_ftorso", "left_btorso", "left_ftorso"],
    );

    check_panels(
        &spec,
        &[
            ("left_btorso", 13, 13, 157.723856, [0.0, 103.895647, -25.0]),
            ("left_ftorso", 12, 12, 172.536822, [0.0, 97.857756, 30.0]),
            ("right_btorso", 13, 13, 157.723856, [0.0, 103.895647, -25.0]),
            ("right_ftorso", 12, 12, 172.536822, [0.0, 97.857756, 30.0]),
        ],
        1e-2,
        1e-4,
    );
}

/// A pencil skirt with a straight waistband: darts, slits and stitch matching
/// that subdivides one side of a seam.
#[test]
fn pencil_skirt_matches_reference() {
    let design = Design::load(assets().join("design_params/default.yaml")).unwrap();
    use fabelgeist_garment_code::design::Value;
    design.set_v("meta.upper", Value::Null);
    design.set_v("meta.bottom", Value::Str("PencilSkirt".into()));
    design.set_v("meta.wb", Value::Str("StraightWB".into()));

    let body = Body::load(assets().join("bodies/mean_all.yaml")).unwrap();
    let mut spec = MetaGarment::new("pencil_skirt", &body, &design).assembly();

    assert_eq!(spec.stitches.len(), 16, "stitch count");
    assert_eq!(
        spec.panel_order(false),
        vec!["skirt_back", "wb_back", "wb_front", "skirt_front"],
    );

    check_panels(
        &spec,
        &[
            (
                "skirt_back",
                18,
                18,
                372.379022,
                [-27.411850, 41.086188, -20.0],
            ),
            (
                "skirt_front",
                6,
                6,
                209.219966,
                [-24.411850, 42.086188, 25.0],
            ),
            ("wb_back", 8, 8, 87.665080, [-19.567900, 104.772500, -15.0]),
            ("wb_front", 4, 4, 99.789480, [-22.599000, 104.772500, 20.0]),
        ],
        // 0.1 mm on a ~3.7 m outline: the fitted side seams come from
        // `curve_from_tangents`, so they carry the optimiser's last few digits.
        1e-2,
        1e-4,
    );
}

/// Pants with a fitted (yoke) waistband: curved crotch seams and arc panels.
#[test]
fn pants_match_reference() {
    let design = Design::load(assets().join("design_params/default.yaml")).unwrap();
    use fabelgeist_garment_code::design::Value;
    design.set_v("meta.upper", Value::Null);
    design.set_v("meta.bottom", Value::Str("Pants".into()));
    design.set_v("meta.wb", Value::Str("FittedWB".into()));

    let body = Body::load(assets().join("bodies/mean_all.yaml")).unwrap();
    let mut spec = MetaGarment::new("pants_only", &body, &design).assembly();

    assert_eq!(spec.stitches.len(), 24, "stitch count");
    assert_eq!(
        spec.panel_order(false),
        vec![
            "pant_b_r", "pant_f_r", "wb_back", "wb_front", "pant_b_l", "pant_f_l"
        ],
    );

    check_panels(
        &spec,
        &[
            ("pant_b_l", 13, 13, 237.206858, [34.5, 46.254136, -20.0]),
            ("pant_b_r", 13, 13, 237.206858, [-34.5, 46.254136, -20.0]),
            ("pant_f_l", 7, 7, 151.867351, [30.5, 48.602506, 25.0]),
            ("pant_f_r", 7, 7, 151.867351, [-30.5, 48.602506, 25.0]),
            ("wb_back", 9, 9, 90.601257, [-19.250810, 108.772500, -15.0]),
            ("wb_front", 5, 5, 100.682143, [-22.565001, 108.772500, 20.0]),
        ],
        1e-2,
        1e-4,
    );
}

/// Every stitch must name a real panel and a real edge of it.
#[test]
fn stitches_reference_real_edges() {
    let spec = build("design_params/t-shirt.yaml", "t-shirt");

    for stitch in &spec.stitches {
        for side in &stitch.sides {
            let panel = spec
                .panel(&side.panel)
                .unwrap_or_else(|| panic!("stitch names unknown panel {}", side.panel));
            assert!(
                side.edge < panel.edges.len(),
                "stitch edge {} out of range on {} ({} edges)",
                side.edge,
                side.panel,
                panel.edges.len()
            );
        }
    }
}

/// Panels must form closed loops with consistent endpoints.
#[test]
fn panels_are_closed_loops() {
    for (design, name) in [("design_params/t-shirt.yaml", "t-shirt")] {
        let spec = build(design, name);
        for (panel_name, panel) in &spec.panels {
            assert!(!panel.edges.is_empty(), "{panel_name}: no edges");
            for i in 0..panel.edges.len() {
                let next = (i + 1) % panel.edges.len();
                assert_eq!(
                    panel.edges[i].endpoints[1], panel.edges[next].endpoints[0],
                    "{panel_name}: edge {i} does not chain into edge {next}"
                );
                for v in panel.edges[i].endpoints {
                    assert!(
                        v < panel.vertices.len(),
                        "{panel_name}: vertex {v} out of range"
                    );
                }
            }
        }
    }
}

/// The generated pattern must survive a round trip through the JSON writer.
#[test]
fn json_is_well_formed() {
    let mut spec = build("design_params/t-shirt.yaml", "t-shirt");
    spec.panel_order(false);
    let text = spec.to_json();

    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(parsed["pattern"]["panels"].as_object().unwrap().len(), 8);
    assert_eq!(parsed["pattern"]["stitches"].as_array().unwrap().len(), 16);
    assert_eq!(parsed["properties"]["units_in_meter"], 100);
    assert_eq!(parsed["properties"]["curvature_coords"], "relative");
}

/// The SVG renderer must emit one closed path per panel.
#[test]
fn svg_has_a_path_per_panel() {
    use fabelgeist_garment_code::pattern::{Annotations, Layout};

    let mut spec = build("design_params/t-shirt.yaml", "t-shirt");
    let svg = spec.to_svg(
        Layout::Placed,
        Annotations {
            names: true,
            ids: false,
        },
        2.0,
    );

    assert_eq!(svg.matches("<path").count(), 8, "one path per panel");
    for name in spec.panel_names() {
        assert!(svg.contains(&format!(">{name}<")), "missing label {name}");
    }
}

/// No panel of a sane design should cross itself.
#[test]
fn presets_are_not_self_intersecting() {
    let body = Body::load(assets().join("bodies/mean_all.yaml")).unwrap();
    {
        let design_file = "design_params/t-shirt.yaml";
        let design = Design::load(assets().join(design_file)).unwrap();
        let piece = MetaGarment::new("check", &body, &design);
        assert!(
            !piece.is_self_intersecting(),
            "{design_file} produced a self-intersecting panel"
        );
    }
}

/// Every body preset shipped with the crate must build the default design.
#[test]
fn all_body_presets_build() {
    let design = Design::load(assets().join("design_params/t-shirt.yaml")).unwrap();
    for body_file in [
        "bodies/mean_all.yaml",
        "bodies/mean_female.yaml",
        "bodies/mean_male.yaml",
        "bodies/f_smpl_average_A40.yaml",
        "bodies/m_smpl_average_A40.yaml",
    ] {
        let body = Body::load(assets().join(body_file)).unwrap_or_else(|e| {
            panic!("loading {body_file}: {e}");
        });
        let spec = MetaGarment::new("b", &body, &design).assembly();
        assert!(!spec.panels.is_empty(), "{body_file} produced no panels");
    }
}
