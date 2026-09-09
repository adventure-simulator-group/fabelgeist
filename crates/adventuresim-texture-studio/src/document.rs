//! Portable authoring documents: texture, material response and viewing conditions.
use adventuresim_procedural_textures::{TextureParameters, TextureRecipeId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub name: String,
    pub recipe: TextureRecipeId,
    pub texture: TextureParameters,
    pub environment: Environment,
    pub view: View,
    pub surface: Surface,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            name: "Untitled material".into(),
            recipe: TextureRecipeId::HewnOak,
            texture: TextureParameters::default(),
            environment: Environment::default(),
            view: View::default(),
            surface: Surface::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub key_lux: f32,
    pub key_color: [f32; 3],
    pub azimuth_degrees: f32,
    pub incidence_degrees: f32,
    pub ambient: f32,
    pub reflection_strength: f32,
    pub exposure_ev: f32,
    pub background: [f32; 3],
    pub fill_lux: f32,
    pub fill_color: [f32; 3],
    pub shadows: bool,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            key_lux: 7000.0,
            key_color: [1.0; 3],
            azimuth_degrees: -35.0,
            incidence_degrees: 0.28_f32.atan().to_degrees(),
            ambient: 120.0,
            reflection_strength: 0.0,
            exposure_ev: 0.0,
            background: [0.035, 0.043, 0.052],
            fill_lux: 0.0,
            fill_color: [0.72, 0.83, 1.0],
            shadows: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Shape {
    Plane,
    Sphere,
    BeveledCube,
    Cylinder,
    Beam,
    FoldedSheet,
    Pane,
    CrownStrip,
}
impl Shape {
    pub const ALL: [Self; 8] = [
        Self::Plane,
        Self::Sphere,
        Self::BeveledCube,
        Self::Cylinder,
        Self::Beam,
        Self::FoldedSheet,
        Self::Pane,
        Self::CrownStrip,
    ];
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub shape: Shape,
    pub repeats: f32,
    pub offset: [f32; 2],
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub orthographic: bool,
    pub turntable: bool,
    pub displacement: f32,
    pub backdrop_distance: f32,
}
impl Default for View {
    fn default() -> Self {
        Self {
            shape: Shape::Plane,
            repeats: 1.0,
            offset: [0.0; 2],
            yaw: 0.0,
            pitch: 0.0,
            distance: 3.2,
            orthographic: true,
            turntable: false,
            displacement: 0.0,
            backdrop_distance: 0.95,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub bark_projection_sharpness: f32,
    pub bark_branch_alignment: f32,
    pub bark_parallax: f32,
    pub bark_fade_metres: f32,
    pub litter_colors_linear: [[f32; 3]; 3],
    pub normal_strength: f32,
    pub ao_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub pigment: [f32; 3],
    pub leaf_transmission: f32,
    pub leaf_thickness_metres: f32,
}
impl Default for Surface {
    fn default() -> Self {
        Self {
            bark_projection_sharpness: 4.0,
            bark_branch_alignment: 0.92,
            bark_parallax: 0.52,
            bark_fade_metres: 12.0,
            litter_colors_linear: [
                [0.016, 0.010, 0.006],
                [0.035, 0.021, 0.011],
                [0.062, 0.036, 0.018],
            ],
            normal_strength: 1.0,
            ao_strength: 1.0,
            roughness: 1.0,
            metallic: 1.0,
            pigment: [0.34, 0.28, 0.20],
            leaf_transmission: 0.46,
            leaf_thickness_metres: 0.0002,
        }
    }
}

impl Document {
    pub fn from_json(source: &str) -> Result<Self, String> {
        if source.len() > 512 * 1024 {
            return Err("Preset exceeds 512 KiB".into());
        }
        let value: serde_json::Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
        TextureParameters::from_value(
            value
                .get("texture")
                .ok_or("Missing texture parameters")?
                .clone(),
        )
        .map_err(|e| e.to_string())?;
        let document: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
        let e = &document.environment;
        let v = &document.view;
        let s = &document.surface;
        if !(0.0..=200_000.0).contains(&e.key_lux)
            || !(0.0..=200_000.0).contains(&e.fill_lux)
            || !(0.0..=10_000.0).contains(&e.ambient)
            || !(0.0..=10_000.0).contains(&e.reflection_strength)
            || !(0.1..=5.0).contains(&v.backdrop_distance)
            || !(-10.0..=10.0).contains(&e.exposure_ev)
            || !(0.1..=100.0).contains(&v.distance)
            || !(0.01..=32.0).contains(&v.repeats)
            || !(0.0..=4.0).contains(&v.displacement)
            || (v.shape != Shape::Plane && v.displacement > 0.0)
            || !(-180.0..=180.0).contains(&e.azimuth_degrees)
            || !(0.0..=90.0).contains(&e.incidence_degrees)
            || !v.yaw.is_finite()
            || !(-1.5..=1.5).contains(&v.pitch)
            || !v.offset.iter().all(|v| (0.0..=1.0).contains(v))
            || !(0.0..=3.0).contains(&s.normal_strength)
            || !(1.0..=12.0).contains(&s.bark_projection_sharpness)
            || !(1.0..=40.0).contains(&s.bark_fade_metres)
            || !(0.00005..=0.005).contains(&s.leaf_thickness_metres)
            || ![
                s.ao_strength,
                s.roughness,
                s.metallic,
                s.leaf_transmission,
                s.bark_branch_alignment,
                s.bark_parallax,
            ]
            .into_iter()
            .all(|v| (0.0..=1.0).contains(&v))
            || ![e.key_color, e.fill_color, e.background, s.pigment]
                .into_iter()
                .chain(s.litter_colors_linear)
                .flatten()
                .all(|v| (0.0..=1.0).contains(&v))
        {
            return Err("Viewing conditions are outside supported bounds".into());
        }
        Ok(document)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("finite validated document")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_review_views_round_trip_and_bound_lighting() {
        let mut document = Document::default();
        for shape in [Shape::FoldedSheet, Shape::Pane, Shape::CrownStrip] {
            document.view.shape = shape;
            document.view.backdrop_distance = 2.0;
            document.environment.reflection_strength = 700.0;
            let loaded = Document::from_json(&document.to_json()).unwrap();
            assert_eq!(loaded.view.shape, shape);
            assert_eq!(loaded.environment.reflection_strength, 700.0);
        }
        document.environment.reflection_strength = -1.0;
        assert!(Document::from_json(&document.to_json()).is_err());
        document.environment.reflection_strength = 700.0;
        document.view.backdrop_distance = 0.0;
        assert!(Document::from_json(&document.to_json()).is_err());
    }
    #[test]
    fn portable_document_round_trips_and_rejects_invalid_preview_values() {
        let mut document = Document::default();
        assert_eq!(
            Document::from_json(&document.to_json()).unwrap().to_json(),
            document.to_json()
        );
        document.surface.normal_strength = -1.0;
        assert!(Document::from_json(&document.to_json()).is_err());
        document.surface.normal_strength = 1.0;
        document.environment.fill_color[0] = 5.0;
        assert!(Document::from_json(&document.to_json()).is_err());
    }
}
