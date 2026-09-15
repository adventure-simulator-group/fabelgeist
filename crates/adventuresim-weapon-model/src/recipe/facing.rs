//! Explicit directional dependencies between independently parameterized heads.
use super::*;
use std::collections::BTreeSet;

fn direction(shape: &Shape) -> Option<Direction> {
    match shape {
        Shape::Axe(p) => Some(p.side.unwrap_or(Direction::Positive)),
        Shape::Beak(p) => Some(p.direction.unwrap_or(Direction::Negative)),
        Shape::FacetedBeak(p) => Some(p.direction.unwrap_or(Direction::Negative)),
        Shape::Hammer(p) => Some(p.direction.unwrap_or(Direction::Positive)),
        _ => None,
    }
}

pub(crate) fn directions(recipe: &Recipe) -> Result<Vec<(usize, Direction)>, String> {
    fn resolve(
        index: usize,
        recipe: &Recipe,
        visiting: &mut BTreeSet<usize>,
    ) -> Result<Direction, String> {
        if !visiting.insert(index) {
            return Err("working-end facing dependencies contain a cycle".into());
        }
        let component = &recipe.components[index];
        let own = direction(&component.shape).ok_or("component has no directional working end")?;
        let result = if let Some(target) = &component.opposed_to {
            let parent = recipe
                .components
                .iter()
                .enumerate()
                .position(|(i, c)| c.resolved_id(i) == *target)
                .ok_or("opposed working end names a missing component")?;
            match resolve(parent, recipe, visiting)? {
                Direction::Positive => Direction::Negative,
                Direction::Negative => Direction::Positive,
            }
        } else {
            own
        };
        visiting.remove(&index);
        Ok(result)
    }
    recipe
        .components
        .iter()
        .enumerate()
        .filter(|(_, c)| c.opposed_to.is_some())
        .map(|(i, _)| resolve(i, recipe, &mut BTreeSet::new()).map(|d| (i, d)))
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) fn apply(recipe: &mut Recipe) -> Result<(), String> {
    for (index, direction) in directions(recipe)? {
        let c = &mut recipe.components[index];
        match &mut c.shape {
            Shape::Axe(p) => p.side = Some(direction),
            Shape::Beak(p) => p.direction = Some(direction),
            Shape::FacetedBeak(p) => p.direction = Some(direction),
            Shape::Hammer(p) => p.direction = Some(direction),
            _ => return Err("component has no directional working end".into()),
        }
        if let Some(offset) = &mut c.offset {
            offset[0] = Metres::new(offset[0].get().abs() * direction.sign())?;
        }
        if let Some(offset) = c.attach.as_mut().and_then(|a| a.offset.as_mut()) {
            offset[0] = Metres::new(offset[0].get().abs() * direction.sign())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opposed_ends_use_resolved_ids_and_equivalent_attachment_offsets() {
        for side in [Direction::Negative, Direction::Positive] {
            let catalog: serde_json::Value =
                serde_json::from_str(include_str!("../../catalog/authoring.json")).unwrap();
            let mut mounted: Recipe = serde_json::from_value(
                catalog["presets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["id"] == "halberd-1540")
                    .unwrap()["definition"]
                    .clone(),
            )
            .unwrap();
            mounted.components[1].id = None;
            mounted.components[1].label = None;
            let Shape::Axe(p) = &mut mounted.components[1].shape else {
                panic!()
            };
            p.side = Some(side);
            mounted.components[2].opposed_to = Some("component-1".into());
            let mut attached = mounted.clone();
            let rear = &mut attached.components[2];
            rear.mount = None;
            rear.attach = Some(Attachment {
                to: "shaft.top".into(),
                at: Some(AttachmentAnchor::Origin),
                offset: rear.offset.take(),
                overlap: None,
            });
            let a = crate::generate_model(&mounted, crate::construction::Detail::Low).unwrap();
            let b = crate::generate_model(&attached, crate::construction::Detail::Low).unwrap();
            assert_eq!(a.positions, b.positions);
            let rear = &a.resolved_definition.recipe.components[2];
            let Shape::Beak(p) = &rear.shape else {
                panic!()
            };
            assert_eq!(p.direction.unwrap().sign(), -side.sign());
        }
    }
}
