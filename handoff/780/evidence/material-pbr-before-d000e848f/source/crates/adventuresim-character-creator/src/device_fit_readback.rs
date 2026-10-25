//! Read and validate all staged realizations before assembling one fitted piece.
use crate::device_fit::DevicePiece;
use crate::device_frames::DeviceFrame;
use crate::device_piece::DeviceRecording;
use anyhow::{Context, Result};
use fabelgeist_armor::ArmorGpu;
use fabelgeist_armor::gpu::{Staging, body::Correspondence};

pub(super) async fn read(
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
            DeviceFrame::check_status(results.status(&frame)?, *region)?;
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
    let base = parts.next().context("wearer has no fitted part")?;
    Ok(DevicePiece {
        skin_aliases: recordings[0].part.outer_aliases(),
        base,
        skin: skin.finish(&results, skin_slots)?,
        endpoints: parts.collect(),
    })
}
