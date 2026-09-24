//! Parametric armor fitted, thickened and skinned on the device.
//!
//! The wearer's body and every morph sample are uploaded once, each piece is
//! recorded and submitted against every one of them, and everything is read
//! back together.

use super::*;
use adventuresim_character_creator::{
    armor_frames::{FitRegion, Wearer},
    armor_gpu,
    device_body::DeviceBody,
    device_frames::DeviceFrame,
    device_frames::DeviceWearer,
    device_piece::DeviceRecording,
};
use fabelgeist_armor::gpu::Staging;
use fabelgeist_armor::gpu::body::{BodySurface, Correspondence, GpuBody, Skin};
use fabelgeist_armor::{ArmorGpu, BuiltPart, PartFrame};
use fabelgeist_compute::KernelBatch;

/// A piece fitted to the wearer and to each morph sample, with the wearer's
/// skin carried onto it.
pub(super) struct DevicePiece {
    pub base: BuiltPart,
    pub skin: Skin,
    pub endpoints: Vec<BuiltPart>,
}

/// Each body vertex's anatomical surface coordinate.
pub(super) fn vertex_texcoords(model: &BodyModel) -> Vec<[f32; 2]> {
    let character = &model.mhr.character;
    let mut uv = vec![[0.0; 2]; character.skin_weights.index.len()];
    for (face, uv_face) in character
        .mesh
        .faces
        .iter()
        .zip(&character.mesh.texcoord_faces)
    {
        for corner in 0..3 {
            uv[face[corner] as usize] = character.mesh.texcoords[uv_face[corner] as usize];
        }
    }
    uv
}

/// One shape of the wearer: the base body or a morph sample.
struct Realization<'a> {
    positions: &'a [[f32; 3]],
    normals: &'a [[f32; 3]],
    joints: &'a [[f32; 8]],
    device: &'a DeviceBody,
}

impl<'a> Realization<'a> {
    fn of(generated: &'a GeneratedCharacter) -> Self {
        Self {
            positions: &generated.positions,
            normals: &generated.normals,
            joints: &generated.global_joint_states,
            device: &generated.device,
        }
    }

    /// The body on the device, uploaded once per shape.
    fn upload(
        &self,
        gpu: &ArmorGpu,
        model: &'a BodyModel,
        texcoords: &[[f32; 2]],
    ) -> Result<&'a GpuBody> {
        let character = &model.mhr.character;
        self.device.get_or_upload(|| {
            Ok(GpuBody::new(
                gpu,
                BodySurface {
                    positions: self.positions,
                    normals: self.normals,
                    faces: &character.mesh.faces,
                    texcoords,
                    joint_indices: &character.skin_weights.index,
                    joint_weights: &character.skin_weights.weight,
                    joints: self.joints,
                },
            )?)
        })
    }

    fn wearer(&self, model: &'a BodyModel) -> Wearer<'a> {
        let character = &model.mhr.character;
        Wearer {
            faces: &character.mesh.faces,
            positions: self.positions,
            normals: self.normals,
            joints: self.joints,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
        }
    }
}

/// The part frame of `region` fitted to the wearer on the device.
pub(super) fn frame(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    region: FitRegion,
) -> Result<PartFrame> {
    let gpu = armor_gpu()?;
    let realization = Realization::of(generated);
    let body = realization.upload(gpu, model, &vertex_texcoords(model))?;
    let host = realization.wearer(model);
    let wearer = DeviceWearer {
        gpu,
        body,
        host: &host,
    };
    wearer.read_frame(region)
}

/// Fit a piece to the wearer and its morph samples on the device.
///
/// `record` records the piece against one realization of the body; it runs
/// once for the wearer and once for each morph sample.
pub(super) fn fitted(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    morphs: &[ForearmMorphSample],
    record: impl Fn(&DeviceWearer, &mut KernelBatch) -> Result<DeviceRecording>,
) -> Result<DevicePiece> {
    let gpu = armor_gpu()?;
    let texcoords = vertex_texcoords(model);
    let realizations = std::iter::once(Realization::of(generated))
        .chain(morphs.iter().map(|sample| Realization {
            positions: &sample.positions,
            normals: &sample.normals,
            joints: &sample.global_joint_states,
            device: &sample.device,
        }))
        .collect::<Vec<_>>();
    let bodies = realizations
        .iter()
        .map(|r| r.upload(gpu, model, &texcoords))
        .collect::<Result<Vec<_>>>()?;
    let hosts = realizations
        .iter()
        .map(|r| r.wearer(model))
        .collect::<Vec<_>>();
    // One submission per realization: a kernel's uniform ring must not wrap
    // within a batch, and a piece and its morphs together can exceed it.
    let mut recordings = Vec::with_capacity(bodies.len());
    let mut skin = None;
    for (body, host) in bodies.iter().copied().zip(&hosts) {
        let mut batch = gpu.batch("fitted armor");
        let device = DeviceWearer { gpu, body, host };
        let mut recording = record(&device, &mut batch)?;
        recording.part.record_shells(gpu, &mut batch)?;
        if skin.is_none() {
            skin = Some(Correspondence::record(
                gpu,
                &mut batch,
                body,
                recording.part.positions(),
                recording.part.vertex_count(),
            )?);
        }
        batch.submit();
        recordings.push(recording);
    }
    read(
        gpu,
        &recordings,
        &skin.expect("the wearer is always fitted"),
        morphs,
    )
}

fn read(
    gpu: &ArmorGpu,
    recordings: &[DeviceRecording],
    skin: &Correspondence,
    morphs: &[ForearmMorphSample],
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
    let results = gpu.read_staged(staging)?;
    let mut parts = Vec::with_capacity(recordings.len());
    for (index, (recording, (frames, part))) in recordings.iter().zip(slots).enumerate() {
        for ((_, region), frame) in recording.frames.iter().zip(frames) {
            DeviceFrame::check_status(results.status(frame), *region)?;
        }
        let context = || match index {
            0 => "fitting armor".to_string(),
            i => format!("fitting armor morph {}", morphs[i - 1].name),
        };
        for check in &recording.checks {
            check(gpu).with_context(context)?;
        }
        parts.push(
            recording
                .part
                .finish(&results, part)
                .with_context(context)?,
        );
    }
    let mut parts = parts.into_iter();
    let base = parts.next().expect("the wearer is always fitted");
    Ok(DevicePiece {
        base,
        skin: skin.finish(&results, skin_slots),
        endpoints: parts.collect(),
    })
}
