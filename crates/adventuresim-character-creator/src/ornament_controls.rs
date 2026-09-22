//! Controls for a procedural ornament: its motif and proportions.
use super::*;
use armor_preview::slider;
use fabelgeist_armor::ornament::{Motif, Ornament};

pub(super) fn ornament(ui: &mut egui::Ui, ornament: &mut Ornament) {
    egui::ComboBox::from_label("Motif")
        .selected_text(ornament.motif.name())
        .show_ui(ui, |ui| {
            for motif in Motif::ALL {
                let same =
                    std::mem::discriminant(&motif) == std::mem::discriminant(&ornament.motif);
                let choice = ui.selectable_label(same, motif.name());
                if choice.clicked() && !same {
                    ornament.motif = motif;
                    ornament.repeats = motif.suggested_repeats();
                }
            }
        });
    ui.add(egui::Slider::new(&mut ornament.repeats, 1..=Ornament::MAX_REPEATS).text("Repeats"));
    slider(
        ui,
        "Line width",
        &mut ornament.line,
        Ornament::MIN_LINE..=Ornament::MAX_LINE,
    );
    ui.checkbox(&mut ornament.fillets, "Fillet lines");
    match &mut ornament.motif {
        Motif::Wave { amplitude } | Motif::Zigzag { amplitude } => {
            slider(ui, "Height", amplitude, 0.0..=1.0);
        }
        Motif::Guilloche { amplitude, strands } => {
            slider(ui, "Height", amplitude, 0.0..=1.0);
            ui.add(egui::Slider::new(strands, 1..=Motif::MAX_STRANDS).text("Strands"));
        }
        Motif::Rope { slant } => slider(ui, "Twist", slant, 0.1..=3.0),
        Motif::Beads { radius } => slider(ui, "Bead size", radius, 0.1..=1.0),
        Motif::Vine { amplitude, leaf } => {
            slider(ui, "Height", amplitude, 0.0..=1.0);
            slider(ui, "Leaf size", leaf, 0.0..=1.0);
        }
    }
}
