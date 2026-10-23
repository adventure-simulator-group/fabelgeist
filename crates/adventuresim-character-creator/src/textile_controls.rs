//! Reusable fabric controls for skirts, dresses, sleeves and hose.
use super::skirt::proportion;
use bevy_egui::egui;
use fabelgeist_armor::{
    MAX_TEXTILE_LAYERS, Permille, TextileAppearance, TextileAxis, TextileColor, TextileFill,
    TextileFinish, TextileLayer, TextileMotif, TextileRegion, TextileRepeatCount, TextileSpan,
    TextileThreadCount,
};

enum LayerAction {
    Remove(usize),
    Earlier(usize),
    Later(usize),
}

pub(super) fn show(ui: &mut egui::Ui, fabric: &mut TextileAppearance) -> bool {
    let mut changed = ui.color_edit_button_srgb(&mut fabric.color.0).changed();
    ui.horizontal(|ui| {
        changed |= ui
            .selectable_value(
                &mut fabric.chart_boundary,
                fabelgeist_armor::TextileChartBoundary::CyclicAcross,
                "Wrap around garment",
            )
            .changed();
        changed |= ui
            .selectable_value(
                &mut fabric.chart_boundary,
                fabelgeist_armor::TextileChartBoundary::Open,
                "Open cloth edges",
            )
            .changed();
    });
    changed |= proportion(
        ui,
        &mut fabric.opacity,
        TextileAppearance::OPACITY_RANGE,
        "Cloth opacity",
    );
    ui.horizontal(|ui| {
        for (finish, label) in [
            (TextileFinish::Linen, "Linen"),
            (TextileFinish::Wool, "Wool"),
            (TextileFinish::Silk, "Silk"),
            (TextileFinish::Velvet, "Velvet"),
            (TextileFinish::MetallicThread, "Metal thread"),
        ] {
            changed |= ui
                .selectable_value(&mut fabric.finish, finish, label)
                .changed();
        }
    });
    let mut action = None;
    let count = fabric.layers.0.len();
    for (index, layer) in fabric.layers.0.iter_mut().enumerate() {
        ui.push_id(index, |ui| {
            ui.collapsing(format!("Fabric layer {}", index + 1), |ui| {
                changed |= layer_controls(ui, layer);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(index > 0, egui::Button::new("Earlier"))
                        .clicked()
                    {
                        action = Some(LayerAction::Earlier(index));
                    }
                    if ui
                        .add_enabled(index + 1 < count, egui::Button::new("Later"))
                        .clicked()
                    {
                        action = Some(LayerAction::Later(index));
                    }
                    if ui.button("Remove layer").clicked() {
                        action = Some(LayerAction::Remove(index));
                    }
                });
            });
        });
    }
    if let Some(action) = action {
        match action {
            LayerAction::Remove(index) => {
                fabric.layers.0.remove(index);
            }
            LayerAction::Earlier(index) => fabric.layers.0.swap(index, index - 1),
            LayerAction::Later(index) => fabric.layers.0.swap(index, index + 1),
        }
        changed = true;
    }
    if ui
        .add_enabled(
            count < MAX_TEXTILE_LAYERS,
            egui::Button::new("Add fabric layer"),
        )
        .clicked()
    {
        fabric.layers.0.push(TextileLayer {
            region: TextileRegion::Band {
                axis: TextileAxis::Along,
                start: Permille(750),
                end: Permille(850),
            },
            fill: TextileFill::Solid,
            color: TextileColor([177, 140, 60]),
        });
        changed = true;
    }
    changed |= super::needlework::show(ui, fabric);
    changed
}

fn layer_controls(ui: &mut egui::Ui, layer: &mut TextileLayer) -> bool {
    let mut changed = ui.color_edit_button_srgb(&mut layer.color.0).changed();
    changed |= region_controls(ui, &mut layer.region);
    changed |= fill_selection(ui, &mut layer.fill);
    match &mut layer.fill {
        TextileFill::Solid => {}
        TextileFill::Weave { warp, weft, width } => {
            changed |= ui
                .add(
                    egui::Slider::new(&mut warp.0, TextileThreadCount::RANGE)
                        .text("Warp yarns across"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut weft.0, TextileThreadCount::RANGE)
                        .text("Weft yarns along"),
                )
                .changed();
            changed |= proportion(ui, width, 1..=900, "Yarn width within spacing");
        }
        TextileFill::Stripes {
            axis,
            repeats,
            width,
            phase,
        } => {
            changed |= axis_controls(ui, axis);
            changed |= ui
                .add(egui::Slider::new(&mut repeats.0, 1..=64).text("Stripe repeats"))
                .changed();
            changed |= proportion(ui, width, 1..=1000, "Stripe width");
            changed |= proportion(ui, phase, 0..=1000, "Stripe phase");
        }
        TextileFill::Motifs {
            space,
            motif,
            across,
            along,
            scale,
            rotation,
            stagger,
            offset,
        } => {
            ui.horizontal(|ui| {
                changed |= ui
                    .selectable_value(
                        space,
                        fabelgeist_armor::TextilePatternSpace::Region,
                        "Pattern fills region",
                    )
                    .changed();
                changed |= ui
                    .selectable_value(
                        space,
                        fabelgeist_armor::TextilePatternSpace::Garment,
                        "Continuous garment pattern",
                    )
                    .changed();
            });
            changed |= motif_controls(ui, motif);
            changed |= ui
                .add(egui::Slider::new(&mut across.0, 1..=64).text("Repeats across"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut along.0, 1..=64).text("Repeats along"))
                .changed();
            changed |= proportion(ui, scale, 100..=1000, "Motif scale");
            changed |= proportion(ui, rotation, 0..=1000, "Motif rotation");
            changed |= proportion(ui, stagger, 0..=1000, "Alternate row offset");
            changed |= proportion(ui, &mut offset.across, 0..=1000, "Pattern offset across");
            changed |= proportion(ui, &mut offset.along, 0..=1000, "Pattern offset along");
        }
    }
    changed
}

fn axis_controls(ui: &mut egui::Ui, axis: &mut TextileAxis) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui
            .selectable_value(axis, TextileAxis::Along, "Position along garment")
            .changed();
        changed |= ui
            .selectable_value(axis, TextileAxis::Across, "Position across garment")
            .changed();
    });
    changed
}

fn motif_controls(ui: &mut egui::Ui, motif: &mut TextileMotif) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        for (label, candidate) in [
            ("Lozenge", TextileMotif::Lozenge),
            (
                "Ringed medallion",
                TextileMotif::Medallion {
                    petals: TextileRepeatCount(4),
                    rings: TextileRepeatCount(3),
                    threads: TextileRepeatCount(32),
                    stroke: Permille(25),
                },
            ),
            (
                "Rosette",
                TextileMotif::Rosette {
                    petals: TextileRepeatCount(6),
                },
            ),
            (
                "Foliage",
                TextileMotif::Palmette {
                    leaf_pairs: TextileRepeatCount(3),
                    spread: Permille(900),
                },
            ),
            (
                "Nested damask",
                TextileMotif::Damask {
                    leaf_pairs: TextileRepeatCount(4),
                    spread: Permille(900),
                },
            ),
            (
                "Scrolling vines",
                TextileMotif::Scrollwork {
                    branches: TextileRepeatCount(2),
                    curl: Permille(900),
                },
            ),
        ] {
            let selected = std::mem::discriminant(motif) == std::mem::discriminant(&candidate);
            if ui.selectable_label(selected, label).clicked() && !selected {
                *motif = candidate;
                changed = true;
            }
        }
    });
    match motif {
        TextileMotif::Rosette { petals } => {
            changed |= ui
                .add(egui::Slider::new(&mut petals.0, 3..=12).text("Petals"))
                .changed();
        }
        TextileMotif::Medallion {
            petals,
            rings,
            threads,
            stroke,
        } => {
            changed |= ui
                .add(egui::Slider::new(&mut petals.0, 3..=12).text("Petals"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut rings.0, 1..=4).text("Thread rings"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut threads.0, 8..=64).text("Thread density"))
                .changed();
            changed |= proportion(ui, stroke, 5..=80, "Thread stroke");
        }
        TextileMotif::Palmette { leaf_pairs, spread }
        | TextileMotif::Damask { leaf_pairs, spread } => {
            changed |= ui
                .add(egui::Slider::new(&mut leaf_pairs.0, 1..=5).text("Leaf pairs"))
                .changed();
            changed |= proportion(ui, spread, 250..=1000, "Leaf spread");
        }
        TextileMotif::Scrollwork { branches, curl } => {
            changed |= ui
                .add(egui::Slider::new(&mut branches.0, 1..=5).text("Vine branches"))
                .changed();
            changed |= proportion(ui, curl, 250..=1000, "Branch curl");
        }
        TextileMotif::Lozenge => {}
    }
    changed
}

fn fill_selection(ui: &mut egui::Ui, fill: &mut TextileFill) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        for (label, candidate) in [
            ("Solid", TextileFill::Solid),
            (
                "Plain weave",
                TextileFill::Weave {
                    warp: TextileThreadCount(256),
                    weft: TextileThreadCount(192),
                    width: Permille(140),
                },
            ),
            (
                "Stripes",
                TextileFill::Stripes {
                    axis: TextileAxis::Across,
                    repeats: TextileRepeatCount(8),
                    width: Permille(150),
                    phase: Permille(0),
                },
            ),
            (
                "Repeating motifs",
                TextileFill::Motifs {
                    space: fabelgeist_armor::TextilePatternSpace::Region,
                    motif: TextileMotif::Rosette {
                        petals: TextileRepeatCount(6),
                    },
                    across: TextileRepeatCount(8),
                    along: TextileRepeatCount(4),
                    scale: Permille(800),
                    rotation: Permille(0),
                    stagger: Permille(500),
                    offset: Default::default(),
                },
            ),
        ] {
            let selected = std::mem::discriminant(fill) == std::mem::discriminant(&candidate);
            if ui.selectable_label(selected, label).clicked() && !selected {
                *fill = candidate;
                changed = true;
            }
        }
    });
    changed
}

pub(super) fn region_controls(ui: &mut egui::Ui, region: &mut TextileRegion) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        for (label, candidate) in [
            ("Whole fabric", TextileRegion::Whole),
            (
                "Band",
                TextileRegion::Band {
                    axis: TextileAxis::Along,
                    start: Permille(750),
                    end: Permille(900),
                },
            ),
            (
                "Panel",
                TextileRegion::Panel {
                    across: TextileSpan {
                        start: Permille(470),
                        end: Permille(530),
                    },
                    along: TextileSpan {
                        start: Permille(0),
                        end: Permille(1000),
                    },
                },
            ),
        ] {
            let selected = std::mem::discriminant(region) == std::mem::discriminant(&candidate);
            if ui.selectable_label(selected, label).clicked() && !selected {
                *region = candidate;
                changed = true;
            }
        }
    });
    match region {
        TextileRegion::Whole => {}
        TextileRegion::Band { axis, start, end } => {
            changed |= axis_controls(ui, axis);
            changed |= span_controls(ui, start, end);
        }
        TextileRegion::Panel { across, along } => {
            ui.label("Across panel");
            changed |= span_controls(ui, &mut across.start, &mut across.end);
            ui.label("Along panel");
            changed |= span_controls(ui, &mut along.start, &mut along.end);
        }
    }
    changed
}

fn span_controls(ui: &mut egui::Ui, start: &mut Permille, end: &mut Permille) -> bool {
    let mut changed = proportion(ui, start, 0..=end.0 - 1, "Start");
    changed |= proportion(ui, end, start.0 + 1..=1000, "End");
    changed
}
