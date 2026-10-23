//! Shared cloth-path editor: stitches, braid, pearl rows and folded seams.
use bevy_egui::egui;
use fabelgeist_armor::{
    Micrometers, Millimeters, Permille, TextileAppearance, TextileChartSize, TextileColor,
    TextileNeedlework, TextilePathClosure, TextilePathPoint, TextileRegion, TextileRepeatCount,
    TextileStroke, TextileStrokeStyle,
};

#[path = "needlework_style_controls.rs"]
mod style_controls;

pub(super) fn show(ui: &mut egui::Ui, fabric: &mut TextileAppearance) -> bool {
    let mut enabled = fabric.needlework.is_some();
    let mut changed = ui
        .checkbox(&mut enabled, "Stitches, braid and ornament paths")
        .changed();
    if changed {
        fabric.needlework = enabled.then(|| {
            Box::new(TextileNeedlework {
                chart: fabric.relief.as_ref().map_or(
                    TextileChartSize {
                        across: Millimeters(1000),
                        along: Millimeters(1000),
                    },
                    |r| r.chart,
                ),
                strokes: vec![default_stroke()],
            })
        });
    }
    if let Some(needlework) = &mut fabric.needlework {
        let mut remove = None;
        for (i, stroke) in needlework.strokes.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                ui.collapsing(format!("Cloth path {}", i + 1), |ui| {
                    changed |= edit(ui, stroke);
                    let remove_response = ui.button("Remove path");
                    if remove_response.clicked() {
                        remove = Some(i);
                    }
                });
            });
        }
        if let Some(i) = remove {
            needlework.strokes.remove(i);
            changed = true;
        }
        let add_path = ui.add_enabled(
            needlework.strokes.len() < TextileNeedlework::MAXIMUM_STROKES,
            egui::Button::new("Add ornament path"),
        );
        if add_path.clicked() {
            needlework.strokes.push(default_stroke());
            changed = true;
        }
        if needlework.strokes.is_empty() {
            fabric.needlework = None;
        }
    }
    changed
}
fn default_stroke() -> TextileStroke {
    TextileStroke {
        region: TextileRegion::Whole,
        across: TextileRepeatCount(1),
        along: TextileRepeatCount(1),
        points: vec![
            TextilePathPoint {
                across: Permille(20),
                along: Permille(50),
            },
            TextilePathPoint {
                across: Permille(980),
                along: Permille(50),
            },
        ],
        closure: TextilePathClosure::Open,
        style: TextileStrokeStyle::Cord {
            diameter: Micrometers(500),
            color: TextileColor([190, 145, 55]),
        },
    }
}
fn edit(ui: &mut egui::Ui, stroke: &mut TextileStroke) -> bool {
    let mut changed = super::textile::region_controls(ui, &mut stroke.region);
    changed |= ui
        .add(
            egui::Slider::new(&mut stroke.across.0, TextileNeedlework::REPEAT_RANGE)
                .text("Repeats across"),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut stroke.along.0, TextileNeedlework::REPEAT_RANGE)
                .text("Repeats along"),
        )
        .changed();
    let mut closed = stroke.closure == TextilePathClosure::Closed;
    let closed_response = ui.checkbox(&mut closed, "Closed path");
    if closed_response.changed() {
        stroke.closure = if closed {
            TextilePathClosure::Closed
        } else {
            TextilePathClosure::Open
        };
        changed = true;
    }
    let mut remove = None;
    let count = stroke.points.len();
    for (i, point) in stroke.points.iter_mut().enumerate() {
        ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
                changed |= super::number(ui, &mut point.across.0, 0..=1000, "Across");
                changed |= super::number(ui, &mut point.along.0, 0..=1000, "Along");
                let remove_point = ui.add_enabled(count > 2, egui::Button::new("Remove point"));
                if remove_point.clicked() {
                    remove = Some(i);
                }
            });
        });
    }
    if let Some(i) = remove {
        stroke.points.remove(i);
        changed = true;
    }
    let add_point = ui.add_enabled(
        count < *TextileNeedlework::POINT_COUNT_RANGE.end(),
        egui::Button::new("Add point"),
    );
    if add_point.clicked() {
        stroke.points.push(TextilePathPoint {
            across: Permille(500),
            along: Permille(500),
        });
        changed = true;
    }
    changed | style_controls::show(ui, &mut stroke.style)
}
