//! The sewing-pattern specification -- the JSON format shared with the
//! GarmentCodeData dataset.
//!
//! Ports the data model of `pygarment.pattern.core`.

use crate::math::{V2, V3};

/// Edge curvature, in the edge's own local frame.
#[derive(Debug, Clone, PartialEq)]
pub enum Curvature {
    /// SVG-style circular arc.
    Circle {
        radius: f64,
        large_arc: bool,
        right: bool,
    },
    /// One control point.
    Quadratic { params: Vec<V2> },
    /// Two control points.
    Cubic { params: Vec<V2> },
}

impl Curvature {
    /// Flip the local Y of every control point -- used when a panel is mirrored
    /// during edge-loop normalisation.
    pub fn flip_y(&mut self) {
        match self {
            Curvature::Circle { right, .. } => *right = !*right,
            Curvature::Quadratic { params } | Curvature::Cubic { params } => {
                for p in params.iter_mut() {
                    p[1] = -p[1];
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeSpec {
    pub endpoints: [usize; 2],
    pub label: Option<String>,
    pub curvature: Option<Curvature>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanelSpec {
    pub translation: V3,
    /// Euler angles in degrees.
    pub rotation: V3,
    pub vertices: Vec<V2>,
    pub edges: Vec<EdgeSpec>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StitchSide {
    pub panel: String,
    pub edge: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stitch {
    pub sides: [StitchSide; 2],
    /// Stitch this side's right face to the other's wrong face.
    pub right_wrong: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Properties {
    pub curvature_coords: String,
    pub normalize_panel_translation: bool,
    pub normalized_edge_loops: bool,
    pub units_in_meter: i64,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            curvature_coords: "relative".into(),
            normalize_panel_translation: false,
            normalized_edge_loops: true,
            units_in_meter: 100,
        }
    }
}

/// A whole pattern: named panels plus the stitches joining them.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternSpec {
    pub name: String,
    /// Panels in insertion order.
    pub panels: Vec<(String, PanelSpec)>,
    pub stitches: Vec<Stitch>,
    pub panel_order: Option<Vec<String>>,
    pub properties: Properties,
}

impl PatternSpec {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            panels: Vec::new(),
            stitches: Vec::new(),
            panel_order: None,
            properties: Properties::default(),
        }
    }

    pub fn panel(&self, name: &str) -> Option<&PanelSpec> {
        self.panels.iter().find(|(n, _)| n == name).map(|(_, p)| p)
    }

    pub fn panel_mut(&mut self, name: &str) -> Option<&mut PanelSpec> {
        self.panels
            .iter_mut()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p)
    }

    pub fn panel_names(&self) -> Vec<String> {
        self.panels.iter().map(|(n, _)| n.clone()).collect()
    }

    /// Merge another pattern's panels and stitches into this one.
    pub fn merge(&mut self, other: PatternSpec) {
        for (name, panel) in other.panels {
            match self.panel_mut(&name) {
                Some(slot) => *slot = panel,
                None => self.panels.push((name, panel)),
            }
        }
        self.stitches.extend(other.stitches);
    }
}

// ----- JSON output -----
//
// Hand-rolled rather than via `serde_json`, so that key order matches the
// reference byte for byte (Python dicts preserve insertion order) without
// forcing a `preserve_order` feature on the rest of the workspace.

/// Format a float the way Python's `json.dump` does: always with a decimal
/// point, so `30` serialises as `30.0`.
fn f(v: f64) -> String {
    if v.is_nan() {
        return "NaN".into();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

struct Json {
    out: String,
}

impl Json {
    fn new() -> Self {
        Self { out: String::new() }
    }

    fn pad(&mut self, depth: usize) {
        self.out.push_str(&"  ".repeat(depth));
    }

    fn num_list(&mut self, values: &[String], depth: usize) {
        if values.is_empty() {
            self.out.push_str("[]");
            return;
        }
        self.out.push_str("[\n");
        for (i, v) in values.iter().enumerate() {
            self.pad(depth + 1);
            self.out.push_str(v);
            if i + 1 < values.len() {
                self.out.push(',');
            }
            self.out.push('\n');
        }
        self.pad(depth);
        self.out.push(']');
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

impl PatternSpec {
    /// Serialise to the pattern-specification JSON, with two-space indentation.
    pub fn to_json(&self) -> String {
        let mut j = Json::new();
        j.out.push_str("{\n");

        // "pattern"
        j.pad(1);
        j.out.push_str("\"pattern\": {\n");

        // panels
        j.pad(2);
        j.out.push_str("\"panels\": {");
        if self.panels.is_empty() {
            j.out.push('}');
        } else {
            j.out.push('\n');
            for (i, (name, panel)) in self.panels.iter().enumerate() {
                j.pad(3);
                j.out.push_str(&format!("\"{}\": {{\n", escape(name)));
                write_panel(&mut j, panel, 4);
                j.pad(3);
                j.out.push('}');
                if i + 1 < self.panels.len() {
                    j.out.push(',');
                }
                j.out.push('\n');
            }
            j.pad(2);
            j.out.push('}');
        }
        j.out.push_str(",\n");

        // stitches
        j.pad(2);
        j.out.push_str("\"stitches\": ");
        if self.stitches.is_empty() {
            j.out.push_str("[]");
        } else {
            j.out.push_str("[\n");
            for (i, st) in self.stitches.iter().enumerate() {
                j.pad(3);
                j.out.push_str("[\n");
                for (k, side) in st.sides.iter().enumerate() {
                    j.pad(4);
                    j.out.push_str("{\n");
                    j.pad(5);
                    j.out
                        .push_str(&format!("\"panel\": \"{}\",\n", escape(&side.panel)));
                    j.pad(5);
                    j.out.push_str(&format!("\"edge\": {}\n", side.edge));
                    j.pad(4);
                    j.out.push('}');
                    if k + 1 < st.sides.len() || st.right_wrong {
                        j.out.push(',');
                    }
                    j.out.push('\n');
                }
                if st.right_wrong {
                    j.pad(4);
                    j.out.push_str("\"right_wrong\"\n");
                }
                j.pad(3);
                j.out.push(']');
                if i + 1 < self.stitches.len() {
                    j.out.push(',');
                }
                j.out.push('\n');
            }
            j.pad(2);
            j.out.push(']');
        }

        // panel_order
        if let Some(order) = &self.panel_order {
            j.out.push_str(",\n");
            j.pad(2);
            j.out.push_str("\"panel_order\": ");
            let items: Vec<String> = order.iter().map(|n| format!("\"{}\"", escape(n))).collect();
            j.num_list(&items, 2);
        }

        j.out.push('\n');
        j.pad(1);
        j.out.push_str("},\n");

        // parameters / parameter_order (kept for format compatibility)
        j.pad(1);
        j.out.push_str("\"parameters\": {},\n");
        j.pad(1);
        j.out.push_str("\"parameter_order\": [],\n");

        // properties
        j.pad(1);
        j.out.push_str("\"properties\": {\n");
        j.pad(2);
        j.out.push_str(&format!(
            "\"curvature_coords\": \"{}\",\n",
            escape(&self.properties.curvature_coords)
        ));
        j.pad(2);
        j.out.push_str(&format!(
            "\"normalize_panel_translation\": {},\n",
            self.properties.normalize_panel_translation
        ));
        j.pad(2);
        j.out.push_str(&format!(
            "\"normalized_edge_loops\": {},\n",
            self.properties.normalized_edge_loops
        ));
        j.pad(2);
        j.out.push_str(&format!(
            "\"units_in_meter\": {}\n",
            self.properties.units_in_meter
        ));
        j.pad(1);
        j.out.push_str("}\n");

        j.out.push('}');
        j.out
    }
}

fn write_panel(j: &mut Json, panel: &PanelSpec, depth: usize) {
    j.pad(depth);
    j.out.push_str("\"translation\": ");
    j.num_list(
        &panel.translation.iter().map(|v| f(*v)).collect::<Vec<_>>(),
        depth,
    );
    j.out.push_str(",\n");

    j.pad(depth);
    j.out.push_str("\"rotation\": ");
    j.num_list(
        &panel.rotation.iter().map(|v| f(*v)).collect::<Vec<_>>(),
        depth,
    );
    j.out.push_str(",\n");

    j.pad(depth);
    j.out.push_str("\"vertices\": ");
    if panel.vertices.is_empty() {
        j.out.push_str("[]");
    } else {
        j.out.push_str("[\n");
        for (i, v) in panel.vertices.iter().enumerate() {
            j.pad(depth + 1);
            j.num_list(&[f(v[0]), f(v[1])], depth + 1);
            if i + 1 < panel.vertices.len() {
                j.out.push(',');
            }
            j.out.push('\n');
        }
        j.pad(depth);
        j.out.push(']');
    }
    j.out.push_str(",\n");

    j.pad(depth);
    j.out.push_str("\"edges\": ");
    if panel.edges.is_empty() {
        j.out.push_str("[]");
    } else {
        j.out.push_str("[\n");
        for (i, e) in panel.edges.iter().enumerate() {
            j.pad(depth + 1);
            j.out.push_str("{\n");
            write_edge(j, e, depth + 2);
            j.pad(depth + 1);
            j.out.push('}');
            if i + 1 < panel.edges.len() {
                j.out.push(',');
            }
            j.out.push('\n');
        }
        j.pad(depth);
        j.out.push(']');
    }

    if let Some(label) = &panel.label {
        j.out.push_str(",\n");
        j.pad(depth);
        j.out
            .push_str(&format!("\"label\": \"{}\"\n", escape(label)));
    } else {
        j.out.push('\n');
    }
}

fn write_edge(j: &mut Json, e: &EdgeSpec, depth: usize) {
    j.pad(depth);
    j.out.push_str("\"endpoints\": ");
    j.num_list(
        &[e.endpoints[0].to_string(), e.endpoints[1].to_string()],
        depth,
    );

    if let Some(label) = &e.label {
        j.out.push_str(",\n");
        j.pad(depth);
        j.out.push_str(&format!("\"label\": \"{}\"", escape(label)));
    }

    if let Some(c) = &e.curvature {
        j.out.push_str(",\n");
        j.pad(depth);
        j.out.push_str("\"curvature\": {\n");
        j.pad(depth + 1);
        let type_name = match c {
            Curvature::Circle { .. } => "circle",
            Curvature::Quadratic { .. } => "quadratic",
            Curvature::Cubic { .. } => "cubic",
        };
        j.out.push_str(&format!("\"type\": \"{type_name}\",\n"));
        j.pad(depth + 1);
        j.out.push_str("\"params\": ");
        match c {
            Curvature::Circle {
                radius,
                large_arc,
                right,
            } => {
                j.num_list(
                    &[
                        f(*radius),
                        (*large_arc as i32).to_string(),
                        (*right as i32).to_string(),
                    ],
                    depth + 1,
                );
            }
            Curvature::Quadratic { params } | Curvature::Cubic { params } => {
                j.out.push_str("[\n");
                for (i, p) in params.iter().enumerate() {
                    j.pad(depth + 2);
                    j.num_list(&[f(p[0]), f(p[1])], depth + 2);
                    if i + 1 < params.len() {
                        j.out.push(',');
                    }
                    j.out.push('\n');
                }
                j.pad(depth + 1);
                j.out.push(']');
            }
        }
        j.out.push('\n');
        j.pad(depth);
        j.out.push('}');
    }

    j.out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_keep_a_decimal_point() {
        assert_eq!(f(0.0), "0.0");
        assert_eq!(f(30.0), "30.0");
        assert_eq!(f(-0.5), "-0.5");
    }

    #[test]
    fn json_round_trips_through_a_parser() {
        let spec = PatternSpec {
            name: "test".into(),
            panels: vec![(
                "front".into(),
                PanelSpec {
                    translation: [0.0, 91.4, 30.0],
                    rotation: [-0.0, 0.0, 0.0],
                    vertices: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0]],
                    edges: vec![
                        EdgeSpec {
                            endpoints: [0, 1],
                            label: None,
                            curvature: None,
                        },
                        EdgeSpec {
                            endpoints: [1, 2],
                            label: Some("lower_interface".into()),
                            curvature: Some(Curvature::Cubic {
                                params: vec![[0.2, 0.35], [0.5, 0.2]],
                            }),
                        },
                        EdgeSpec {
                            endpoints: [2, 0],
                            label: None,
                            curvature: Some(Curvature::Circle {
                                radius: 4.5,
                                large_arc: false,
                                right: true,
                            }),
                        },
                    ],
                    label: Some("body".into()),
                },
            )],
            stitches: vec![Stitch {
                sides: [
                    StitchSide {
                        panel: "front".into(),
                        edge: 0,
                    },
                    StitchSide {
                        panel: "front".into(),
                        edge: 2,
                    },
                ],
                right_wrong: true,
            }],
            panel_order: Some(vec!["front".into()]),
            properties: Properties::default(),
        };

        let text = spec.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

        assert_eq!(parsed["properties"]["units_in_meter"], 100);
        let panel = &parsed["pattern"]["panels"]["front"];
        assert_eq!(panel["label"], "body");
        assert_eq!(panel["translation"][1], 91.4);
        assert_eq!(panel["edges"][1]["label"], "lower_interface");
        assert_eq!(panel["edges"][1]["curvature"]["type"], "cubic");
        assert_eq!(panel["edges"][2]["curvature"]["params"][2], 1);
        assert_eq!(parsed["pattern"]["stitches"][0][2], "right_wrong");
        assert_eq!(parsed["pattern"]["panel_order"][0], "front");
    }

    /// Key order must match the reference's, since the dataset tooling reads
    /// these files positionally in places.
    #[test]
    fn key_order_matches_the_reference() {
        let spec = PatternSpec {
            name: "t".into(),
            panels: vec![(
                "p".into(),
                PanelSpec {
                    translation: [0.0; 3],
                    rotation: [0.0; 3],
                    vertices: vec![[0.0, 0.0], [1.0, 1.0]],
                    edges: vec![EdgeSpec {
                        endpoints: [0, 1],
                        label: Some("l".into()),
                        curvature: Some(Curvature::Quadratic {
                            params: vec![[0.5, 0.5]],
                        }),
                    }],
                    label: None,
                },
            )],
            stitches: vec![],
            panel_order: None,
            properties: Properties::default(),
        };
        let text = spec.to_json();

        let endpoints = text.find("\"endpoints\"").unwrap();
        let label = text.find("\"label\"").unwrap();
        let curvature = text.find("\"curvature\"").unwrap();
        assert!(endpoints < label && label < curvature, "{text}");

        let translation = text.find("\"translation\"").unwrap();
        let rotation = text.find("\"rotation\"").unwrap();
        let vertices = text.find("\"vertices\"").unwrap();
        let edges = text.find("\"edges\"").unwrap();
        assert!(translation < rotation && rotation < vertices && vertices < edges);
    }
}
