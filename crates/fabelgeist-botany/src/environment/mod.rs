//! The environment passes: everything in a step that couples modules to each
//! other rather than to their own plant.
//!
//! * **Light** (Sec. 5.2.1 and 6.2): every module's local light exposure
//!   `Q = exp(-f_collisions)` from the overlap of its bounding sphere with its
//!   neighbours', and the global shadow `Q_G` cast on it from above.
//! * **Orientation** (Sec. 5.2.3, App. A.1): a few steps of pattern search
//!   over small rotations of each new module, minimising
//!   `w1 f_collisions + w2 f_tropism`.
//!
//! Solids ([`crate::obstacles`]) count as occupied space in both: in a
//! module's crowding as the cap of its sphere past the nearest surface, in
//! the shadow as cells they fill.
//!
//! Both are embarrassingly parallel over modules and both query a spatial
//! hash of all module spheres, so both exist twice: [`host`] on the CPU with
//! rayon, and (with the `gpu` feature) [`gpu`] as compute kernels. They
//! implement the same arithmetic and are tested against each other.

pub mod host;
pub mod shadow;

#[cfg(feature = "gpu")]
pub mod gpu;
#[cfg(feature = "gpu")]
mod wgsl;

use std::sync::Arc;

use fabelgeist_math::{Vec3, Vec4};
use fabelgeist_spatial_hash::{sphere_overlap_volume, sphere_volume};
use rayon::prelude::*;

use crate::obstacles::Obstacles;

pub use shadow::ShadowGrid;

/// Every module sphere in the scene, flattened, and the solids among them.
#[derive(Clone, Debug, Default)]
pub struct SphereSet {
    /// Centre and radius.
    pub spheres: Vec<[f32; 4]>,
    /// Global index of each module's parent module, or [`NONE`](crate::prototype::NONE).
    pub parents: Vec<u32>,
    /// Buildings: space no module may take.
    pub obstacles: Option<Arc<Obstacles>>,
}

impl SphereSet {
    pub fn len(&self) -> usize {
        self.spheres.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spheres.is_empty()
    }

    /// A grid cell size that keeps every overlapping pair within the 27-cell
    /// neighbourhood: twice the largest radius.
    pub fn cell_size(&self) -> f32 {
        2.0 * self.spheres.iter().map(|s| s[3]).fold(0.05f32, f32::max)
    }
}

/// A new module whose orientation is to be optimised.
///
/// Five `vec4`s, which is how the GPU kernel reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OrientRequest {
    /// Where the module attaches.
    pub root: [f32; 3],
    /// Radius of the module once mature.
    pub radius: f32,
    /// Centre of the mature module's sphere, module-local.
    pub local_center: [f32; 3],
    /// This module's index in the [`SphereSet`].
    pub index: u32,
    /// Starting orientation.
    pub rotation: [f32; 4],
    /// The module's growth axis, module-local.
    pub axis: [f32; 3],
    /// `cos(alpha_tropism)`.
    pub cos_target: f32,
    /// `w1`.
    pub collision_weight: f32,
    /// `w2`.
    pub tropism_weight: f32,
    /// Parent's index in the [`SphereSet`], or [`NONE`](crate::prototype::NONE).
    pub parent: u32,
    /// One when the module can reach a solid of [`SphereSet::obstacles`],
    /// which then joins its crowding; zero spares it the solids.
    pub obstacles: u32,
}

/// Where the orientation search left a new module.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oriented {
    pub rotation: Vec4,
    /// `f_collisions` there: how much of the module's mature sphere lies
    /// inside other plants' and branches' spheres, and inside solids.
    pub crowding: f32,
}

/// Knobs of the environment passes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentSettings {
    /// Scales `f_collisions` before `Q = exp(-f)`. The overlap is measured as
    /// a fraction of the module's own sphere, so one means "a module wholly
    /// inside others' spheres gets `e^-1` of the light".
    pub light_sensitivity: f32,
    /// Pattern-search steps per new module (the paper finds three enough).
    pub orientation_steps: u32,
    /// The pattern's rotation, radians.
    pub orientation_epsilon: f32,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            light_sensitivity: 1.0,
            orientation_steps: 4,
            orientation_epsilon: 0.12,
        }
    }
}

/// `sum_w V(B_u, B_w) / V(B_u)` over the neighbours `visit` reports,
/// skipping `index` itself, its parent, its children and its siblings: their
/// overlap is the prototype's own arrangement, not competition for space.
pub(crate) fn crowding(
    set: &SphereSet,
    index: u32,
    parent: u32,
    center: Vec3,
    radius: f32,
    neighbours: impl FnOnce(&mut dyn FnMut(u32)),
) -> f32 {
    let mut sum = 0.0;
    neighbours(&mut |j| {
        let pj = set.parents[j as usize];
        if j == index || j == parent || pj == index || (parent != u32::MAX && pj == parent) {
            return;
        }
        let s = set.spheres[j as usize];
        let d = (Vec3::new(s[0], s[1], s[2]) - center).length();
        sum += sphere_overlap_volume(radius, s[3], d);
    });
    sum / sphere_volume(radius).max(1e-9)
}

/// Eq. 3 for one candidate orientation.
pub(crate) fn distribution_cost(
    request: &OrientRequest,
    rotation: Vec4,
    crowding_at: impl FnOnce(Vec3) -> f32,
) -> f32 {
    let root = Vec3::from_array(request.root);
    let center = root + rotation.rotate_vec3(Vec3::from_array(request.local_center));
    let axis = rotation.rotate_vec3(Vec3::from_array(request.axis));
    // Eq. 4: distance of the axis' angle from the preferred one, in cosines.
    let tropism = (request.cos_target - axis.y).abs();
    request.collision_weight * crowding_at(center) + request.tropism_weight * tropism
}

/// The pattern of App. A.1: `+-epsilon` about the module's local x and z.
pub(crate) fn pattern(epsilon: f32) -> [Vec4; 4] {
    let x = Vec3::new(1.0, 0.0, 0.0);
    let z = Vec3::new(0.0, 0.0, 1.0);
    [
        Vec4::from_axis_angle(x, epsilon),
        Vec4::from_axis_angle(x, -epsilon),
        Vec4::from_axis_angle(z, epsilon),
        Vec4::from_axis_angle(z, -epsilon),
    ]
}

/// Where the environment passes run.
pub enum Environment {
    Host(host::HostEnvironment),
    #[cfg(feature = "gpu")]
    Gpu(Box<gpu::GpuEnvironment>),
}

impl Environment {
    /// `[Q, Q_G]` per module.
    pub async fn light(
        &mut self,
        set: &SphereSet,
        shadow: Option<&ShadowGrid>,
        settings: &EnvironmentSettings,
    ) -> anyhow::Result<Vec<[f32; 2]>> {
        let mut light = match self {
            Environment::Host(host) => host.light(set, shadow, settings),
            #[cfg(feature = "gpu")]
            Environment::Gpu(gpu) => gpu.light(set, shadow, settings).await?,
        };
        // Masonry crowds a module as other modules do: Q = exp(-s (f_modules
        // + f_solids)), the second factor here.
        if let Some(obstacles) = &set.obstacles {
            light.par_iter_mut().zip(&set.spheres).for_each(|(l, s)| {
                let f = obstacles.crowding(Vec3::new(s[0], s[1], s[2]), s[3]);
                if f > 0.0 {
                    l[0] *= (-settings.light_sensitivity * f).exp();
                }
            });
        }
        Ok(light)
    }

    /// The optimised rotation of each request.
    pub async fn orient(
        &mut self,
        set: &SphereSet,
        requests: &[OrientRequest],
        settings: &EnvironmentSettings,
    ) -> anyhow::Result<Vec<Oriented>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        match self {
            Environment::Host(host) => Ok(host.orient(set, requests, settings)),
            #[cfg(feature = "gpu")]
            Environment::Gpu(gpu) => gpu.orient(set, requests, settings).await,
        }
    }

    pub fn is_gpu(&self) -> bool {
        !matches!(self, Environment::Host(_))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Host(host::HostEnvironment::default())
    }
}
