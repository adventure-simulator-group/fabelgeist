//! The armory: every parametric catalog piece fitted to the character's body
//! and hung on a display wall, for inspecting and reshaping the catalog designs.
use super::*;
use adventuresim_character_creator::{
    armor_metal::is_plate_steel,
    decoration::Decoration,
    item_catalog_schema::{ItemKind, Slot},
    item_design::ItemDesign,
};
use fabelgeist_armor::Construction;
use studio_generation::PreviewScene;
use studio_scene::{OrbitCamera, OrbitGoal, framing_radius};

use bare_body::BareBody;
use scene::spawn_exhibit;
use scene::{ArmoryBody, ArmoryMesh};
pub(super) use exhibit::{Exhibit, ExhibitFit};
pub(super) use scene::{display, frame_camera, setup};

mod exhibit;
#[path = "armory_scene.rs"]
mod scene;

/// Space between neighbouring pieces on the wall, in metres.
const GAP: f32 = 0.14;
/// Extra space under each row for the piece labels, in metres.
const LABEL_SPACE: f32 = 0.12;
/// Narrowest wall space a piece takes, so neighbouring labels stay apart.
const MIN_WIDTH: f32 = 0.3;

/// How a steel exhibit is finished: built and decorated as designed.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Finish {
    pub decoration: Decoration,
    pub construction: Construction,
}

#[derive(Resource, Default)]
pub(super) struct Armory {
    /// The body the exhibits were fitted to.
    body: Option<BareBody>,
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
    /// The construction being tried, shown on the selected piece.
    pub construction: Construction,
    /// The name the decoration is saved to the library under.
    pub decoration_name: String,
    /// The character view to return to on leaving the armory.
    saved_camera: Option<OrbitCamera>,
    body_material: Option<(Handle<StandardMaterial>, bool)>,
    /// Whether the wall is laid out for both sides, once it is laid out.
    arranged: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Region {
    Head,
    Torso,
    Waist,
    Arms,
    Legs,
    Clothing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ArmoryView {
    /// Every piece side by side.
    #[default]
    Wall,
    /// The selected piece where it is worn, on the body.
    OnBody,
}

/// What the camera should frame next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Frame {
    Wall,
    Selected,
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

    /// The finish exhibit `index` should show: the one being designed on the
    /// selected steel piece, and the plain solid piece elsewhere.
    fn finish_for(&self, index: usize) -> Finish {
        if self.selected == Some(index) && self.exhibits[index].steel {
            Finish {
                decoration: self.decoration.clone(),
                construction: self.construction.clone(),
            }
        } else {
            Finish::default()
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
    match (active, armory.saved_camera) {
        (true, None) => {
            armory.saved_camera = Some(*orbit);
            armory.refit_all |= armory
                .body
                .as_ref()
                .is_none_or(|body| !body.is_current(&model, &studio.recipe));
            armory.frame = Some(if armory.selected.is_some() {
                Frame::Selected
            } else {
                Frame::Wall
            });
        }
        (false, Some(saved)) => {
            *orbit = saved;
            armory.saved_camera = None;
            if std::mem::take(&mut armory.defaults_changed) {
                studio.dirty = true;
            }
        }
        _ => {}
    }
}

/// Fit the pieces that need it and spawn their preview meshes.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the armory, studio, model, catalog, scene, queries and UI context"
)]
pub(crate) fn refit(
    mut armory: ResMut<Armory>,
    mut studio: ResMut<Studio>,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut scene: PreviewScene,
    old: Query<(Entity, &ArmoryMesh)>,
    bodies: Query<Entity, With<ArmoryBody>>,
    mut contexts: EguiContexts,
) {
    if studio.tab != studio_ui::StudioTab::Armory || studio_generation::dragging(&mut contexts) {
        return;
    }
    let started = std::time::Instant::now();
    let all = std::mem::take(&mut armory.refit_all);
    if all {
        match BareBody::generate(&model, &studio.recipe) {
            Ok(body) => {
                for entity in &bodies {
                    scene.commands.entity(entity).despawn();
                }
                armory.body_material = Some((body.spawn(&mut scene, &model, ArmoryBody), false));
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
        let Some(body) = armory.body.as_ref().map(|body| &body.generated) else {
            studio.status = "Armory fitting requires a generated body".into();
            return;
        };
        let fitted = fit_exhibits(&model, body, &catalog, &armory.exhibits, &stale);
        for (i, fitted) in stale.iter().copied().zip(fitted) {
            let exhibit = &mut armory.exhibits[i];
            exhibit.fitted = fitted.into_iter().map(ExhibitFit::from).collect();
            exhibit.stale = false;
        }
        armory.arranged = None;
    }
    // Show what was refitted, and every piece whose finish changed.
    let respawn: Vec<usize> = (0..armory.exhibits.len())
        .filter(|i| stale.contains(i) || armory.exhibits[*i].finished != armory.finish_for(*i))
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
        let finish = armory.finish_for(i);
        let exhibit = &mut armory.exhibits[i];
        exhibit.preview_error = spawn_exhibit(&mut scene, &catalog, i, exhibit, &finish).err();
        exhibit.finished = finish;
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

/// Every wearable catalog item with a parametric design, all stale.
fn exhibits(catalog: &EquipmentCatalog) -> Vec<Exhibit> {
    catalog
        .wearable()
        .filter_map(|item| {
            let loaded = catalog.design(item)?;
            let placements = item
                .retained_placements()
                .filter(|placement| !placement.selection().surface.is_empty())
                .collect::<Vec<_>>();
            Some(Exhibit {
                name: item.display_name.clone(),
                region: Region::of(item.definition()),
                fitted: placements
                    .iter()
                    .map(|_| ExhibitFit::Pending)
                    .collect(),
                preview_error: None,
                placements,
                loaded,
                steel: equipment_material(item.definition()).is_some_and(is_plate_steel),
                finished: Finish::default(),
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
) -> Vec<Vec<anyhow::Result<GeneratedArmor>>> {
    let fit = |exhibit: &Exhibit| {
        let design = catalog.design(exhibit.item());
        exhibit
            .placements
            .iter()
            .map(|placement| {
                parametric_equipment::fitted_catalog_item(
                    model,
                    body,
                    placement.selection(),
                    design
                        .as_ref()
                        .context("exhibit has no parametric design")?,
                    &[],
                    &[],
                )
                .with_context(|| format!("fitting {}", placement.selection().definition().id))
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
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}
