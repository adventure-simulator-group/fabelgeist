//! Freeze lower-layer support ownership on actual trimmed cloth triangles.
use super::domain::{CutDomain, PlaneSupport};
use super::{LayerConstraint, SupportScope, bvh, referenced_normals};
use crate::armor_frames::{FitRegion, Wearer};
use crate::armor_layer::ArmorLayerSurface;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use fabelgeist_bvh::TriangleBvh;
use std::collections::BTreeMap;

const BOUNDARY_SHARE_TOLERANCE: f32 = 0.0001;

/// Native canonical metre-space lower samples and frozen vertex ordinals.
struct InnerSample {
    vertex: usize,
    point: Vec3,
    outward: Vec3,
    normal: Vec3,
}

struct CarrierQueries {
    tree: TriangleBvh,
    edges: BTreeMap<[u32; 2], usize>,
}

impl CarrierQueries {
    fn new(domain: &CutDomain<'_>) -> Self {
        Self {
            tree: TriangleBvh::new(
                domain.positions.iter().copied().map(bvh).collect(),
                domain.faces.to_vec(),
            ),
            edges: edge_counts(domain.faces),
        }
    }

    fn collect(
        &self,
        domain: &CutDomain<'_>,
        samples: &[InnerSample],
        banks: &crate::cloth_panel::banks::ContactBanks,
        minimum_gap_metres: f32,
    ) -> Result<Vec<LayerConstraint>> {
        let positions = &domain.positions;
        let faces = domain.faces;
        let mut constraints = Vec::new();
        for sample in samples {
            let vertex = sample.vertex;
            let point = sample.point;
            let outward = sample.outward;
            let mut hits = Vec::new();
            for sign in [-1., 1.] {
                if let Some(hit) = super::support::oriented_hit(
                    &self.tree,
                    positions,
                    faces,
                    point,
                    outward,
                    sign,
                    super::support::SupportBank::OwnedAnatomical {
                        directions: &domain.directions,
                        source_faces: banks.sources(vertex),
                    },
                )? {
                    hits.push(hit);
                }
            }
            let Some(hit) = hits
                .into_iter()
                .min_by(|a, b| a.distance.total_cmp(&b.distance))
            else {
                continue;
            };
            let face = faces[hit.triangle];
            let [a, b, c] = face.map(|v| Vec3::from_array(positions[v as usize]));
            let normal = hit.normal;
            if normal.dot(sample.normal) <= 0. {
                continue;
            }
            let hit_direction = hit.ray_direction;
            let hit = hit.point;
            let weights = barycentric(a, b, c, hit);
            if on_open_border(face, weights, &self.edges) {
                continue;
            }
            let required_metres = minimum_gap_metres - (hit - point).dot(normal);
            ensure!(
                required_metres.is_finite() && weights.iter().all(|v| v.is_finite()),
                "invalid frozen layer constraint"
            );
            let weights = weights.map(|v| v.max(0.));
            let sum = weights.iter().sum::<f32>();
            let weights = weights.map(|v| v / sum);
            constraints.push(domain.lift(PlaneSupport {
                face,
                weights,
                normal,
                required_metres,
                query_direction: hit_direction,
                sample: point,
                hit,
            })?);
        }
        Ok(constraints)
    }
}

pub(super) fn collect(
    domain: &CutDomain<'_>,
    body: &Wearer<'_>,
    layers: &[ArmorLayerSurface<'_>],
    scope: SupportScope<'_>,
    layer_banks: &[crate::cloth_panel::banks::ContactBanks],
) -> Result<Vec<LayerConstraint>> {
    let faces = domain.faces;
    let queries = CarrierQueries::new(domain);
    let mut constraints = match scope {
        SupportScope::GarmentLayers => body_contacts(domain, body)?,
        SupportScope::RetainedAnatomy { .. } => Vec::new(),
    };
    for (layer_index, layer) in layers.iter().enumerate() {
        if let SupportScope::RetainedAnatomy { banks, .. } = scope {
            banks.validate(faces.len(), layer.faces.len(), layer.positions.len())?;
        }
        let banks = match scope {
            SupportScope::GarmentLayers => layer_banks
                .get(layer_index)
                .context("missing frozen garment correspondence")?,
            SupportScope::RetainedAnatomy { banks, .. } => banks,
        };
        banks.validate(faces.len(), layer.faces.len(), layer.positions.len())?;
        let torso_panel = match scope {
            SupportScope::GarmentLayers => torso_mask(layer),
            SupportScope::RetainedAnatomy { .. } => vec![true; layer.positions.len()],
        };
        constraints.extend(super::support::at_carrier_samples(
            domain,
            layer,
            &torso_panel,
            scope.rest_ease_metres(),
            Some(banks),
        )?);
        let torso_affinity =
            body.support_weights_for(FitRegion::Torso, layer.joint_indices, layer.joint_weights)?;
        constraints.extend(super::overlap::collect(
            domain,
            layer,
            &torso_panel,
            match scope {
                SupportScope::GarmentLayers => super::overlap::ContactScope::TorsoEnclosure {
                    weights: &torso_affinity,
                    banks,
                },
                SupportScope::RetainedAnatomy { banks, .. } => {
                    super::overlap::ContactScope::RetainedAnatomy {
                        minimum_gap_metres: scope.rest_ease_metres(),
                        banks,
                    }
                }
            },
        )?);
        constraints.extend(queries.collect(
            domain,
            &inner_samples(body, layer, scope, &torso_panel)?,
            banks,
            scope.rest_ease_metres(),
        )?);
    }
    Ok(constraints)
}

/// Seam ownership is frozen at the original attachment. Refreshing a nearby
/// sleeve search after each move could walk torso cloth down the whole arm.
/// Initial body clearance must remain a constraint during later layer seating.
/// Physical nearby contacts may involve other anatomical banks; unlike deep
/// enclosure ownership, these candidates follow the current finite geometry.
fn body_contacts(domain: &CutDomain<'_>, body: &Wearer<'_>) -> Result<Vec<LayerConstraint>> {
    let layer = ArmorLayerSurface::anatomy(body);
    let eligible = vec![true; body.positions.len()];
    let minimum_gap_metres =
        crate::cloth_panel::MINIMUM_CLOTH_CLEARANCE_M + super::CLEARANCE_TOLERANCE_M;
    let mut contacts =
        super::support::at_carrier_samples(domain, &layer, &eligible, minimum_gap_metres, None)?;
    contacts.extend(super::overlap::collect(
        domain,
        &layer,
        &eligible,
        super::overlap::ContactScope::BodyClearance { minimum_gap_metres },
    )?);
    Ok(contacts)
}

fn inner_samples(
    body: &Wearer<'_>,
    layer: &ArmorLayerSurface<'_>,
    scope: SupportScope<'_>,
    eligible: &[bool],
) -> Result<Vec<InnerSample>> {
    let layer_normals = referenced_normals(layer.positions, layer.faces)?;
    let layer_directions = layer_normals
        .iter()
        .map(|normal| normal.unwrap_or([0.; 3]))
        .collect::<Vec<_>>();
    let vertices = match scope {
        SupportScope::GarmentLayers => {
            body.support_indices_for(FitRegion::Torso, layer.joint_indices, layer.joint_weights)?
        }
        SupportScope::RetainedAnatomy { .. } => (0..layer.positions.len()).collect(),
    };

    Ok(vertices
        .into_iter()
        .filter(|&v| eligible[v])
        .filter_map(|vertex| {
            layer_normals[vertex].map(|normal| InnerSample {
                vertex,
                point: Vec3::from_array(layer.positions[vertex]),
                outward: Vec3::from_array(layer_directions[vertex]),
                normal: Vec3::from_array(normal),
            })
        })
        .collect())
}

fn edge_counts(faces: &[[u32; 3]]) -> BTreeMap<[u32; 2], usize> {
    let mut edges = BTreeMap::new();
    for face in faces {
        for i in 0..3 {
            let mut edge = [face[(i + 1) % 3], face[(i + 2) % 3]];
            edge.sort_unstable();
            *edges.entry(edge).or_default() += 1;
        }
    }
    edges
}

fn on_open_border(face: [u32; 3], weights: [f32; 3], edges: &BTreeMap<[u32; 2], usize>) -> bool {
    (0..3).any(|i| {
        let mut edge = [face[(i + 1) % 3], face[(i + 2) % 3]];
        edge.sort_unstable();
        weights[i] <= BOUNDARY_SHARE_TOLERANCE && edges.get(&edge) == Some(&1)
    })
}

pub(super) fn barycentric(a: Vec3, b: Vec3, c: Vec3, p: Vec3) -> [f32; 3] {
    // Cross-product barycentrics avoid cancellation of nearly equal Gram
    // products on the cutter's valid long, thin triangles. The query points
    // and source geometry stay f32; only this numerical boundary uses f64.
    let [a, b, c, p] = [a, b, c, p].map(|point| point.as_dvec3());
    let [ab, ac, ap] = [b - a, c - a, p - a];
    let normal = ab.cross(ac);
    let denominator = normal.length_squared();
    let u = ap.cross(ac).dot(normal) / denominator;
    let v = ab.cross(ap).dot(normal) / denominator;
    [(1. - u - v) as f32, u as f32, v as f32]
}

/// Bone affinity alone cannot distinguish a torso panel from a puffy sleeve.
/// Only body construction supplies torso enclosure; openings remain open.
fn torso_support_role(role: fabelgeist_armor::ArmorComponentRole) -> bool {
    use fabelgeist_armor::ArmorComponentRole;
    matches!(
        role,
        ArmorComponentRole::Plate
            | ArmorComponentRole::OuterFabric
            | ArmorComponentRole::BodiceFabric
            | ArmorComponentRole::Undercloth
            | ArmorComponentRole::Lining
            | ArmorComponentRole::Interlining
            | ArmorComponentRole::WaistSupportBand
            | ArmorComponentRole::WaistFrontBand
            | ArmorComponentRole::Boning
            | ArmorComponentRole::ClothTies
            | ArmorComponentRole::ClothFasteners
    )
}

/// Native component ranges establish semantic torso coverage before bone masks.
pub(super) fn torso_mask(layer: &ArmorLayerSurface<'_>) -> Vec<bool> {
    let mut torso_panel = vec![false; layer.positions.len()];
    for component in layer
        .components
        .iter()
        .filter(|component| torso_support_role(component.role))
    {
        for flag in &mut torso_panel[component.vertices.clone()] {
            *flag = true;
        }
    }
    torso_panel
}
