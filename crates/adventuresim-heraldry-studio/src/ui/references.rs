//! Reference recipes and source credits in the authoring interface.
use super::*;
use adventuresim_heraldry::presets;
pub(super) fn recipes(ui: &mut egui::Ui, d: &mut Document) {
    ui.label("Reference studies");
    for (id, label) in [
        ("imperial-eagle", "Dürer · double eagle"),
        ("burgkmair-eagle", "Burgkmair · single eagle"),
        ("german-lion", "German lion · c. 1530"),
        ("durer-lion", "Dürer · rampant lion"),
        ("woensam-lions", "Woensam · paired lions"),
        ("quartered", "Quartering and inescutcheon"),
        ("counterchanged", "Counterchanged eagle"),
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
            ("Dürer, 1521 · The Met", "https://www.metmuseum.org/art/collection/search/387586"),
            ("Dürer, c. 1502 · The Met", "https://www.metmuseum.org/art/collection/search/391113"),
            ("Burgkmair, c. 1505 · NGA", "https://www.nga.gov/artworks/128425-coat-arms-single-eagle"),
            ("Woensam, 1530 · British Museum", "https://www.britishmuseum.org/collection/object/P_1900-1019-79"),
            ("German lion · Tom-L / Rinaldum · CC BY-SA 3.0", "https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg"),
            ("German armorial, c. 1530 · BSB Cod.icon. 391", "https://www.digitale-sammlungen.de/en/view/bsb00007681"),
            ("Clean eagle reference · Wikimedia Commons", "https://commons.wikimedia.org/wiki/File:Aigle_bicéphale_éployée.svg"),
            ("Behaim shields · conservation report", "https://resources.metmuseum.org/resources/metpublications/pdf/Appendix_Notes_on_the_Restoration_of_the_Behaim_Shields_The_Metropolitan_Museum_Journal_v_30_1995.pdf"),
            ("G. W. Eve · Heraldry as Art (1907)", "https://www.gutenberg.org/files/69298/69298-h/69298-h.htm"),
            ("Cennini · pigment preparations", "https://noteaccess.com/Texts/Cennini/2.htm"),
            ("Cennini · paint and binder recipes", "https://noteaccess.com/Texts/Cennini/6TM.htm"),
            ("Wilton Diptych · gilding techniques", "https://www.nationalgallery.org.uk/paintings/catalogues/national-gallery-2024/the-wilton-diptych"),
            ("Burnishing and gold appearance · Wu et al.", "https://www.nature.com/articles/s40494-020-00463-3"),
        ] { ui.hyperlink_to(label, url); }
        ui.label("The lion adapts Tom-L's SVG after Rinaldum, based on a German armorial of c. 1530. Its painted modeling is separate from the light; the modern redraw is not a color facsimile. Exports carry credit and CC BY-SA 3.0.");
        ui.label("Gilding methods follow conservation evidence. Roughness, relief and glaze depth are adjustable interpretations, not measurements of a museum object.");
    });
}
