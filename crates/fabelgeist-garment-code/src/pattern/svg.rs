//! SVG rendering of a sewing pattern.
//!
//! Ports the drawing half of `pygarment.pattern.wrappers.VisPattern`. Two
//! layouts are produced by the reference and both are supported here:
//!
//! * the *pattern* view, where panels are placed and rotated according to their
//!   3D placement, flattened onto XY, and
//! * the *printable* view, where panels are laid out side by side without
//!   overlap and left unfilled.
//!
//! Rasterisation (the reference's cairosvg step to PNG/PDF) is out of scope --
//! the SVG is the output.

use std::path::Path as FsPath;

use anyhow::{Context, Result};

use crate::curve::{Curve, Path};
use crate::math::*;

use super::core::edge_as_curve;
use super::spec::PatternSpec;

const FILL: &str = "rgb(227,175,186)";
const STROKE: &str = "rgb(51,51,51)";
const STROKE_WIDTH: &str = "0.2";

/// Layout for [`PatternSpec::to_svg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Panels placed by their 3D position, flattened onto XY.
    Placed,
    /// Panels laid out side by side for printing, unfilled.
    Printable,
}

/// Text annotations to draw.
#[derive(Debug, Clone, Copy)]
pub struct Annotations {
    /// Panel names at panel centres.
    pub names: bool,
    /// Vertex and edge indices.
    pub ids: bool,
}

impl Default for Annotations {
    fn default() -> Self {
        Self {
            names: true,
            ids: true,
        }
    }
}

struct DrawnPanel {
    name: String,
    path: Path,
    front: bool,
}

/// One edge of a laid-out panel.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeDrawing {
    /// Index into the panel's `edges`, which is what a [`Stitch`] refers to.
    ///
    /// [`Stitch`]: super::spec::Stitch
    pub index: usize,
    /// The `d` attribute for this edge alone, starting with its own `M`.
    pub d: String,
    /// The edge's first vertex, in laid-out coordinates.
    pub start: V2,
    /// The point half-way along the edge, by arc length.
    pub mid: V2,
}

/// One panel, laid out and flattened onto the drawing plane.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelDrawing {
    pub name: String,
    /// The closed outline, as an SVG `d` attribute.
    pub d: String,
    /// Whether the panel faces the viewer (`translation.z >= 0`).
    pub front: bool,
    /// `[min_x, max_x, min_y, max_y]`.
    pub bbox: [f64; 4],
    /// The bounding box centre, where the panel's name is drawn.
    pub center: V2,
    /// Outline length, in the pattern's units (centimetres).
    pub perimeter: f64,
    pub edges: Vec<EdgeDrawing>,
}

/// A whole pattern laid out for drawing.
///
/// [`PatternSpec::to_svg`] is one renderer over this; a front-end that wants
/// its own -- per-panel colours, hover, a seam overlay -- builds it from here
/// rather than re-deriving the layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternDrawing {
    /// `[min_x, min_y, width, height]`, margin included.
    pub view_box: [f64; 4],
    /// Panels in draw order: back-most first, front panels before back ones.
    pub panels: Vec<PanelDrawing>,
}

impl PatternDrawing {
    fn empty() -> Self {
        Self {
            view_box: [0.0, 0.0, 1.0, 1.0],
            panels: Vec::new(),
        }
    }

    /// Find a panel by name.
    pub fn panel(&self, name: &str) -> Option<&PanelDrawing> {
        self.panels.iter().find(|p| p.name == name)
    }
}

/// Convert vertices and the panel's XY translation into image coordinates:
/// flip Y, then move the bounding box's upper-left corner to the origin.
fn verts_to_px_coords(vertices: &[V2], translation_2d: V2) -> (Vec<V2>, V2) {
    let flipped: Vec<V2> = vertices.iter().map(|v| [v[0], -v[1]]).collect();
    let translation_2d = [translation_2d[0], -translation_2d[1]];

    let offset = [
        flipped.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min),
        flipped.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min),
    ];

    (
        flipped.iter().map(|v| sub2(*v, offset)).collect(),
        add2(translation_2d, offset),
    )
}

fn draw_panel(spec: &PatternSpec, name: &str, apply_transform: bool) -> DrawnPanel {
    let panel = spec.panel(name).expect("unknown panel");
    let (vertices, translation) = verts_to_px_coords(
        &panel.vertices,
        [panel.translation[0], panel.translation[1]],
    );

    let segs: Vec<Curve> = panel
        .edges
        .iter()
        .map(|e| edge_as_curve(&vertices, e, true))
        .collect();
    let mut path = Path::new(segs);

    if apply_transform {
        // Place and rotate according to the 3D pose, flattened onto XY.
        // Only the rotation visible in the XY plane is kept, estimated from
        // where the panel's local Y axis ends up.
        //
        // NOTE: heuristic. Ox sometimes flips because of gimbal locks in this
        // Euler representation.
        let rotation = Rotation::from_euler_xyz(panel.rotation, true);
        let res = rotation.apply([0.0, 1.0, 0.0]);
        let flat_rot_angle = vector_angle([0.0, 1.0], [res[0], res[1]]).to_degrees();

        path = path.rotated(-flat_rot_angle, vertices[0]);
        // NOTE: rotation before translation -- the order matters.
        path = path.translated(translation);
    }

    DrawnPanel {
        name: name.to_string(),
        path,
        front: panel.translation[2] >= 0.0,
    }
}

impl PatternSpec {
    /// Lay the pattern out for drawing.
    ///
    /// `margin` is the padding around the whole drawing, and in the printable
    /// layout also the gap between neighbouring panels.
    pub fn drawing(&mut self, layout: Layout, margin: f64) -> PatternDrawing {
        let order = self.panel_order(false);
        if order.is_empty() {
            return PatternDrawing::empty();
        }

        // Draw back-to-front so the frontmost panels render on top.
        let mut z_sorted: Vec<(f64, String)> = order
            .iter()
            .map(|n| (self.panel(n).unwrap().translation[2], n.clone()))
            .collect();
        z_sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then_with(|| a.1.cmp(&b.1)));

        let flat = layout == Layout::Printable;
        let mut front: Vec<DrawnPanel> = Vec::new();
        let mut back: Vec<DrawnPanel> = Vec::new();
        let (mut shift_x_front, mut shift_x_back) = (margin, margin);

        for (_, name) in &z_sorted {
            let mut drawn = draw_panel(self, name, !flat);
            if flat {
                let shift = if drawn.front {
                    shift_x_front
                } else {
                    shift_x_back
                };
                drawn.path = drawn.path.translated([shift, 0.0]);
                let bbox = drawn.path.bbox();
                let diff = (bbox[1] - bbox[0]) + margin;
                if drawn.front {
                    shift_x_front += diff;
                } else {
                    shift_x_back += diff;
                }
            }
            if drawn.front {
                front.push(drawn);
            } else {
                back.push(drawn);
            }
        }

        // Separate the back panels from the front ones.
        if !front.is_empty() && !back.is_empty() {
            let front_max_x = front
                .iter()
                .map(|p| p.path.bbox()[1])
                .fold(f64::NEG_INFINITY, f64::max);
            let back_min_x = back
                .iter()
                .map(|p| p.path.bbox()[0])
                .fold(f64::INFINITY, f64::min);

            let (shift_x, shift_y) = if flat {
                let front_max_y = front
                    .iter()
                    .map(|p| p.path.bbox()[3])
                    .fold(f64::NEG_INFINITY, f64::max);
                let back_min_y = back
                    .iter()
                    .map(|p| p.path.bbox()[2])
                    .fold(f64::INFINITY, f64::min);
                (0.0, front_max_y - back_min_y + 10.0)
            } else {
                (front_max_x - back_min_x + 10.0, 0.0)
            };

            for p in back.iter_mut() {
                p.path = p.path.translated([shift_x, shift_y]);
            }
        }

        let panels: Vec<DrawnPanel> = front.into_iter().chain(back).collect();

        let boxes: Vec<[f64; 4]> = panels.iter().map(|p| p.path.bbox()).collect();
        let min_x = boxes.iter().map(|b| b[0]).fold(f64::INFINITY, f64::min);
        let max_x = boxes.iter().map(|b| b[1]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = boxes.iter().map(|b| b[2]).fold(f64::INFINITY, f64::min);
        let max_y = boxes.iter().map(|b| b[3]).fold(f64::NEG_INFINITY, f64::max);

        let viewbox = [
            min_x - margin,
            min_y - margin,
            (max_x - min_x) + 2.0 * margin,
            (max_y - min_y) + 2.0 * margin,
        ];

        PatternDrawing {
            view_box: [viewbox[0], viewbox[1], viewbox[2], viewbox[3]],
            panels: panels.iter().map(drawn_to_panel).collect(),
        }
    }

    /// Render to SVG.
    pub fn to_svg(&mut self, layout: Layout, annotations: Annotations, margin: f64) -> String {
        let drawing = self.drawing(layout, margin);
        if drawing.panels.is_empty() {
            return empty_svg();
        }
        let viewbox = drawing.view_box;

        let mut out = String::new();
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" \
             viewBox=\"{} {} {} {}\" width=\"{}cm\" height=\"{}cm\">\n",
            viewbox[0], viewbox[1], viewbox[2], viewbox[3], viewbox[2], viewbox[3]
        ));

        let fill = if layout == Layout::Printable {
            "rgb(255,255,255)"
        } else {
            FILL
        };
        for p in &drawing.panels {
            out.push_str(&format!(
                "  <path d=\"{}\" fill=\"{fill}\" stroke=\"{STROKE}\" stroke-width=\"{STROKE_WIDTH}\" />\n",
                p.d
            ));
        }

        for p in &drawing.panels {
            out.push_str(&annotate(p, annotations));
        }

        out.push_str("</svg>\n");
        out
    }

    /// Write both SVG views next to the specification.
    pub fn save_svg(
        &mut self,
        dir: impl AsRef<FsPath>,
        tag: &str,
        annotations: Annotations,
        printable: bool,
    ) -> Result<()> {
        let dir = dir.as_ref();
        let name = self.name.clone();

        let svg = self.to_svg(Layout::Placed, annotations, 2.0);
        let file = dir.join(format!("{name}{tag}_pattern.svg"));
        std::fs::write(&file, svg).with_context(|| format!("writing {}", file.display()))?;

        if printable {
            let svg = self.to_svg(Layout::Printable, annotations, 10.0);
            let file = dir.join(format!("{name}{tag}_print_pattern.svg"));
            std::fs::write(&file, svg).with_context(|| format!("writing {}", file.display()))?;
        }
        Ok(())
    }
}

fn empty_svg() -> String {
    "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\" />\n".to_string()
}

/// Measure a laid-out panel: the geometry both renderers need.
fn drawn_to_panel(panel: &DrawnPanel) -> PanelDrawing {
    let bbox = panel.path.bbox();
    let edges = panel
        .path
        .segments
        .iter()
        .enumerate()
        .map(|(index, seg)| {
            let t = seg.ilength(seg.length() / 2.0, 1e-3);
            EdgeDrawing {
                index,
                d: format!("M {},{} {}", seg.start()[0], seg.start()[1], segment_d(seg)),
                start: seg.start(),
                mid: seg.point(t),
            }
        })
        .collect();

    PanelDrawing {
        name: panel.name.clone(),
        d: path_d(&panel.path),
        front: panel.front,
        bbox,
        center: [(bbox[0] + bbox[1]) / 2.0, (bbox[2] + bbox[3]) / 2.0],
        perimeter: panel.path.length(),
        edges,
    }
}

fn annotate(panel: &PanelDrawing, annotations: Annotations) -> String {
    let mut out = String::new();

    if annotations.names {
        out.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" fill=\"rgb(31,31,31)\" font-size=\"7\" \
             text-anchor=\"middle\" dominant-baseline=\"middle\">{}</text>\n",
            panel.center[0],
            panel.center[1],
            escape_xml(&panel.name)
        ));
    }

    if annotations.ids {
        for edge in &panel.edges {
            let i = edge.index;
            out.push_str(&format!(
                "  <text x=\"{}\" y=\"{}\" fill=\"rgb(245,96,66)\" font-size=\"7\">{i}</text>\n",
                edge.start[0], edge.start[1]
            ));
        }
        for edge in &panel.edges {
            let i = edge.index;
            out.push_str(&format!(
                "  <text x=\"{}\" y=\"{}\" fill=\"rgb(44,131,68)\" font-size=\"7\" \
                 text-anchor=\"middle\">{i}</text>\n",
                edge.mid[0],
                // Sit slightly above the line.
                edge.mid[1] - 3.0
            ));
        }
    }

    out
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The `d` attribute for a path.
fn path_d(path: &Path) -> String {
    let mut out = String::new();
    let mut prev_end: Option<V2> = None;

    for seg in &path.segments {
        let start = seg.start();
        if prev_end.map(|p| dist2(p, start) > 1e-9).unwrap_or(true) {
            out.push_str(&format!("M {},{} ", start[0], start[1]));
        }
        out.push_str(&segment_d(seg));
        out.push(' ');
        prev_end = Some(seg.end());
    }

    out.push('Z');
    out.trim().to_string()
}

/// One segment's `d` command, without a leading `M`.
fn segment_d(seg: &Curve) -> String {
    match seg {
        Curve::Line { end, .. } => format!("L {},{}", end[0], end[1]),
        Curve::Quad { control, end, .. } => {
            format!("Q {},{} {},{}", control[0], control[1], end[0], end[1])
        }
        Curve::Cubic { c1, c2, end, .. } => format!(
            "C {},{} {},{} {},{}",
            c1[0], c1[1], c2[0], c2[1], end[0], end[1]
        ),
        Curve::Arc(a) => format!(
            "A {},{} {} {},{} {},{}",
            a.radius[0],
            a.radius[1],
            a.rotation,
            a.large_arc as i32,
            a.sweep as i32,
            a.end[0],
            a.end[1]
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::spec::{EdgeSpec, PanelSpec, PatternSpec};

    fn two_panel_pattern() -> PatternSpec {
        let square = |t: V3| PanelSpec {
            translation: t,
            rotation: [0.0; 3],
            vertices: vec![[0.0, 0.0], [10.0, 0.0], [10.0, 20.0], [0.0, 20.0]],
            edges: (0..4)
                .map(|i| EdgeSpec {
                    endpoints: [i, (i + 1) % 4],
                    label: None,
                    curvature: None,
                })
                .collect(),
            label: None,
        };

        let mut spec = PatternSpec::empty();
        spec.name = "test".into();
        spec.panels = vec![
            ("front".into(), square([0.0, 50.0, 20.0])),
            ("back".into(), square([0.0, 50.0, -20.0])),
        ];
        spec
    }

    #[test]
    fn renders_both_panels() {
        let mut spec = two_panel_pattern();
        let svg = spec.to_svg(Layout::Placed, Annotations::default(), 2.0);

        assert_eq!(svg.matches("<path").count(), 2, "{svg}");
        assert!(svg.contains(">front<"));
        assert!(svg.contains(">back<"));
        assert!(svg.starts_with("<svg"));
        assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn printable_layout_separates_panels() {
        let mut spec = two_panel_pattern();
        let placed = spec.to_svg(
            Layout::Placed,
            Annotations {
                names: false,
                ids: false,
            },
            2.0,
        );
        let printable = spec.to_svg(
            Layout::Printable,
            Annotations {
                names: false,
                ids: false,
            },
            10.0,
        );

        // The printable sheet spreads panels out, so it is taller/wider.
        let vb = |s: &str| -> Vec<f64> {
            let start = s.find("viewBox=\"").unwrap() + 9;
            let end = s[start..].find('"').unwrap() + start;
            s[start..end]
                .split_whitespace()
                .map(|v| v.parse().unwrap())
                .collect()
        };
        assert!(vb(&printable)[3] > vb(&placed)[3], "{}", printable);
    }

    #[test]
    fn path_data_closes_the_loop() {
        let mut spec = two_panel_pattern();
        let svg = spec.to_svg(
            Layout::Placed,
            Annotations {
                names: false,
                ids: false,
            },
            2.0,
        );
        for line in svg.lines().filter(|l| l.contains("<path")) {
            assert!(line.contains("Z\""), "path not closed: {line}");
            assert!(line.contains("M "), "path has no move-to: {line}");
        }
    }

    #[test]
    fn curved_edges_emit_curve_commands() {
        let mut spec = two_panel_pattern();
        spec.panel_mut("front").unwrap().edges[1].curvature =
            Some(crate::pattern::spec::Curvature::Cubic {
                params: vec![[0.3, 0.2], [0.7, 0.1]],
            });
        spec.panel_mut("front").unwrap().edges[2].curvature =
            Some(crate::pattern::spec::Curvature::Circle {
                radius: 8.0,
                large_arc: false,
                right: true,
            });

        let svg = spec.to_svg(
            Layout::Placed,
            Annotations {
                names: false,
                ids: false,
            },
            2.0,
        );
        assert!(svg.contains(" C "), "{svg}");
        assert!(svg.contains(" A "), "{svg}");
    }

    /// The drawing is what `to_svg` renders, so every outline it reports must
    /// appear in the SVG verbatim.
    #[test]
    fn drawing_is_what_the_svg_draws() {
        let mut spec = two_panel_pattern();
        let drawing = spec.drawing(Layout::Placed, 2.0);
        let svg = spec.to_svg(
            Layout::Placed,
            Annotations {
                names: false,
                ids: false,
            },
            2.0,
        );

        assert_eq!(drawing.panels.len(), 2);
        assert!(svg.contains(&format!(
            "viewBox=\"{} {} {} {}\"",
            drawing.view_box[0], drawing.view_box[1], drawing.view_box[2], drawing.view_box[3]
        )));

        for panel in &drawing.panels {
            assert!(svg.contains(&panel.d), "{} is not in the SVG", panel.name);
            assert_eq!(panel.edges.len(), 4);
            assert!((panel.perimeter - 60.0).abs() < 1e-9, "{}", panel.perimeter);
            for edge in &panel.edges {
                assert!(edge.d.starts_with("M "), "{}", edge.d);
            }
        }

        // The front panel is drawn first and sits left of the back one.
        assert!(drawing.panels[0].front);
        assert!(!drawing.panels[1].front);
        assert!(drawing.panels[0].bbox[1] <= drawing.panels[1].bbox[0]);
    }

    /// Edge indices must line up with the panel's own edge list, or a stitch
    /// overlay drawn from them would join the wrong seams.
    #[test]
    fn edge_midpoints_lie_on_their_edge() {
        let mut spec = two_panel_pattern();
        let drawing = spec.drawing(Layout::Printable, 5.0);
        let panel = drawing.panel("front").expect("front panel");

        assert_eq!(panel.edges.len(), spec.panel("front").unwrap().edges.len());
        for edge in &panel.edges {
            let next = &panel.edges[(edge.index + 1) % panel.edges.len()];
            // A square's edge midpoint is half-way between its endpoints.
            let expected = [
                (edge.start[0] + next.start[0]) / 2.0,
                (edge.start[1] + next.start[1]) / 2.0,
            ];
            assert!(dist2(edge.mid, expected) < 1e-6, "edge {}", edge.index);
        }
    }
}
