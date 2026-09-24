//! Underlayers cut from the body and fitted on the device, assembled into
//! armor with their own surface attributes and morph targets.

use anyhow::Result;
use fabelgeist_armor::{
    ArmorComponent, ArmorComponentRole, ArmorGpu, ArmorMorph, GeneratedArmor, Millimeters,
    Permille, TrunkHoseDesign,
};

use crate::armor_frames::Wearer;
use crate::device_fit::deltas;
use crate::device_underlayer::{self, BodyShape, SurfaceDomain};
use crate::underlayer::{UnderlayerDesign, UnderlayerKind};

/// The body an underlayer is cut from, and where its surface coordinates are.
#[derive(Clone, Copy)]
pub struct UnderlayerBody<'a> {
    pub wearer: &'a Wearer<'a>,
    pub domain: SurfaceDomain<'a>,
    /// The name of the surface coordinates' domain.
    pub surface_domain: &'a str,
    /// Static body shapes the fit must also clear, beyond the morphs.
    pub proportions: &'a [BodyShape<'a>],
}

/// Cut an underlayer from the wearer and fit it to every morph, named.
pub fn fit_underlayer(
    gpu: &ArmorGpu,
    design: &UnderlayerDesign,
    placement: &str,
    body: &UnderlayerBody<'_>,
    morphs: &[(&str, BodyShape<'_>)],
) -> Result<GeneratedArmor> {
    let shapes = morphs.iter().map(|(_, shape)| *shape).collect::<Vec<_>>();
    let fitted = device_underlayer::fit(
        gpu,
        design,
        placement,
        body.wearer,
        body.domain,
        body.proportions,
        &shapes,
    )?;
    let base = fitted.base;
    let (vertex_count, index_count) = (base.positions.len(), fitted.indices.len());
    let targets = morphs
        .iter()
        .zip(fitted.endpoints)
        .map(|((name, _), endpoint)| ArmorMorph {
            name: (*name).to_owned(),
            position_deltas: deltas(&base.positions, &endpoint.positions),
            normal_deltas: deltas(&base.normals, &endpoint.normals),
            direct_positions: endpoint.positions,
        })
        .collect();
    Ok(GeneratedArmor {
        design_hash: fabelgeist_armor::parametric_design_hash(&serde_json::to_vec(design)?),
        surface_domain: body.surface_domain.into(),
        positions: base.positions,
        normals: base.normals,
        texcoords: fitted.texcoords,
        joint_indices: fitted.joint_indices,
        joint_weights: fitted.joint_weights,
        indices: fitted.indices,
        // Cut from the body: a quilted layer, not a thickened plate.
        faces: Vec::new(),
        trim: None,
        grids: Vec::new(),
        morphs: targets,
        components: fabric_component(design, vertex_count, index_count),
    })
}

/// Cloth is dyed its design's colour; mail keeps its own surface maps.
fn fabric_component(
    design: &UnderlayerDesign,
    vertex_count: usize,
    index_count: usize,
) -> Vec<ArmorComponent> {
    if design.kind.is_mail() {
        return Vec::new();
    }
    vec![ArmorComponent {
        role: ArmorComponentRole::OuterFabric,
        vertices: 0..vertex_count,
        indices: 0..index_count,
        hinge: None,
        material: Some(design.color.material()),
    }]
}

/// Fit trunk hose as a quilted layer over the hips and alternate its
/// vertical panes between the two fabrics.
pub fn fit_trunk_hose(
    gpu: &ArmorGpu,
    design: &TrunkHoseDesign,
    placement: &str,
    body: &UnderlayerBody<'_>,
    morphs: &[(&str, BodyShape<'_>)],
) -> Result<GeneratedArmor> {
    design.validate().map_err(anyhow::Error::new)?;
    let carrier = UnderlayerDesign {
        kind: UnderlayerKind::MailBrayette,
        clearance: design.clearance,
        thickness: design.thickness,
        color: design.primary_color,
        length: design.length,
        sleeve_length: Permille(1_000),
        patch_width: Millimeters(80),
        cuts: Vec::new(),
    };
    let mut armor = fit_underlayer(gpu, &carrier, placement, body, morphs)?;
    armor.design_hash = fabelgeist_armor::parametric_design_hash(&serde_json::to_vec(design)?);
    apply_trunk_hose_panes(&mut armor, design);
    Ok(armor)
}

/// Sort the triangles into panes by their angle around the hips, even panes
/// first, and give each run its fabric.
fn apply_trunk_hose_panes(armor: &mut GeneratedArmor, design: &TrunkHoseDesign) {
    let count = armor.positions.len() as f32;
    let center = [
        armor.positions.iter().map(|point| point[0]).sum::<f32>() / count,
        armor.positions.iter().map(|point| point[2]).sum::<f32>() / count,
    ];
    let repeat = std::f32::consts::TAU / f32::from(design.panel_count);
    let mut primary = Vec::new();
    let mut secondary = Vec::new();
    for face in armor.indices.as_chunks::<3>().0 {
        let centroid = face.iter().fold([0.0; 2], |mut centroid, index| {
            let point = armor.positions[*index as usize];
            centroid[0] += point[0] / 3.0;
            centroid[1] += point[2] / 3.0;
            centroid
        });
        let angle = (centroid[0] - center[0])
            .atan2(centroid[1] - center[1])
            .rem_euclid(std::f32::consts::TAU);
        let pane = ((angle / repeat + 0.5).floor() as u8) % design.panel_count;
        if pane.is_multiple_of(2) {
            primary.extend_from_slice(face);
        } else {
            secondary.extend_from_slice(face);
        }
    }
    let split = primary.len();
    primary.append(&mut secondary);
    armor.indices = primary;
    armor.components = [
        (0..split, design.primary_color),
        (split..armor.indices.len(), design.secondary_color),
    ]
    .into_iter()
    .map(|(indices, color)| ArmorComponent {
        role: ArmorComponentRole::OuterFabric,
        vertices: 0..armor.positions.len(),
        indices,
        hinge: None,
        material: Some(color.material()),
    })
    .collect();
}
