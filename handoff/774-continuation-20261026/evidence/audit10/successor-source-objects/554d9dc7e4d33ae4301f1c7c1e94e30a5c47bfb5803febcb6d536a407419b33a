//! Admit an opaque complete preparation at the native map installation boundary.
use super::*;
use crate::presentation::generation::activate_city;
use adventuresim_tactical_core::regional_city::RegionalCityInput;

enum Admission {
    Resident,
    Replacement(Box<InstalledCity>),
}

pub(in crate::presentation::regional_map) fn install(
    world: &mut World,
    json: &str,
    preparation: PreparationTicket,
) {
    let result = admit(world, json, preparation);
    let retired = {
        let mut state = world.resource_mut::<MapState>();
        match result {
            Ok(Admission::Resident) => {
                state.city.status = Some(InstallationStatus {
                    preparation,
                    phase: InstallationPhase::Ready,
                    error: None,
                });
                None
            }
            Ok(Admission::Replacement(city)) => {
                let old = state.city.pending.replace(*city);
                state.city.status = Some(InstallationStatus {
                    preparation,
                    phase: InstallationPhase::Preparing,
                    error: None,
                });
                state.city.settled_frames = 0;
                old.map(|old| old.root)
            }
            Err(error) => {
                state.city.status = Some(InstallationStatus::failed(preparation, error));
                None
            }
        }
    };
    if let Some(root) = retired
        && let Ok(entity) = world.get_entity_mut(root)
    {
        entity.despawn();
    }
}

fn admit(
    world: &mut World,
    json: &str,
    preparation: PreparationTicket,
) -> std::result::Result<Admission, InstallationFailure> {
    let document: RegionalCityInput = serde_json::from_str(json).map_err(|error| {
        warn!(%error, "Invalid focused city document");
        InstallationFailure::Input
    })?;
    let state = world.resource::<MapState>();
    let origin = state
        .presented
        .as_ref()
        .filter(|surface| &surface.source == document.source())
        .filter(|_| state.requested_city() == Some(document.place()))
        .map(|surface| surface.request.origin)
        .ok_or(InstallationFailure::Focus)?;
    let product = activate_city(preparation, &document).map_err(|error| {
        warn!(%error, "Could not activate focused city preparation");
        InstallationFailure::Preparation
    })?;
    let residency = &world.resource::<MapState>().city;
    if residency.pending.is_none()
        && residency
            .installed
            .as_ref()
            .is_some_and(|city| Arc::ptr_eq(&city.product, &product))
    {
        return Ok(Admission::Resident);
    }
    let frame = CityFrame::from_geographic_city(&document, origin);
    let owner = PresentationOwner::RegionalMap;
    let root = world
        .spawn((
            Name::new("Focused map city"),
            owner,
            owner.render_layers(),
            Transform::from_matrix(frame.world_from_city()),
            Visibility::Hidden,
        ))
        .id();
    let installation = FocusedGround::install(world, &product, root, frame)
        .map_err(|error| {
            warn!(%error, "Could not install canonical city ground");
            InstallationFailure::Ground
        })
        .and_then(|ground| {
            buildings::queue_focused_city(world, &document, root, origin).map_err(|error| {
                warn!(%error, "Could not queue complete focused city");
                InstallationFailure::Buildings
            })?;
            Ok(InstalledCity {
                root,
                product,
                ground,
                frame,
            })
        });
    if installation.is_err()
        && let Ok(entity) = world.get_entity_mut(root)
    {
        entity.despawn();
    }
    installation.map(|city| Admission::Replacement(Box::new(city)))
}
