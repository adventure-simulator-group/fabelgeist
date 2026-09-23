//! The armory: every parametric catalog piece fitted to the character's body
//! and hung on a display wall, for inspecting and reshaping the catalog designs.
use super::*;
use adventuresim_character_creator::{
    armor_metal::is_plate_steel,
    decoration::Decoration,
    item_catalog_schema::{ItemKind, Slot},
    item_design::ItemDesign,
};
use studio_generation::PreviewScene;
use studio_scene::{OrbitCamera, OrbitGoal, framing_radius};

#[path = "armory_scene.rs"]
mod scene;
use scene::{ArmoryBody, ArmoryMesh};
pub(super) use scene::{display, frame_camera, setup};
use scene::{spawn_body, spawn_exhibit};

/// Space between neighbouring pieces on the wall, in metres.
const GAP: f32 = 0.14;
/// Extra space under each row for the piece labels, in metres.
const LABEL_SPACE: f32 = 0.12;
/// Narrowest wall space a piece takes, so neighbouring labels stay apart.
const MIN_WIDTH: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Region {
    Head,
    Torso,
    Waist,
    Arms,
    Legs,
    Clothing,
}

impl Region {
    pub const ALL: [Self; 6] = [
        Self::Head,
        Self::Torso,
        Self::Waist,
        Self::Arms,
        Self::Legs,
        Self::Clothing,
    ];

    fn of(item: &ItemDefinition) -> Self {
        let ItemKind::Armor { slot, .. } = item.kind else {
            return Self::Clothing;
        };
        match slot {
            Slot::Head => Self::Head,
            Slot::Chest => Self::Torso,
            Slot::Stomach => Self::Waist,
            Slot::AnyArm | Slot::LeftArm | Slot::RightArm => Self::Arms,
            Slot::AnyLeg | Slot::LeftLeg | Slot::RightLeg => Self::Legs,
            _ => Self::Clothing,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Head => "Head",
            Self::Torso => "Torso",
            Self::Waist => "Waist",
            Self::Arms => "Arms",
            Self::Legs => "Legs",
            Self::Clothing => "Clothing",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ArmoryView {
    /// Every piece side by side.
    #[default]
    Wall,
    /// The selected piece where it is worn, on the body.
    OnBody,
}

/// One catalog item on display, fitted in each of its placements.
pub(super) struct Exhibit {
    pub item_id: String,
    pub name: String,
    pub region: Region,
    pub placements: Vec<String>,
    /// Each placement's fitted mesh, or why fitting failed.
    pub fitted: Vec<Result<GeneratedArmor, String>>,
    /// The catalog default when the armory opened, for reverting edits.
    pub loaded: ItemDesign,
    /// Whether it is plate steel, which can be engraved and trimmed.
    pub steel: bool,
    /// The decoration its preview meshes show.
    decorated: Decoration,
    /// Added to the worn position to hang the piece on the wall.
    pub offset: Vec3,
    stale: bool,
}

impl Exhibit {
    /// Placements shown on the wall: one side, unless both are asked for.
    pub fn shown(&self, both_sides: bool) -> usize {
        if both_sides { self.placements.len() } else { 1 }
    }

    /// World bounds of the shown placements where they are worn.
    pub fn bounds(&self, both_sides: bool) -> Option<(Vec3, Vec3)> {
        self.fitted[..self.shown(both_sides)]
            .iter()
            .flatten()
            .flat_map(|armor| &armor.positions)
            .map(|p| Vec3::from_array(*p))
            .fold(None, |bounds, p| {
                Some(bounds.map_or((p, p), |(lo, hi): (Vec3, Vec3)| (lo.min(p), hi.max(p))))
            })
    }

    pub fn errors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.placements
            .iter()
            .zip(&self.fitted)
            .filter_map(|(placement, fit)| Some((placement.as_str(), fit.as_ref().err()?.as_str())))
    }

    pub fn triangles(&self) -> usize {
        self.fitted
            .iter()
            .flatten()
            .map(|a| a.indices.len() / 3)
            .sum()
    }
}

/// What the camera should frame next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Frame {
    Wall,
    Selected,
}

/// The body the exhibits were fitted to, and what it was generated from.
struct FittedBody {
    key: BodyKey,
    generated: GeneratedCharacter,
}

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

#[derive(Resource, Default)]
pub(super) struct Armory {
    body: Option<FittedBody>,
    pub exhibits: Vec<Exhibit>,
    pub selected: Option<usize>,
    pub view: ArmoryView,
    pub both_sides: bool,
    /// Hide every other piece on the wall.
    pub solo: bool,
    /// See the piece's clearance through a translucent body.
    pub ghost_body: bool,
    pub refit_all: bool,
    pub frame: Option<Frame>,
    /// Catalog defaults changed here, so the worn outfit must be rebuilt.
    pub defaults_changed: bool,
    /// The engraving and trim being designed, shown on the selected piece.
    pub decoration: Decoration,
    /// The name the decoration is saved to the library under.
    pub decoration_name: String,
    /// The character view to return to on leaving the armory.
    saved_camera: Option<OrbitCamera>,
    body_material: Option<(Handle<StandardMaterial>, bool)>,
    /// Whether the wall is laid out for both sides, once it is laid out.
    arranged: Option<bool>,
}

impl Armory {
    /// Select a piece and bring it into view.
    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
        self.frame = Some(Frame::Selected);
    }

    /// Refit a piece whose catalog default changed.
    pub fn refit(&mut self, index: usize) {
        self.exhibits[index].stale = true;
        self.defaults_changed = true;
    }

    pub fn ready(&self) -> bool {
        self.body.is_some()
    }

    /// The decoration exhibit `index` should show: the one being designed on
    /// the selected steel piece, and none elsewhere.
    fn decoration_for(&self, index: usize) -> Decoration {
        if self.selected == Some(index) && self.exhibits[index].steel {
            self.decoration.clone()
        } else {
            Decoration::default()
        }
    }

    /// Where every exhibit hangs on the wall, rows by body region from the head down.
    fn arrange(&mut self) {
        let rows: Vec<Vec<usize>> = Region::ALL
            .iter()
            .map(|region| {
                (0..self.exhibits.len())
                    .filter(|i| self.exhibits[*i].region == *region)
                    .collect::<Vec<_>>()
            })
            .filter(|row| !row.is_empty())
            .collect();
        let bounds: Vec<_> = self
            .exhibits
            .iter()
            .map(|exhibit| exhibit.bounds(self.both_sides))
            .collect();
        let size = |i: usize| bounds[i].map_or(Vec3::splat(0.2), |(lo, hi)| hi - lo);
        let heights: Vec<f32> = rows
            .iter()
            .map(|row| row.iter().map(|i| size(*i).y).fold(0.0, f32::max))
            .collect();
        let mut top = 0.15 + heights.iter().map(|h| h + GAP + LABEL_SPACE).sum::<f32>();
        for (row, height) in rows.iter().zip(heights) {
            let bottom = top - height;
            let cell = |i: usize| size(i).x.max(MIN_WIDTH);
            let width = row.iter().map(|i| cell(*i) + GAP).sum::<f32>() - GAP;
            let mut left = -width * 0.5;
            for &i in row {
                let (lo, hi) = bounds[i].unwrap_or((Vec3::ZERO, Vec3::splat(0.2)));
                let x = left + (cell(i) - size(i).x) * 0.5 - lo.x;
                self.exhibits[i].offset = Vec3::new(x, bottom - lo.y, -(lo.z + hi.z) * 0.5);
                left += cell(i) + GAP;
            }
            top = bottom - GAP - LABEL_SPACE;
        }
    }

    /// Bounds of what the camera should frame, as displayed.
    fn target(&self, frame: Frame) -> Option<(Vec3, Vec3)> {
        let displayed = |i: usize| {
            let exhibit = &self.exhibits[i];
            let offset = if self.view == ArmoryView::Wall {
                exhibit.offset
            } else {
                Vec3::ZERO
            };
            let both = self.both_sides || self.view == ArmoryView::OnBody;
            exhibit
                .bounds(both)
                .map(|(lo, hi)| (lo + offset, hi + offset))
        };
        let indices: Vec<usize> = match (frame, self.selected) {
            (Frame::Selected, Some(i)) => vec![i],
            _ => (0..self.exhibits.len()).collect(),
        };
        indices
            .into_iter()
            .filter_map(displayed)
            .reduce(|(a, b), (c, d)| (a.min(c), b.max(d)))
    }
}

/// Enter and leave the armory: swap the camera and rebuild the worn outfit
/// if catalog defaults changed meanwhile.
pub(crate) fn enter_or_leave(
    mut armory: ResMut<Armory>,
    mut studio: ResMut<Studio>,
    model: Res<BodyModel>,
    mut camera: Query<&mut OrbitCamera>,
) {
    let Ok(mut orbit) = camera.single_mut() else {
        return;
    };
    let active = studio.tab == studio_ui::StudioTab::Armory;
    match (active, armory.saved_camera.is_some()) {
        (true, false) => {
            armory.saved_camera = Some(*orbit);
            let key = BodyKey::new(&model, &studio.recipe);
            armory.refit_all |= armory.body.as_ref().is_none_or(|body| body.key != key);
            armory.frame = Some(if armory.selected.is_some() {
                Frame::Selected
            } else {
                Frame::Wall
            });
        }
        (false, true) => {
            *orbit = armory.saved_camera.take().expect("checked above");
            if std::mem::take(&mut armory.defaults_changed) {
                studio.dirty = true;
            }
        }
        _ => {}
    }
}

/// Fit the pieces that need it and spawn their preview meshes.
pub(crate) fn refit(
    mut armory: ResMut<Armory>,
    mut studio: ResMut<Studio>,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut scene: PreviewScene,
    old: Query<(Entity, &ArmoryMesh)>,
    bodies: Query<Entity, With<ArmoryBody>>,
) {
    if studio.tab != studio_ui::StudioTab::Armory {
        return;
    }
    let started = std::time::Instant::now();
    let all = std::mem::take(&mut armory.refit_all);
    if all {
        match fit_body(&model, &studio.recipe) {
            Ok(body) => {
                for entity in &bodies {
                    scene.commands.entity(entity).despawn();
                }
                armory.body_material = Some(spawn_body(&mut scene, &model, &body.generated));
                armory.exhibits = exhibits(&catalog);
                armory.body = Some(body);
                armory.selected = armory.selected.filter(|i| *i < armory.exhibits.len());
                armory.frame.get_or_insert(Frame::Wall);
            }
            Err(error) => {
                studio.status = format!("Armory body failed: {error:#}");
                return;
            }
        }
    }
    let stale: Vec<usize> = (0..armory.exhibits.len())
        .filter(|i| armory.exhibits[*i].stale)
        .collect();
    if !stale.is_empty() {
        let body = &armory.body.as_ref().expect("fitted above").generated;
        let fitted = fit_exhibits(&model, body, &catalog, &armory.exhibits, &stale);
        for (i, fitted) in stale.iter().copied().zip(fitted) {
            let exhibit = &mut armory.exhibits[i];
            exhibit.fitted = fitted;
            exhibit.stale = false;
        }
        armory.arranged = None;
    }
    // Show what was refitted, and every piece whose decoration changed.
    let respawn: Vec<usize> = (0..armory.exhibits.len())
        .filter(|i| stale.contains(i) || armory.exhibits[*i].decorated != armory.decoration_for(*i))
        .collect();
    if respawn.is_empty() {
        return;
    }
    for (entity, mesh) in &old {
        if all || respawn.contains(&mesh.exhibit) {
            scene.commands.entity(entity).despawn();
        }
    }
    // Keep only the steels this and the previous preview baked.
    scene.equipment_maps.begin_generation();
    for i in respawn {
        let decoration = armory.decoration_for(i);
        let exhibit = &mut armory.exhibits[i];
        if let Err(error) = spawn_exhibit(&mut scene, &catalog, i, exhibit, &decoration) {
            exhibit.fitted[0] = Err(format!("{error:#}"));
        }
        exhibit.decorated = decoration;
    }
    if stale.is_empty() {
        return;
    }
    let failed = armory
        .exhibits
        .iter()
        .filter(|e| e.errors().next().is_some())
        .count();
    studio.status = format!(
        "Armory fitted {} piece{} in {:.1} s · {failed} with problems",
        stale.len(),
        if stale.len() == 1 { "" } else { "s" },
        started.elapsed().as_secs_f32(),
    );
}

fn fit_body(model: &BodyModel, recipe: &CharacterRecipe) -> Result<FittedBody> {
    Ok(FittedBody {
        key: BodyKey::new(model, recipe),
        generated: generate_character(model, recipe)?,
    })
}

/// Every wearable catalog item with a parametric design, all stale.
fn exhibits(catalog: &EquipmentCatalog) -> Vec<Exhibit> {
    catalog
        .wearable()
        .filter_map(|item| {
            let loaded = catalog.design(&item.id)?;
            let placements: Vec<String> = item
                .equipment
                .as_ref()?
                .placements
                .iter()
                .filter(|placement| !placement.surface.is_empty())
                .map(|placement| placement.id.clone())
                .collect();
            Some(Exhibit {
                item_id: item.id.clone(),
                name: item.display_name.clone(),
                region: Region::of(item),
                fitted: placements
                    .iter()
                    .map(|_| Err("not fitted".into()))
                    .collect(),
                placements,
                loaded,
                steel: equipment_material(item).is_some_and(is_plate_steel),
                decorated: Decoration::default(),
                offset: Vec3::ZERO,
                stale: true,
            })
        })
        .collect()
}

fn equipment_material(
    item: &ItemDefinition,
) -> Option<adventuresim_character_creator::item_catalog_schema::EquipmentMaterial> {
    item.equipment.as_ref()?.material
}

/// Fit the given exhibits in every placement, in parallel.
fn fit_exhibits(
    model: &BodyModel,
    body: &GeneratedCharacter,
    catalog: &EquipmentCatalog,
    exhibits: &[Exhibit],
    indices: &[usize],
) -> Vec<Vec<Result<GeneratedArmor, String>>> {
    let fit = |exhibit: &Exhibit| {
        let item = catalog
            .item(&exhibit.item_id)
            .expect("exhibits come from the catalog");
        let design = catalog
            .design(&exhibit.item_id)
            .expect("exhibits are parametric");
        exhibit
            .placements
            .iter()
            .map(|placement| {
                parametric_equipment::fitted_catalog_item(
                    model,
                    body,
                    item,
                    &design,
                    placement,
                    &[],
                )
                .map_err(|error| format!("{error:#}"))
            })
            .collect::<Vec<_>>()
    };
    let threads = std::thread::available_parallelism().map_or(4, usize::from);
    let chunk = indices.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = indices
            .chunks(chunk)
            .map(|chunk| {
                scope.spawn(move || chunk.iter().map(|i| fit(&exhibits[*i])).collect::<Vec<_>>())
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("armor fitting panicked"))
            .collect()
    })
}
