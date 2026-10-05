use super::*;

#[test]
fn occupied_programme_reuses_the_accepted_recipe_after_memo_transfer() {
    let mut palette = CityRecipePalette::default();
    let selected = palette
        .get(
            BuildingArchetype::TownHouse,
            Some(BuildingUse::Dwelling),
            None,
            42,
        )
        .unwrap();
    let mut transferred = palette.clone();
    let reused = transferred.for_program(&selected.program).unwrap();
    assert!(Arc::ptr_eq(&selected, &reused));
    assert_eq!(palette, transferred);
    assert_eq!(reused.program, selected.program);
}

#[test]
fn an_invalid_occupied_programme_is_rejected_without_recipe_reselection() {
    let mut palette = CityRecipePalette::default();
    let selected = palette
        .get(
            BuildingArchetype::TownHouse,
            Some(BuildingUse::Dwelling),
            None,
            42,
        )
        .unwrap();
    let mut invalid = selected.program.clone();
    invalid.storeys[0].rooms.clear();
    let before = palette.clone();
    assert!(matches!(
        palette.for_program(&invalid),
        Err(CityCompileError::Recipe {
            source: adventuresim_building_generator::GenerationError::EmptyStorey { level: _ },
            ..
        })
    ));
    assert_eq!(palette, before);
    assert!(Arc::ptr_eq(
        &palette.for_program(&selected.program).unwrap(),
        &selected
    ));
}

#[test]
fn curated_upper_dwelling_programmes_remain_buildable_without_reselection() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        let mut palette = CityRecipePalette::default();
        for choice in 0..CURATED_RECIPE_SEEDS.len() {
            let seed = catalogue::seed(archetype, BuildingUse::Dwelling, choice);
            let programme =
                BuildingProgram::settlement(archetype, Some(BuildingUse::Dwelling), seed);
            let recipe = palette
                .get(archetype, Some(BuildingUse::Dwelling), None, seed)
                .unwrap();
            assert_eq!(recipe.program, programme);
            assert_eq!(palette.for_program(&programme).unwrap().program, programme);
        }
    }
}
