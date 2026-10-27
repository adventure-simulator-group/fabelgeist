use super::*;
mod urban;
pub use urban::{UrbanGroundLookup, UrbanGroundSurfaces};

/// Physical material below a tactical ground-cover layer.
///
/// This is authoritative semantic data rather than a render-material index so
/// server gameplay can query it without depending on client asset catalogs.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum GroundSubstrate {
    #[default]
    Soil,
    Stone,
    Gravel,
    Mud,
    Road,
    Water,
}

/// Mutually exclusive ground-cover profile at one tactical surface sample.
///
/// A profile can render several compatible details (for example leaves and
/// twigs), but a location never simultaneously claims two cover profiles.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum GroundCover {
    Bare,
    #[default]
    TallGrass,
    LeafLitter,
    LooseStone,
    Reeds,
}

/// Compact server-queryable description of one ground sample.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct GroundSurface {
    pub substrate: GroundSubstrate,
    pub cover: GroundCover,
    pub cover_density_bps: u16,
    pub cover_height_cm: u16,
}

/// Authoritative spatial ground semantics shared by the tactical server and
/// clients. Samples use the same centered row-major grid convention as
/// [`SceneTerrain`], allowing presentation and future gameplay systems to ask
/// the same world-position question.
#[derive(Component, Serialize, Deserialize, Debug, Reflect, Clone, PartialEq)]
#[component(immutable)]
pub struct SceneGround {
    samples: Vec<GroundSurface>,
    pub urban: UrbanGroundSurfaces,
    width: usize,
    scale: f32,
}

impl SceneGround {
    pub fn uniform_for_terrain(terrain: &SceneTerrain, surface: GroundSurface) -> Self {
        Self {
            samples: vec![surface; terrain.grid_width() * terrain.grid_depth()],
            width: terrain.grid_width(),
            scale: terrain.grid_scale(),
            urban: UrbanGroundSurfaces::default(),
        }
    }

    pub fn from_samples(
        grid_width: usize,
        grid_depth: usize,
        scale: f32,
        samples: Vec<GroundSurface>,
    ) -> Option<Self> {
        if grid_width < 2
            || grid_depth < 2
            || !scale.is_finite()
            || scale <= 0.0
            || samples.len() != grid_width.checked_mul(grid_depth)?
            || samples.iter().any(|sample| {
                sample.cover_density_bps > BASIS_POINTS_PER_WHOLE || sample.cover_height_cm > 1_000
            })
        {
            return None;
        }
        Some(Self {
            samples,
            urban: UrbanGroundSurfaces::default(),
            width: grid_width,
            scale,
        })
    }

    pub fn grid_width(&self) -> usize {
        self.width
    }

    pub fn grid_depth(&self) -> usize {
        self.samples.len() / self.width
    }

    pub fn width(&self) -> f32 {
        self.grid_width().saturating_sub(1) as f32 * self.scale
    }

    pub fn depth(&self) -> f32 {
        self.grid_depth().saturating_sub(1) as f32 * self.scale
    }

    pub fn grid_scale(&self) -> f32 {
        self.scale
    }

    pub fn samples(&self) -> &[GroundSurface] {
        &self.samples
    }

    /// Returns exact urban ownership, then the natural sample at a world point.
    /// Enum values are intentionally not interpolated across boundaries.
    pub fn ground_at(&self, pos: Vec2) -> Option<GroundSurface> {
        let grid = (pos + Vec2::new(self.width(), self.depth()) * 0.5) / self.scale;
        if grid.x < 0.0
            || grid.y < 0.0
            || grid.x > (self.grid_width() - 1) as f32
            || grid.y > (self.grid_depth() - 1) as f32
        {
            return None;
        }
        let x = (grid.x.floor() as usize).min(self.grid_width() - 1);
        let z = (grid.y.floor() as usize).min(self.grid_depth() - 1);
        self.urban
            .surface_at(pos)
            .or_else(|| self.samples.get(z * self.width + x).copied())
    }

    /// Conservatively checks a square scatter footprint. Macro grass patches
    /// use this to avoid extending blades across a leaf-litter boundary.
    pub fn cover_intersects_square(
        &self,
        centre: Vec2,
        half_extent: f32,
        cover: GroundCover,
    ) -> bool {
        if !half_extent.is_finite() || half_extent < 0.0 {
            return false;
        }
        let half_size = Vec2::new(self.width(), self.depth()) * 0.5;
        let minimum = ((centre - Vec2::splat(half_extent) + half_size) / self.scale)
            .floor()
            .max(Vec2::ZERO);
        let maximum = ((centre + Vec2::splat(half_extent) + half_size) / self.scale)
            .ceil()
            .min(Vec2::new(
                (self.grid_width() - 1) as f32,
                (self.grid_depth() - 1) as f32,
            ));
        if minimum.x > maximum.x || minimum.y > maximum.y {
            return false;
        }
        for z in minimum.y as usize..=maximum.y as usize {
            for x in minimum.x as usize..=maximum.x as usize {
                if self.samples[z * self.width + x].cover == cover {
                    return true;
                }
            }
        }
        false
    }

    pub fn cover_count(&self, cover: GroundCover) -> usize {
        self.samples
            .iter()
            .filter(|sample| sample.cover == cover)
            .count()
    }
}
