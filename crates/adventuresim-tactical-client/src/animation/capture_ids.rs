use bevy::{animation::AnimationTargetId, prelude::*};

pub(crate) fn capture_animation_target_id(target: AnimationTargetId) -> String {
    format!("AnimationTargetId({})", target.0)
}

pub(crate) fn capture_entity_id(entity: Entity) -> String {
    if entity == Entity::PLACEHOLDER {
        "PLACEHOLDER".to_owned()
    } else {
        format!("{}v{}", entity.index_u32(), entity.generation().to_bits())
    }
}

#[cfg(test)]
mod capture_id_tests {
    use super::*;

    #[test]
    fn capture_ids_have_explicit_stable_encodings() {
        assert_eq!(capture_entity_id(Entity::PLACEHOLDER), "PLACEHOLDER");
        assert_eq!(capture_entity_id(Entity::from_raw_u32(1).unwrap()), "1v0");
        let target = AnimationTargetId::from_name(&Name::new("capture-bone"));
        assert_eq!(
            capture_animation_target_id(target),
            "AnimationTargetId(4fa8b1ad-0d70-5ad3-9c90-7f17995275f4)"
        );
    }
}
