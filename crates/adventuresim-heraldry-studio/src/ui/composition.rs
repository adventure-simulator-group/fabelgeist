use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldForm {
    Solid,
    Divided,
    Patterned,
    Quarterly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Symbol {
    Lion,
    Eagle,
    Roundel,
    Lozenge,
    Star,
}
const EDITOR_MAX_DEPTH: usize = 5;
pub(super) fn arms(ui: &mut egui::Ui, a: &mut ArmsDesign, depth: usize) {
    field(ui, &mut a.field, depth);
    ui.collapsing("Ordinaries", |ui| {
        let mut remove = None;
        for (i, o) in a.ordinaries.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                combo(
                    ui,
                    "Ordinary",
                    &mut o.kind,
                    &[
                        OrdinaryKind::Pale,
                        OrdinaryKind::Fess,
                        OrdinaryKind::Bend,
                        OrdinaryKind::BendSinister,
                        OrdinaryKind::Chevron,
                        OrdinaryKind::Cross,
                        OrdinaryKind::Saltire,
                        OrdinaryKind::Chief,
                        OrdinaryKind::Bordure,
                    ],
                );
                tincture(ui, "Tincture", &mut o.tincture);
                slider(ui, "Width", &mut o.width.0, 0.03..=0.5);
                boundary(ui, &mut o.boundary);
                if ui.button("Remove ordinary").clicked() {
                    remove = Some(i);
                }
                ui.separator();
            });
        }
        if let Some(i) = remove {
            a.ordinaries.remove(i);
        }
        if ui.button("Add ordinary").clicked() {
            a.ordinaries.push(Ordinary {
                kind: OrdinaryKind::Fess,
                width: Ratio(0.2),
                boundary: Boundary::Straight,
                tincture: Tincture::Argent,
            });
        }
    });
    let mut remove = None;
    let mut duplicate = None;
    for (i, c) in a.charges.iter_mut().enumerate() {
        egui::CollapsingHeader::new(format!("Charge {}", i + 1))
            .id_salt(("charge", i))
            .default_open(i == 0)
            .show(ui, |ui| {
                charge(ui, c);
                ui.horizontal(|ui| {
                    if ui.button("Duplicate").clicked() {
                        duplicate = Some(c.clone());
                    }
                    if ui.button("Remove charge").clicked() {
                        remove = Some(i);
                    }
                });
            });
    }
    if let Some(i) = remove {
        a.charges.remove(i);
    }
    if let Some(mut c) = duplicate {
        c.center[0] = (c.center[0] + 0.1).min(1.0);
        a.charges.push(c);
    }
    if ui.button("Add charge").clicked() {
        a.charges
            .push(Charge::new(ChargeKind::Roundel, Tincture::Or));
    }
    if let Some(inset) = &mut a.inescutcheon {
        ui.collapsing("Inescutcheon", |ui| arms(ui, inset, depth + 1));
        if ui.button("Remove inescutcheon").clicked() {
            a.inescutcheon = None;
        }
    } else if depth < EDITOR_MAX_DEPTH && ui.button("Add inescutcheon").clicked() {
        a.inescutcheon = Some(Box::new(ArmsDesign::plain(Tincture::Gules)));
    }
}
fn field(ui: &mut egui::Ui, f: &mut Field, depth: usize) {
    let mut form = match f {
        Field::Solid { .. } => FieldForm::Solid,
        Field::Divided { .. } => FieldForm::Divided,
        Field::Patterned { .. } => FieldForm::Patterned,
        Field::Quarterly { .. } => FieldForm::Quarterly,
    };
    let before = form;
    let choices = if depth < EDITOR_MAX_DEPTH {
        &[
            FieldForm::Solid,
            FieldForm::Divided,
            FieldForm::Patterned,
            FieldForm::Quarterly,
        ][..]
    } else {
        &[FieldForm::Solid, FieldForm::Divided, FieldForm::Patterned][..]
    };
    combo(ui, "Field", &mut form, choices);
    if before != form {
        *f = match form {
            FieldForm::Solid => Field::Solid {
                tincture: Tincture::Azure,
            },
            FieldForm::Divided => Field::Divided {
                division: Division::Pale,
                boundary: Boundary::Straight,
                tinctures: [Tincture::Azure, Tincture::Argent],
            },
            FieldForm::Patterned => Field::Patterned {
                pattern: Pattern::Checks,
                repeats: 6,
                tinctures: [Tincture::Azure, Tincture::Argent],
            },
            FieldForm::Quarterly => Field::Quarterly {
                quarters: Box::new(std::array::from_fn(|i| {
                    ArmsDesign::plain(if i % 3 == 0 {
                        Tincture::Or
                    } else {
                        Tincture::Gules
                    })
                })),
            },
        };
    }
    match f {
        Field::Solid { tincture: t } => tincture(ui, "Field tincture", t),
        Field::Divided {
            division,
            boundary: b,
            tinctures,
        } => {
            combo(
                ui,
                "Division",
                division,
                &[
                    Division::Pale,
                    Division::Fess,
                    Division::Bend,
                    Division::BendSinister,
                    Division::Chevron,
                    Division::Saltire,
                ],
            );
            boundary(ui, b);
            pair(ui, tinctures);
        }
        Field::Patterned {
            pattern,
            repeats,
            tinctures,
        } => {
            combo(
                ui,
                "Pattern",
                pattern,
                &[Pattern::Stripes, Pattern::Checks, Pattern::Lozenges],
            );
            ui.add(egui::Slider::new(repeats, 2..=16).text("Repeats"));
            pair(ui, tinctures);
        }
        Field::Quarterly { quarters } => {
            for (i, q) in quarters.iter_mut().enumerate() {
                ui.collapsing(format!("Quarter {}", i + 1), |ui| arms(ui, q, depth + 1));
            }
        }
    }
}
fn pair(ui: &mut egui::Ui, t: &mut [Tincture; 2]) {
    tincture(ui, "First tincture", &mut t[0]);
    tincture(ui, "Second tincture", &mut t[1]);
}
fn boundary(ui: &mut egui::Ui, b: &mut Boundary) {
    combo(
        ui,
        "Boundary",
        b,
        &[
            Boundary::Straight,
            Boundary::Wavy,
            Boundary::Indented,
            Boundary::Embattled,
        ],
    );
}
fn charge(ui: &mut egui::Ui, c: &mut Charge) {
    let mut symbol = match c.shape {
        ChargeKind::Lion { .. } => Symbol::Lion,
        ChargeKind::Eagle { .. } => Symbol::Eagle,
        ChargeKind::Roundel => Symbol::Roundel,
        ChargeKind::Lozenge => Symbol::Lozenge,
        ChargeKind::Star { .. } => Symbol::Star,
    };
    let before = symbol;
    combo(
        ui,
        "Charge",
        &mut symbol,
        &[
            Symbol::Lion,
            Symbol::Eagle,
            Symbol::Roundel,
            Symbol::Lozenge,
            Symbol::Star,
        ],
    );
    if symbol != before {
        c.shape = match symbol {
            Symbol::Lion => ChargeKind::Lion {
                tails: LionTails::One,
                facing: Facing::Dexter,
            },
            Symbol::Eagle => ChargeKind::Eagle {
                heads: EagleHeads::One,
                facing: Facing::Dexter,
            },
            Symbol::Roundel => ChargeKind::Roundel,
            Symbol::Lozenge => ChargeKind::Lozenge,
            Symbol::Star => ChargeKind::Star { points: 6 },
        };
    }
    match &mut c.shape {
        ChargeKind::Lion { tails, facing } => {
            combo(ui, "Tails", tails, &[LionTails::One, LionTails::Two]);
            combo(ui, "Facing", facing, &[Facing::Dexter, Facing::Sinister]);
            ui.weak("Dexter faces the observer's left.");
            tincture(ui, "Claws / teeth", &mut c.armed);
            tincture(ui, "Tongue", &mut c.langued);
        }
        ChargeKind::Star { points } => {
            ui.add(egui::Slider::new(points, 3..=16).text("Points"));
        }
        ChargeKind::Eagle { heads, facing } => {
            combo(ui, "Heads", heads, &[EagleHeads::One, EagleHeads::Two]);
            combo(ui, "Facing", facing, &[Facing::Dexter, Facing::Sinister]);
            tincture(ui, "Beak / legs / claws", &mut c.armed);
            tincture(ui, "Tongue", &mut c.langued);
            if *heads == EagleHeads::Two {
                ui.weak("The source's halos follow the beak tincture.");
            }
        }
        _ => (),
    }
    let mut counter = matches!(c.color, Coloring::Counterchanged { .. });
    if ui.checkbox(&mut counter, "Counterchanged").changed() {
        c.color = if counter {
            Coloring::Counterchanged {
                tinctures: [Tincture::Azure, Tincture::Argent],
            }
        } else {
            Coloring::Solid {
                tincture: Tincture::Or,
            }
        };
    }
    match &mut c.color {
        Coloring::Solid { tincture: t } => tincture(ui, "Charge tincture", t),
        Coloring::Counterchanged { tinctures } => pair(ui, tinctures),
    }
    slider(ui, "Horizontal position", &mut c.center[0], 0.0..=1.0);
    slider(ui, "Vertical position", &mut c.center[1], 0.0..=1.0);
    slider(ui, "Width", &mut c.size[0].0, 0.03..=1.5);
    slider(ui, "Height", &mut c.size[1].0, 0.03..=1.5);
    slider(ui, "Rotation", &mut c.rotation.0, -180.0..=180.0);
}
