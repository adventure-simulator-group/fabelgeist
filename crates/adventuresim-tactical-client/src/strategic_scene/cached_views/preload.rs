//! Prepare unvisited residents' portraits using the current portrait dimensions.
use super::*;

#[cfg(test)]
mod tests;

pub(super) fn specs(view: &StrategicView, scene: &RetainedScene) -> Vec<ViewSpec> {
    let Some(template) = view.portraits.first() else {
        return Vec::new();
    };
    view.people
        .iter()
        .filter_map(|person| {
            let retained = scene.people.get(&person.id)?;
            let mut spec = super::super::views::person_view(
                template.rect,
                retained.anchor,
                retained.facing,
                true,
                super::super::views::focused_layers(scene, Some(person.id)),
            );
            spec.cache = Some(ViewKey::Portrait {
                person: person.id,
                slot: 0,
            });
            Some(spec)
        })
        .collect()
}
