//! Exact branch-and-bound for the shared rail's existing lexicographic score.

type Quality = (f32, f32);
const EPSILON: f32 = 1e-5;

pub(crate) fn improves(candidate: Quality, incumbent: Quality) -> bool {
    candidate.0 > incumbent.0 + EPSILON
        || ((candidate.0 - incumbent.0).abs() <= EPSILON && candidate.1 < incumbent.1 - EPSILON)
}

fn cannot_improve(partial: Quality, incumbent: Quality) -> bool {
    // More endpoints can only lower the minimum angle and raise the maximum
    // aspect. Use the comparator's actual f32 expressions, including its tie
    // band, rather than replacing it with an approximately equivalent score.
    (partial.0 < incumbent.0 && (partial.0 - incumbent.0).abs() > EPSILON)
        || (partial.0 <= incumbent.0 + EPSILON && partial.1 >= incumbent.1 - EPSILON)
}

pub(crate) fn worst_first(qualities: impl IntoIterator<Item = Quality>) -> Vec<usize> {
    let mut indexed = qualities.into_iter().enumerate().collect::<Vec<_>>();
    indexed.sort_by(|(left_index, left), (right_index, right)| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| right.1.total_cmp(&left.1))
            .then_with(|| left_index.cmp(right_index))
    });
    indexed.into_iter().map(|(index, _)| index).collect()
}

pub(crate) fn improving_family<T>(
    mut quality: Quality,
    incumbent: Quality,
    order: &[usize],
    mut evaluate: impl FnMut(usize) -> Option<(T, Quality)>,
) -> Option<(Quality, Vec<T>)> {
    if cannot_improve(quality, incumbent) {
        return None;
    }
    let mut shapes = (0..order.len()).map(|_| None).collect::<Vec<_>>();
    for &index in order {
        let (shape, endpoint) = evaluate(index)?;
        quality = (quality.0.min(endpoint.0), quality.1.max(endpoint.1));
        if cannot_improve(quality, incumbent) {
            return None;
        }
        assert!(shapes[index].replace(shape).is_none(), "duplicate endpoint");
    }
    improves(quality, incumbent).then(|| {
        (
            quality,
            shapes
                .into_iter()
                .map(|shape| shape.expect("missing endpoint"))
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pruning_never_rejects_a_possible_winner_at_float_tie_boundaries() {
        for angle in [0.0_f32, 2.5, 10.0, 12.5, 60.0, 180.0] {
            let boundaries = [angle - EPSILON, angle, angle + EPSILON];
            let mut angles = vec![0.0, 180.0];
            for boundary in boundaries {
                angles.extend([boundary.next_down(), boundary, boundary.next_up()]);
            }
            for aspect in [1.0_f32, 9.0, f32::INFINITY] {
                let boundary = aspect - EPSILON;
                let aspects = [
                    0.0,
                    boundary.next_down(),
                    boundary,
                    boundary.next_up(),
                    aspect,
                    f32::INFINITY,
                ];
                for &partial_angle in &angles {
                    for &partial_aspect in &aspects {
                        let partial = (partial_angle, partial_aspect);
                        if !cannot_improve(partial, (angle, aspect)) {
                            continue;
                        }
                        for &next_angle in &angles {
                            for &next_aspect in &aspects {
                                let complete = (
                                    partial_angle.min(next_angle),
                                    partial_aspect.max(next_aspect),
                                );
                                assert!(
                                    !improves(complete, (angle, aspect)),
                                    "{partial:?} -> {complete:?} beats {:?}",
                                    (angle, aspect)
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn exhaustive_and_pruned_search_accept_identical_sequences_and_indexed_shapes() {
        let mut trials = vec![
            vec![Some((10.0, 9.0)); 4],
            vec![
                Some((11.0, 8.0)),
                Some((12.0, 7.0)),
                None,
                Some((12.0, 7.0)),
            ],
            vec![Some((10.0, f32::INFINITY)); 4],
        ];
        for step in 0..300 {
            trials.push(
                (0..4)
                    .map(|endpoint| {
                        Some((
                            10.0 + ((step * 17 + endpoint * 23) % 19) as f32 * 0.000001,
                            9.0 - ((step * 11 + endpoint * 7) % 31) as f32 * 0.000001,
                        ))
                    })
                    .collect(),
            );
        }
        trials.push(vec![Some((10.0, 8.99998)); 4]);
        trials.push(vec![Some((10.2, 9.0)); 4]);
        trials.push(vec![Some((10.2, 8.0)); 4]);
        let mut exhaustive_best = (0.0, f32::INFINITY);
        let mut pruned_best = exhaustive_best;
        let mut exhaustive_winners = Vec::new();
        let mut pruned_winners = Vec::new();
        let mut order = vec![3, 1, 0, 2];
        for (trial_index, trial) in trials.iter().enumerate() {
            let complete = trial
                .iter()
                .try_fold((180.0_f32, 0.0_f32), |score, endpoint| {
                    endpoint.map(|q| (score.0.min(q.0), score.1.max(q.1)))
                });
            if let Some(score) = complete.filter(|score| improves(*score, exhaustive_best)) {
                exhaustive_best = score;
                exhaustive_winners.push(trial_index);
            }
            if let Some((score, shapes)) =
                improving_family((180.0, 0.0), pruned_best, &order, |index| {
                    trial[index].map(|quality| ((trial_index, index), quality))
                })
            {
                assert_eq!(
                    shapes,
                    (0..4).map(|index| (trial_index, index)).collect::<Vec<_>>()
                );
                pruned_best = score;
                pruned_winners.push(trial_index);
                order = worst_first(trial.iter().map(|q| q.unwrap()));
            }
            assert_eq!(exhaustive_winners, pruned_winners);
            assert_eq!(exhaustive_best, pruned_best);
        }
        assert!(exhaustive_winners.len() > 1);
    }

    #[test]
    fn rejecting_owner_short_circuits_but_potential_winner_checks_invalid_later_body() {
        let mut visited = Vec::new();
        assert!(
            improving_family((15.0, 4.0), (10.0, 9.0), &[2, 0, 1], |index| {
                visited.push(index);
                Some((index, (9.0, 8.0)))
            })
            .is_none()
        );
        assert_eq!(visited, [2]);
        visited.clear();
        assert!(
            improving_family((15.0, 4.0), (10.0, 9.0), &[2, 0, 1], |index| {
                visited.push(index);
                (index != 1).then_some((index, (12.0, 8.0)))
            })
            .is_none()
        );
        assert_eq!(visited, [2, 0, 1]);
        assert_eq!(
            worst_first([(11.0, 8.0), (10.0, 7.0), (10.0, 9.0)]),
            [2, 1, 0]
        );
    }
}
