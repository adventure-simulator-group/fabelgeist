//! Resolve reduced groove fans at their actual float32 envelope cutoffs.
use super::*;

pub(super) fn refine(
    stations: &mut Vec<f64>,
    fuller: &FullerParameters,
    minimum_envelope: f64,
    ring: &impl Fn(f64, f64) -> Vec<Point>,
    detail: Detail,
) -> Result<(), String> {
    let mut cutoffs: Vec<_> = fuller
        .grooves
        .iter()
        .flat_map(|g| {
            [GrooveEnd::Entry, GrooveEnd::Exit].map(|end| end.cutoff(g, minimum_envelope))
        })
        .collect();
    cutoffs.sort_by(f64::total_cmp);
    cutoffs.dedup();
    // Every unsuccessful pass must insert at least one previously absent cutoff.
    for _ in 0..=cutoffs.len() {
        let mut next = vec![stations[0]];
        let mut changed = false;
        for pair in stations.windows(2) {
            if let Err(error) =
                check_interval([pair[0], pair[1]], fuller, minimum_envelope, ring, detail)
            {
                let inside: Vec<_> = cutoffs
                    .iter()
                    .copied()
                    .filter(|&y| y > pair[0] && y < pair[1])
                    .collect();
                if inside.is_empty() {
                    return Err(error);
                }
                next.extend(inside);
                changed = true;
            }
            next.push(pair[1]);
        }
        if !changed {
            return Ok(());
        }
        construction_budget((next.len() * ring(0.0, minimum_envelope).len() * 2) as f64)?;
        *stations = next;
    }
    Err("groove transition exceeds its bounded cutoff refinement".into())
}

fn check_interval(
    [a, b]: [f64; 2],
    fuller: &FullerParameters,
    minimum_envelope: f64,
    ring: &impl Fn(f64, f64) -> Vec<Point>,
    detail: Detail,
) -> Result<(), String> {
    let [left, right] = [ring(a, minimum_envelope), ring(b, minimum_envelope)];
    let [raw_left, raw_right] = [ring(a, 0.0), ring(b, 0.0)];
    for side in 0..left.len() {
        let next = (side + 1) % left.len();
        let quad = [left[side], left[next], right[next], right[side]];
        if quad
            != [
                raw_left[side],
                raw_left[next],
                raw_right[next],
                raw_right[side],
            ]
            && let Some(flat) = blade_reduction::flat_indices(fuller, side)
        {
            blade_reduction::check([a, b], side, flat, &quad, |y| ring(y, 0.0), detail)?;
        }
    }
    Ok(())
}

enum GrooveEnd {
    Entry,
    Exit,
}

impl GrooveEnd {
    fn cutoff(self, groove: &FullerGroove, minimum_envelope: f64) -> f64 {
        let [mut lo, mut hi] = match self {
            Self::Entry => [
                groove.start.get(),
                groove.start.get() + groove.entry_length.get(),
            ],
            Self::Exit => [
                groove.end.get() - groove.exit_length.get(),
                groove.end.get(),
            ],
        };
        for _ in 0..f64::MANTISSA_DIGITS {
            let mid = (lo + hi) / 2.0;
            let retained = groove.envelope(mid) >= minimum_envelope;
            if retained == matches!(self, Self::Entry) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        // Keep the bracket endpoint on the representable side of the cutoff.
        match self {
            Self::Entry => hi,
            Self::Exit => lo,
        }
    }
}
