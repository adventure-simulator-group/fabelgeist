//! Export fitted skirt variants against an existing body review JSON.
//! This exercises the runtime fitter without requiring the offline body model.

use adventuresim_character_creator::{
    armor_recipes::ParametricDesign,
    device_body::DeviceBody,
    device_fit::{self, FitBody, Fitted, Realization},
};
use anyhow::Result;
use clap::Parser;
use fabelgeist_armor::{ArmorGpu, Permille, SkirtDesign, SkirtProfile};
use fabelgeist_rig::RigJointName;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Body review JSON containing native mesh and rig attributes.
    body: PathBuf,
    /// Output consumed by scripts/render_armor_review.py.
    output: PathBuf,
    /// Additional direct-fit body realizations sharing the reference topology.
    #[arg(long = "morph-body")]
    morph_bodies: Vec<PathBuf>,
    /// Join a minimal body-conforming upper section.
    #[arg(long)]
    dress: bool,
    /// Use one serialized SkirtDesign instead of the default three profiles.
    #[arg(long)]
    design: Option<PathBuf>,
}

/// Native attributes at the review-file boundary, in canonical metres.
#[derive(Deserialize)]
struct Body {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    texcoords: Vec<[f32; 2]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    joint_names: Vec<RigJointName>,
    joints: Vec<[f32; 8]>,
}

struct Variant {
    name: &'static str,
    profile: SkirtProfile,
}

impl Body {
    fn fit_body(&self) -> FitBody<'_> {
        FitBody {
            faces: &self.faces,
            texcoords: &self.texcoords,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
        }
    }

    fn realization<'a>(&'a self, device: &'a DeviceBody) -> Realization<'a> {
        Realization {
            positions: &self.positions,
            normals: &self.normals,
            joints: &self.joints,
            device,
        }
    }
}

fn variants() -> [Variant; 3] {
    [
        Variant {
            name: "straight_skirt",
            profile: SkirtProfile::Straight,
        },
        Variant {
            name: "flared_skirt",
            profile: SkirtProfile::Flared {
                width: Permille(1600),
                depth: Permille(1600),
            },
        },
        Variant {
            name: "bell_skirt",
            profile: SkirtProfile::Bell {
                width: Permille(2200),
                depth: Permille(1800),
                roundness: Permille(1000),
            },
        },
    ]
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes = std::fs::read(&args.body)?;
    let body: Body = serde_json::from_slice(&bytes)?;
    std::fs::create_dir_all(&args.output)?;
    std::fs::write(args.output.join("body.json"), &bytes)?;
    let gpu = ArmorGpu::open()?;
    let device = DeviceBody::default();
    let samples = args
        .morph_bodies
        .iter()
        .map(|path| Ok(serde_json::from_slice::<Body>(&std::fs::read(path)?)?))
        .collect::<Result<Vec<_>>>()?;
    let devices = samples
        .iter()
        .map(|_| DeviceBody::default())
        .collect::<Vec<_>>();
    let names = args
        .morph_bodies
        .iter()
        .map(|path| {
            path.file_stem()
                .and_then(|v| v.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid morph filename"))
        })
        .collect::<Result<Vec<_>>>()?;
    let morphs = samples
        .iter()
        .zip(&devices)
        .zip(&names)
        .map(|((body, device), name)| (*name, body.realization(device)))
        .collect::<Vec<_>>();
    let fit = body.fit_body();
    let wearer = body.realization(&device);
    let custom = args
        .design
        .as_ref()
        .map(|path| -> Result<SkirtDesign> { Ok(serde_json::from_slice(&std::fs::read(path)?)?) })
        .transpose()?;
    for Variant { name, profile } in
        variants()
            .into_iter()
            .take(if custom.is_some() { 1 } else { 3 })
    {
        let mut skirt = custom.clone().unwrap_or(SkirtDesign {
            profile,
            ..Default::default()
        });
        if args.dress {
            skirt.bodice = Some(Default::default());
        }
        let design = ParametricDesign::Skirt(skirt);
        eprintln!("Fitting {name}");
        let piece = device_fit::fit_recipe(&gpu, &fit, &wearer, &morphs, &design, "worn", &[])?;
        let generated = device_fit::assemble_recipe(
            &design,
            piece,
            &Fitted {
                placement: "worn",
                morphs: &names,
                domain: "review_body",
                joint_names: &body.joint_names,
                joints: &body.joints,
            },
        )?;
        let textile_maps = write_textile_maps(&args.output, &generated)?;
        std::fs::write(
            args.output.join(format!("{name}--worn.json")),
            serde_json::to_vec(&serde_json::json!({
                "id": name, "placement": "worn", "design": design,
                "positions": generated.positions, "normals": generated.normals,
                "texcoords": generated.texcoords, "textile_maps": textile_maps,
                "indices": generated.indices, "components": generated.components,
                "joint_indices": generated.joint_indices, "joint_weights": generated.joint_weights,
                "morphs": generated.morphs.iter().map(|morph| serde_json::json!({
                    "name": morph.name, "positions": morph.direct_positions,
                    "normal_deltas": morph.normal_deltas,
                })).collect::<Vec<_>>(),
                "generator_version": fabelgeist_armor::GENERATOR_VERSION,
            }))?,
        )?;
    }
    Ok(())
}

fn write_textile_maps(
    output: &std::path::Path,
    armor: &fabelgeist_armor::GeneratedArmor,
) -> Result<Vec<serde_json::Value>> {
    let mut maps = Vec::new();
    for (index, component) in armor.components.iter().enumerate() {
        if let Some(fabric) = &component.textile
            && let Some(textures) =
                adventuresim_character_creator::textile_surface::textures(fabric)?
        {
            let name = format!("textile-{index}-color.png");
            std::fs::write(output.join(&name), textures.base_color_png)?;
            maps.push(serde_json::json!({"component":index,"base_color":name}));
        }
    }
    Ok(maps)
}
