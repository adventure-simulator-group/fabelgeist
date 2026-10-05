//! Projection passes for vertex/triangle and edge/edge contacts.
use super::*;
use fabelgeist_xpbd::{
    ConstraintGradient, EffectiveInverseMass, MassValidity, ProjectionActivity, ProjectionDepth,
    RelativeNormalSpeed,
};

struct ContactState<'a> {
    positions: &'a mut [Vec3],
    previous: &'a [Vec3],
    inverse_masses: &'a [ParticleInverseMass],
    velocities: Option<&'a mut [Vec3]>,
    thickness: f32,
}
impl SurfaceContacts {
    pub(super) fn solve_inner(
        &self,
        positions: &mut [Vec3],
        previous: &[Vec3],
        inverse_masses: &[ParticleInverseMass],
        thickness: f32,
        iterations: u32,
        velocities: Option<&mut [Vec3]>,
    ) -> usize {
        assert_eq!(positions.len(), self.seam_copies.len());
        assert_eq!(positions.len(), previous.len());
        assert_eq!(positions.len(), inverse_masses.len());
        for mass in inverse_masses {
            assert_eq!(mass.validity(), MassValidity::FiniteNonnegative);
        }
        assert!(positions.iter().chain(previous).all(|p| p.is_finite()));
        if !thickness.is_finite() || thickness <= 0.0 {
            return 0;
        }
        let mut state = ContactState {
            positions,
            previous,
            inverse_masses,
            velocities,
            thickness,
        };
        let mut contacts = 0;
        for _ in 0..iterations {
            contacts += self.vertex_contacts(&mut state);
            contacts += self.edge_contacts(&mut state);
        }
        contacts
    }

    fn vertex_contacts(&self, state: &mut ContactState<'_>) -> usize {
        let thickness = state.thickness;
        let search_radius = thickness.max(self.static_clearance);
        let dynamic_count = state.positions.len() - self.static_positions.len();
        let positions = &mut *state.positions;
        let previous = state.previous;
        let inverse_masses = state.inverse_masses;
        let velocities = &mut state.velocities;
        let mut contacts = 0;
        let face_count = self
            .fixed_bounds
            .as_ref()
            .map_or(self.faces.len(), |b| b.face_offset);
        let bounds: Vec<_> = self.faces[..face_count]
            .iter()
            .map(|face| {
                Aabb::from_points(
                    face.iter()
                        .flat_map(|&i| [positions[i as usize], previous[i as usize]]),
                )
                .expand(search_radius)
            })
            .collect();
        let tree = Bvh::build(&bounds);
        // Fixed vertices only need to query dynamic faces, avoiding
        // expensive body-against-body searches on a dense animated mesh.
        let dynamic_faces: Vec<_> = self.faces[..face_count]
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                f.iter()
                    .any(|&v| inverse_masses[v as usize].mobility() == ParticleMobility::Dynamic)
                    .then_some(i)
            })
            .collect();
        let dynamic_bounds: Vec<_> = dynamic_faces.iter().map(|&f| bounds[f]).collect();
        let dynamic_tree = Bvh::build(&dynamic_bounds);
        let mut candidates = Vec::new();
        for v in 0..dynamic_count {
            candidates.clear();
            let query = Aabb::from_points([positions[v], previous[v]]).expand(search_radius);
            if inverse_masses[v].mobility() == ParticleMobility::Dynamic {
                tree.query_aabb(&bounds, &query, |f| candidates.push(f as usize));
                if let Some(fixed) = &self.fixed_bounds {
                    fixed.faces.query(&query.expand(search_radius), |f| {
                        candidates.push(f + fixed.face_offset)
                    });
                }
            } else {
                dynamic_tree.query_aabb(&dynamic_bounds, &query, |f| {
                    candidates.push(dynamic_faces[f as usize])
                });
            }
            for &f in &candidates {
                let face = self.faces[f];
                if face
                    .iter()
                    .any(|&i| i as usize == v || self.seam_copies[v].contains(&i))
                {
                    continue;
                }
                contacts += resolve(
                    Pair::VertexTriangle,
                    positions,
                    previous,
                    inverse_masses,
                    [v, face[0] as usize, face[1] as usize, face[2] as usize],
                    if v >= dynamic_count || face.iter().any(|&i| i as usize >= dynamic_count) {
                        self.static_clearance
                    } else {
                        thickness
                    },
                    velocities.as_deref_mut(),
                );
            }
        }
        if let Some(fixed) = &self.fixed_bounds {
            // Obstacle vertices matter only near cloth faces. Visit them from
            // the swept face bounds instead of querying every body vertex.
            let mut pairs = Vec::new();
            for (&f, face_bounds) in dynamic_faces.iter().zip(&dynamic_bounds) {
                fixed
                    .vertices
                    .query(&face_bounds.expand(search_radius), |i| {
                        pairs.push((i + dynamic_count, f))
                    });
            }
            for (v, f) in pairs {
                let face = self.faces[f];
                contacts += resolve(
                    Pair::VertexTriangle,
                    positions,
                    previous,
                    inverse_masses,
                    [v, face[0] as usize, face[1] as usize, face[2] as usize],
                    self.static_clearance,
                    velocities.as_deref_mut(),
                );
            }
        }
        contacts
    }

    fn edge_contacts(&self, state: &mut ContactState<'_>) -> usize {
        let thickness = state.thickness;
        let search_radius = thickness.max(self.static_clearance);
        let dynamic_count = state.positions.len() - self.static_positions.len();
        let positions = &mut *state.positions;
        let previous = state.previous;
        let inverse_masses = state.inverse_masses;
        let velocities = &mut state.velocities;
        let mut contacts = 0;
        let mut candidates = Vec::new();
        let edge_count = self
            .fixed_bounds
            .as_ref()
            .map_or(self.edges.len(), |b| b.edge_offset);
        let bounds: Vec<_> = self.edges[..edge_count]
            .iter()
            .map(|edge| {
                Aabb::from_points(
                    edge.iter()
                        .flat_map(|&i| [positions[i as usize], previous[i as usize]]),
                )
                .expand(search_radius)
            })
            .collect();
        let tree = Bvh::build(&bounds);
        for (i, edge) in self.edges[..edge_count].iter().enumerate() {
            if edge
                .iter()
                .all(|&v| inverse_masses[v as usize].mobility() == ParticleMobility::Prescribed)
            {
                continue;
            }
            candidates.clear();
            tree.query_aabb(&bounds, &bounds[i], |j| {
                let j = j as usize;
                if j > i
                    || self.edges[j].iter().all(|&v| {
                        inverse_masses[v as usize].mobility() == ParticleMobility::Prescribed
                    })
                {
                    candidates.push(j);
                }
            });
            if let Some(fixed) = &self.fixed_bounds {
                fixed.edges.query(&bounds[i].expand(search_radius), |j| {
                    candidates.push(j + fixed.edge_offset)
                });
            }
            for &j in &candidates {
                let other = self.edges[j];
                if edge.iter().any(|a| {
                    other
                        .iter()
                        .any(|b| a == b || self.seam_copies[*a as usize].contains(b))
                }) {
                    continue;
                }
                contacts += resolve(
                    Pair::EdgeEdge,
                    positions,
                    previous,
                    inverse_masses,
                    [
                        edge[0] as usize,
                        edge[1] as usize,
                        other[0] as usize,
                        other[1] as usize,
                    ],
                    if edge
                        .iter()
                        .chain(other.iter())
                        .any(|&i| i as usize >= dynamic_count)
                    {
                        self.static_clearance
                    } else {
                        thickness
                    },
                    velocities.as_deref_mut(),
                );
            }
        }
        contacts
    }
}

fn resolve(
    pair: Pair,
    positions: &mut [Vec3],
    previous: &[Vec3],
    masses: &[ParticleInverseMass],
    ids: [usize; 4],
    thickness: f32,
    velocities: Option<&mut [Vec3]>,
) -> usize {
    let start = ids.map(|i| previous[i]);
    let end = ids.map(|i| positions[i]);
    // The CCD guard is inside the resting contact shell. This lets touching
    // cloth slide tangentially without returning time zero on every step.
    let contact = ccd::sweep(pair, start, end, thickness * 0.5)
        .unwrap_or_else(|| ccd::proximity(pair, end, start));
    debug_assert!((0.0..=1.0).contains(&contact.time));
    let separation = ids
        .iter()
        .zip(contact.weights)
        .fold(
            Vec3::default(),
            |sum: Vec3, (&i, w): (&usize, ConstraintGradient)| -> Vec3 {
                sum + w.contribution(positions[i])
            },
        )
        .dot(contact.normal);
    let depth = ProjectionDepth::from(thickness - separation);
    if depth.activity() == ProjectionActivity::Inactive {
        return 0;
    }
    let denominator: EffectiveInverseMass = ids
        .iter()
        .zip(contact.weights)
        .map(
            |(&i, w): (&usize, ConstraintGradient)| -> EffectiveInverseMass {
                w.inverse_response(masses[i])
            },
        )
        .sum();
    if denominator.contact_activity() == ProjectionActivity::Inactive {
        return 0;
    }
    if let Some(velocities) = velocities {
        let relative = RelativeNormalSpeed::from(
            ids.iter()
                .zip(contact.weights)
                .fold(
                    Vec3::default(),
                    |sum: Vec3, (&i, w): (&usize, ConstraintGradient)| -> Vec3 {
                        sum + w.contribution(velocities[i])
                    },
                )
                .dot(contact.normal),
        );
        if relative.activity() == ProjectionActivity::Active {
            for (i, w) in ids.into_iter().zip(contact.weights) {
                velocities[i] -= relative
                    .contact_correction(w, masses[i], denominator)
                    .along(contact.normal);
            }
        }
    }
    for (i, w) in ids.into_iter().zip(contact.weights) {
        positions[i] += depth
            .contact_correction(w, masses[i], denominator)
            .along(contact.normal);
    }
    1
}

#[cfg(test)]
mod tests;
