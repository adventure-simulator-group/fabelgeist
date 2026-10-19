//! Native file IO and orbit framing around the shared checked camera boundary.

use crate::stage::{self, Orbit};
use adventuresim_environment::render::view::{ForestView, ForestViewDocument};
use anyhow::{Result, ensure};
use bevy::prelude::*;
use serde_json::Value;
use std::path::Path;

const MAX_VIEW_BYTES: u64 = 4096;

#[derive(Clone)]
pub struct CaptureView {
    pub orbit: Orbit,
    view: ForestView,
}

impl CaptureView {
    pub fn read(path: &Path) -> Result<Self> {
        ensure!(
            std::fs::metadata(path)?.len() <= MAX_VIEW_BYTES,
            "capture view is too large"
        );
        let document: ForestViewDocument = serde_json::from_slice(&std::fs::read(path)?)?;
        let view = document.admit(stage::distance_fog())?;
        let offset = view.transform().translation - view.target();
        let radius = offset.length();
        Ok(Self {
            orbit: Orbit {
                focus: view.target(),
                radius,
                yaw: offset.x.atan2(offset.z),
                pitch: (offset.y / radius).clamp(-1.0, 1.0).asin(),
            },
            view,
        })
    }
    pub fn framed(orbit: Orbit) -> Self {
        Self {
            orbit,
            view: ForestView::from_renderer(
                orbit.focus,
                orbit.transform(),
                PerspectiveProjection::default(),
                stage::distance_fog(),
            ),
        }
    }
    pub fn lens(&self) -> impl Bundle {
        self.view.lens()
    }
    pub fn transform(&self) -> Transform {
        self.view.transform()
    }
    pub fn evidence(&self) -> Value {
        self.view.evidence()
    }
}
