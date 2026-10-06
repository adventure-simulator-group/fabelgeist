use super::*;
use adventuresim_building_generator::{
    BuildingCollision, BuildingProgram, DoorSpec, ServiceBuildingSize, compile_building_collision,
    compile_building_detail, compile_operable_doors, generate,
};
use bevy::math::Vec3;
use std::{collections::BTreeMap, sync::Arc};
mod catalogue;
#[cfg(test)]
mod tests;

const RECIPE_SELECTION_DOMAIN: StreamId = StreamId::new("city.building-recipe");
const CURATED_RECIPE_SEEDS: [u64; 3] = [42, 47, 101];

/// Lightweight immutable recipes retained from accepted city compilation.
/// The memo contains programmes, bearings, thresholds and measured envelopes;
/// it does not retain facade meshes or acquire simulation authority.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CityRecipePalette {
    entries: BTreeMap<RecipeKey, Arc<Recipe>>,
    occupied: Vec<Arc<Recipe>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RecipeKey {
    archetype_slug: &'static str,
    usage: Option<BuildingUse>,
    size: Option<ServiceBuildingSize>,
    seed: u64,
}

#[derive(Debug, PartialEq)]
pub(super) struct Recipe {
    pub program: BuildingProgram,
    pub collision: BuildingCollision,
    /// Complete detailed envelope, relative to the collision-centred placement.
    pub render_min: Vec2,
    pub render_max: Vec2,
    pub doors: Vec<DoorSpec<adventuresim_building_generator::spatial_geometry::Architectural>>,
    pub ground_entrances: Vec<adventuresim_building_generator::BuildingEntrance>,
}

impl CityRecipePalette {
    pub(super) fn front(
        &mut self,
        seed: fabelgeist_determinism::Seed,
        lot: CityBuildingLot,
    ) -> Result<Arc<Recipe>, CityCompileError> {
        let choice = RECIPE_SELECTION_DOMAIN
            .rng(seed, &[lot.id])
            .index(CURATED_RECIPE_SEEDS.len());
        let archetype = lot.archetype();
        let usage = lot.building_use().unwrap_or(BuildingUse::Dwelling);
        self.get(
            archetype,
            Some(usage),
            lot.service_size(),
            catalogue::seed(archetype, usage, choice),
        )
    }

    pub(super) fn range(&mut self) -> Result<Arc<Recipe>, CityCompileError> {
        self.get(
            BuildingArchetype::StorageRange,
            None,
            None,
            CURATED_RECIPE_SEEDS[0],
        )
    }

    pub(super) fn get(
        &mut self,
        archetype: BuildingArchetype,
        usage: Option<BuildingUse>,
        size: Option<ServiceBuildingSize>,
        seed: u64,
    ) -> Result<Arc<Recipe>, CityCompileError> {
        let key = RecipeKey {
            archetype_slug: archetype.slug(),
            usage,
            size,
            seed,
        };
        if let Some(recipe) = self.entries.get(&key) {
            return Ok(recipe.clone());
        }
        let program = match usage {
            Some(usage) if catalogue::exact_upper_dwelling(archetype, Some(usage)) => {
                let program = BuildingProgram::settlement(archetype, Some(usage), seed);
                match size {
                    Some(size) => program.with_service_size(size),
                    None => program,
                }
            }
            Some(usage) => BuildingProgram::validated_settlement(archetype, usage, seed, size)
                .map_err(|source| CityCompileError::Recipe {
                    archetype,
                    seed,
                    source,
                })?,
            None => BuildingProgram::fixture(archetype, seed),
        };
        let recipe = Recipe::compile(&program)?;
        self.occupied.push(recipe.clone());
        self.entries.insert(key, recipe.clone());
        Ok(recipe)
    }
    /// Reconstruct an already occupied programme exactly. A cache miss does
    /// not run recipe selection or search for a different valid seed.
    pub(super) fn for_program(
        &mut self,
        program: &BuildingProgram,
    ) -> Result<Arc<Recipe>, CityCompileError> {
        if let Some(recipe) = self
            .occupied
            .iter()
            .find(|recipe| recipe.program == *program)
        {
            return Ok(recipe.clone());
        }
        let recipe = Recipe::compile(program)?;
        self.occupied.push(recipe.clone());
        Ok(recipe)
    }
}

impl Recipe {
    fn compile(program: &BuildingProgram) -> Result<Arc<Self>, CityCompileError> {
        let archetype = program.archetype;
        let seed = program.seed;
        let plan = generate(program).map_err(|source| CityCompileError::Recipe {
            archetype,
            seed,
            source,
        })?;
        if plan.domestic_heating.is_some() {
            adventuresim_building_generator::interior::validate_circulation(&plan).map_err(|source| CityCompileError::Recipe {
                archetype, seed,
                source: adventuresim_building_generator::GenerationError::BlockedDomesticCirculation(source),
            })?;
        }
        let fail = |source| CityCompileError::Recipe {
            archetype,
            seed,
            source,
        };
        let collision = compile_building_collision(&plan).map_err(|e| fail(e.into()))?;
        let origin = collision
            .bounds
            .centre()
            .map_err(|e| fail(e.into()))?
            .metres();
        let (render_min, render_max) = compile_building_detail(&plan)
            .map_err(fail)?
            .meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .map(|v| v.position - origin)
            .map(|p| Vec2::new(p.x, p.z))
            .fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), p| (min.min(p), max.max(p)),
            );
        let recipe = Arc::new(Self {
            program: program.clone(),
            collision,
            render_min,
            render_max,
            doors: compile_operable_doors(&plan).map_err(|e| fail(e.into()))?,
            ground_entrances: adventuresim_building_generator::compile_ground_entrances(&plan)
                .map_err(|e| fail(e.into()))?,
        });
        Ok(recipe)
    }

    pub fn from_generated(
        building: &crate::scene_input::GeneratedBuilding,
    ) -> Result<Self, adventuresim_building_generator::GenerationError> {
        let origin = building.collision.bounds.centre()?.metres();
        let (render_min, render_max) = compile_building_detail(&building.plan)?
            .meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .map(|v| Vec2::new(v.position.x - origin.x, v.position.z - origin.z))
            .fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), p| (min.min(p), max.max(p)),
            );
        Ok(Self {
            program: building.placement.program.clone(),
            collision: building.collision.clone(),
            render_min,
            render_max,
            doors: compile_operable_doors(&building.plan)?,
            ground_entrances: adventuresim_building_generator::compile_ground_entrances(
                &building.plan,
            )?,
        })
    }

    pub fn place(
        &self,
        id: crate::scene_input::SceneBuildingId,
        centre_metres: Vec2,
        orientation: BuildingOrientation,
    ) -> Result<TacticalBuildingPlacement, CityCompileError> {
        // Lots use a street-facing -Y frame. The basilica's west portal is -X;
        // compose its physical frame once for every scene representation.
        let orientation = match self.program.frontage_direction() {
            adventuresim_building_generator::Direction::West => BuildingOrientation::from_radians(
                orientation.yaw_radians() - core::f32::consts::FRAC_PI_2,
            )
            .ok_or(
                adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection,
            )?,
            _ => orientation,
        };
        Ok(TacticalBuildingPlacement {
            base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
            id,
            program: self.program.clone(),
            centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(centre_metres)?,
            orientation,
        })
    }

    pub fn door_point(
        &self,
        placement: &TacticalBuildingPlacement,
        outward: adventuresim_building_generator::Direction,
    ) -> Result<Option<crate::scene_coordinates::ScenePlanPoint>, CityCompileError> {
        let mut doors = self
            .doors
            .iter()
            .filter(|door| door.outward.vector() == outward.offset().as_vec2());
        let Some(door) = doors.next().filter(|_| doors.next().is_none()) else {
            return Ok(None);
        };
        let centre = door.hinge_centre.metres()
            + Vec3::new(door.tangent.vector().x, 0.0, door.tangent.vector().y)
                * door.size_metres.metres().x
                * 0.5;
        let project = || {
            use adventuresim_building_generator::plan_geometry::ArchitecturalPlanPoint;
            let projection = crate::scene_coordinates::ArchitecturalPlanProjection::from_placement(
                placement,
                self.collision.bounds,
            )?;
            projection.point(ArchitecturalPlanPoint::try_from(Vec2::new(
                centre.x, centre.z,
            ))?)
        };
        project()
            .map(Some)
            .map_err(|source| CityCompileError::Recipe {
                archetype: self.program.archetype,
                seed: self.program.seed,
                source: source.into(),
            })
    }

    pub fn fits(&self, placement: &TacticalBuildingPlacement, bounds: CityPlotBounds) -> bool {
        [
            self.render_min,
            Vec2::new(self.render_max.x, self.render_min.y),
            self.render_max,
            Vec2::new(self.render_min.x, self.render_max.y),
        ]
        .into_iter()
        .all(|p| {
            bounds.contains(
                placement.centre_metres.metres() + placement.orientation.local_to_world(p),
            )
        })
    }
}
