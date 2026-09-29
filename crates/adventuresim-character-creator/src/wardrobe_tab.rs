//! The wardrobe: cloth draped and settled on the character's bare body and
//! saved by name, and saved garments fitted to the current body without
//! simulating.
use super::*;
use adventuresim_character_creator::{
    garment::{DrapeInput, DrapeStage, DrapedGarment, GarmentSelection, SettledGarment},
    wardrobe::{LibraryName, Wardrobe},
};
use bare_body::BareBody;
use drape_worker::DrapeWorker;
use studio_generation::PreviewScene;

/// The wardrobe library and where it is saved.
pub(super) struct WardrobeLibrary {
    pub library: Wardrobe,
    pub path: String,
}

impl WardrobeLibrary {
    /// Save `garment` under `name` and write the library.
    pub(super) fn save(&mut self, name: &str, garment: SettledGarment) -> Result<LibraryName> {
        let name = LibraryName::try_from(name.to_owned()).map_err(anyhow::Error::msg)?;
        let mut library = self.library.clone();
        library.insert(name.clone(), garment)?;
        library.save(std::path::Path::new(&self.path))?;
        self.library = library;
        Ok(name)
    }

    /// Delete the garment called `name` and write the library.
    pub(super) fn delete(&mut self, name: &LibraryName) -> Result<()> {
        let mut library = self.library.clone();
        library.remove(name);
        library.save(std::path::Path::new(&self.path))?;
        self.library = library;
        Ok(())
    }
}

/// What the wardrobe shows on the body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Showing {
    /// The garment being designed, draped by simulation.
    Design,
    /// A saved garment, fitted to the current body from its saved drape.
    Saved(LibraryName),
}

#[derive(Component)]
pub(super) struct WardrobeBody;

#[derive(Component)]
pub(super) struct WardrobeGarment;

/// One drape of the garment being designed.
struct Drape {
    /// The settings it was draped from.
    selection: GarmentSelection,
    /// Its latest preview, or its result once finished.
    garment: Option<DrapedGarment>,
    finished: bool,
    /// Problems of the finished drape; the garment is still shown.
    problems: Option<String>,
}

#[derive(Resource)]
pub(super) struct WardrobeTab {
    /// The body garments are draped on and fitted to.
    body: Option<BareBody>,
    /// The garment being designed.
    pub garment: GarmentSelection,
    pub showing: Showing,
    worker: DrapeWorker<()>,
    drape: Option<Drape>,
    /// Problems fitting the shown saved garment; it is still shown.
    fit_problems: Option<String>,
    /// Drape the designed garment again, from its first changed stage.
    pub redrape: bool,
    /// Show `showing` again, as the body or the saved garment changed.
    refresh: bool,
}

impl Default for WardrobeTab {
    fn default() -> Self {
        Self {
            body: None,
            garment: GarmentSelection::default(),
            showing: Showing::Design,
            worker: DrapeWorker::default(),
            drape: None,
            fit_problems: None,
            redrape: true,
            refresh: true,
        }
    }
}

impl WardrobeTab {
    pub fn ready(&self) -> bool {
        self.body.is_some()
    }

    pub fn draping(&self) -> bool {
        self.worker.running()
    }

    /// How far the designed garment's drape has come.
    pub fn stage(&self) -> Option<DrapeStage> {
        Some(self.drape.as_ref()?.garment.as_ref()?.stage)
    }

    /// The designed garment, fully settled as it is designed now: ready to save.
    pub fn settled(&self) -> Option<&DrapedGarment> {
        let drape = self.drape.as_ref()?;
        let garment = drape.garment.as_ref()?;
        let complete = matches!(garment.stage, DrapeStage::Settling { step, of } if step == of);
        (drape.finished && complete && drape.selection.same_simulation(&self.garment))
            .then_some(garment)
    }

    /// Problems of what is shown, which is shown anyway.
    pub fn problems(&self) -> Option<&str> {
        match self.showing {
            Showing::Design => self.drape.as_ref()?.problems.as_deref(),
            Showing::Saved(_) => self.fit_problems.as_deref(),
        }
    }

    pub fn show(&mut self, showing: Showing) {
        self.showing = showing;
        self.refresh = true;
    }

    /// Design from a saved garment's settings, draping it again.
    pub fn edit(&mut self, garment: &SettledGarment) {
        self.garment = garment.selection.clone();
        self.redrape = true;
        self.show(Showing::Design);
    }

    /// Drape the designed garment again from its placed panels.
    pub fn restart(&mut self) {
        self.worker.restart_from_placement();
        self.redrape = true;
    }

    /// Save the settled garment to the library under its name.
    pub fn save(&self, model: &BodyModel, library: &mut WardrobeLibrary) -> Result<LibraryName> {
        let body = self.body.as_ref().context("the wardrobe has no body yet")?;
        let settled = self
            .settled()
            .context("the garment has not settled as designed")?;
        let input = drape_preview::input(model, &body.generated, self.garment.clone());
        library.save(
            &self.garment.name,
            SettledGarment::capture(&input, settled)?,
        )
    }

    fn input(&self, model: &BodyModel, selection: GarmentSelection) -> Option<DrapeInput> {
        let body = self.body.as_ref()?;
        Some(drape_preview::input(model, &body.generated, selection))
    }
}

/// Keep the wardrobe's body current, drape the designed garment, and show
/// what the wardrobe shows.
pub(super) fn update(
    mut wardrobe: ResMut<WardrobeTab>,
    mut studio: ResMut<Studio>,
    model: Res<BodyModel>,
    mut scene: PreviewScene,
    mut mail: ResMut<drape_preview::MailMaterials>,
    bodies: Query<Entity, With<WardrobeBody>>,
    garments: Query<Entity, With<WardrobeGarment>>,
) {
    let wardrobe = &mut *wardrobe;
    collect(wardrobe, &mut studio);
    if studio.tab != studio_ui::StudioTab::Wardrobe {
        return;
    }
    if wardrobe
        .body
        .as_ref()
        .is_none_or(|body| !body.is_current(&model, &studio.recipe))
    {
        match BareBody::generate(&model, &studio.recipe) {
            Ok(body) => {
                for entity in &bodies {
                    scene.commands.entity(entity).despawn();
                }
                body.spawn(&mut scene, &model, WardrobeBody);
                wardrobe.body = Some(body);
                wardrobe.redrape = true;
                wardrobe.refresh = true;
            }
            Err(error) => {
                studio.status = format!("Wardrobe body failed: {error:#}");
                return;
            }
        }
    }
    if std::mem::take(&mut wardrobe.redrape) {
        request(wardrobe, &model, &mut studio);
    }
    if !std::mem::take(&mut wardrobe.refresh) {
        return;
    }
    let shown = match wardrobe.showing.clone() {
        // The design's own appearance, so chainmail edits show without draping.
        Showing::Design => wardrobe
            .drape
            .as_ref()
            .and_then(|drape| Some((wardrobe.garment.clone(), drape.garment.clone()?))),
        Showing::Saved(name) => fit_saved(wardrobe, &model, &studio, &name),
    };
    for entity in &garments {
        scene.commands.entity(entity).despawn();
    }
    if let Some((selection, garment)) = shown {
        spawn(&mut scene, &mut mail, &selection, garment);
    }
}

/// Take the running drape's progress, and show it when the design is shown.
fn collect(wardrobe: &mut WardrobeTab, studio: &mut Studio) {
    let progress = wardrobe.worker.poll();
    let Some(drape) = wardrobe.drape.as_mut() else {
        return;
    };
    if let Some(garment) = progress
        .preview
        .and_then(|garments| garments.into_iter().next())
    {
        drape.garment = Some(garment);
        wardrobe.refresh |= wardrobe.showing == Showing::Design;
    }
    if let Some(outcome) = progress.finished {
        drape.finished = true;
        drape.problems = outcome.problems();
        studio.status = match &drape.problems {
            None => format!(
                "{} settled; ready to save to the wardrobe",
                drape.selection.name
            ),
            Some(problems) => format!("{} draped with problems: {problems}", drape.selection.name),
        };
    }
}

/// Drape the designed garment on the wardrobe's body.
fn request(wardrobe: &mut WardrobeTab, model: &BodyModel, studio: &mut Studio) {
    let Some(input) = wardrobe.input(model, wardrobe.garment.clone()) else {
        return;
    };
    wardrobe.drape = Some(Drape {
        selection: wardrobe.garment.clone(),
        garment: None,
        finished: false,
        problems: None,
    });
    wardrobe.worker.request(vec![((), input)]);
    studio.status = format!("Draping {} in the wardrobe…", wardrobe.garment.name);
}

/// Fit a saved garment to the wardrobe's body from its saved drape.
fn fit_saved(
    wardrobe: &mut WardrobeTab,
    model: &BodyModel,
    studio: &Studio,
    name: &LibraryName,
) -> Option<(GarmentSelection, DrapedGarment)> {
    let saved = studio.wardrobe.library.get(name)?;
    let input = wardrobe.input(model, saved.selection.clone())?;
    let outcome = saved.drape.wear(&input);
    let mut problems = outcome.warnings;
    let garment = match outcome.result {
        Ok(garment) => Some((saved.selection.clone(), garment)),
        Err(error) => {
            problems.insert(0, format!("{error:#}"));
            None
        }
    };
    wardrobe.fit_problems = (!problems.is_empty()).then(|| problems.join("; "));
    garment
}

fn spawn(
    scene: &mut PreviewScene,
    mail: &mut drape_preview::MailMaterials,
    selection: &GarmentSelection,
    mut garment: DrapedGarment,
) {
    // The wardrobe shows the garment standing still: no skin to animate it.
    garment.indices.clear();
    garment.weights.clear();
    let material = mail.material_for(&mut scene.images, selection);
    drape_preview::spawn_garment(
        &mut scene.commands,
        &mut scene.meshes,
        &mut scene.materials,
        garment,
        material,
        (WardrobeGarment, Visibility::Hidden),
    );
}

/// Show the wardrobe's body and garment on its tab only.
pub(super) fn display(
    studio: Res<Studio>,
    mut shown: Query<&mut Visibility, Or<(With<WardrobeBody>, With<WardrobeGarment>)>>,
) {
    let visibility = if studio.tab == studio_ui::StudioTab::Wardrobe {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut shown in &mut shown {
        shown.set_if_neq(visibility);
    }
}
