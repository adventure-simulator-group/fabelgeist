fn exact_action_retry(
    attempt: &InvestigationActionAttempt,
    actor_id: u64,
    action_id: &str,
    method: &str,
    expected_version: u32,
) -> bool {
    attempt.owner_character_id == actor_id
        && attempt.capability_id == action_id
        && attempt.method == method
        && attempt.expected_version == expected_version
}
