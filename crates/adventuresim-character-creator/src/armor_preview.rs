use super::*;
use bevy::{
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_armor::{Armor, ArmorPart, Construction};

pub fn editor(ui: &mut egui::Ui, selection: &mut Option<Armor>) -> bool {
    let before = selection.clone();
    let mut valid = true;
    ui.collapsing("Fabelgeist armor", |ui| {
        let mut enabled = selection.is_some();
        if ui.checkbox(&mut enabled, "Breastplate").changed() {
            *selection = enabled.then(Armor::default);
        }
        let Some(a) = selection else {
            return;
        };
        ui.horizontal(|ui| {
            for (label, mode) in [
                ("Solid", Construction::Solid),
                ("Lamellar", Construction::Lamellar),
                ("Scale", Construction::Scale),
            ] {
                if ui.selectable_label(a.construction == mode, label).clicked() {
                    a.construction = mode;
                    if mode == Construction::Scale {
                        a.plate.roundness = 1.0;
                        a.plate.stagger = 0.5;
                        a.plate.hole_pairs = 1;
                    }
                    if mode == Construction::Lamellar {
                        a.plate.roundness = 0.25;
                        a.plate.stagger = 0.0;
                        a.plate.hole_pairs = 3;
                    }
                }
            }
        });
        slider(ui, "Width (m)", &mut a.width, 0.2..=0.8);
        slider(ui, "Height (m)", &mut a.height, 0.2..=0.8);
        slider(ui, "Depth (m)", &mut a.depth, 0.03..=0.35);
        slider(ui, "Waist ratio", &mut a.waist, 0.5..=1.2);
        slider(ui, "Neck opening (m)", &mut a.neck, 0.0..=0.12);
        slider(ui, "Arm opening (m)", &mut a.arm_cut, 0.0..=0.09);
        slider(ui, "Thickness (m)", &mut a.thickness, 0.001..=0.012);
        a.plate.bevel = a.plate.bevel.min(a.thickness * 0.45);
        slider(ui, "Center ridge (m)", &mut a.ridge, 0.0..=0.12);
        slider(ui, "Ridge sharpness", &mut a.ridge_sharpness, 1.0..=6.0);
        slider(ui, "Bottom point (m)", &mut a.center_point, 0.0..=0.08);
        ui.collapsing("Placement", |ui| {
            for (i, label) in ["X (m)", "Y (m)", "Z (m)"].iter().enumerate() {
                slider(ui, label, &mut a.translation[i], -3.0..=3.0);
            }
        });
        if a.construction != Construction::Solid || a.fauld.construction != Construction::Solid {
            ui.collapsing("Small plates", |ui| {
                let p = &mut a.plate;
                slider(ui, "Plate gap (m)", &mut p.gap, 0.0..=0.005);
                slider(ui, "Edge bevel (m)", &mut p.bevel, 0.0..=a.thickness * 0.45);
                slider(ui, "Piece width (m)", &mut p.width, 0.025..=0.15);
                slider(ui, "Piece height (m)", &mut p.height, 0.03..=0.2);
                slider(ui, "Rounded bottom", &mut p.roundness, 0.0..=1.0);
                slider(ui, "Row overlap", &mut p.overlap, 0.0..=0.5);
                slider(ui, "Row stagger", &mut p.stagger, 0.0..=1.0);
                slider(ui, "Hole radius (m)", &mut p.hole_radius, 0.0..=0.0049);
                ui.add(egui::Slider::new(&mut p.hole_pairs, 0..=3).text("Hole pairs"));
            });
        }
        ui.collapsing("Fauld generator", |ui| {
            ui.horizontal(|ui| {
                ui.label("Construction");
                for (label, mode) in [
                    ("Solid", Construction::Solid),
                    ("Lamellar", Construction::Lamellar),
                    ("Scale", Construction::Scale),
                ] {
                    if ui
                        .selectable_label(a.fauld.construction == mode, label)
                        .clicked()
                    {
                        a.fauld.construction = mode;
                        if mode == Construction::Scale {
                            a.plate.roundness = 1.0;
                            a.plate.stagger = 0.5;
                            a.plate.hole_pairs = 1;
                        }
                        if mode == Construction::Lamellar {
                            a.plate.roundness = 0.25;
                            a.plate.stagger = 0.0;
                            a.plate.hole_pairs = 3;
                        }
                    }
                }
            });
            ui.add(egui::Slider::new(&mut a.fauld.layer_count, 0..=12).text("Layer count"));
            slider(
                ui,
                "Layer height (m)",
                &mut a.fauld.layer_height,
                0.025..=0.15,
            );
            slider(ui, "Overlap", &mut a.fauld.overlap, 0.0..=0.5);
            slider(ui, "Flare (m)", &mut a.fauld.flare, 0.0..=0.05);
            let total_height =
                a.fauld.layer_count as f32 * a.fauld.layer_height * (1.0 - a.fauld.overlap);
            ui.label(format!("Generated fauld length: {total_height:.3} m"));
        });
        ui.collapsing("Metal and scratches", |ui| {
            let m = &mut a.metal;
            ui.color_edit_button_rgb(&mut m.color);
            slider(ui, "Roughness", &mut m.roughness, 0.08..=0.9);
            ui.add(egui::Slider::new(&mut m.scratch_density, 0..=3000).text("Scratch count"));
            slider(ui, "Scratch length", &mut m.scratch_length, 0.005..=0.4);
            slider(ui, "Scratch width", &mut m.scratch_width, 0.5..=3.0);
            slider(ui, "Scratch depth", &mut m.scratch_depth, 0.0..=1.0);
            slider(
                ui,
                "Scratch angle",
                &mut m.scratch_angle,
                -std::f32::consts::PI..=std::f32::consts::PI,
            );
            slider(
                ui,
                "Angle spread",
                &mut m.scratch_spread,
                0.0..=std::f32::consts::PI,
            );
            ui.add(egui::DragValue::new(&mut m.seed).prefix("Seed "));
        });
        if let Err(e) = a.validate() {
            ui.colored_label(egui::Color32::LIGHT_RED, e);
            valid = false;
        }
    });
    valid && *selection != before
}
fn slider(ui: &mut egui::Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>) {
    ui.add(egui::Slider::new(value, range).text(label));
}

pub struct RiggedArmorPart {
    pub part: ArmorPart,
    pub indices: Vec<[u32; 8]>,
    pub weights: Vec<[f32; 8]>,
}
pub fn rigged_parts(
    a: &Armor,
    model: &BodyModel,
    states: &[[f32; 8]],
) -> Result<Vec<RiggedArmorPart>, String> {
    let parts = fabelgeist_armor::build(a)?;
    let candidates = model
        .mhr
        .character
        .skeleton
        .names
        .iter()
        .enumerate()
        .filter(|(_, n)| n.contains("spine") || n.contains("pelvis"))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err("MHR skeleton has no torso joints".into());
    }
    Ok(parts
        .into_iter()
        .map(|part| {
            let y = part.mesh.positions.iter().map(|p| p[1]).sum::<f32>()
                / part.mesh.positions.len() as f32;
            let joint = *candidates
                .iter()
                .min_by(|&&i, &&j| {
                    (states[i][1] - y)
                        .abs()
                        .total_cmp(&(states[j][1] - y).abs())
                })
                .unwrap();
            let n = part.mesh.positions.len();
            RiggedArmorPart {
                part,
                indices: vec![[joint as u32; 8]; n],
                weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; n],
            }
        })
        .collect())
}
pub fn spawn(
    a: &Armor,
    model: &BodyModel,
    states: &[[f32; 8]],
    walk: &WalkPreview,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Result<(), String> {
    let parts = rigged_parts(a, model, states)?;
    let textures = a.metal.textures(512)?;
    let mut texture = |data| {
        let mut image = Image::new(
            Extent3d {
                width: textures.size,
                height: textures.size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        });
        images.add(image)
    };
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(a.metal.color[0], a.metal.color[1], a.metal.color[2]),
        metallic: 1.0,
        perceptual_roughness: 1.0,
        normal_map_texture: Some(texture(textures.normal)),
        metallic_roughness_texture: Some(texture(textures.metal_roughness)),
        ..default()
    });
    for p in parts {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p.part.mesh.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.part.mesh.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.part.mesh.uvs)
        .with_inserted_indices(Indices::U32(
            p.part.mesh.faces.into_iter().flatten().collect(),
        ));
        mesh.generate_tangents()
            .map_err(|e| format!("Armor tangents: {e}"))?;
        animation_preview::skin_mesh(&mut mesh, &p.indices, &p.weights);
        commands.spawn((
            CharacterMesh,
            Name::new(p.part.name),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            animation_preview::skin(walk).ok_or("Missing preview rig")?,
        ));
    }
    Ok(())
}
