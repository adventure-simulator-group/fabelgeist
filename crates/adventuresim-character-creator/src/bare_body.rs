//! The character's body without its outfit, which the armory and the wardrobe
//! fit to, and what it was generated from.
use super::*;
use studio_generation::PreviewScene;

/// What a bare body is generated from: the model and the recipe without its
/// outfit or name, which never change the body.
#[derive(PartialEq)]
struct BodyKey {
    lod: u8,
    correctives: bool,
    recipe: CharacterRecipe,
}

impl BodyKey {
    fn new(model: &BodyModel, recipe: &CharacterRecipe) -> Self {
        let mut recipe = recipe.clone();
        recipe.inventory = default();
        recipe.name.clear();
        Self {
            lod: model.lod,
            correctives: model.correctives,
            recipe,
        }
    }
}

pub(super) struct BareBody {
    key: BodyKey,
    pub generated: GeneratedCharacter,
}

impl BareBody {
    pub(super) fn generate(model: &BodyModel, recipe: &CharacterRecipe) -> Result<Self> {
        Ok(Self {
            key: BodyKey::new(model, recipe),
            generated: generate_character(model, recipe)?,
        })
    }

    /// Whether the character's body is still this one.
    pub(super) fn is_current(&self, model: &BodyModel, recipe: &CharacterRecipe) -> bool {
        self.key == BodyKey::new(model, recipe)
    }

    /// Show the body in skin, hidden until its tab shows it. Returns its
    /// material, for making it see-through.
    pub(super) fn spawn(
        &self,
        scene: &mut PreviewScene,
        model: &BodyModel,
        marker: impl Bundle,
    ) -> Handle<StandardMaterial> {
        let mesh =
            studio_generation::visible_body_mesh(&self.generated, &model.mhr.character.mesh.faces);
        let material = scene.materials.add(StandardMaterial {
            base_color: Color::srgb(0.64, 0.39, 0.30),
            perceptual_roughness: 0.52,
            reflectance: 0.46,
            ..default()
        });
        scene.commands.spawn((
            marker,
            Name::new("bare body"),
            Mesh3d(scene.meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Visibility::Hidden,
        ));
        material
    }
}
