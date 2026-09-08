//! Interior-certified posterior seating without a query-direction fallback.

use crate::TorsoShoulderSample;
use crate::breastplate_crest_curve::{dot, normalized};
use crate::breastplate_seated_band::ShoulderSeat;
use crate::breastplate_shoulder_band::{BandError, CrestSample, ShoulderBand};
use serde::Serialize;

const RAY_DETERMINANT_EPSILON: f64 = 1e-10;
const BARYCENTRIC_TOLERANCE: f64 = 1e-5;
const WINDING_TOLERANCE: f64 = 1e-6;
const MIN_FRONT_NORMAL: f64 = 0.15;
const MIN_SUPERIOR_NORMAL: f64 = 0.45;

pub(crate) fn hash_enclosure_geometry(
    hash: &mut blake3::Hasher,
    vertices: &[TorsoShoulderSample],
    faces: &[[u32; 3]],
) {
    hash.update(b"full-body-enclosure-domain-v1");
    hash.update(&(vertices.len() as u64).to_le_bytes());
    for vertex in vertices {
        for value in vertex.position.into_iter().chain(vertex.normal) {
            hash.update(&value.to_bits().to_le_bytes());
        }
    }
    hash.update(&(faces.len() as u64).to_le_bytes());
    for index in faces.iter().flatten() {
        hash.update(&index.to_le_bytes());
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regional_patch_is_not_an_interior_domain_but_full_enclosure_is() {
        let (vertices, faces) = octahedron();
        let make = |domain_faces| {
            SeatedCrestQueries::new(
                &vertices,
                domain_faces,
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0],
                ShoulderSeat::new(0.125).unwrap(),
                0.01,
            )
        };
        let mut regional = make(&faces[..4]);
        assert!(matches!(
            regional.query([0.1, 0.0, 0.4], 0, 0.3),
            Err(BandError::InvalidSeed)
        ));
        assert!((regional.evidence[0].original_winding.abs() - 1.0).abs() > WINDING_TOLERANCE);
        let mut enclosure = make(&faces);
        assert!(enclosure.query([0.1, 0.0, 0.4], 0, 0.3).is_ok());
    }

    #[test]
    fn morph_pairs_keep_cropped_support_and_enclosure_in_separate_index_domains() {
        use crate::{ClearancePoseError, TorsoClearanceMesh, TorsoClearancePose};
        let (vertices, faces) = octahedron();
        let positions = vertices.iter().map(|v| v.position).collect::<Vec<_>>();
        let normals = vertices.iter().map(|v| v.normal).collect::<Vec<_>>();
        let crop = [0, 2, 4];
        let base = TorsoClearancePose::from_full_body(&positions, &normals, &crop).unwrap();
        let mut morphed = positions.clone();
        morphed[5][2] *= 1.4;
        let morph = TorsoClearancePose::from_full_body(&morphed, &normals, &crop).unwrap();
        assert_eq!(base.vertices, morph.vertices);
        assert_ne!(base.enclosure_vertices, morph.enclosure_vertices);
        for (local, source) in crop.into_iter().enumerate() {
            assert_eq!(base.vertices[local], base.enclosure_vertices[source]);
            assert_eq!(morph.vertices[local], morph.enclosure_vertices[source]);
        }
        let mut domains = TorsoClearanceMesh {
            base,
            faces: vec![[0, 1, 2]],
            enclosure_faces: faces,
            morphs: vec![morph],
        };
        assert!(domains.has_corresponding_domains(1));
        let query = |pose: &TorsoClearancePose| {
            SeatedCrestQueries::new(
                &pose.enclosure_vertices,
                &domains.enclosure_faces,
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0],
                ShoulderSeat::new(0.125).unwrap(),
                0.01,
            )
            .query([0.1, 0.0, 0.4], 0, 0.3)
            .unwrap()
        };
        assert_ne!(
            query(&domains.base).position,
            query(&domains.morphs[0]).position
        );
        domains.morphs[0].enclosure_vertices.pop();
        assert!(!domains.has_corresponding_domains(1));
        assert!(matches!(
            TorsoClearancePose::from_full_body(&positions, &normals[..3], &crop),
            Err(ClearancePoseError::MismatchedArrays)
        ));
        assert!(matches!(
            TorsoClearancePose::from_full_body(&positions, &normals, &[99]),
            Err(ClearancePoseError::InvalidCroppedIndex)
        ));
    }

    #[test]
    fn enclosure_cache_covers_posterior_geometry_normals_and_face_domain() {
        let (vertices, faces) = octahedron();
        let hash = |v: &[TorsoShoulderSample], f: &[[u32; 3]]| {
            let mut h = blake3::Hasher::new();
            hash_enclosure_geometry(&mut h, v, f);
            h.finalize()
        };
        let original = hash(&vertices, &faces);
        assert_eq!(original, hash(&vertices, &faces));
        let mut changed = vertices.clone();
        changed[5].position[2] -= 0.2;
        assert_ne!(original, hash(&changed, &faces));
        changed = vertices.clone();
        changed[5].normal[0] += 0.1;
        assert_ne!(original, hash(&changed, &faces));
        assert_ne!(original, hash(&vertices, &faces[..faces.len() - 1]));
        let mut reversed = faces.clone();
        reversed[0].swap(1, 2);
        assert_ne!(original, hash(&vertices, &reversed));
    }

    fn octahedron() -> (Vec<TorsoShoulderSample>, Vec<[u32; 3]>) {
        let vertices = vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ]
        .into_iter()
        .map(|p| TorsoShoulderSample {
            position: p,
            normal: p,
        })
        .collect::<Vec<_>>();
        let mut faces = Vec::new();
        for x in [0, 1] {
            for y in [2, 3] {
                for z in [4, 5] {
                    let mut face = [x, y, z];
                    let [a, b, c] = face.map(|i| vertices[i as usize].position.map(f64::from));
                    if dot(cross(sub(b, a), sub(c, a)), a) < 0.0 {
                        face.swap(1, 2);
                    }
                    faces.push(face);
                }
            }
        }
        (vertices, faces)
    }

    #[test]
    fn seated_seed_uses_unpadded_first_exit_interval_and_one_output_padding() {
        let (vertices, faces) = octahedron();
        let mut queries = SeatedCrestQueries::new(
            &vertices,
            &faces,
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            ShoulderSeat::new(0.125).unwrap(),
            0.01,
        );
        let target = queries.query([0.1, 0.0, 0.4], 0, 0.3).unwrap();
        let evidence = &queries.evidence[0];
        let posterior = evidence.posterior_first_geometric_hit.as_ref().unwrap();
        assert!((posterior.raw_position_world[2] + 0.9).abs() < 1e-7);
        assert!((evidence.shifted_seed_world.unwrap()[2] - 0.2375).abs() < 1e-7);
        assert!((evidence.original_winding - 1.0).abs() < 1e-12);
        assert!((evidence.shifted_winding.unwrap() - 1.0).abs() < 1e-12);
        let superior = evidence.superior_first_geometric_hit.as_ref().unwrap();
        let offset = sub(target.position, superior.raw_position_world);
        assert!((dot(offset, offset).sqrt() - 0.01).abs() < 1e-8);
        assert!(posterior.is_outward() && superior.is_outward());
    }

    #[test]
    fn outside_origin_cannot_skip_entry_and_select_a_later_outward_exit() {
        let (vertices, faces) = octahedron();
        let mut queries = SeatedCrestQueries::new(
            &vertices,
            &faces,
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            ShoulderSeat::new(0.125).unwrap(),
            0.01,
        );
        let first = queries
            .first_crossing([0.1, 0.0, 2.0], [0.0, 0.0, -1.0])
            .unwrap();
        assert!(!first.is_outward());
        assert!(first.raw_position_world[2] > 0.0);
        assert!(matches!(
            queries.query([0.1, 0.0, 2.0], 0, 0.3),
            Err(BandError::InvalidSeed)
        ));
        assert!(queries.evidence[0].posterior_first_geometric_hit.is_none());
        assert!(queries.evidence[0].original_winding.abs() < 1e-12);
    }

    #[test]
    fn rejected_pure_superior_normal_does_not_try_an_anterior_direction() {
        let (vertices, faces) = octahedron();
        let mut queries = SeatedCrestQueries::new(
            &vertices,
            &faces,
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            ShoulderSeat::new(0.125).unwrap(),
            0.01,
        );
        assert!(matches!(
            queries.query([0.1, 0.0, 0.1], 0, 0.3),
            Err(BandError::InvalidNormal)
        ));
        assert_eq!(queries.evidence.len(), 1);
        assert!(queries.evidence[0].superior_first_geometric_hit.is_some());
        assert!(queries.evidence[0].padded_target.is_none());
    }
}

#[derive(Clone, Debug, Serialize)]
struct RayHit {
    triangle: usize,
    distance_m: f64,
    raw_position_world: [f64; 3],
    normal_world: [f64; 3],
    geometric_outward_dot: f64,
    interpolated_outward_dot: f64,
}

impl RayHit {
    fn is_outward(&self) -> bool {
        self.geometric_outward_dot > 0.0 && self.interpolated_outward_dot > 0.0
    }
}

#[derive(Debug, Serialize)]
struct QueryEvidence {
    side: usize,
    seed_fraction: f32,
    original_seed_world: [f64; 3],
    shifted_seed_world: Option<[f64; 3]>,
    original_winding: f64,
    shifted_winding: Option<f64>,
    posterior_first_geometric_hit: Option<RayHit>,
    superior_first_geometric_hit: Option<RayHit>,
    posterior_fraction: ShoulderSeat,
    padding_m: f64,
    padded_target: Option<CrestSample>,
    failure: Option<String>,
}

pub(crate) struct SeatedCrestQueries<'a> {
    vertices: &'a [TorsoShoulderSample],
    faces: &'a [[u32; 3]],
    front: [f64; 3],
    superior: [f64; 3],
    seat: ShoulderSeat,
    padding_m: f64,
    evidence: Vec<QueryEvidence>,
}

impl<'a> SeatedCrestQueries<'a> {
    pub(crate) fn new(
        vertices: &'a [TorsoShoulderSample],
        faces: &'a [[u32; 3]],
        front: [f32; 3],
        superior: [f32; 3],
        seat: ShoulderSeat,
        padding_m: f32,
    ) -> Self {
        Self {
            vertices,
            faces,
            front: front.map(f64::from),
            superior: superior.map(f64::from),
            seat,
            padding_m: padding_m as f64,
            evidence: Vec::new(),
        }
    }

    fn winding(&self, origin: [f64; 3]) -> f64 {
        self.faces
            .iter()
            .map(|face| {
                let [a, b, c] =
                    face.map(|i| sub(self.vertices[i as usize].position.map(f64::from), origin));
                let [la, lb, lc] = [a, b, c].map(|v| dot(v, v).sqrt());
                let numerator = dot(a, cross(b, c));
                let denominator = la * lb * lc + dot(a, b) * lc + dot(b, c) * la + dot(c, a) * lb;
                2.0 * numerator.atan2(denominator)
            })
            .sum::<f64>()
            / (4.0 * std::f64::consts::PI)
    }

    fn first_crossing(&self, origin: [f64; 3], direction: [f64; 3]) -> Option<RayHit> {
        let mut nearest: Option<RayHit> = None;
        for (triangle, face) in self.faces.iter().enumerate() {
            let vertices = face.map(|i| self.vertices[i as usize]);
            let [a, b, c] = vertices.map(|v| v.position.map(f64::from));
            let e1 = sub(b, a);
            let e2 = sub(c, a);
            let p = cross(direction, e2);
            let determinant = dot(e1, p);
            if determinant.abs() < RAY_DETERMINANT_EPSILON {
                continue;
            }
            let t = sub(origin, a);
            let u = dot(t, p) / determinant;
            let q = cross(t, e1);
            let v = dot(direction, q) / determinant;
            let distance = dot(e2, q) / determinant;
            if u < -BARYCENTRIC_TOLERANCE
                || v < -BARYCENTRIC_TOLERANCE
                || u + v > 1.0 + BARYCENTRIC_TOLERANCE
                || distance <= 0.0
            {
                continue;
            }
            if nearest
                .as_ref()
                .is_some_and(|hit| hit.distance_m <= distance)
            {
                continue;
            }
            let weights = [1.0 - u - v, u, v];
            let normal = normalized(std::array::from_fn(|axis| {
                (0..3)
                    .map(|i| vertices[i].normal[axis] as f64 * weights[i])
                    .sum()
            }))
            .ok()?;
            nearest = Some(RayHit {
                triangle,
                distance_m: distance,
                raw_position_world: std::array::from_fn(|i| origin[i] + direction[i] * distance),
                normal_world: normal,
                geometric_outward_dot: dot(cross(e1, e2), direction),
                interpolated_outward_dot: dot(normal, direction),
            });
        }
        nearest
    }

    pub(crate) fn query(
        &mut self,
        seed: [f32; 3],
        side: usize,
        seed_fraction: f32,
    ) -> Result<CrestSample, BandError> {
        let seed = seed.map(f64::from);
        let mut evidence = QueryEvidence {
            side,
            seed_fraction,
            original_seed_world: seed,
            shifted_seed_world: None,
            original_winding: self.winding(seed),
            shifted_winding: None,
            posterior_first_geometric_hit: None,
            superior_first_geometric_hit: None,
            posterior_fraction: self.seat,
            padding_m: self.padding_m,
            padded_target: None,
            failure: None,
        };
        let result = self.certified_query(&mut evidence);
        match &result {
            Ok(sample) => evidence.padded_target = Some(*sample),
            Err(error) => evidence.failure = Some(error.to_string()),
        }
        self.evidence.push(evidence);
        result
    }

    fn certified_query(&self, evidence: &mut QueryEvidence) -> Result<CrestSample, BandError> {
        let inside = |w: f64| w.is_finite() && (w.abs() - 1.0).abs() <= WINDING_TOLERANCE;
        if !inside(evidence.original_winding) {
            return Err(BandError::InvalidSeed);
        }
        evidence.posterior_first_geometric_hit =
            self.first_crossing(evidence.original_seed_world, self.front.map(|v| -v));
        let posterior = evidence
            .posterior_first_geometric_hit
            .as_ref()
            .ok_or(BandError::InvalidSeed)?;
        if !posterior.is_outward() {
            return Err(BandError::InvalidSeed);
        }
        // This open segment precedes the FIRST geometric exit; seating cannot
        // leave and re-enter a nonconvex component unnoticed.
        let shifted = std::array::from_fn(|i| {
            evidence.original_seed_world[i]
                + self.seat.fraction()
                    * (posterior.raw_position_world[i] - evidence.original_seed_world[i])
        });
        evidence.shifted_seed_world = Some(shifted);
        let winding = self.winding(shifted);
        evidence.shifted_winding = Some(winding);
        if !inside(winding) {
            return Err(BandError::InvalidSeed);
        }
        evidence.superior_first_geometric_hit = self.first_crossing(shifted, self.superior);
        let superior = evidence
            .superior_first_geometric_hit
            .as_ref()
            .ok_or(BandError::InvalidSeed)?;
        if !superior.is_outward() {
            return Err(BandError::InvalidSeed);
        }
        if dot(superior.normal_world, self.front) < MIN_FRONT_NORMAL
            || dot(superior.normal_world, self.superior) < MIN_SUPERIOR_NORMAL
        {
            return Err(BandError::InvalidNormal);
        }
        Ok(CrestSample {
            position: std::array::from_fn(|i| {
                superior.raw_position_world[i] + self.padding_m * superior.normal_world[i]
            }),
            normal: superior.normal_world,
        })
    }

    pub(crate) fn write_dump(&self) -> Result<(), BandError> {
        ShoulderBand::write_dump(
            "shoulder-seat-queries",
            &serde_json::json!({
                "domain": "full-body-enclosure; independent of cropped clearance/support surface",
                "domain_vertices": self.vertices.len(), "domain_faces": self.faces.len(),
                "space": "world-y-up metres", "front": self.front, "superior": self.superior,
                "minimum_front_normal": MIN_FRONT_NORMAL, "minimum_superior_normal": MIN_SUPERIOR_NORMAL,
                "winding_tolerance": WINDING_TOLERANCE, "queries": self.evidence,
            }),
        )
    }
}
