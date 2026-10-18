//! UI events admitted into semantic ecosystem policies and units.
use super::*;
use crate::identity::SpeciesId;
use crate::identity::VineSpeciesId;
use crate::policy::GlobalShadowing;
use crate::policy::Mortality;
use crate::policy::Seeding;

impl State {
    pub(super) fn apply_command(
        &mut self,
        command: Command,
        build: &impl Fn(Scene, &Profile) -> anyhow::Result<Ecosystem>,
        step_once: &mut bool,
    ) -> anyhow::Result<()> {
        match command {
            Command::Reset(scene) => {
                self.ecosystem = build(scene, &self.profile)?;
                self.generation += 1;
                self.terrain_sent = false;
            }
            Command::Play(playing) => self.playing = playing,
            Command::Step => *step_once = true,
            Command::Speed(speed) => self.years_per_second = speed,
            Command::Climate {
                temperature,
                precipitation,
            } => {
                self.ecosystem.climate.temperature = temperature;
                self.ecosystem.climate.precipitation = precipitation;
                self.ecosystem.refresh_species();
            }
            Command::Seeding(enabled) => {
                self.ecosystem.settings.seeding = Seeding::from_enabled(enabled)
            }
            Command::Mortality(enabled) => {
                self.ecosystem.settings.mortality = Mortality::from_enabled(enabled)
            }
            Command::GlobalShadowing(enabled) => {
                self.ecosystem.settings.global_shadowing = GlobalShadowing::from_enabled(enabled)
            }
            Command::Profile(profile) => {
                self.profile = profile;
                self.ecosystem.settings.dt = profile.step_years;
            }
            Command::Gap { x, z, radius } => {
                self.ecosystem.clear(x, z, radius);
            }
            Command::Sow {
                species,
                origin,
                direction,
            } => {
                let ecosystem = &mut self.ecosystem;
                let thrown = (Vec3::from_array(origin), Vec3::from_array(direction));
                if let Some(ground) = ecosystem.landing(thrown.0, thrown.1) {
                    let plants = ecosystem.species.len();
                    if species < plants {
                        let _ = ecosystem.plant(SpeciesId::from_word(species as u16), ground);
                    } else {
                        let _ = ecosystem.plant_vine(
                            VineSpeciesId::from_word((species - plants) as u16),
                            ground,
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
