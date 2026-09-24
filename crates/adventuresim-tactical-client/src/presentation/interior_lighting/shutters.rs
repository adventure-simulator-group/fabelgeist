//! Projected open aperture follows the replicated leaf transform.
use super::field::{InteriorField, LightSample};
use adventuresim_tactical_core::prelude::{SceneBuilding, SceneWindow};
use bevy::prelude::*;

const APERTURE_CHANGE_TOLERANCE: f32 = 0.001;

#[derive(Clone)]
pub(super) struct ShutterLight {
    pub opening_id: u64,
    pub contributions: Vec<(usize, LightSample)>,
    pub openness: f32,
}

impl InteriorField {
    fn update_apertures(&mut self, apertures: &[(u64, f32)]) -> bool {
        let mut changed = false;
        for shutter in &mut self.shutters {
            let openness = apertures
                .iter()
                .find(|entry| entry.0 == shutter.opening_id)
                .map_or(0.0, |entry| entry.1.clamp(0.0, 1.0));
            if (shutter.openness - openness).abs() > APERTURE_CHANGE_TOLERANCE {
                shutter.openness = openness;
                changed = true;
            }
        }
        if changed {
            self.samples.clone_from(&self.base_samples);
            for shutter in &self.shutters {
                for &(index, light) in &shutter.contributions {
                    self.samples[index].positive += light.positive * shutter.openness;
                    self.samples[index].negative += light.negative * shutter.openness;
                }
            }
            for sample in &mut self.samples {
                sample.positive = sample
                    .positive
                    .truncate()
                    .min(Vec3::ONE)
                    .extend(sample.positive.w);
                sample.negative = sample.negative.truncate().min(Vec3::ONE).extend(0.0);
            }
        }
        changed
    }
}

pub(super) fn update_shutter_light(
    mut fields: Query<(&SceneBuilding, &mut InteriorField)>,
    windows: Query<(&SceneWindow, &GlobalTransform)>,
) {
    for (building, mut field) in &mut fields {
        if field.shutters.is_empty() {
            continue;
        }
        let apertures = windows
            .iter()
            .filter(|(window, _)| window.building_id == building.id)
            .map(|(window, transform)| {
                let tangent = transform
                    .affine()
                    .transform_vector3(Vec3::X)
                    .normalize_or_zero();
                (
                    window.opening_id,
                    1.0 - tangent.dot(window.tangent).abs().clamp(0.0, 1.0),
                )
            })
            .collect::<Vec<_>>();
        if field.bypass_change_detection().update_apertures(&apertures) {
            field.set_changed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::{BuildingArchetype, BuildingProgram, generate};

    #[test]
    fn opening_and_closing_a_shutter_adds_and_removes_only_its_daylight() {
        let plan = generate(&BuildingProgram::fixture(
            BuildingArchetype::FachwerkCottage,
            42,
        ))
        .unwrap();
        let mut field = InteriorField::from_plan(&plan, Vec3::ZERO);
        assert!(!field.shutters.is_empty());
        let opening = field.shutters[0].opening_id;
        let closed = field.samples.clone();
        assert!(field.update_apertures(&[(opening, 0.8)]));
        assert!(
            field
                .samples
                .iter()
                .zip(&closed)
                .any(|(a, b)| a.positive.x > b.positive.x || a.negative.x > b.negative.x)
        );
        assert!(
            field
                .samples
                .iter()
                .zip(&closed)
                .all(|(a, b)| a.positive.w == b.positive.w)
        );
        assert!(!field.update_apertures(&[(opening, 0.8)]));
        assert!(field.update_apertures(&[]));
        assert_eq!(field.samples, closed);
    }
}
