//! Exercise runtime construction pipelines before the client becomes interactive.
use super::*;
use fabelgeist_armor::{
    BreastplateConstruction, GarmentArmorDesign, GarmentArmorKind, GarmentPlateShape,
    WrappedTassetDesign,
};

/// Completed device preparation; temporary meshes are discarded, kernels remain.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct WarmupReport {
    pub fitted_placements: usize,
    pub cached_kernels: usize,
}

/// Warm every authored armor placement and the alternative construction paths.
///
/// Use the canonical unposed body during initial loading. These are temporary
/// client-generated meshes, not NPC fits or served assets. Body dimensions do
/// not specialize the kernels; subsequent wearers reuse the same device cache.
pub async fn warm_up(
    body: &RuntimeBody,
    bracer: &BracerDesign,
    breastplate: &BreastplateDesign,
) -> Result<WarmupReport> {
    let mut fitted_placements = 0;
    for canonical in ITEM_CATALOG
        .iter()
        .filter(|item| armor_recipes::is_parametric(item.definition))
    {
        let item = canonical.definition;
        for placement in item
            .placements()
            .filter(|placement| !placement.surface.is_empty())
        {
            generate(body, placement, bracer, breastplate, &[])
                .await
                .with_context(|| format!("preparing equipment {} {}", item.id, placement.id))?;
            fitted_placements += 1;
        }
    }
    // This runs the completed-lower-layer projection path as well as bare fits.
    let cuirass_selection = CatalogPlacement::find(item_catalog::catalog(), "cuirass", "worn")
        .context("missing authored cuirass placement")?;
    let cuirass = generate(body, cuirass_selection, bracer, breastplate, &[])
        .await
        .context("preparing the lower cuirass surface")?;
    let pauldron = CatalogPlacement::find(item_catalog::catalog(), "pauldron", "left")
        .context("missing authored pauldron placement")?;
    generate(body, pauldron, bracer, breastplate, &[&cuirass])
        .await
        .context("preparing layered pauldrons")?;
    fitted_placements += 2;

    let mut alternate = breastplate.clone();
    alternate.construction = match &breastplate.construction {
        BreastplateConstruction::Solid => BreastplateConstruction::Anime(Default::default()),
        BreastplateConstruction::Anime(_) => BreastplateConstruction::Solid,
    };
    generate(body, cuirass_selection, bracer, &alternate, &[])
        .await
        .context("preparing alternate cuirass construction")?;
    fitted_placements += 1;

    // Wrapped courses are an alternative to the authored sector waist recipe.
    let mut tassets = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
    tassets.lame_count = 8;
    tassets.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign::default());
    let design = ParametricDesign::Garment(tassets);
    let placement = CatalogPlacement::find(item_catalog::catalog(), "tassets", "worn")
        .context("missing authored tassets placement")?;
    generate_parametric(
        crate::armor_gpu_async().await?,
        body,
        &design,
        placement,
        &[],
    )
    .await
    .context("preparing wrapped tassets")?;
    fitted_placements += 1;
    Ok(WarmupReport {
        fitted_placements,
        cached_kernels: crate::armor_gpu_async().await?.cache().len(),
    })
}
