//! Inclusive CPU wall spans at public render schedule boundaries.

use std::collections::BTreeMap;
use std::time::Duration;
use web_time::Instant;

use bevy::app::SubApp;
use bevy::prelude::*;
use bevy::render::diagnostic::resolve_encoder;
use bevy::render::renderer::{RenderGraph, RenderGraphSystems};
use bevy::render::{Render, RenderSystems};

use super::super::{ForestBenchmark, ValidationPhase};
use super::{ProfileFailure, RenderProfile, RenderStage};

#[derive(Default)]
pub(super) struct ProfileData {
    current: Option<Frame>,
    pub(super) frames: Vec<BTreeMap<RenderStage, Duration>>,
    pub(super) failures: Vec<ProfileFailure>,
    pub(super) wrapped_cull_systems: usize,
}

#[derive(Default)]
struct Frame {
    starts: BTreeMap<RenderStage, Instant>,
    elapsed: BTreeMap<RenderStage, Duration>,
}

impl ProfileData {
    pub(super) fn finish_frame(&mut self) {
        let Some(frame) = self.current.take() else {
            return;
        };
        let complete = frame.starts.is_empty()
            && [
                RenderStage::Queue,
                RenderStage::Prepare,
                RenderStage::GraphEncode,
                RenderStage::GraphSubmit,
                RenderStage::GpuWait,
            ]
            .iter()
            .all(|stage| frame.elapsed.get(stage).is_some_and(|time| !time.is_zero()));
        if complete {
            self.frames.push(frame.elapsed);
        } else {
            self.failures.push(ProfileFailure::IncompleteFrame);
        }
    }
}

impl RenderProfile {
    pub(super) fn begin(&self, stage: RenderStage) {
        if let Ok(mut data) = self.data.lock()
            && let Some(frame) = &mut data.current
            && frame.starts.insert(stage, Instant::now()).is_some()
        {
            data.failures.push(ProfileFailure::DuplicateStage);
        }
    }

    pub(super) fn end(&self, stage: RenderStage) {
        if let Ok(mut data) = self.data.lock()
            && let Some(frame) = &mut data.current
        {
            if let Some(started) = frame.starts.remove(&stage) {
                if frame.elapsed.insert(stage, started.elapsed()).is_some() {
                    data.failures.push(ProfileFailure::DuplicateStage);
                }
            } else {
                data.failures.push(ProfileFailure::UnpairedStage);
            }
        }
    }
}

pub(super) fn install(render: &mut SubApp) {
    render.add_systems(
        Render,
        (
            open_queue
                .after(RenderSystems::PrepareViews)
                .before(RenderSystems::Queue),
            end_queue
                .after(RenderSystems::Queue)
                .before(RenderSystems::PhaseSort),
            begin_prepare
                .after(RenderSystems::PhaseSort)
                .before(RenderSystems::Prepare),
            end_prepare
                .after(RenderSystems::Prepare)
                .before(RenderSystems::Render),
        ),
    );
    render.add_systems(
        RenderGraph,
        (
            begin_encode
                .after(RenderGraphSystems::Begin)
                .before(RenderGraphSystems::Render),
            end_encode
                .after(RenderGraphSystems::Render)
                .before(resolve_encoder),
            begin_submit
                .after(resolve_encoder)
                .before(RenderGraphSystems::Submit),
            end_submit
                .after(RenderGraphSystems::Submit)
                .before(RenderGraphSystems::Finish),
        ),
    );
}

fn open_queue(profile: Res<RenderProfile>, probe: Res<ForestBenchmark>) {
    let active = probe
        .validation()
        .evidence()
        .is_some_and(|state| state.phase == ValidationPhase::Sampling);
    if let Ok(mut data) = profile.data.lock() {
        if data.current.is_some() {
            data.failures.push(ProfileFailure::IncompleteFrame);
        }
        data.current = active.then(Frame::default);
    }
    profile.begin(RenderStage::Queue);
}

fn end_queue(profile: Res<RenderProfile>) {
    profile.end(RenderStage::Queue);
}

fn begin_prepare(profile: Res<RenderProfile>) {
    profile.begin(RenderStage::Prepare);
}

fn end_prepare(profile: Res<RenderProfile>) {
    profile.end(RenderStage::Prepare);
}

fn begin_encode(profile: Res<RenderProfile>) {
    profile.begin(RenderStage::GraphEncode);
}

fn end_encode(profile: Res<RenderProfile>) {
    profile.end(RenderStage::GraphEncode);
}

fn begin_submit(profile: Res<RenderProfile>) {
    profile.begin(RenderStage::GraphSubmit);
}

fn end_submit(profile: Res<RenderProfile>) {
    profile.end(RenderStage::GraphSubmit);
}
