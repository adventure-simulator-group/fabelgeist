//! Seed dispersal and renewal of shrub clumps.
use super::*;
use crate::units::GroundPoint;
use fabelgeist_math::units::Metres;

impl Ecosystem {
    /// Place seeds around their parents (a Gaussian of the seeding radius),
    /// where the soil is open and no plant already stands.
    pub(super) fn disperse(&mut self, seeds: &[(SpeciesId, Vec3, f32, u32)]) -> u32 {
        if seeds.is_empty() {
            return 0;
        }
        let spacing = self.settings.seed_spacing.max(1e-3);
        let mut bases: Vec<[f32; 3]> = self
            .plants
            .iter()
            .map(|p| [p.position.x, 0.0, p.position.z])
            .collect();
        let grid = HashGrid::new(bases.iter().copied(), spacing);
        let mut germinated = 0;
        for &(species, origin, radius, count) in seeds {
            for _ in 0..count {
                if self.plants.len() >= self.settings.max_plants {
                    return germinated;
                }
                let x = origin.x + self.rng.gaussian() * radius;
                let z = origin.z + self.rng.gaussian() * radius;
                let mut crowded = false;
                grid.for_each_near([x, 0.0, z], 1, |j| {
                    let b = bases[j as usize];
                    crowded |= (b[0] - x).powi(2) + (b[2] - z).powi(2) < spacing * spacing;
                });
                // Seedlings placed this step are checked by brute force; there
                // are few of them.
                let start = grid.len();
                crowded |= bases[start..]
                    .iter()
                    .any(|b| (b[0] - x).powi(2) + (b[2] - z).powi(2) < spacing * spacing);
                if crowded {
                    continue;
                }
                if self
                    .plant(
                        species,
                        GroundPoint::new(Metres::from_metres(x), Metres::from_metres(z)),
                    )
                    .is_ok()
                {
                    bases.push([x, 0.0, z]);
                    germinated += 1;
                }
            }
        }
        germinated
    }

    /// New stems from around the bases of plants that sprout
    /// ([`PlantType::sprouts_per_year`]), while their clump has room. Each
    /// leans out of the clump; its leader turns back up as it grows, so a
    /// clump opens into a vase.
    pub(super) fn sprout(&mut self, dt: f32) -> u32 {
        /// Radius of a clump, metres.
        const CLUMP: f32 = 0.6;
        /// How far a new stem leans out, radians.
        const LEAN: f32 = 0.35;
        if self.species.iter().all(|s| s.sprouts_per_year <= 0.0) {
            return 0;
        }
        let mut stems: Vec<(SpeciesId, [f32; 3])> = self
            .plants
            .iter()
            .map(|p| (p.species, [p.position.x, 0.0, p.position.z]))
            .collect();
        let grid = HashGrid::new(stems.iter().map(|s| s.1), CLUMP);
        let mut sprouted = 0;
        for i in 0..self.plants.len() {
            // Stems sprout from the clump's root crown, not their own: a
            // clump holds its ground rather than creeping away.
            // Sprouting draws on the roots' reserves: a stem in the open
            // sprouts freely, a shaded one seldom.
            let (species, origin, age, reserves) = {
                let p = &self.plants[i];
                let s = &self.species[p.species.index()];
                // A kept stem sends up no more.
                let reserves = if p.preserved {
                    0.0
                } else {
                    (p.root_vigor / p.root_vigor_cap(s).max(1e-6)).clamp(0.0, 1.0)
                };
                (p.species, p.clump, p.age.years(), reserves)
            };
            let (rate, most) = {
                let s = &self.species[species.index()];
                (
                    s.sprouts_per_year * reserves * reserves,
                    s.clump_stems as usize,
                )
            };
            if rate <= 0.0 || age < 2.0 {
                continue;
            }
            for _ in 0..self.rng.stochastic_round(rate * dt) {
                if self.plants.len() >= self.settings.max_plants {
                    return sprouted;
                }
                let near = |b: [f32; 3]| {
                    (b[0] - origin.x).powi(2) + (b[2] - origin.z).powi(2) < CLUMP * CLUMP
                };
                let mut clump = 0;
                grid.for_each_near([origin.x, 0.0, origin.z], 1, |j| {
                    let (other, b) = stems[j as usize];
                    clump += usize::from(other == species && near(b));
                });
                clump += stems[grid.len()..]
                    .iter()
                    .filter(|(other, b)| *other == species && near(*b))
                    .count();
                if clump >= most {
                    break;
                }
                let angle = self.rng.range(0.0, std::f32::consts::TAU);
                let out = Vec3::new(angle.cos(), 0.0, angle.sin());
                let at = origin + out * (CLUMP * 0.5 * self.rng.next_f32().sqrt());
                let Ok(new) = self.plant(
                    species,
                    GroundPoint::new(Metres::from_metres(at.x), Metres::from_metres(at.z)),
                ) else {
                    continue;
                };
                let lean = Vec4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0).cross(out), LEAN);
                let s = &self.species[species.index()];
                let plant = &mut self.plants[new.index()];
                plant.clump = origin;
                plant.modules[0].orientation = lean.mul_quat(plant.modules[0].orientation);
                plant.update_geometry(s, &self.prototypes);
                stems.push((species, [at.x, 0.0, at.z]));
                sprouted += 1;
            }
        }
        sprouted
    }
}
