//! Filesystem authoring boundary. Export runs entirely on the CPU.
use adventuresim_heraldry::{
    bake::{Baked, Resolution},
    document::Document,
    export, presets,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
#[derive(Parser)]
#[command(about = "Parametric heraldry recipes, maps, display objects and review captures")]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Clone, Copy, ValueEnum)]
enum Quality {
    Draft,
    Preview,
    High,
    Final,
}
impl From<Quality> for Resolution {
    fn from(q: Quality) -> Self {
        match q {
            Quality::Draft => Self::Draft,
            Quality::Preview => Self::Preview,
            Quality::High => Self::High,
            Quality::Final => Self::Final,
        }
    }
}
#[derive(Subcommand)]
enum Command {
    /// Solve a measured paint request; omit input to print an editable example.
    Mix { input: Option<PathBuf> },
    /// Write a complete editable reference recipe. Omit name to list presets.
    Preset {
        name: Option<String>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate structure and print non-blocking heraldic advice.
    Validate { input: PathBuf },
    /// Export JSON, SVG, flat PNG, PBR maps, mips and a closed GLB display support.
    Export {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "final")]
        quality: Quality,
    },
    /// Render the shared Bevy material scene, optionally reloading an exported GLB.
    Capture {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "high")]
        quality: Quality,
        #[arg(long)]
        reload_glb: Option<PathBuf>,
        /// Include the editor with the Or paint mixer open.
        #[arg(long, conflicts_with = "reload_glb")]
        paint_mixer: bool,
    },
    /// Render two recipes under the right-hand recipe's viewing conditions.
    Compare {
        left: PathBuf,
        right: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, value_enum, default_value = "high")]
        quality: Quality,
    },
}
fn read(path: &Path) -> Result<Document, Box<dyn std::error::Error>> {
    Ok(Document::from_json(&std::fs::read_to_string(path)?)?)
}
fn write(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Args::parse().command {
        Command::Mix { input } => {
            use adventuresim_heraldry::paint::mixing::{SearchRequest, provenance};
            if let Some(path) = input {
                let request: SearchRequest = serde_json::from_str(&std::fs::read_to_string(path)?)?;
                let result = request.solve()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({"request":request,"result":result,"calibration":provenance()})
                    )?
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&SearchRequest::default())?
                );
            }
        }
        Command::Preset { name: None, .. } => {
            for preset in presets::PRESETS {
                println!("{preset}");
            }
        }
        Command::Preset {
            name: Some(name),
            output,
        } => {
            let source = presets::preset(&name)?.to_json()?;
            if let Some(path) = output {
                write(&path, source.as_bytes())?;
            } else {
                println!("{source}");
            }
        }
        Command::Validate { input } => {
            let d = read(&input)?;
            println!("Valid: {}", d.name);
            for message in d.advice() {
                println!("Advice: {message}");
            }
        }
        Command::Export {
            input,
            output,
            quality,
        } => {
            let d = read(&input)?;
            let resolution = quality.into();
            let baked = Baked::generate(&d, resolution)?;
            for file in export::bundle(&d, &baked, resolution)? {
                write(&output.join(file.name), &file.bytes)?;
            }
            write(&output.join("material.bake"), &baked.to_bytes())?;
            println!(
                "Exported {} at {} px to {}",
                d.name,
                baked.size,
                output.display()
            );
        }
        Command::Capture {
            input,
            output,
            quality,
            reload_glb,
            paint_mixer,
        } => adventuresim_heraldry_studio::review::capture(
            read(&input)?,
            None,
            quality.into(),
            &output,
            reload_glb.as_deref(),
            paint_mixer.then_some(adventuresim_heraldry::document::Tincture::Or),
        )?,
        Command::Compare {
            left,
            right,
            output,
            quality,
        } => adventuresim_heraldry_studio::review::capture(
            read(&right)?,
            Some(read(&left)?),
            quality.into(),
            &output,
            None,
            None,
        )?,
    }
    Ok(())
}
