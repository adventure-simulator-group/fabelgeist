use super::*;
pub(super) fn buttons(ui: &mut egui::Ui, s: &mut Studio, d: &mut Document) {
    if ui.button("JSON / files").clicked() {
        s.json = d.to_json().unwrap();
        s.show_json = true;
    }
    #[cfg(target_family = "wasm")]
    if ui.button("Import JSON").clicked() {
        crate::exchange::choose_import();
    }
    if ui.button("Save JSON").clicked() {
        s.status = match crate::exchange::save_bytes(
            "arms.json",
            d.to_json().unwrap().as_bytes(),
            "application/json",
        ) {
            Ok(()) => "Saved arms.json".into(),
            Err(e) => e,
        };
    }
}
pub(super) fn json_window(ctx: &egui::Context, s: &mut Studio, d: &mut Document) {
    let mut open = s.show_json;
    egui::Window::new("Document · JSON and local files")
        .open(&mut open)
        .default_size([680.0, 600.0])
        .show(ctx, |ui| {
            #[cfg(not(target_family = "wasm"))]
            ui.horizontal(|ui| {
                ui.label("File");
                ui.text_edit_singleline(&mut s.exchange_path);
                if ui.button("Read").clicked() {
                    match std::fs::read_to_string(&s.exchange_path) {
                        Ok(source) => s.json = source,
                        Err(e) => s.status = e.to_string(),
                    }
                }
                if ui.button("Write").clicked() {
                    s.status = match write(&s.exchange_path, &s.json) {
                        Ok(()) => format!("Saved {}", s.exchange_path),
                        Err(e) => e,
                    };
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Apply validated JSON").clicked() {
                    match Document::from_json(&s.json) {
                        Ok(document) => {
                            *d = document;
                            s.status = "Document loaded".into();
                        }
                        Err(e) => s.status = e.to_string(),
                    }
                }
                if ui.button("Reset text to current").clicked() {
                    s.json = d.to_json().unwrap();
                }
            });
            ui.label(&s.status);
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut s.json)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(28),
                );
            });
        });
    s.show_json = open;
}
#[cfg(not(target_family = "wasm"))]
fn write(path: &str, source: &str) -> Result<(), String> {
    Document::from_json(source).map_err(|e| e.to_string())?;
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, source).map_err(|e| e.to_string())
}
