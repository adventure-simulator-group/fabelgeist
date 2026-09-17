//! Portable render outputs for clients that load recipes prepared offline.
use crate::{signs::*, *};
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

const RECIPE_HASH_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Increment when compiled render geometry changes without a programme schema change.
const PREPARED_RENDER_VERSION: u16 = 3;
const RECIPE_HASH_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Serialize, Deserialize)]
pub struct PreparedBuilding {
    #[serde(with = "program_json")]
    pub program: BuildingProgram,
    pub local_origin: Vec3,
    pub floor_offset_metres: f32,
    pub sign_sites: Vec<(SignMount, SignSite)>,
    pub lod0: Vec<LodMesh>,
    pub lod1: Vec<LodMesh>,
    pub lod2: Vec<LodMesh>,
}

mod program_json {
    use super::BuildingProgram;
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer, de::Error as _, ser::Error as _,
    };

    pub(super) fn serialize<S>(program: &BuildingProgram, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_json::to_vec(program)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<BuildingProgram, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        serde_json::from_slice(&bytes).map_err(D::Error::custom)
    }
}

impl PreparedBuilding {
    /// Compile once natively; the browser only uploads these existing meshes.
    pub fn from_plan(program: BuildingProgram, plan: &BuildingPlan) -> Self {
        let collision = compile_building_collision(plan);
        let local_origin = collision.bounds.centre();
        let sign_sites = SignSite::for_plan(plan).map_or_else(Vec::new, |site| {
            [SignMount::Wall, SignMount::Projecting]
                .into_iter()
                .filter(|mount| site.supports(plan, *mount))
                .map(|mount| (mount, site))
                .collect()
        });
        Self {
            program,
            local_origin,
            floor_offset_metres: local_origin.y - collision.bounds.min.y,
            sign_sites,
            lod0: compile_building_detail(plan).meshes,
            lod1: compile_building_lod(plan, BuildingLodLevel::Facade).meshes,
            lod2: compile_building_lod(plan, BuildingLodLevel::Shell).meshes,
        }
    }

    /// Extract full detail into a separately requested asset.
    pub fn take_detail(&mut self) -> Self {
        Self {
            program: self.program.clone(),
            local_origin: self.local_origin,
            floor_offset_metres: self.floor_offset_metres,
            sign_sites: Vec::new(),
            lod0: std::mem::take(&mut self.lod0),
            lod1: Vec::new(),
            lod2: Vec::new(),
        }
    }

    /// Extract the facade while leaving the cheapest shell in the overview.
    pub fn take_facade(&mut self) -> Self {
        Self {
            program: self.program.clone(),
            local_origin: self.local_origin,
            floor_offset_metres: self.floor_offset_metres,
            sign_sites: std::mem::take(&mut self.sign_sites),
            lod0: Vec::new(),
            lod1: std::mem::take(&mut self.lod1),
            lod2: Vec::new(),
        }
    }
}

/// Stable content identity for a serialized recipe, independent of placement.
pub fn recipe_key(program: &BuildingProgram) -> String {
    let bytes = serde_json::to_vec(program).expect("building programs serialize");
    let hash = PREPARED_RENDER_VERSION
        .to_le_bytes()
        .into_iter()
        .chain(bytes)
        .fold(RECIPE_HASH_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(RECIPE_HASH_PRIME)
        });
    format!("{hash:016x}")
}
