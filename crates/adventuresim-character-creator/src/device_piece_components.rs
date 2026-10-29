//! Material partitions copy shading attributes, never independently fit a seam.
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{ArmorComponent, ArmorComponentRole, BuiltPart, PlateFace};
use std::collections::{BTreeMap, BTreeSet};

use crate::device_fit::DevicePiece;

/// Native mesh remapping frozen from the base topology and reused for endpoints.
struct ComponentPartition {
    vertices: Vec<usize>,
    indices: Vec<u32>,
    faces: Vec<PlateFace>,
    components: Vec<ArmorComponent>,
}

struct MaterialGroup {
    component: ArmorComponent,
    triangles: Vec<usize>,
}

impl DevicePiece {
    /// Each cut fabric has its own along-cloth pattern domain. The original
    /// device chart remains the angular chart; no geometry is evaluated here.
    pub(crate) fn normalize_cloth_charts(&mut self, roles: &[ArmorComponentRole]) -> Result<()> {
        let part = &mut self.base;
        let coordinates = part
            .texcoords
            .as_mut()
            .context("cloth has no fabric chart")?;
        for component in &part.components {
            if !roles.contains(&component.role) {
                continue;
            }
            let points = &mut coordinates[component.vertices.clone()];
            let low = points.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
            let high = points
                .iter()
                .map(|uv| uv[1])
                .fold(f32::NEG_INFINITY, f32::max);
            anyhow::ensure!(high > low, "cloth has no along-cloth extent");
            for uv in points {
                uv[1] = (uv[1] - low) / (high - low);
            }
        }
        Ok(())
    }

    pub(crate) fn append(&mut self, other: Self) -> Result<()> {
        ensure!(
            self.endpoints.len() == other.endpoints.len(),
            "assembled garments have different morph counts"
        );
        let first = self.base.positions.len();
        self.base.append(other.base);
        self.skin.append(other.skin)?;
        self.skin_aliases
            .extend(other.skin_aliases.into_iter().map(|i| i + first));
        for (part, next) in self.endpoints.iter_mut().zip(other.endpoints) {
            part.append(next);
        }
        Ok(())
    }

    /// Partition only the first shell's material. Shared seam coordinates,
    /// normals, skin weights and endpoint positions are copied exactly.
    pub(crate) fn partition_first_component(
        &mut self,
        classify: impl Fn(&[u32; 3], PlateFace) -> ArmorComponentRole,
    ) -> Result<()> {
        let partition = ComponentPartition::new(&self.base, &self.skin_aliases, classify)?;
        let mut remap = BTreeMap::new();
        for (new, &old) in partition.vertices.iter().enumerate() {
            remap.entry(old).or_insert(new);
        }
        let aliases = partition
            .vertices
            .iter()
            .map(|&old| {
                remap
                    .get(&self.skin_aliases[old])
                    .copied()
                    .context("material partition lost a physical seam source")
            })
            .collect::<Result<Vec<_>>>()?;
        self.skin.texcoords = partition
            .vertices
            .iter()
            .map(|&v| self.skin.texcoords[v])
            .collect();
        self.skin.joint_indices = self
            .skin
            .joint_indices
            .select(partition.vertices.iter().copied());
        self.skin.joint_weights = partition
            .vertices
            .iter()
            .map(|&v| self.skin.joint_weights[v])
            .collect();
        self.skin_aliases = aliases;
        partition.apply(&mut self.base);
        for endpoint in &mut self.endpoints {
            partition.apply(endpoint);
        }
        Ok(())
    }
}

impl ComponentPartition {
    fn new(
        part: &BuiltPart,
        aliases: &[usize],
        classify: impl Fn(&[u32; 3], PlateFace) -> ArmorComponentRole,
    ) -> Result<Self> {
        ensure!(
            part.grids.is_empty(),
            "material partition requires a triangle cloth surface"
        );
        let first = part.components.first().context("cloth has no component")?;
        let mut groups: Vec<MaterialGroup> = Vec::new();
        for (triangle, indices) in part.indices[first.indices.clone()]
            .as_chunks::<3>()
            .0
            .iter()
            .enumerate()
        {
            let role = classify(indices, part.faces[first.indices.start / 3 + triangle]);
            let group =
                if let Some(index) = groups.iter().position(|group| group.component.role == role) {
                    index
                } else {
                    let mut component = first.clone();
                    component.role = role;
                    groups.push(MaterialGroup {
                        component,
                        triangles: Vec::new(),
                    });
                    groups.len() - 1
                };
            groups[group]
                .triangles
                .push(first.indices.start / 3 + triangle);
        }
        for component in part.components.iter().skip(1) {
            groups.push(MaterialGroup {
                component: component.clone(),
                triangles: (component.indices.start / 3..component.indices.end / 3).collect(),
            });
        }
        let mut result = Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            faces: Vec::new(),
            components: Vec::new(),
        };
        for MaterialGroup {
            mut component,
            triangles,
        } in groups
        {
            let mut used = triangles
                .iter()
                .flat_map(|&i| part.indices[i * 3..i * 3 + 3].iter().map(|&v| v as usize))
                .collect::<BTreeSet<_>>();
            // Keep canonical physical owners even when a buried outer face
            // is omitted; surviving inner vertices still inherit that owner.
            let owners = used
                .iter()
                .map(|&vertex| aliases[vertex])
                .collect::<Vec<_>>();
            used.extend(owners);
            let first = result.vertices.len();
            let remap = used
                .iter()
                .enumerate()
                .map(|(i, &v)| (v, (first + i) as u32))
                .collect::<BTreeMap<_, _>>();
            result.vertices.extend(used);
            let start = result.indices.len();
            for triangle in triangles {
                result.indices.extend(
                    part.indices[triangle * 3..triangle * 3 + 3]
                        .iter()
                        .map(|&v| remap[&(v as usize)]),
                );
                result.faces.push(part.faces[triangle]);
            }
            component.vertices = first..result.vertices.len();
            component.indices = start..result.indices.len();
            result.components.push(component);
        }
        Ok(result)
    }

    fn apply(&self, part: &mut BuiltPart) {
        part.positions = self.vertices.iter().map(|&v| part.positions[v]).collect();
        part.normals = self.vertices.iter().map(|&v| part.normals[v]).collect();
        if let Some(texcoords) = &mut part.texcoords {
            *texcoords = self.vertices.iter().map(|&v| texcoords[v]).collect();
        }
        part.indices.clone_from(&self.indices);
        part.faces.clone_from(&self.faces);
        part.components.clone_from(&self.components);
    }
}
