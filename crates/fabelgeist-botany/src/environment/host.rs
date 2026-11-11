//! The environment passes on the CPU.

use fabelgeist_math::{Vec3, Vec4};
use fabelgeist_spatial_hash::HashGrid;
use rayon::prelude::*;

use super::{
    EnvironmentSettings, OrientRequest, Oriented, ShadowGrid, SphereSet, crowding,
    distribution_cost, pattern,
};

#[derive(Default)]
pub struct HostEnvironment {
    grid: HashGrid,
}

impl HostEnvironment {
    fn build(&mut self, set: &SphereSet) {
        self.grid.rebuild(
            set.spheres.iter().map(|s| [s[0], s[1], s[2]]),
            set.cell_size(),
        );
    }

    pub fn light(
        &mut self,
        set: &SphereSet,
        shadow: Option<&ShadowGrid>,
        settings: &EnvironmentSettings,
    ) -> Vec<[f32; 2]> {
        self.build(set);
        let field = shadow.map(|grid| (grid, grid.propagate(&set.spheres)));
        let grid = &self.grid;
        (0..set.len() as u32)
            .into_par_iter()
            .map(|i| {
                let s = set.spheres[i as usize];
                let center = Vec3::new(s[0], s[1], s[2]);
                let f = crowding(set, i, set.parents[i as usize], center, s[3], |visit| {
                    grid.for_each_near([s[0], s[1], s[2]], 1, visit)
                });
                let q = (-settings.light_sensitivity * f).exp();
                let qg = field.as_ref().map_or(1.0, |(grid, shadow)| {
                    grid.light_at(shadow, [s[0], s[1], s[2]])
                });
                [q, qg]
            })
            .collect()
    }

    pub fn orient(
        &mut self,
        set: &SphereSet,
        requests: &[OrientRequest],
        settings: &EnvironmentSettings,
    ) -> Vec<Oriented> {
        self.build(set);
        let grid = &self.grid;
        let moves = pattern(settings.orientation_epsilon);
        requests
            .par_iter()
            .map(|request| {
                let solids = set.obstacles.as_ref().filter(|_| request.obstacles != 0);
                let crowding_at = |center: Vec3| {
                    crowding(
                        set,
                        request.index,
                        request.parent,
                        center,
                        request.radius,
                        |visit| grid.for_each_near(center.to_array(), 1, visit),
                    ) + solids.map_or(0.0, |o| o.crowding(center, request.radius))
                };
                let cost = |rotation: Vec4| distribution_cost(request, rotation, crowding_at);
                let r = request.rotation;
                let mut rotation = Vec4::new(r[0], r[1], r[2], r[3]);
                let mut best = cost(rotation);
                for _ in 0..settings.orientation_steps {
                    let mut improved = None;
                    for m in moves {
                        let candidate = rotation.mul_quat(m).normalize();
                        let c = cost(candidate);
                        if c < best {
                            best = c;
                            improved = Some(candidate);
                        }
                    }
                    match improved {
                        Some(candidate) => rotation = candidate,
                        None => break,
                    }
                }
                let center = Vec3::from_array(request.root)
                    + rotation.rotate_vec3(Vec3::from_array(request.local_center));
                Oriented {
                    rotation,
                    crowding: crowding_at(center),
                }
            })
            .collect()
    }
}
