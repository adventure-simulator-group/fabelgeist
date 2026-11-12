//! A background thread draping one outfit at a time, so draping never blocks
//! the studio. A new request cancels the running drape.
use adventuresim_character_creator::garment::{
    DrapeCheckpoints, DrapeInput, DrapedGarment, OutfitOutcome, drape_outfit,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Drapes garments, each tagged with what it belongs to.
pub(super) struct DrapeWorker<T> {
    handle: Option<std::thread::JoinHandle<OutfitOutcome>>,
    pending: Option<Vec<(T, DrapeInput)>>,
    /// The tag of each garment the running drape produces.
    draping: Vec<T>,
    cancel: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<Vec<DrapedGarment>>>>,
    /// Completed stages of the most recent drape, reused by the next request.
    checkpoints: Vec<DrapeCheckpoints>,
}

impl<T> Default for DrapeWorker<T> {
    fn default() -> Self {
        Self {
            handle: None,
            pending: None,
            draping: Vec::new(),
            cancel: Arc::default(),
            latest: Arc::default(),
            checkpoints: Vec::new(),
        }
    }
}

impl<T> Drop for DrapeWorker<T> {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// What changed since the worker was last polled.
#[derive(Default)]
pub(super) struct DrapeProgress {
    /// A drape began; its garments are tagged by [`DrapeWorker::draping`].
    pub started: bool,
    /// Garments to show now, in drape order.
    pub preview: Option<Vec<DrapedGarment>>,
    /// The drape's result, once it finished without being cancelled.
    pub finished: Option<OutfitOutcome>,
}

impl<T: Send + 'static> DrapeWorker<T> {
    /// Drape garments, innermost first, cancelling any drape in progress.
    pub fn request(&mut self, input: Vec<(T, DrapeInput)>) {
        self.cancel.store(true, Ordering::Relaxed);
        self.pending = (!input.is_empty()).then_some(input);
        *self.latest.lock().unwrap() = None;
    }

    /// Make the next request simulate every stage again.
    pub fn restart_from_placement(&mut self) {
        self.checkpoints.clear();
    }

    /// Whether a drape is queued or running.
    pub fn running(&self) -> bool {
        self.handle.is_some() || self.pending.is_some()
    }

    /// The tags of the garments the latest drape produces.
    pub fn draping(&self) -> &[T] {
        &self.draping
    }

    /// Collect the running drape's progress, and start a queued one.
    pub fn poll(&mut self) -> DrapeProgress {
        let mut progress = DrapeProgress {
            preview: if self.cancel.load(Ordering::Relaxed) {
                None
            } else {
                self.latest.lock().unwrap().take()
            },
            ..DrapeProgress::default()
        };
        if self
            .handle
            .as_ref()
            .is_some_and(std::thread::JoinHandle::is_finished)
        {
            let mut outcome = self
                .handle
                .take()
                .expect("checked above")
                .join()
                .unwrap_or_else(|_| OutfitOutcome {
                    garments: Vec::new(),
                    warnings: Vec::new(),
                    error: Some(anyhow::anyhow!("Drape worker failed")),
                    checkpoints: Vec::new(),
                });
            // Keep completed stages even from a failed or cancelled drape.
            self.checkpoints = std::mem::take(&mut outcome.checkpoints);
            if !self.cancel.load(Ordering::Relaxed) {
                // Whatever draped is shown and usable, even when some of it failed.
                progress.preview = Some(outcome.garments.clone());
                progress.finished = Some(outcome);
            }
        }
        if self.handle.is_none()
            && let Some(pending) = self.pending.take()
        {
            let (tags, input): (Vec<_>, Vec<_>) = pending.into_iter().unzip();
            self.draping = tags;
            let cancel = Arc::new(AtomicBool::new(false));
            self.cancel = cancel.clone();
            let latest = Arc::new(Mutex::new(None));
            self.latest = latest.clone();
            let previous = std::mem::take(&mut self.checkpoints);
            self.handle = Some(std::thread::spawn(move || {
                drape_outfit(input, previous, &cancel, |garments| {
                    if !cancel.load(Ordering::Relaxed) {
                        *latest.lock().unwrap() = Some(garments);
                    }
                })
            }));
            progress.started = true;
        }
        progress
    }
}
