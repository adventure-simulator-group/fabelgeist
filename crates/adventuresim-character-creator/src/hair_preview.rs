//! The hair on the previewed character.
//!
//! A worker thread grows a hairstyle on the hair crate's bust (once per
//! style) and places it on the generated body (once per body): the strands
//! are carried onto the character's head and every level of detail is
//! prepared there, kept outside the character's own skin. The studio then
//! draws the level the Hair page picks, skinned wholly to the head joint so
//! it follows the walk preview, and shades the scalp under it.
use super::*;
use adventuresim_character_creator::hair::{
    BodySurface, Groom, GroomedHair, HairDesign, HeadPlacement, HeadSkin, PlacedHair, SkinCover,
    variant_label,
};
use fabelgeist_hair::lod::Form;
use std::sync::Arc;
use std::thread::JoinHandle;

mod assets;
#[cfg(test)]
mod render_test;
mod scalp_shade;
mod strand_shading;

use assets::HairAssets;
pub(super) use strand_shading::HairShadingPlugin;

/// The hair mesh shown on the character: the design and the form it draws.
#[derive(Component)]
pub(super) struct HairMesh {
    design: HairDesign,
    form: Form,
}

/// The grown and placed hair, and the worker preparing the next.
#[derive(Resource, Default)]
pub(super) struct HairPreview {
    grown: Option<Arc<GroomedHair>>,
    placed: Option<PlacedStyle>,
    worker: Option<Worker>,
}

/// Hair placed on one generated body.
pub(super) struct PlacedStyle {
    pub(super) hair: PlacedHair,
    body: Arc<GeneratedCharacter>,
    atlas: Handle<Image>,
}

struct Worker {
    groom: Groom,
    body: Arc<GeneratedCharacter>,
    handle: JoinHandle<Result<Prepared>>,
}

/// What a worker hands back.
struct Prepared {
    grown: Arc<GroomedHair>,
    placed: PlacedHair,
}

/// A body's copy of what placing hair on it reads, for the worker to own.
pub(super) struct BodyHead {
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    joint_names: Vec<String>,
    global_joint_states: Vec<[f32; 8]>,
}

impl BodyHead {
    pub(super) fn new(model: &BodyModel, generated: &GeneratedCharacter) -> Self {
        let character = &model.mhr.character;
        Self {
            positions: generated.positions.clone(),
            faces: character.mesh.faces.clone(),
            joint_indices: character.skin_weights.index.clone(),
            joint_weights: character.skin_weights.weight.clone(),
            joint_names: character.skeleton.names.clone(),
            global_joint_states: generated.global_joint_states.clone(),
        }
    }

    /// Where the bust's head lies on this body.
    pub(super) fn placement(&self, hair: &GroomedHair) -> Result<HeadPlacement> {
        let skin = HeadSkin {
            positions: &self.positions,
            joint_indices: &self.joint_indices,
            joint_weights: &self.joint_weights,
            joint_names: &self.joint_names,
            global_joint_states: &self.global_joint_states,
        };
        Ok(HeadPlacement::fit(&skin, hair.head())?)
    }

    /// `hair` placed on this body, kept outside its skin.
    pub(super) fn place(&self, hair: &GroomedHair) -> Result<PlacedHair> {
        let placement = self.placement(hair)?;
        let body = BodySurface {
            positions: &self.positions,
            faces: &self.faces,
        };
        Ok(hair.place(&placement, &body))
    }
}

impl HairPreview {
    /// The placed hair, if it is `groom` on `body`.
    fn placed(&self, groom: Groom, body: &Arc<GeneratedCharacter>) -> Option<&PlacedStyle> {
        self.placed
            .as_ref()
            .filter(|placed| placed.hair.groom == groom && Arc::ptr_eq(&placed.body, body))
    }

    /// Start preparing `groom` on `body` unless that is placed or under way
    /// already; true when a worker started. A worker on another style or
    /// body is left to finish and its hair dropped.
    fn request(&mut self, groom: Groom, body: &Arc<GeneratedCharacter>, head: BodyHead) -> bool {
        let under_way = self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.groom == groom && Arc::ptr_eq(&worker.body, body));
        if self.placed(groom, body).is_some() || under_way {
            return false;
        }
        let grown = self.grown.clone().filter(|grown| grown.groom == groom);
        let handle = std::thread::spawn(move || {
            let grown = grown.unwrap_or_else(|| Arc::new(GroomedHair::grow(groom)));
            let placed = head.place(&grown)?;
            Ok(Prepared { grown, placed })
        });
        self.worker = Some(Worker {
            groom,
            body: body.clone(),
            handle,
        });
        true
    }

    /// Take the worker's hair once it has finished.
    fn collect(&mut self, images: &mut Assets<Image>) -> Result<()> {
        if !self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.handle.is_finished())
        {
            return Ok(());
        }
        let Some(worker) = self.worker.take() else {
            return Ok(());
        };
        let prepared = worker
            .handle
            .join()
            .map_err(|_| anyhow::anyhow!("the hair worker panicked"))??;
        let atlas = images.add(assets::atlas_image(&prepared.placed));
        self.grown = Some(prepared.grown);
        self.placed = Some(PlacedStyle {
            hair: prepared.placed,
            body: worker.body,
            atlas,
        });
        Ok(())
    }
}

/// A level of detail the Hair page offers, and its name there.
pub(super) struct FormChoice {
    pub(super) form: Form,
    pub(super) label: String,
}

impl HairPreview {
    /// The levels of detail of the placed hair, finest first; none until
    /// it is placed.
    pub(super) fn form_choices(&self) -> Vec<FormChoice> {
        let Some(placed) = &self.placed else {
            return Vec::new();
        };
        let hair = &placed.hair;
        hair.forms()
            .into_iter()
            .map(|form| FormChoice {
                form,
                label: match form {
                    Form::Strands { level, .. } => {
                        format!("Strands, {} points each", hair.strand_points(level))
                    }
                    Form::Cards { level } => format!("Cards, {} strips", hair.card_count(level)),
                },
            })
            .collect()
    }

    /// How many strands `form` draws of the placed hair's, and of how many.
    pub(super) fn strands_drawn(&self, form: Form) -> Option<StrandsDrawn> {
        let hair = &self.placed.as_ref()?.hair;
        Some(StrandsDrawn {
            drawn: hair.strands_drawn(form),
            all: hair.strand_count(),
        })
    }

    /// Whether hair is still being grown or placed.
    pub(super) fn preparing(&self) -> bool {
        self.worker.is_some()
    }
}

/// Strands drawn out of a groom's.
pub(super) struct StrandsDrawn {
    pub(super) drawn: usize,
    pub(super) all: usize,
}

/// Keep the shown hair in step with the recipe and the Hair page: grow and
/// place a new style, place it again on every body the studio regenerates
/// (which despawns it with the rest of the character), and redraw it when
/// its colour or level of detail changes.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the studio, body caches, the shown meshes and their assets"
)]
pub(super) fn refresh(
    mut commands: Commands,
    mut preview: ResMut<HairPreview>,
    mut studio: ResMut<Studio>,
    mut cache: ResMut<studio_cache::StudioCache>,
    model: Res<BodyModel>,
    walk: Res<WalkPreview>,
    shown: Query<(Entity, &HairMesh)>,
    mut assets: HairAssets,
    mut contexts: EguiContexts,
) {
    if let Err(error) = preview.collect(&mut assets.images) {
        studio.status = format!("Hair could not be placed: {error:#}");
    }
    let wanted = studio.recipe.hair.map(|design| (design, studio.hair_form));
    for (entity, mesh) in &shown {
        if Some((mesh.design, mesh.form)) != wanted {
            commands.entity(entity).despawn();
        }
    }
    let Some((design, form)) = wanted else {
        return;
    };
    // A level of detail is drawn once its slider is let go.
    if studio.dirty || walk.joints.is_empty() || studio_generation::dragging(&mut contexts) {
        return;
    }
    let recipe = studio.recipe.clone();
    let body = match cache.body(&model, &recipe) {
        Ok(body) => body,
        Err(error) => {
            studio.status = format!("{error:#}");
            return;
        }
    };
    let Some(placed) = preview.placed(design.groom(), &body) else {
        if preview.request(design.groom(), &body, BodyHead::new(&model, &body)) {
            studio.status = format!(
                "Growing {} hair",
                variant_label(design.preset).to_lowercase()
            );
        }
        return;
    };
    if shown
        .iter()
        .any(|(_, mesh)| (mesh.design, mesh.form) == (design, form))
    {
        return;
    }
    let Some(skin) = animation_preview::skin(&walk) else {
        return;
    };
    let drawn = assets.hair_mesh(placed, form).and_then(|hair| {
        let cap = assets.scalp_cap(placed, design)?;
        Ok((hair, cap))
    });
    match drawn {
        Ok((mesh, (cap, skin_material))) => {
            let material = assets.hair_material(design, form, placed.atlas.clone());
            commands.spawn((
                CharacterMesh,
                HairMesh { design, form },
                Name::new("Hair"),
                skin.clone(),
                Mesh3d(mesh),
                MeshMaterial3d(material),
            ));
            commands.spawn((
                CharacterMesh,
                HairMesh { design, form },
                Name::new("Scalp"),
                skin,
                Mesh3d(cap),
                MeshMaterial3d(skin_material),
            ));
        }
        Err(error) => studio.status = format!("Hair could not be drawn: {error:#}"),
    }
}
