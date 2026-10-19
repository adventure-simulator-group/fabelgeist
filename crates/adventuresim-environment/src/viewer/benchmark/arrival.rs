//! Capture the first GPU-ready forest frame before settling or steady sampling.
//! Keep its pixels in memory; PNG encoding runs only after timing is complete.

use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use serde_json::{Value, json};

use super::{Measurement, MeasurementError};

pub(super) enum ArrivalFrame {
    Pending,
    Requested,
    Captured {
        elapsed: Duration,
        image: Box<Image>,
    },
}

impl Measurement {
    /// Called only after fresh GPU-complete expected forest commands.
    pub fn capture_arrival(&mut self, commands: &mut Commands, target: Handle<Image>) {
        if !matches!(self.arrival_frame, ArrivalFrame::Pending) {
            return;
        }
        self.arrival_frame = ArrivalFrame::Requested;
        commands.spawn(Screenshot::image(target)).observe(
            |captured: On<ScreenshotCaptured>,
             mut measurement: ResMut<Measurement>,
             mut exit: MessageWriter<AppExit>| {
                if let Err(error) = measurement.receive_arrival(&captured.image) {
                    error!("{error}");
                    exit.write(AppExit::error());
                }
            },
        );
    }

    pub fn arrival_ready(&self) -> bool {
        matches!(self.arrival_frame, ArrivalFrame::Captured { .. })
    }

    fn receive_arrival(&mut self, image: &Image) -> Result<(), MeasurementError> {
        if !matches!(self.arrival_frame, ArrivalFrame::Requested) {
            return Err(MeasurementError::UnexpectedArrival);
        }
        if image.data.as_ref().is_none_or(Vec::is_empty) {
            return Err(MeasurementError::MissingPixels);
        }
        self.arrival_frame = ArrivalFrame::Captured {
            elapsed: self.request.started.elapsed(),
            image: Box::new(image.clone()),
        };
        Ok(())
    }

    pub(super) fn arrival_evidence(&self) -> Result<Value, MeasurementError> {
        let ArrivalFrame::Captured { elapsed, image } = &self.arrival_frame else {
            return Err(MeasurementError::MissingArrival);
        };
        let path: PathBuf = self.request.report.with_extension("arrival.png");
        let decoded = image
            .as_ref()
            .clone()
            .try_into_dynamic()
            .map_err(|error| MeasurementError::ArrivalImage(Box::new(error)))?;
        decoded
            .to_rgb8()
            .save(&path)
            .map_err(|error| MeasurementError::ArrivalImage(Box::new(error)))?;
        Ok(json!({
            "elapsed_from_request_ms": elapsed.as_secs_f64() * 1000.0,
            "image": path,
            "resolution": [image.width(), image.height()],
            "boundary": "first screenshot requested after fresh GPU-complete forest commands; includes readback, excludes PNG encoding; pixel coverage requires independent review",
        }))
    }
}
