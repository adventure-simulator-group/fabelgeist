//! Indexed surfaces, renderer metadata, and solid mass integrals.
use super::*;
use serde::Serialize;
use std::collections::BTreeMap;

pub(crate) struct PartSource {
    pub(crate) animation_channel: Option<AnimationChannel>,
    pub(crate) smoothing_cosine: f64,
    pub(crate) solid: Solid,
    pub(crate) material: Material,
    pub(crate) label: String,
    pub(crate) component_id: String,
    pub(crate) animation_pivot: Option<Point>,
    pub(crate) bore: Option<BoreConnection>,
    pub(crate) shield_role: Option<ShieldRole>,
}
impl PartSource {
    pub(crate) fn new(solid: Solid, material: Material, label: &str, component_id: &str) -> Self {
        Self {
            animation_channel: None,
            smoothing_cosine: 0.25,
            solid,
            material,
            label: label.into(),
            component_id: component_id.into(),
            animation_pivot: None,
            bore: None,
            shield_role: None,
        }
    }
    pub(crate) fn animate(&mut self, pivot: Point, channel: AnimationChannel) {
        self.animation_pivot = Some(pivot);
        self.animation_channel = Some(channel);
    }
}

/// Mechanical animation ownership, independent of a part's display label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationChannel {
    BowString,
    CrossbowString,
    FirearmLock,
    PouchFlap,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ShieldRole {
    Body,
    Rim,
    Boss,
    Fitting,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoreConnection {
    pub touchhole_centerline: [Point; 2],
    pub bore_center: Point,
    pub bore_radius: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct MaterialAppearance {
    pub color: [f64; 3],
    pub density: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_channel: Option<AnimationChannel>,
    pub positions: Vec<f64>,
    pub normals: Vec<f64>,
    pub colors: Vec<f64>,
    pub indices: Vec<u32>,
    pub material: MaterialAppearance,
    pub label: String,
    pub component_id: String,
    pub material_id: Material,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield_role: Option<ShieldRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_pivot: Option<Point>,
    #[serde(flatten)]
    pub bore: Option<BoreConnection>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModelBounds {
    pub min: Point,
    pub max: Point,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStats {
    pub bounds: ModelBounds,
    pub dimensions: Point,
    pub radius: f64,
    pub triangles: usize,
    pub vertices: usize,
    pub volume: f64,
    pub part_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ResolvedRecipe {
    #[serde(flatten)]
    pub(crate) recipe: Recipe,
    #[serde(rename = "_frames")]
    pub(crate) frames: BTreeMap<String, Point>,
    #[serde(rename = "_resolutionErrors")]
    pub(crate) errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedModel {
    pub positions: Vec<f64>,
    pub normals: Vec<f64>,
    pub colors: Vec<f64>,
    pub indices: Vec<u32>,
    pub parts: Vec<ModelPart>,
    pub stats: ModelStats,
    pub physical: PhysicalProperties,
    pub(crate) resolved_definition: ResolvedRecipe,
}

impl ModelPart {
    pub(crate) fn from_source(source: PartSource) -> Self {
        #[derive(Eq, PartialEq, Ord, PartialOrd)]
        enum Surface {
            Smooth(u32),
            Flat([i64; 3]),
        }
        let mut lookup: BTreeMap<(Surface, [i64; 3]), Vec<usize>> = BTreeMap::new();
        let mut positions: Vec<Point> = Vec::new();
        let mut sums: Vec<Point> = Vec::new();
        let mut neighbors: Vec<Vec<Point>> = Vec::new();
        let mut indices = Vec::new();
        for (triangle, &[a, b, c]) in source.solid.faces.iter().enumerate() {
            let points = [
                source.solid.positions[a],
                source.solid.positions[b],
                source.solid.positions[c],
            ];
            let face = normalize(cross(sub(points[1], points[0]), sub(points[2], points[0])));
            for corner in 0..3 {
                let point = points[corner];
                let surface = if source.solid.surfaces[triangle] == 0 {
                    Surface::Flat(face.map(|v| (v * 1e8).round() as i64))
                } else {
                    Surface::Smooth(source.solid.surfaces[triangle])
                };
                let candidates = lookup
                    .entry((surface, point.map(|v| (v * 1e9).round() as i64)))
                    .or_default();
                let index = candidates
                    .iter()
                    .copied()
                    .find(|&i| {
                        neighbors[i]
                            .iter()
                            .all(|&neighbor| dot(neighbor, face) > source.smoothing_cosine)
                    })
                    .unwrap_or_else(|| {
                        let index = positions.len();
                        candidates.push(index);
                        positions.push(point);
                        sums.push([0.0; 3]);
                        neighbors.push(Vec::new());
                        index
                    });
                neighbors[index].push(face);
                let a = normalize(sub(points[(corner + 1) % 3], point));
                let b = normalize(sub(points[(corner + 2) % 3], point));
                let angle = dot(a, b).clamp(-1.0, 1.0).acos();
                sums[index] = add(sums[index], mul(face, angle));
                indices.push(index as u32);
            }
        }
        let colors = positions
            .iter()
            .flat_map(|_| source.material.color())
            .collect();
        Self {
            positions: positions.into_iter().flatten().collect(),
            normals: sums.into_iter().flat_map(normalize).collect(),
            colors,
            indices,
            material: MaterialAppearance {
                color: source.material.color(),
                density: source.material.density(),
            },
            material_id: source.material,
            label: source.label,
            component_id: source.component_id,
            animation_pivot: source.animation_pivot,
            animation_channel: source.animation_channel,
            bore: source.bore,
            shield_role: source.shield_role,
        }
    }
}

impl GeneratedModel {
    pub(crate) fn frames(&self) -> &BTreeMap<String, Point> {
        &self.resolved_definition.frames
    }

    pub(crate) fn from_sources(
        sources: Vec<PartSource>,
        resolved_definition: ResolvedRecipe,
        physical: PhysicalProperties,
    ) -> Self {
        let volume = sources.iter().map(|p| p.solid.volume()).sum();
        let parts: Vec<_> = sources.into_iter().map(ModelPart::from_source).collect();
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        for part in &parts {
            let base = positions.len() / 3;
            indices.extend(part.indices.iter().map(|&i| i + base as u32));
            positions.extend(&part.positions);
            normals.extend(&part.normals);
            colors.extend(&part.colors);
        }
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for point in positions.as_chunks::<3>().0 {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        let dimensions = sub(max, min);
        let stats = ModelStats {
            bounds: ModelBounds { min, max },
            dimensions,
            radius: magnitude(dimensions) / 2.0,
            triangles: indices.len() / 3,
            vertices: positions.len() / 3,
            volume,
            part_count: parts.len(),
        };
        Self {
            positions,
            normals,
            colors,
            indices,
            parts,
            stats,
            physical,
            resolved_definition,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalProperties {
    pub control_point: Point,
    pub mass_kg: f64,
    pub center_of_mass: Point,
    pub center_of_mass_from_grip_m: f64,
    pub moment_of_inertia_kg_m2: f64,
    pub balance: f64,
    pub grip_to_tip_m: f64,
    pub components: Vec<PartMass>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartMass {
    pub id: String,
    pub label: String,
    pub material: Material,
    pub mass_kg: f64,
    pub center_of_mass: Point,
}

impl PhysicalProperties {
    pub(crate) fn from_sources(parts: &[PartSource], control: Point) -> Self {
        let mut mass_kg = 0.0;
        let mut first = [0.0; 3];
        let mut inertia = 0.0;
        let mut tip = control[1];
        let mut components = Vec::new();
        for part in parts {
            let mut mass = 0.0;
            let mut moment = [0.0; 3];
            let mut transverse = 0.0;
            for &[a, b, c] in &part.solid.faces {
                let [a, b, c] = [
                    part.solid.positions[a],
                    part.solid.positions[b],
                    part.solid.positions[c],
                ];
                let tetra_mass = dot(a, cross(b, c)) / 6.0 * part.material.density();
                mass += tetra_mass;
                for axis in 0..3 {
                    let first = tetra_mass * (a[axis] + b[axis] + c[axis]) / 4.0;
                    moment[axis] += first;
                    let second = tetra_mass
                        * (a[axis] * a[axis]
                            + b[axis] * b[axis]
                            + c[axis] * c[axis]
                            + a[axis] * b[axis]
                            + a[axis] * c[axis]
                            + b[axis] * c[axis])
                        / 10.0;
                    let shifted = second - 2.0 * control[axis] * first
                        + control[axis] * control[axis] * tetra_mass;
                    transverse += shifted * if axis == 1 { 1.0 } else { 0.5 };
                }
            }
            mass_kg += mass;
            first = add(first, moment);
            inertia += transverse;
            tip = part
                .solid
                .positions
                .iter()
                .map(|p| p[1])
                .fold(tip, f64::max);
            components.push(PartMass {
                id: part.component_id.clone(),
                label: part.label.clone(),
                material: part.material,
                mass_kg: mass,
                center_of_mass: mul(moment, 1.0 / mass),
            });
        }
        let center_of_mass = mul(first, 1.0 / mass_kg);
        let grip_to_tip_m = (tip - control[1]).max(0.0);
        Self {
            mass_kg,
            control_point: control,
            center_of_mass,
            center_of_mass_from_grip_m: center_of_mass[1] - control[1],
            moment_of_inertia_kg_m2: inertia,
            balance: if grip_to_tip_m > 0.0 {
                (inertia / mass_kg).sqrt() / grip_to_tip_m
            } else {
                1.0
            },
            grip_to_tip_m,
            components,
        }
    }
}
