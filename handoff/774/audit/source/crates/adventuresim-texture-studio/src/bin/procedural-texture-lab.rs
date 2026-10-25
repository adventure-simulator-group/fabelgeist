//! Headless review and export using exactly the same documents and renderer as Texture Studio.
use adventuresim_procedural_textures::{
    BakedRecipe, PROCEDURAL_TEXTURE_CATALOGUE, TextureRecipeId,
};
use adventuresim_texture_studio::{document::Document, review};
use clap::{Parser, Subcommand};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    List,
    Export {
        recipe: String,
        #[arg(long)]
        preset: Option<PathBuf>,
        #[arg(long, default_value = "target/procedural-texture-lab")]
        output: PathBuf,
    },
    Capture {
        #[arg(long)]
        preset: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    Compare {
        #[arg(long)]
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Write the canonical complete document for a recipe, ready to edit or import.
    Preset {
        recipe: String,
        #[arg(long)]
        output: PathBuf,
    },
}
fn load(path: &Path) -> Result<Document, String> {
    Document::from_json(&fs::read_to_string(path).map_err(|e| e.to_string())?)
}
fn recipe(slug: &str) -> Result<TextureRecipeId, String> {
    PROCEDURAL_TEXTURE_CATALOGUE
        .iter()
        .find(|r| r.id.slug() == slug)
        .map(|r| r.id)
        .ok_or_else(|| format!("Unknown recipe {slug}"))
}
fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, bytes).map_err(|e| e.to_string())
}
fn main() -> Result<(), String> {
    match Arguments::parse().command {
        Command::List => {
            for r in PROCEDURAL_TEXTURE_CATALOGUE {
                println!("{}\t{:?}", r.id.slug(), r.family);
            }
            Ok(())
        }
        Command::Preset {
            recipe: slug,
            output,
        } => {
            let document = Document {
                recipe: recipe(&slug)?,
                name: slug,
                ..Default::default()
            };
            write(&output, document.to_json().as_bytes())
        }
        Command::Export {
            recipe: slug,
            preset,
            output,
        } => {
            let selected = recipe(&slug)?;
            let document = if let Some(path) = preset {
                load(&path)?
            } else {
                Document {
                    recipe: selected,
                    ..Default::default()
                }
            };
            if document.recipe != selected {
                return Err("Preset and requested recipe differ".into());
            }
            let baked = BakedRecipe::generate(selected, &document.texture);
            write(&output.join(format!("{slug}.bake")), &baked.to_bytes())?;
            write(
                &output.join(format!("{slug}.texture.json")),
                document.to_json().as_bytes(),
            )?;
            adventuresim_texture_studio::export_recipe(&baked, &output)
        }
        Command::Capture { preset, output } => review::capture(load(&preset)?, None, &output),
        Command::Compare {
            before,
            after,
            output,
        } => review::capture(load(&after)?, Some(load(&before)?), &output),
    }
}
