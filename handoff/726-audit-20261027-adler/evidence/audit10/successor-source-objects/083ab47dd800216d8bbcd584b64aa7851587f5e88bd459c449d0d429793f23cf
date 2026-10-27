//! A groom: the stack of operations that turns guides into hair.

use serde::{Deserialize, Serialize};
use web_time::Instant;

use crate::ops::{Context, Op};
use crate::strands::Strands;

/// A stack of operations and the resolution they run at -- the recipe of
/// one hairstyle, as a Blender object's modifier stack of hair node groups
/// is. Serialises to JSON, so a dataset sample records exactly how it was
/// made.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Groom {
    /// Points per strand. DiffLocks' strands have 256; tight curls need
    /// about eight a turn.
    pub points: usize,
    pub ops: Vec<Op>,
}

/// How long each operation took.
#[derive(Clone, Debug, Default)]
pub struct GrowReport {
    pub steps: Vec<(&'static str, f32, usize)>,
}

impl GrowReport {
    pub fn total_ms(&self) -> f32 {
        self.steps.iter().map(|s| s.1).sum()
    }
}

impl std::fmt::Display for GrowReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, ms, strands) in &self.steps {
            writeln!(f, "{name:>12}: {ms:8.1} ms  ({strands} strands)")?;
        }
        write!(f, "{:>12}: {:8.1} ms", "total", self.total_ms())
    }
}

impl Groom {
    /// Run the stack over `guides`: they are resampled to the groom's point
    /// count, and every operation applied in order.
    pub fn grow(&self, guides: &Strands, context: &Context) -> Strands {
        self.grow_reporting(guides, context).0
    }

    pub fn grow_reporting(&self, guides: &Strands, context: &Context) -> (Strands, GrowReport) {
        self.grow_while(guides, context, || true)
            .expect("never stopped")
    }

    /// Grow, asking `keep_going` before every operation: `None` if it
    /// said to stop -- the groom is no longer wanted, and the rest of the
    /// stack need not run.
    pub fn grow_while(
        &self,
        guides: &Strands,
        context: &Context,
        keep_going: impl Fn() -> bool,
    ) -> Option<(Strands, GrowReport)> {
        let mut strands = guides.resampled(self.points.max(2));
        let mut report = GrowReport::default();
        for op in &self.ops {
            if !keep_going() {
                return None;
            }
            let started = Instant::now();
            op.apply(&mut strands, context);
            report.steps.push((
                op.name(),
                started.elapsed().as_secs_f32() * 1e3,
                strands.len(),
            ));
        }
        Some((strands, report))
    }
}
