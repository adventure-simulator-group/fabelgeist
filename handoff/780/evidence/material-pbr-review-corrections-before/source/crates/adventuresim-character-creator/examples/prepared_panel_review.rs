//! Inspect a frozen coarse anatomical carrier on independent unposed bodies.
use adventuresim_character_creator::surface_cut::{Plane, SurfaceCut, interpolate};
use adventuresim_character_creator::{
    armor_frames::Wearer,
    cloth_panel::{PanelPlan, PanelSurface},
    device_body::DeviceBody,
};
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::{GownDesign, PartFrame};
use fabelgeist_rig::RigJointName;
use std::path::{Path, PathBuf};
#[path = "textile_review/body.rs"]
mod body;

fn main() -> Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let body_path = PathBuf::from(arguments.next().context("expected body JSON")?);
    let design_path = PathBuf::from(arguments.next().context("expected gown design JSON")?);
    let output = PathBuf::from(arguments.next().context("expected output directory")?);
    let bytes = std::fs::read(&body_path)?;
    let body: body::Body = serde_json::from_slice(&bytes)?;
    let design: GownDesign = serde_json::from_slice(&std::fs::read(design_path)?)?;
    let device = DeviceBody::default();
    // Review inputs use the canonical unposed body axes; only axes enter cuts.
    let head = PartFrame {
        origin: [0.; 3],
        axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        half_extents: [1.; 3],
    };
    let (plan, surface) = PanelPlan::new(
        &design.bodice,
        &wearer(&body, &device),
        &head,
        design.sleeves.as_ref(),
    )?;
    let root = RigJointName::ROOT.require_in(&body.joint_names)?;
    let proof = SurfaceCut::new(
        &surface.positions,
        &surface.faces,
        &surface.faces,
        &[vec![Plane {
            normal: [0., -1., 0.],
            offset: -body.joints[root][1],
        }]],
        &[],
    );
    write_case(&output, &bytes, &surface, &proof)?;
    let Some(directory) = arguments.next() else {
        return Ok(());
    };
    let mut paths = std::fs::read_dir(directory)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|p| p.extension().is_some_and(|ext| ext == "json"));
    paths.sort();
    let mut cases = Vec::new();
    for path in paths {
        let name = path.file_stem().context("body has no file stem")?;
        let bytes = std::fs::read(&path)?;
        let body: body::Body = serde_json::from_slice(&bytes)?;
        let target = plan
            .realize(&wearer(&body, &device))
            .with_context(|| format!("refitting {}", path.display()))?;
        ensure!(
            target.faces == surface.faces && target.positions.len() == surface.positions.len(),
            "carrier refit changed correspondence"
        );
        let case = output.join(name);
        write_case(&case, &bytes, &target, &proof)?;
        cases.push(serde_json::json!({"body":path,"output":case,"vertices":target.positions.len(),"triangles":target.faces.len()}));
        println!("refit {}", path.display());
    }
    std::fs::write(
        output.join("refits.json"),
        serde_json::to_vec_pretty(&cases)?,
    )?;
    Ok(())
}

/// Native review-file boundary, canonical unposed metres and body index space.
fn wearer<'a>(body: &'a body::Body, device: &'a DeviceBody) -> Wearer<'a> {
    let realization = body.realization(device);
    let fit = body.fit_body();
    Wearer {
        positions: realization.positions,
        normals: realization.normals,
        faces: fit.faces,
        joint_indices: fit.joint_indices,
        joint_weights: fit.joint_weights,
        joint_names: fit.joint_names,
        joints: realization.joints,
    }
}

fn write_case(
    output: &Path,
    bytes: &[u8],
    surface: &PanelSurface,
    proof: &SurfaceCut,
) -> Result<()> {
    std::fs::create_dir_all(output)?;
    std::fs::write(output.join("body.json"), bytes)?;
    let indices = surface.faces.iter().flatten().copied().collect::<Vec<_>>();
    std::fs::write(
        output.join("prepared-source.json"),
        serde_json::to_vec(&serde_json::json!({
            "positions":surface.positions,"normals":surface.normals,"indices":indices,
            "cap_seams":[],"plate_faces":vec!["outer";surface.faces.len()],
            "components":[{"role":"bodice_fabric","vertices":{"start":0,"end":surface.positions.len()},
            "indices":{"start":0,"end":indices.len()},"material":{"base_color":[0.3,0.5,0.7,1.],
            "roughness":0.8,"metallic":0.},"textile":null}],
            "stage":"trimmed anatomical panel source; no physical shell or finishing sheets",
        }))?,
    )?;
    let positions = proof
        .points
        .iter()
        .map(|point| {
            interpolate(
                surface.faces[point.triangle].map(|v| surface.positions[v as usize]),
                point.weights,
            )
        })
        .collect::<Vec<_>>();
    let normals = proof
        .points
        .iter()
        .map(|point| {
            bevy::math::Vec3::from_array(interpolate(
                surface.faces[point.triangle].map(|v| surface.normals[v as usize]),
                point.weights,
            ))
            .normalize()
            .to_array()
        })
        .collect::<Vec<_>>();
    let indices = proof.faces.iter().flatten().copied().collect::<Vec<_>>();
    std::fs::write(
        output.join("legal-hem-source.json"),
        serde_json::to_vec(&serde_json::json!({
            "positions":positions,"normals":normals,"indices":indices,
            "plate_faces":vec!["outer";proof.faces.len()],
            "components":[{"role":"bodice_fabric","vertices":{"start":0,"end":positions.len()},
            "indices":{"start":0,"end":indices.len()},"material":{"base_color":[0.3,0.5,0.7,1.],"roughness":0.8,"metallic":0.},"textile":null}],
            "stage":"canonical source trimmed once at minimum legal garment hem; frozen proof topology, not finished clothing",
        }))?,
    )?;
    Ok(())
}
