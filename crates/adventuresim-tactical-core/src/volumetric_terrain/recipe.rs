use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainLandformKind {
    FaultScarp,
    SandstoneAlcove,
    CarbonateDissolution,
    GraniteJointRockfall,
    BasaltCoolingColumns,
    CohesiveSlumpHeadscarp,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[component(immutable)]
#[serde(try_from = "QuantizedLandformRecipe", into = "QuantizedLandformRecipe")]
pub struct TerrainLandformRecipe {
    pub(super) kind: TerrainLandformKind,
    pub(super) surface: TerrainSurfaceRecipe,
    pub(super) seed: fabelgeist_determinism::Seed,
    pub(super) origin_cm: [i32; 2],
    /// Unit tangent encoded in ten-thousandths.
    pub(super) tangent_permyriad: [i16; 2],
    pub(super) relief_cm: u16,
    pub(super) half_length_cm: u16,
    pub(super) half_width_cm: u16,
    pub(super) collar_cm: u16,
    pub(super) lod: TerrainLandformLod,
    transition_collar: TerrainTransitionCollar,
}

/// Native scene document admission: positions and dimensions use centimetres;
/// tangent components use ten-thousandths. This record is not admitted geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantizedLandformRecipe {
    pub kind: TerrainLandformKind,
    pub surface: TerrainSurfaceRecipe,
    pub seed: fabelgeist_determinism::Seed,
    pub origin_cm: [i32; 2],
    /// Unit tangent encoded in ten-thousandths.
    pub tangent_permyriad: [i16; 2],
    pub relief_cm: u16,
    pub half_length_cm: u16,
    pub half_width_cm: u16,
    pub collar_cm: u16,
    pub lod: TerrainLandformLod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainLandformLod {
    Detail,
    Fringe,
}

impl TerrainLandformLod {
    pub(crate) const fn voxel_cm(self) -> u16 {
        match self {
            Self::Detail => 50,
            Self::Fringe => 100,
        }
    }
}

impl TerrainLandformRecipe {
    pub fn validate(self, terrain: &SceneTerrain) -> TerrainRecipeResult<()> {
        self.validate_parameters()?;
        let tangent = Vec2::new(
            f32::from(self.tangent_permyriad[0]),
            f32::from(self.tangent_permyriad[1]),
        ) / 10_000.0;
        let origin = Vec2::new(self.origin_cm[0] as f32, self.origin_cm[1] as f32) / 100.0;
        let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        let half_length = f32::from(self.half_length_cm) / 100.0;
        let half_width = f32::from(self.half_width_cm) / 100.0 + SCARP_RUPTURE_WANDER_METRES;
        let normal = Vec2::new(-tangent.y, tangent.x);
        let extent = tangent.abs() * half_length + normal.abs() * half_width;
        if origin.x.abs() > half.x + extent.x || origin.y.abs() > half.y + extent.y {
            return Err(TerrainRecipeError::OutsidePlayable);
        }
        Ok(())
    }

    pub fn transition_collar(self) -> TerrainTransitionCollar {
        self.transition_collar
    }
    fn validate_parameters(self) -> TerrainRecipeResult<()> {
        self.surface.validate()?;
        let tangent = Vec2::new(
            f32::from(self.tangent_permyriad[0]),
            f32::from(self.tangent_permyriad[1]),
        ) / 10_000.0;
        if !(0.98..=1.02).contains(&tangent.length()) {
            return Err(TerrainRecipeError::Tangent);
        }
        if !(100..=2_000).contains(&self.relief_cm)
            || !(400..=5_000).contains(&self.half_length_cm)
            || !(300..=2_000).contains(&self.half_width_cm)
            || self.collar_cm < 100
            || self.collar_cm >= self.half_length_cm
            || u32::from(self.collar_cm) * 2 >= u32::from(self.half_width_cm)
        {
            return Err(TerrainRecipeError::Dimensions);
        }
        Ok(())
    }
    pub fn from_quantized(wire: QuantizedLandformRecipe) -> TerrainRecipeResult<Self> {
        let origin = Vec2::new(wire.origin_cm[0] as f32, wire.origin_cm[1] as f32) / 100.0;
        let tangent = Vec2::new(
            f32::from(wire.tangent_permyriad[0]),
            f32::from(wire.tangent_permyriad[1]),
        ) / 10_000.0;
        let transition_collar =
            TerrainTransitionCollar::irregular_ellipse(crate::prelude::TerrainCollarParameters {
                origin: crate::scene_coordinates::ScenePlanPoint::try_from(origin)
                    .map_err(|_| TerrainRecipeError::Dimensions)?,
                tangent:
                    adventuresim_building_generator::spatial_geometry::PlanDirection::from_vector(
                        tangent,
                    )
                    .map_err(|_| TerrainRecipeError::Dimensions)?,
                half_length:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        f32::from(wire.half_length_cm) / 100.0,
                    )
                    .map_err(|_| TerrainRecipeError::Dimensions)?,
                half_width:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        f32::from(wire.half_width_cm) / 100.0,
                    )
                    .map_err(|_| TerrainRecipeError::Dimensions)?,
                width:
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        f32::from(wire.collar_cm) / 100.0,
                    )
                    .map_err(|_| TerrainRecipeError::Dimensions)?,
                seed: wire.seed,
                wander: crate::prelude::RuptureWander::from_metres(SCARP_RUPTURE_WANDER_METRES)
                    .ok_or(TerrainRecipeError::Dimensions)?,
                width_variation: crate::prelude::CollarWidthVariation::from_basis_points(
                    SCARP_WIDTH_VARIATION_BPS,
                )
                .ok_or(TerrainRecipeError::Dimensions)?,
            })
            .ok_or(TerrainRecipeError::Dimensions)?;
        let recipe = Self {
            kind: wire.kind,
            surface: wire.surface,
            seed: wire.seed,
            origin_cm: wire.origin_cm,
            tangent_permyriad: wire.tangent_permyriad,
            relief_cm: wire.relief_cm,
            half_length_cm: wire.half_length_cm,
            half_width_cm: wire.half_width_cm,
            collar_cm: wire.collar_cm,
            lod: wire.lod,
            transition_collar,
        };
        recipe.validate_parameters()?;
        Ok(recipe)
    }
}
impl TryFrom<QuantizedLandformRecipe> for TerrainLandformRecipe {
    type Error = TerrainRecipeError;
    fn try_from(wire: QuantizedLandformRecipe) -> TerrainRecipeResult<Self> {
        Self::from_quantized(wire)
    }
}
impl From<TerrainLandformRecipe> for QuantizedLandformRecipe {
    fn from(recipe: TerrainLandformRecipe) -> Self {
        Self {
            kind: recipe.kind,
            surface: recipe.surface,
            seed: recipe.seed,
            origin_cm: recipe.origin_cm,
            tangent_permyriad: recipe.tangent_permyriad,
            relief_cm: recipe.relief_cm,
            half_length_cm: recipe.half_length_cm,
            half_width_cm: recipe.half_width_cm,
            collar_cm: recipe.collar_cm,
            lod: recipe.lod,
        }
    }
}
impl TerrainLandformRecipe {
    pub fn kind(self) -> TerrainLandformKind {
        self.kind
    }
    pub fn surface(self) -> TerrainSurfaceRecipe {
        self.surface
    }
    pub fn seed(self) -> fabelgeist_determinism::Seed {
        self.seed
    }
    pub fn origin_cm(self) -> [i32; 2] {
        self.origin_cm
    }
    pub fn tangent_permyriad(self) -> [i16; 2] {
        self.tangent_permyriad
    }
    pub fn relief_cm(self) -> u16 {
        self.relief_cm
    }
    pub fn half_length_cm(self) -> u16 {
        self.half_length_cm
    }
    pub fn half_width_cm(self) -> u16 {
        self.half_width_cm
    }
    pub fn collar_cm(self) -> u16 {
        self.collar_cm
    }
    pub fn lod(self) -> TerrainLandformLod {
        self.lod
    }
}
