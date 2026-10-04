//! Admission limits for strategic disease interval work.
use super::DiseaseIntervalError;

pub fn insert_unique_bounded<K: Ord, V>(
    values: &mut std::collections::BTreeMap<K, V>,
    key: K,
    value: V,
    limit: usize,
) -> Result<bool, DiseaseIntervalError> {
    use std::collections::btree_map::Entry;
    if values.len() >= limit && !values.contains_key(&key) {
        return Err(DiseaseIntervalError::PresenceSpanBound);
    }
    Ok(match values.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(value);
            true
        }
        Entry::Occupied(_) => false,
    })
}

pub fn add_bounded_work(work: &mut u64, amount: u64, max: u64) -> Result<(), DiseaseIntervalError> {
    *work = work.saturating_add(amount);
    (*work <= max)
        .then_some(())
        .ok_or(DiseaseIntervalError::ExposureWorkBound)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_presence_span_and_checkpoint_work_caps_fail_closed() {
        let mut spans = std::collections::BTreeMap::new();
        assert_eq!(insert_unique_bounded(&mut spans, 1, "a", 2), Ok(true));
        assert_eq!(
            insert_unique_bounded(&mut spans, 1, "duplicate", 2),
            Ok(false)
        );
        assert_eq!(insert_unique_bounded(&mut spans, 2, "b", 2), Ok(true));
        assert_eq!(
            insert_unique_bounded(&mut spans, 3, "excess", 2),
            Err(DiseaseIntervalError::PresenceSpanBound)
        );

        let mut work = 0;
        assert_eq!(add_bounded_work(&mut work, 4, 5), Ok(()));
        assert_eq!(
            add_bounded_work(&mut work, 2, 5),
            Err(DiseaseIntervalError::ExposureWorkBound)
        );
    }
}
