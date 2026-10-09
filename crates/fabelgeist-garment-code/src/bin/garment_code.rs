//! Generate a sewing pattern from body measurements and a design.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use fabelgeist_garment_code::pattern::Annotations;
use fabelgeist_garment_code::programs::MetaGarment;
use fabelgeist_garment_code::{Body, Design};

#[derive(Parser, Debug)]
#[command(
    name = "garment-code",
    about = "Generate a parametric sewing pattern (a GarmentCode port)"
)]
struct Args {
    /// Body measurements YAML (the file's `body:` section).
    #[arg(long)]
    body: PathBuf,

    /// Design parameters YAML (the file's `design:` section).
    #[arg(long)]
    design: PathBuf,

    /// Where to write the pattern.
    #[arg(long, default_value = "output")]
    out: PathBuf,

    /// Name for the generated pattern. Defaults to the design file's stem.
    #[arg(long)]
    name: Option<String>,

    /// Write straight into `<out>` instead of `<out>/<name><tag>/`.
    #[arg(long)]
    no_subfolder: bool,

    /// Suffix for the generated file names.
    #[arg(long, default_value = "")]
    tag: String,

    /// Skip the SVG renderings.
    #[arg(long)]
    no_svg: bool,

    /// Draw panel names and vertex/edge indices on the SVG.
    #[arg(long)]
    annotate: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let body = Body::load(&args.body)?;
    let design = Design::load(&args.design)?;

    let name = match &args.name {
        Some(n) => n.clone(),
        None => args
            .design
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "garment".to_string()),
    };

    let piece = MetaGarment::new(&name, &body, &design);
    let mut pattern = piece.assembly();

    if piece.is_self_intersecting() {
        eprintln!("{name} is self-intersecting");
    }

    let folder = pattern
        .serialize(&args.out, !args.no_subfolder, &args.tag, false)
        .context("writing the pattern specification")?;

    if !args.no_svg {
        let annotations = if args.annotate {
            Annotations {
                names: true,
                ids: true,
            }
        } else {
            Annotations {
                names: false,
                ids: false,
            }
        };
        pattern.save_svg(&folder, &args.tag, annotations, true)?;
    }

    body.save(&folder)?;
    std::fs::copy(
        &args.design,
        folder.join(
            args.design
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("design.yaml")),
        ),
    )
    .ok();

    println!("Success! {name} saved to {}", folder.display());
    Ok(())
}
