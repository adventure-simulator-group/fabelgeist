//! Evaluate a wearer's surface before fitting, rather than fitting every endpoint.
use super::*;
use adventuresim_core::{
    character_morph::CharacterMorphWeights,
    character_proportions::{BODY_PROPORTION_COUNT, CharacterProportions},
    skeletal_fit::SkeletalFitMorph,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct BodyShapeKey {
    weights: Vec<u32>,
    proportions: [u32; BODY_PROPORTION_COUNT],
}

impl BodyShapeKey {
    pub(super) fn new(
        names: &[String],
        character: Option<u64>,
        proportions: CharacterProportions,
        reference: CharacterProportions,
    ) -> Self {
        let identity = character.map(CharacterMorphWeights::from_character_id);
        Self {
            weights: names
                .iter()
                .map(|name| {
                    identity
                        .as_ref()
                        .and_then(|identity| identity.named_weight(name))
                        .or_else(|| {
                            SkeletalFitMorph::from_name(name)
                                .map(|m| m.weight(proportions, reference))
                        })
                        .unwrap_or(0.0)
                        .to_bits()
                })
                .collect(),
            proportions: <[f32; BODY_PROPORTION_COUNT]>::from(proportions).map(f32::to_bits),
        }
    }

    pub(super) fn proportions(&self) -> CharacterProportions {
        self.proportions
            .map(f32::from_bits)
            .try_into()
            .expect("validated proportions")
    }

    pub(super) fn apply(
        &self,
        body: &mut RuntimeBody,
        targets: &[MorphAttributes],
    ) -> anyhow::Result<()> {
        let count = body.positions.len();
        anyhow::ensure!(
            targets.len() == count * self.weights.len(),
            "body morph dimensions disagree"
        );
        for (weight, target) in self.weights.iter().zip(targets.chunks(count)) {
            let weight = f32::from_bits(*weight);
            if weight == 0.0 {
                continue;
            }
            for ((position, normal), delta) in
                body.positions.iter_mut().zip(&mut body.normals).zip(target)
            {
                *position = (Vec3::from_array(*position) + delta.position * weight).to_array();
                *normal = (Vec3::from_array(*normal) + delta.normal * weight).to_array();
            }
        }
        for normal in &mut body.normals {
            *normal = Vec3::from_array(*normal).normalize_or_zero().to_array();
        }
        body.device = Default::default();
        Ok(())
    }
}
