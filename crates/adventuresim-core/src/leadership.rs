use crate::identity::CharacterId;
use std::collections::{BTreeMap, BTreeSet};

/// Returns the qualifying challenger with the greatest tally, breaking ties by
/// the lowest character id. The current leader is deliberately not required to
/// retain a threshold after an election.
pub fn elect_leader(
    current_leader: CharacterId,
    current_leader_alive: bool,
    living_members: &[CharacterId],
    votes: &[(CharacterId, CharacterId)],
) -> Option<CharacterId> {
    let living: BTreeSet<CharacterId> = living_members.iter().copied().collect();
    if living.is_empty() {
        return None;
    }
    let mut tallies = BTreeMap::<CharacterId, usize>::new();
    for &(voter, candidate) in votes {
        if living.contains(&voter) && living.contains(&candidate) {
            *tallies.entry(candidate).or_default() += 1;
        }
    }
    tallies
        .into_iter()
        .filter(|(candidate, count)| {
            *candidate != current_leader
                && if current_leader_alive {
                    *count * 100 >= living.len() * 66
                } else {
                    *count * 2 >= living.len()
                }
        })
        .max_by(|(left_id, left_count), (right_id, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_id.cmp(left_id))
        })
        .map(|(candidate, _)| candidate)
}

#[cfg(test)]
mod tests {
    use super::elect_leader;
    use crate::identity::CharacterId;

    #[test]
    fn live_threshold_is_inclusive_and_rounded_by_ratio() {
        let mut members: Vec<_> = (1..=49).map(CharacterId::from).collect();
        members.push(CharacterId::from(99));
        let below: Vec<_> = (1..=32)
            .map(|voter| (CharacterId::from(voter), CharacterId::from(99)))
            .collect();
        let exact: Vec<_> = (1..=33)
            .map(|voter| (CharacterId::from(voter), CharacterId::from(99)))
            .collect();
        assert_eq!(
            elect_leader(CharacterId::from(1), true, &members, &below),
            None
        );
        assert_eq!(
            elect_leader(CharacterId::from(1), true, &members, &exact),
            Some(CharacterId::from(99))
        );
    }

    #[test]
    fn two_of_three_meets_live_threshold_but_one_of_two_does_not() {
        assert_eq!(
            elect_leader(
                CharacterId::from(1),
                true,
                &[
                    CharacterId::from(1),
                    CharacterId::from(2),
                    CharacterId::from(3)
                ],
                &[
                    (CharacterId::from(1), CharacterId::from(2)),
                    (CharacterId::from(3), CharacterId::from(2))
                ]
            ),
            Some(CharacterId::from(2))
        );
        assert_eq!(
            elect_leader(
                CharacterId::from(1),
                true,
                &[CharacterId::from(1), CharacterId::from(2)],
                &[(CharacterId::from(1), CharacterId::from(2))]
            ),
            None
        );
    }

    #[test]
    fn dead_threshold_is_inclusive_at_half() {
        assert_eq!(
            elect_leader(
                CharacterId::from(1),
                false,
                &[
                    CharacterId::from(2),
                    CharacterId::from(3),
                    CharacterId::from(4),
                    CharacterId::from(5)
                ],
                &[
                    (CharacterId::from(2), CharacterId::from(3)),
                    (CharacterId::from(4), CharacterId::from(3))
                ]
            ),
            Some(CharacterId::from(3))
        );
    }

    #[test]
    fn sole_survivor_can_succeed_a_dead_leader_with_system_ballot() {
        assert_eq!(
            elect_leader(
                CharacterId::from(1),
                false,
                &[CharacterId::from(2)],
                &[(CharacterId::from(2), CharacterId::from(2))]
            ),
            Some(CharacterId::from(2))
        );
    }

    #[test]
    fn highest_tally_then_lowest_id_wins() {
        let votes = [
            (CharacterId::from(1), CharacterId::from(9)),
            (CharacterId::from(2), CharacterId::from(9)),
            (CharacterId::from(8), CharacterId::from(8)),
            (CharacterId::from(9), CharacterId::from(8)),
        ];
        assert_eq!(
            elect_leader(
                CharacterId::from(7),
                false,
                &[
                    CharacterId::from(1),
                    CharacterId::from(2),
                    CharacterId::from(8),
                    CharacterId::from(9)
                ],
                &votes
            ),
            Some(CharacterId::from(8))
        );
    }

    #[test]
    fn invalid_ballots_and_incumbent_do_not_trigger_a_change() {
        assert_eq!(
            elect_leader(
                CharacterId::from(1),
                true,
                &[
                    CharacterId::from(1),
                    CharacterId::from(2),
                    CharacterId::from(3)
                ],
                &[
                    (CharacterId::from(1), CharacterId::from(1)),
                    (CharacterId::from(2), CharacterId::from(1)),
                    (CharacterId::from(99), CharacterId::from(2))
                ]
            ),
            None
        );
    }
}
