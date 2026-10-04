//! Directed subject/actor identity for the persisted soft-affinity edge.

use crate::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterAffinityKey {
    subject: CharacterId,
    actor: CharacterId,
}

impl CharacterAffinityKey {
    pub fn new(subject: CharacterId, actor: CharacterId) -> Self {
        Self { subject, actor }
    }
}

impl std::fmt::Display for CharacterAffinityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.subject, self.actor)
    }
}
impl From<CharacterAffinityKey> for String {
    fn from(key: CharacterAffinityKey) -> Self {
        key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_affinity_keys_preserve_direction_and_exact_numeric_encoding() {
        let subject = CharacterId::from(7);
        let actor = CharacterId::from(17);
        let key = CharacterAffinityKey::new(subject, actor);
        assert_eq!(String::from(key), "7:17");
        assert_eq!(
            String::from(CharacterAffinityKey::new(actor, subject)),
            "17:7"
        );
        assert_ne!(key, CharacterAffinityKey::new(actor, subject));
    }
}
