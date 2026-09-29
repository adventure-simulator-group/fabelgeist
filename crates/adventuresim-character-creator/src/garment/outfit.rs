//! Draping a whole outfit, innermost garment first, each over the ones beneath.
use super::*;

/// An outfit's drape result, with checkpoints for every garment attempted.
pub struct OutfitOutcome {
    /// Draped garments in input order, up to the first that could not be draped.
    pub garments: Vec<DrapedGarment>,
    /// Fit problems in the draped garments, each naming its garment.
    pub warnings: Vec<String>,
    /// Why draping stopped before the last garment, if it did.
    pub error: Option<anyhow::Error>,
    pub checkpoints: Vec<DrapeCheckpoints>,
}

impl OutfitOutcome {
    /// Every problem on one line, or `None` when all garments draped cleanly.
    pub fn problems(&self) -> Option<String> {
        let mut problems = self.warnings.clone();
        if let Some(error) = &self.error {
            problems.insert(0, format!("draping stopped: {error:#}"));
        }
        (!problems.is_empty()).then(|| problems.join("; "))
    }
}

/// Drape garments inside to outside. `previous` holds the last outfit drape's
/// checkpoints, so each garment re-runs only from its first changed stage.
pub fn drape_outfit(
    inputs: Vec<DrapeInput>,
    previous: Vec<DrapeCheckpoints>,
    cancel: &AtomicBool,
    mut preview: impl FnMut(Vec<DrapedGarment>),
) -> OutfitOutcome {
    let mut finished = Vec::new();
    let mut warnings = Vec::new();
    let mut checkpoints = Vec::new();
    for (index, mut input) in inputs.into_iter().enumerate() {
        input.obstacles = finished.clone();
        let outcome = match input.settled.clone() {
            Some(saved) => {
                let outcome = saved.wear(&input);
                if let Ok(garment) = &outcome.result {
                    let mut snapshot = finished.clone();
                    snapshot.push(garment.clone());
                    preview(snapshot);
                }
                outcome
            }
            None => drape(input, previous.get(index), cancel, |current| {
                let mut snapshot = finished.clone();
                snapshot.push(current);
                preview(snapshot);
            }),
        };
        checkpoints.push(outcome.checkpoints);
        match outcome.result {
            Ok(garment) => {
                warnings.extend(
                    outcome
                        .warnings
                        .into_iter()
                        .map(|warning| format!("{}: {warning}", garment.name)),
                );
                finished.push(garment);
            }
            Err(error) => {
                return OutfitOutcome {
                    garments: finished,
                    warnings,
                    error: Some(error),
                    checkpoints,
                };
            }
        }
    }
    OutfitOutcome {
        garments: finished,
        warnings,
        error: None,
        checkpoints,
    }
}
