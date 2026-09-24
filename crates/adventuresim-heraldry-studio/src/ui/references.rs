//! Reference recipes and source credits in the authoring interface.
use super::*;
use adventuresim_heraldry::presets;
pub(super) fn recipes(ui: &mut egui::Ui, d: &mut Document) {
    ui.label("Reference studies");
    for (id, label) in [
        ("german-lion", "German lion · c. 1530"),
        ("durer-lion", "Dürer · rampant lion"),
        ("woensam-lions", "Woensam · paired lions"),
        ("wernigerode-eagle", "Wernigerode style · single eagle"),
        ("wernigerode-double-eagle", "Wernigerode · double eagle"),
        ("quartered", "Quartering and inescutcheon"),
        ("counterchanged", "Counterchanged lion"),
    ] {
        if ui.button(label).clicked() {
            *d = presets::preset(id).unwrap();
        }
    }
}
pub(super) fn sources(ui: &mut egui::Ui) {
    ui.collapsing("Sources", |ui| {
        ui.label("Recipes study charge forms; print references do not establish their colors.");
        for (label, url) in [
            ("Dürer, c. 1502 · The Met", "https://www.metmuseum.org/art/collection/search/391113"),
            ("Woensam, 1530 · British Museum", "https://www.britishmuseum.org/collection/object/P_1900-1019-79"),
            ("German lion · Tom-L / Rinaldum · CC BY-SA 3.0", "https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg"),
            ("German armorial, c. 1530 · BSB Cod.icon. 391", "https://www.digitale-sammlungen.de/en/view/bsb00007681"),
            ("Single eagle · Tom-L / Heralder · CC BY-SA 3.0", "https://commons.wikimedia.org/wiki/File:Arms_of_the_King_of_the_Romans_(c.1433-1486).svg"),
            ("Double eagle · Tom-L / Heralder · CC BY-SA 3.0", "https://commons.wikimedia.org/wiki/File:Arms_of_the_Holy_Roman_Emperor_(c.1433-c.1450).svg"),
            ("Wernigerode armorial, c. 1475–1500 · BSB Cod.icon. 308 n", "https://www.digitale-sammlungen.de/en/view/bsb00043104"),
            ("Behaim shields · conservation report", "https://resources.metmuseum.org/resources/metpublications/pdf/Appendix_Notes_on_the_Restoration_of_the_Behaim_Shields_The_Metropolitan_Museum_Journal_v_30_1995.pdf"),
            ("G. W. Eve · Heraldry as Art (1907)", "https://www.gutenberg.org/files/69298/69298-h/69298-h.htm"),
            ("Cennini · pigment preparations", "https://noteaccess.com/Texts/Cennini/2.htm"),
            ("Cennini · paint and binder recipes", "https://noteaccess.com/Texts/Cennini/6TM.htm"),
            ("Wilton Diptych · gilding techniques", "https://www.nationalgallery.org.uk/paintings/catalogues/national-gallery-2024/the-wilton-diptych"),
            ("Burnishing and gold appearance · Wu et al.", "https://www.nature.com/articles/s40494-020-00463-3"),
        ] { ui.hyperlink_to(label, url); }
        ui.label("The lion adapts Tom-L's SVG after Rinaldum, based on a German armorial of c. 1530. Its painted modeling is separate from the light; the modern redraw is not a color facsimile. Exports carry credit and CC BY-SA 3.0.");
        ui.label("The double eagle adapts a modern tracing of the Wernigerode armorial, c. 1475–1500. Its halos replace the manuscript's small crowns. The single eagle is a modern variation in that style. Their painted highlights are adjustable independently of the light; exports carry both artists' credit and CC BY-SA 3.0.");
        ui.label("Gilding methods follow conservation evidence. Roughness, relief and glaze depth are adjustable interpretations, not measurements of a museum object.");
    });
}
