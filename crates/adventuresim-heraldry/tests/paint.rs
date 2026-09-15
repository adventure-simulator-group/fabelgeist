use adventuresim_heraldry::{
    artwork::{Artwork, PaintTone},
    bake::{Baked, Resolution},
    document::*,
    export,
    geometry::Mesh,
    paint::{Paint, PaintRecipe},
    presets,
};

#[test]
fn paint_recipe_changes_color_and_finish_without_changing_the_arms_or_support() {
    let mut d = presets::preset("german-lion").unwrap();
    let arms = d.arms.clone();
    let drawing = d.drawing.clone();
    let mesh = Mesh::generate(&d).unwrap();
    let mineral = Baked::generate(&d, Resolution::Draft).unwrap();
    d.surface.palette[Tincture::Azure] = Paint::Recipe {
        recipe: PaintRecipe::IndigoWhiteSize,
    };
    let size_paint = Baked::generate(&d, Resolution::Draft).unwrap();
    assert_eq!(arms, d.arms);
    assert_eq!(drawing, d.drawing);
    assert_eq!(mesh.positions, Mesh::generate(&d).unwrap().positions);
    assert_eq!(mineral.height, size_paint.height);
    assert_ne!(mineral.flat, size_paint.flat);
    assert_ne!(mineral.albedo, size_paint.albedo);
    assert_ne!(mineral.orm, size_paint.orm);
    assert!(!mineral.matches(&d, Resolution::Draft));
    assert_eq!(mineral.coat, size_paint.coat);
    assert!(size_paint.orm.as_chunks::<4>().0.iter().all(|p| p[2] == 0));
}

#[test]
fn every_catalog_choice_roundtrips_and_exports_its_source_with_the_material() {
    let mut d = Document {
        arms: ArmsDesign::plain(Tincture::Or),
        ..Default::default()
    };
    d.surface.shape = DisplayShape::Panel;
    d.surface.gold = MetalFinish::Pigment;
    for recipe in PaintRecipe::ALL {
        d.surface.palette[Tincture::Or] = Paint::Recipe { recipe };
        let restored = Document::from_json(&d.to_json().unwrap()).unwrap();
        assert_eq!(d, restored);
        let b = Baked::generate(&restored, Resolution::Draft).unwrap();
        let files = export::bundle(&restored, &b, Resolution::Draft).unwrap();
        let record = files
            .iter()
            .find(|f| f.name == "paint-recipes.json")
            .unwrap();
        let manifest: serde_json::Value = serde_json::from_slice(&record.bytes).unwrap();
        assert_eq!(
            manifest["palette"][0]["selection"],
            serde_json::to_value(d.surface.palette[Tincture::Or]).unwrap()
        );
        let definition = &manifest["palette"][0]["definition"];
        assert_eq!(definition["source_url"], recipe.definition().source_url);
        assert!(!definition["ingredients"].as_array().unwrap().is_empty());
        let glb = files.iter().find(|f| f.name == "display.glb").unwrap();
        let json_size = u32::from_le_bytes(glb.bytes[12..16].try_into().unwrap()) as usize;
        let scene: serde_json::Value =
            serde_json::from_slice(&glb.bytes[20..20 + json_size]).unwrap();
        assert_eq!(scene["extras"]["paintRecipes"], manifest);
    }
}

#[test]
fn vector_export_uses_selected_paint_tones_and_lighting_stays_out_of_the_bake() {
    let mut d = presets::preset("german-lion").unwrap();
    d.surface.palette[Tincture::Or] = Paint::Recipe {
        recipe: PaintRecipe::YellowOchreTempera,
    };
    let svg = Artwork::compose(&d).unwrap().svg(&d.surface.palette);
    for tone in PaintTone::ALL {
        let [r, g, b] = d.surface.palette[Tincture::Or].color(tone);
        assert!(svg.contains(&format!("#{r:02x}{g:02x}{b:02x}")));
    }
    let left = Baked::generate(&d, Resolution::Draft).unwrap();
    d.view.light = Degrees(65.0);
    assert!(left.matches(&d, Resolution::Draft));
    let right = Baked::generate(&d, Resolution::Draft).unwrap();
    assert_eq!(left.albedo, right.albedo);
    assert_eq!(left.normal, right.normal);
    assert_eq!(left.orm, right.orm);
}
