use super::*;
use adventuresim_building_generator::{
    BuildingCollision, BuildingProgram, DoorSpec, ServiceBuildingSize, compile_building_collision,
    compile_building_detail, compile_operable_doors, generate,
};
use bevy::math::Vec3;
use std::{collections::BTreeMap, sync::Arc};

const RECIPE_SELECTION_DOMAIN: StreamId = StreamId::new("city.building-recipe");
const CURATED_RECIPE_SEEDS: [u64; 3] = [42, 47, 101];

#[derive(Default)]
pub(super) struct RecipePalette {
    entries: BTreeMap<RecipeKey, Arc<Recipe>>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct RecipeKey {
    archetype_slug: &'static str,
    usage: Option<BuildingUse>,
    size: Option<ServiceBuildingSize>,
    seed: u64,
}

pub(super) struct Recipe {
    pub program: BuildingProgram,
    pub collision: BuildingCollision,
    /// Complete detailed envelope, relative to the collision-centred placement.
    pub render_min: Vec2,
    pub render_max: Vec2,
    pub doors: Vec<DoorSpec>,
}

impl RecipePalette {
    pub fn front(
        &mut self,
        seed: u64,
        lot: CityBuildingLot,
    ) -> Result<Arc<Recipe>, CityCompileError> {
        let choice = RECIPE_SELECTION_DOMAIN
            .rng(seed, &[lot.id])
            .index(CURATED_RECIPE_SEEDS.len());
        self.get(
            lot.archetype(),
            Some(lot.building_use().unwrap_or(BuildingUse::Dwelling)),
            lot.service_size(),
            CURATED_RECIPE_SEEDS[choice],
        )
    }

    pub fn range(&mut self) -> Result<Arc<Recipe>, CityCompileError> {
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
            Some(usage) => BuildingProgram::validated_settlement(archetype, usage, seed, size)
                .map_err(|source| CityCompileError::Recipe {
                    archetype,
                    seed,
                    source,
                })?,
            None => BuildingProgram::fixture(archetype, seed),
        };
        let plan = generate(&program).map_err(|source| CityCompileError::Recipe {
            archetype,
            seed,
            source,
        })?;
        let collision = compile_building_collision(&plan);
        let origin = collision.bounds.centre();
        let (render_min, render_max) = compile_building_detail(&plan)
            .meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .map(|v| v.position - origin)
            .map(|p| Vec2::new(p.x, p.z))
            .fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), p| (min.min(p), max.max(p)),
            );
        let recipe = Arc::new(Recipe {
            program,
            collision,
            render_min,
            render_max,
            doors: compile_operable_doors(&plan),
        });
        self.entries.insert(key, recipe.clone());
        Ok(recipe)
    }
}

impl Recipe {
    pub fn from_generated(building: &crate::scene_input::GeneratedBuilding) -> Self {
        let origin = building.collision.bounds.centre();
        let (render_min, render_max) = compile_building_detail(&building.plan)
            .meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .map(|v| Vec2::new(v.position.x - origin.x, v.position.z - origin.z))
            .fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), p| (min.min(p), max.max(p)),
            );
        Self {
            program: building.placement.program.clone(),
            collision: building.collision.clone(),
            render_min,
            render_max,
            doors: compile_operable_doors(&building.plan),
        }
    }

    pub fn place(
        &self,
        id: u64,
        centre_metres: Vec2,
        orientation: BuildingOrientation,
    ) -> TacticalBuildingPlacement {
        // Lots use a street-facing -Y frame. The basilica's west portal is -X;
        // compose its physical frame once for every scene representation.
        let orientation = match self.program.frontage_direction() {
            adventuresim_building_generator::Direction::West => BuildingOrientation::from_radians(
                orientation.yaw_radians() - core::f32::consts::FRAC_PI_2,
            )
            .expect("finite lot orientation"),
            _ => orientation,
        };
        TacticalBuildingPlacement {
            id,
            program: self.program.clone(),
            centre_metres,
            orientation,
        }
    }

    pub fn door_point(&self, placement: &TacticalBuildingPlacement, outward: Vec2) -> Option<Vec2> {
        let door = self.doors.iter().find(|door| door.outward == outward)?;
        let centre = door.hinge_centre
            + Vec3::new(door.tangent.x, 0.0, door.tangent.y) * door.size_metres.x * 0.5;
        let local = centre - self.collision.bounds.centre();
        Some(
            placement.centre_metres
                + placement
                    .orientation
                    .local_to_world(Vec2::new(local.x, local.z)),
        )
    }

    pub fn fits(&self, placement: &TacticalBuildingPlacement, bounds: CityPlotBounds) -> bool {
        [
            self.render_min,
            Vec2::new(self.render_max.x, self.render_min.y),
            self.render_max,
            Vec2::new(self.render_min.x, self.render_max.y),
        ]
        .into_iter()
        .all(|p| bounds.contains(placement.centre_metres + placement.orientation.local_to_world(p)))
    }
}
