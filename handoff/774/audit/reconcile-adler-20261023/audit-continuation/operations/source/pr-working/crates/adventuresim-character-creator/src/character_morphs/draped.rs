//! Transfer the body's ordered morph channels through fixed surface samples.
use super::*;
use adventuresim_character_creator::garment::DrapedGarment;
use fabelgeist_math::Vec3;

impl CharacterMorphs {
    pub(crate) fn draped(
        &self,
        body: &GeneratedCharacter,
        faces: &[[u32; 3]],
        garments: &[DrapedGarment],
    ) -> Result<Vec<Vec<MorphDelta>>> {
        let tree = fabelgeist_bvh::TriangleBvh::new(
            body.positions
                .iter()
                .copied()
                .map(Vec3::from_array)
                .collect(),
            faces.to_vec(),
        );
        garments
            .iter()
            .map(|garment| {
                let samples = garment
                    .positions
                    .iter()
                    .map(|&p| {
                        let (face, point, _) = tree
                            .closest_point(Vec3::from_array(p), f32::MAX)
                            .context("garment morph has no source surface")?;
                        let face = faces[face as usize];
                        let [a, b, c] = face.map(|i| Vec3::from_array(body.positions[i as usize]));
                        let (u, v, q) = (b - a, c - a, point - a);
                        let determinant = u.dot(u) * v.dot(v) - u.dot(v).powi(2);
                        anyhow::ensure!(determinant > 0.0, "degenerate garment morph source");
                        let y = (v.dot(v) * q.dot(u) - u.dot(v) * q.dot(v)) / determinant;
                        let z = (u.dot(u) * q.dot(v) - u.dot(v) * q.dot(u)) / determinant;
                        Ok((face, [1.0 - y - z, y, z]))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(self
                    .body
                    .iter()
                    .map(|target| {
                        let transfer = |values: &[[f32; 3]]| {
                            samples
                                .iter()
                                .map(|(face, bary)| {
                                    std::array::from_fn(|axis| {
                                        face.iter()
                                            .zip(bary)
                                            .map(|(&i, &w)| values[i as usize][axis] * w)
                                            .sum()
                                    })
                                })
                                .collect()
                        };
                        let positions: Vec<[f32; 3]> = transfer(&target.positions);
                        let deformed: Vec<_> = garment
                            .positions
                            .iter()
                            .zip(&positions)
                            .map(|(base, delta)| {
                                std::array::from_fn(|axis| base[axis] + delta[axis])
                            })
                            .collect();
                        let normals = garment
                            .normals_for(&deformed)
                            .iter()
                            .zip(&garment.normals)
                            .map(|(normal, base)| {
                                std::array::from_fn(|axis| normal[axis] - base[axis])
                            })
                            .collect();
                        MorphDelta {
                            name: target.name.clone(),
                            positions,
                            normals,
                        }
                    })
                    .collect())
            })
            .collect()
    }
}
