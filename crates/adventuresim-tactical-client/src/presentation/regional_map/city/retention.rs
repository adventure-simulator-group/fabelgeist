//! Retain one previous map installation without retaining an unbounded journey.
use super::*;

impl CityResidency {
    pub(in crate::presentation::regional_map) fn clear(&mut self, world: &mut World) {
        let cities = [
            self.pending.take(),
            self.installed.take(),
            self.previous.take(),
        ];
        *self = Self::default();
        for city in cities.into_iter().flatten() {
            city.discard(world);
        }
    }
}

impl InstalledCity {
    fn discard(self, world: &mut World) {
        if let CityGeometry::Retained(geometry) = self.geometry {
            geometry.discard(world);
        }
        if let Ok(entity) = world.get_entity_mut(self.root) {
            entity.despawn();
        }
    }
}

pub(super) fn reuse(
    world: &mut World,
    product: &Arc<PreparedCityProduct>,
    origin: Wgs84CoordinateMicrodegrees,
) -> std::result::Result<Option<InstalledCity>, InstallationFailure> {
    save_current_geometry(world);
    let residency = &world.resource::<MapState>().city;
    let current_matches = residency.installed.as_ref().is_some_and(|city| {
        matches!(city.geometry, CityGeometry::Retained(_))
            && city.product.same_presentation_inputs(product)
            && world.get_entity(city.root).is_ok()
    });
    let previous_matches = residency.previous.as_ref().is_some_and(|city| {
        city.product.same_presentation_inputs(product) && world.get_entity(city.root).is_ok()
    });
    let candidate = {
        let mut state = world.resource_mut::<MapState>();
        if current_matches {
            state.city.installed.take()
        } else if previous_matches {
            state.city.previous.take()
        } else {
            None
        }
    };
    let Some(mut city) = candidate else {
        return Ok(None);
    };
    if let CityGeometry::Retained(geometry) =
        std::mem::replace(&mut city.geometry, CityGeometry::Active)
    {
        geometry.restore(world);
    }
    city.product = product.clone();
    if let Err(error) = city.reanchor(world, origin) {
        warn!(%error, "Could not reanchor retained map city");
        buildings::reset_gpu(world, PresentationOwner::RegionalMap);
        city.discard(world);
        return Err(InstallationFailure::Ground);
    }
    Ok(Some(city))
}

pub(super) fn retire(world: &mut World, residency: &mut CityResidency, city: InstalledCity) {
    if !matches!(city.geometry, CityGeometry::Retained(_)) {
        city.discard(world);
        return;
    }
    if let Some(mut visibility) = world.get_mut::<Visibility>(city.root) {
        *visibility = Visibility::Hidden;
    }
    if let Some(previous) = residency.previous.replace(city) {
        previous.discard(world);
    }
}

fn save_current_geometry(world: &mut World) {
    let root = world
        .resource::<MapState>()
        .city
        .installed
        .as_ref()
        .filter(|city| matches!(city.geometry, CityGeometry::Active))
        .map(|city| city.root);
    let Some(root) = root else {
        return;
    };
    if let Some(geometry) =
        buildings::RetainedCityGeometry::take(world, PresentationOwner::RegionalMap, root)
        && let Some(city) = world.resource_mut::<MapState>().city.installed.as_mut()
    {
        city.geometry = CityGeometry::Retained(Box::new(geometry));
    }
}
