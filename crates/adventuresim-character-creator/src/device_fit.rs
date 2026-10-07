//! Catalog recipes fitted, thickened and skinned on the device.
//!
//! The wearer's body and every morph realization are uploaded once, each
//! piece is recorded and submitted against every one of them, and everything
//! is read back together. The studio and the runtime client share this path.

use anyhow::{Context, Result};
use fabelgeist_armor::gpu::Staging;
use fabelgeist_armor::gpu::body::{BodySurface, Correspondence, GpuBody, Skin};
use fabelgeist_armor::{
    ArmorComponent, ArmorComponentRole, ArmorGpu, ArmorMorph, BuiltPart, GeneratedArmor,
    HelmetDesign, LimbArmorDesign, PartFrame, TASSET_SUSPENSION_GAP_M,
};
use fabelgeist_compute::KernelBatch;
use fabelgeist_compute::KernelBatchLabel;

use crate::armor_frames::{FitRegion, Wearer};
use crate::armor_layer::ArmorLayerSurface;
use crate::armor_recipes::{self, ParametricDesign};
use crate::device_body::DeviceBody;
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_piece::DeviceRecording;

/// What every realization of one body shares: its topology, surface
/// coordinates and skin.
#[derive(Clone, Copy)]
pub struct FitBody<'a> {
    pub faces: &'a [[u32; 3]],
    /// Each vertex's anatomical surface coordinate.
    pub texcoords: &'a [[f32; 2]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
}

/// One shape of the body: the wearer or a morph realization, with its device
/// upload.
#[derive(Clone, Copy)]
pub struct Realization<'a> {
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub joints: &'a [[f32; 8]],
    pub device: &'a DeviceBody,
}

impl<'a> Realization<'a> {
    /// The body on the device, uploaded once per shape.
    fn upload(&self, gpu: &ArmorGpu, body: &FitBody<'_>) -> Result<&'a GpuBody> {
        self.device.get_or_upload(|| {
            Ok(GpuBody::new(
                gpu,
                BodySurface {
                    positions: self.positions,
                    normals: self.normals,
                    faces: body.faces,
                    texcoords: body.texcoords,
                    joint_indices: body.joint_indices,
                    joint_weights: body.joint_weights,
                    joints: self.joints,
                },
            )?)
        })
    }

    pub fn wearer(&self, body: &FitBody<'a>) -> Wearer<'a> {
        Wearer {
            faces: body.faces,
            positions: self.positions,
            normals: self.normals,
            joints: self.joints,
            joint_indices: body.joint_indices,
            joint_weights: body.joint_weights,
            joint_names: body.joint_names,
        }
    }
}

/// A piece fitted to the wearer and to each morph realization, with the
/// wearer's skin carried onto it.
pub struct DevicePiece {
    pub base: BuiltPart,
    pub skin: Skin,
    pub endpoints: Vec<BuiltPart>,
}

/// The part frame of `region` fitted to `wearer` on the device.
pub fn frame(
    gpu: &ArmorGpu,
    body: &FitBody<'_>,
    wearer: &Realization<'_>,
    region: FitRegion,
) -> Result<PartFrame> {
    let device = wearer.upload(gpu, body)?;
    let host = wearer.wearer(body);
    DeviceWearer {
        gpu,
        body: device,
        host: &host,
    }
    .read_frame(region)
}

/// Fit a piece to `wearer` and each of `morphs`, named by `names`.
///
/// `record` records the piece against one realization of the body; it runs
/// once for the wearer and once for each morph.
pub fn fit_piece(
    gpu: &ArmorGpu,
    body: &FitBody<'_>,
    wearer: &Realization<'_>,
    morphs: &[(&str, Realization<'_>)],
    record: impl Fn(&DeviceWearer, &mut KernelBatch, Option<&str>) -> Result<DeviceRecording>,
) -> Result<DevicePiece> {
    pollster::block_on(fit_piece_async(gpu, body, wearer, morphs, record))
}

pub async fn fit_piece_async(
    gpu: &ArmorGpu,
    body: &FitBody<'_>,
    wearer: &Realization<'_>,
    morphs: &[(&str, Realization<'_>)],
    record: impl Fn(&DeviceWearer, &mut KernelBatch, Option<&str>) -> Result<DeviceRecording>,
) -> Result<DevicePiece> {
    let realizations = std::iter::once(wearer)
        .chain(morphs.iter().map(|(_, morph)| morph))
        .collect::<Vec<_>>();
    let bodies = realizations
        .iter()
        .map(|r| r.upload(gpu, body))
        .collect::<Result<Vec<_>>>()?;
    let hosts = realizations
        .iter()
        .map(|r| r.wearer(body))
        .collect::<Vec<_>>();
    // One submission per realization: a kernel's uniform ring must not wrap
    // within a batch, and a piece and its morphs together can exceed it.
    let mut recordings = Vec::with_capacity(bodies.len());
    let mut skin = None;
    for (index, (device, host)) in bodies.iter().copied().zip(&hosts).enumerate() {
        let mut batch = gpu.batch(KernelBatchLabel::from("fitted armor"));
        let wearer = DeviceWearer {
            gpu,
            body: device,
            host,
        };
        let target = index.checked_sub(1).map(|i| morphs[i].0);
        let mut recording = record(&wearer, &mut batch, target)?;
        recording.part.record_shells(gpu, &mut batch)?;
        if skin.is_none() {
            skin = Some(Correspondence::record(
                gpu,
                &mut batch,
                device,
                recording.part.positions(),
                recording.part.vertex_count(),
            )?);
        }
        batch.submit();
        recordings.push(recording);
    }
    let names = morphs.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    read(
        gpu,
        &recordings,
        &skin.expect("the wearer is always fitted"),
        &names,
    )
    .await
}

async fn read(
    gpu: &ArmorGpu,
    recordings: &[DeviceRecording],
    skin: &Correspondence,
    names: &[&str],
) -> Result<DevicePiece> {
    let mut staging = Staging::new();
    let slots = recordings
        .iter()
        .map(|recording| {
            let frames = recording
                .frames
                .iter()
                .map(|(frame, _)| frame.stage(&mut staging))
                .collect::<Vec<_>>();
            (frames, recording.part.stage(&mut staging))
        })
        .collect::<Vec<_>>();
    let skin_slots = skin.stage(&mut staging);
    let results = gpu.read_staged_async(staging).await?;
    let mut parts = Vec::with_capacity(recordings.len());
    for (index, (recording, (frames, part))) in recordings.iter().zip(slots).enumerate() {
        for ((_, region), frame) in recording.frames.iter().zip(frames) {
            DeviceFrame::check_status(results.status(frame), *region)?;
        }
        let context = || match index {
            0 => "fitting armor".to_string(),
            i => format!("fitting armor morph {}", names[i - 1]),
        };
        for check in &recording.checks {
            if let crate::device_piece::DeviceCheck::Device(check) = check {
                check(gpu).await.with_context(context)?;
            }
        }
        let part = recording
            .part
            .finish(&results, part)
            .with_context(context)?;
        for check in &recording.checks {
            if let crate::device_piece::DeviceCheck::Mesh(check) = check {
                check(&part).with_context(context)?;
            }
        }
        parts.push(part);
    }
    let mut parts = parts.into_iter();
    let base = parts.next().expect("the wearer is always fitted");
    Ok(DevicePiece {
        base,
        skin: skin.finish(&results, skin_slots),
        endpoints: parts.collect(),
    })
}

/// Fit a recipe to `wearer` and each of `morphs` in `placement`.
/// Underlayers are cut from the body instead.
pub fn fit_recipe(
    gpu: &ArmorGpu,
    body: &FitBody<'_>,
    wearer: &Realization<'_>,
    morphs: &[(&str, Realization<'_>)],
    design: &ParametricDesign,
    placement: &str,
    layers: &[&GeneratedArmor],
) -> Result<DevicePiece> {
    pollster::block_on(fit_recipe_async(
        gpu, body, wearer, morphs, design, placement, layers,
    ))
}

pub async fn fit_recipe_async(
    gpu: &ArmorGpu,
    body: &FitBody<'_>,
    wearer: &Realization<'_>,
    morphs: &[(&str, Realization<'_>)],
    design: &ParametricDesign,
    placement: &str,
    layers: &[&GeneratedArmor],
) -> Result<DevicePiece> {
    let fit = async |design: &ParametricDesign| {
        fit_piece_async(gpu, body, wearer, morphs, |device, batch, target| {
            let surfaces = layers
                .iter()
                .map(|armor| ArmorLayerSurface::from_generated(armor, target))
                .collect::<Result<Vec<_>>>()?;
            record_recipe(device, batch, design, placement, &surfaces)
        })
        .await
    };
    let ParametricDesign::WaistAssembly(waist) = design else {
        return fit(design).await;
    };
    let fauld = fit(&ParametricDesign::Garment(waist.fauld.clone())).await?;
    let tassets = if matches!(
        waist.tassets.plate_shape,
        fabelgeist_armor::GarmentPlateShape::WrappedTassets(_)
    ) {
        fit_piece_async(gpu, body, wearer, morphs, |device, batch, target| {
            let lower = match target {
                None => &fauld.base,
                Some(name) => {
                    &fauld.endpoints[morphs
                        .iter()
                        .position(|(n, _)| *n == name)
                        .context("missing suspended fauld realization")?]
                }
            };
            let top = lower
                .positions
                .iter()
                .map(|p| p[1])
                .fold(f32::INFINITY, f32::min)
                - TASSET_SUSPENSION_GAP_M;
            let mut surfaces = layers
                .iter()
                .map(|armor| ArmorLayerSurface::from_generated(armor, target))
                .collect::<Result<Vec<_>>>()?;
            surfaces.push(ArmorLayerSurface {
                positions: &lower.positions,
                faces: lower.indices.as_chunks::<3>().0,
                joint_indices: &fauld.skin.joint_indices,
                joint_weights: &fauld.skin.joint_weights,
            });
            device.record_wrapped_tassets(batch, &waist.tassets, &surfaces, Some(top))
        })
        .await?
    } else {
        fit(&ParametricDesign::Garment(waist.tassets.clone())).await?
    };
    Ok(suspended_waist(fauld, tassets, &waist.tassets))
}

/// Hang the tassets below the fauld's hem on every realization, and join
/// both as one waist defense with a component each.
fn suspended_waist(
    fauld: DevicePiece,
    tassets: DevicePiece,
    design: &fabelgeist_armor::GarmentArmorDesign,
) -> DevicePiece {
    let join = |mut fauld: BuiltPart, mut tassets: BuiltPart| {
        let hem = fauld
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let top = tassets
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        // Wrapped carriers were fitted at the actual suspension height; moving
        // them afterward would invalidate anatomical support at shaped edges.
        let shift = if matches!(
            design.plate_shape,
            fabelgeist_armor::GarmentPlateShape::WrappedTassets(_)
        ) {
            0.0
        } else {
            (hem - TASSET_SUSPENSION_GAP_M - top).min(0.0)
        };
        for point in &mut tassets.positions {
            point[1] += shift;
        }
        let whole = |part: &BuiltPart, role| ArmorComponent {
            role,
            vertices: 0..part.positions.len(),
            indices: 0..part.indices.len(),
            hinge: None,
            mount: None,
            material: None,
        };
        fauld.components = vec![whole(&fauld, ArmorComponentRole::Fauld)];
        tassets.components = vec![whole(&tassets, ArmorComponentRole::Tassets)];
        fauld.append(tassets);
        fauld
    };
    let DevicePiece {
        base,
        mut skin,
        endpoints,
    } = fauld;
    skin.append(tassets.skin);
    DevicePiece {
        base: join(base, tassets.base),
        skin,
        endpoints: endpoints
            .into_iter()
            .zip(tassets.endpoints)
            .map(|(fauld, tassets)| join(fauld, tassets))
            .collect(),
    }
}

/// Record a recipe fitted to one realization of the wearer, short of
/// thickening.
fn record_recipe(
    wearer: &DeviceWearer,
    batch: &mut KernelBatch,
    design: &ParametricDesign,
    placement: &str,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<DeviceRecording> {
    match design {
        ParametricDesign::Helmet(HelmetDesign::CloseHelmet(d)) => {
            wearer.record_fitted_close_helmet(batch, d)
        }
        ParametricDesign::Helmet(HelmetDesign::MailCoif(d)) => wearer.record_fitted_coif(batch, d),
        ParametricDesign::Helmet(helmet) => wearer.record_helmet(batch, helmet),
        ParametricDesign::Limb(LimbArmorDesign::Pauldron(d)) => wearer.record_fitted_pauldron(
            batch,
            d,
            armor_recipes::fit_region(design, placement)?,
            layers,
        ),
        ParametricDesign::Limb(limb) => {
            let region = armor_recipes::fit_region(design, placement)?;
            if matches!(
                limb,
                LimbArmorDesign::MittenGauntlet(_)
                    | LimbArmorDesign::Sabaton(_)
                    | LimbArmorDesign::LeatherBoot(_)
            ) {
                wearer.record_fitted_extremity(batch, limb, region)
            } else {
                wearer.record_fitted_limb(batch, limb, region)
            }
        }
        ParametricDesign::Garment(garment)
            if matches!(
                garment.plate_shape,
                fabelgeist_armor::GarmentPlateShape::WrappedTassets(_)
            ) =>
        {
            wearer.record_wrapped_tassets(batch, garment, layers, None)
        }
        ParametricDesign::Garment(garment) => {
            wearer.record_fitted_garment(batch, garment, placement, layers)
        }
        ParametricDesign::Underlayer(_) | ParametricDesign::TrunkHose(_) => {
            anyhow::bail!("underlayers are cut from the body, not recorded as parts")
        }
        ParametricDesign::PuffAndSlash(puff) => wearer.record_fitted_puff(
            batch,
            puff,
            armor_recipes::fit_region(design, placement)?,
            layers,
        ),
        ParametricDesign::WaistAssembly(_) => {
            anyhow::bail!("a waist assembly is fitted as its fauld and tassets")
        }
    }
}

/// The wearer a recipe was fitted to: where its surface coordinates are,
/// and its skeleton.
pub struct Fitted<'a> {
    pub placement: &'a str,
    /// Each morph realization's name, in order.
    pub morphs: &'a [&'a str],
    pub domain: &'a str,
    pub joint_names: &'a [String],
    /// The wearer's global joint states, one per name.
    pub joints: &'a [[f32; 8]],
}

/// The armor of a recipe fitted on the device, with a morph target per
/// morph realization. Rigid plates take their anatomical owner's skin.
pub fn assemble_recipe(
    design: &ParametricDesign,
    piece: DevicePiece,
    fitted: &Fitted<'_>,
) -> Result<GeneratedArmor> {
    let DevicePiece {
        base,
        skin,
        endpoints,
    } = piece;
    let mut targets = Vec::with_capacity(endpoints.len());
    for (name, endpoint) in fitted.morphs.iter().zip(endpoints) {
        anyhow::ensure!(
            endpoint.indices == base.indices && endpoint.positions.len() == base.positions.len(),
            "armor fit changed morph topology"
        );
        anyhow::ensure!(
            endpoint
                .components
                .iter()
                .map(|part| (&part.role, &part.vertices, &part.indices))
                .eq(base
                    .components
                    .iter()
                    .map(|part| (&part.role, &part.vertices, &part.indices))),
            "armor fit changed component correspondence"
        );
        targets.push(ArmorMorph {
            name: (*name).to_owned(),
            position_deltas: deltas(&base.positions, &endpoint.positions),
            normal_deltas: deltas(&base.normals, &endpoint.normals),
            direct_positions: endpoint.positions,
        });
    }
    let bytes = serde_json::to_vec(design)?;
    let mut armor = GeneratedArmor {
        design_hash: fabelgeist_armor::parametric_design_hash(&bytes),
        surface_domain: fitted.domain.into(),
        positions: base.positions,
        normals: base.normals,
        texcoords: skin.texcoords,
        joint_indices: skin.joint_indices,
        joint_weights: skin.joint_weights,
        indices: base.indices,
        faces: base.faces,
        trim: None,
        grids: base.grids,
        morphs: targets,
        components: base.components,
    };
    if let ParametricDesign::PuffAndSlash(puff) = design {
        for component in &mut armor.components {
            component.material = match component.role {
                ArmorComponentRole::Undercloth => Some(puff.undercloth_color.material()),
                ArmorComponentRole::OuterFabric => Some(puff.outer_color.material()),
                _ => component.material,
            };
        }
    }
    crate::skin_rules::attach(
        design,
        fitted.placement,
        fitted.joint_names,
        fitted.joints,
        &mut armor,
    )?;
    Ok(armor)
}

/// Each morph endpoint's offset from the base.
pub fn deltas(base: &[[f32; 3]], sample: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(sample)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}
