//! Restricted resident presentation adapts the generated transport vocabulary.
use adventuresim_core::personality::Presentation;
use adventuresim_stdb_client as sats;

pub(crate) const fn npc_presentation_id(value: sats::Presentation) -> &'static str {
    match value {
        sats::Presentation::Man => Presentation::Man,
        sats::Presentation::Ambiguous => Presentation::Ambiguous,
        sats::Presentation::Woman => Presentation::Woman,
    }
    .stable_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_presentations_use_canonical_identifiers() {
        for (generated, domain) in [
            (sats::Presentation::Man, Presentation::Man),
            (sats::Presentation::Ambiguous, Presentation::Ambiguous),
            (sats::Presentation::Woman, Presentation::Woman),
        ] {
            assert_eq!(npc_presentation_id(generated), domain.stable_id());
        }
    }
}
