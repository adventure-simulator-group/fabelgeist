//! Transport preserves the authority used to admit each floor region.
use super::*;
use crate::{scene_coordinates::ArchitecturalPlanProjection, scene_input::BuildingOrientation};
use adventuresim_building_generator::plan_geometry::{
    ArchitecturalPlanPoint, PlanGeometryError, PlanPolygon,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "FloorRegionWire", try_from = "FloorRegionWire")]
pub struct FloorRegion {
    outline: ScenePlanPolygon,
    admission: FloorRegionWire,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum FloorRegionWire {
    Scene {
        vertices: Vec<ScenePlanPoint>,
    },
    Architectural {
        vertices: Vec<ArchitecturalPlanPoint>,
        centre: ScenePlanPoint,
        origin: ArchitecturalPlanPoint,
        orientation: BuildingOrientation,
    },
}
impl From<FloorRegion> for FloorRegionWire {
    fn from(region: FloorRegion) -> Self {
        region.admission
    }
}
impl TryFrom<FloorRegionWire> for FloorRegion {
    type Error = PlanGeometryError;
    fn try_from(admission: FloorRegionWire) -> Result<Self, Self::Error> {
        let outline = match &admission {
            FloorRegionWire::Scene { vertices } => {
                ScenePlanPolygon::from_ordered_vertices(vertices.clone())?
            }
            FloorRegionWire::Architectural {
                vertices,
                centre,
                origin,
                orientation,
            } => ScenePlanPolygon::from_architectural(
                &PlanPolygon::from_ordered_vertices(vertices.clone())?,
                ArchitecturalPlanProjection {
                    centre: *centre,
                    origin: *origin,
                    orientation: *orientation,
                },
            )?,
        };
        Ok(Self { outline, admission })
    }
}
impl FloorRegion {
    pub fn from_scene(outline: ScenePlanPolygon) -> Result<Self, PlanGeometryError> {
        FloorRegionWire::Scene {
            vertices: outline.vertices().to_vec(),
        }
        .try_into()
    }
    pub fn from_architectural(
        source: &PlanPolygon<ArchitecturalPlanPoint>,
        projection: ArchitecturalPlanProjection,
    ) -> Result<Self, PlanGeometryError> {
        FloorRegionWire::Architectural {
            vertices: source.vertices().to_vec(),
            centre: projection.centre,
            origin: projection.origin,
            orientation: projection.orientation,
        }
        .try_into()
    }
    pub fn outline(&self) -> &ScenePlanPolygon {
        &self.outline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn architectural_transport_rebuilds_exact_projection_and_rejects_bad_source() {
        let p = |x, y| ArchitecturalPlanPoint::from_metres(Vec2::new(x, y)).unwrap();
        let source = PlanPolygon::from_ordered_vertices(vec![
            p(-2.0, -1.0),
            p(0.0, -1.0),
            p(2.0, -1.0),
            p(2.0, 1.0),
            p(-2.0, 1.0),
        ])
        .unwrap();
        let projection = ArchitecturalPlanProjection {
            centre: ScenePlanPoint::from_metres(Vec2::new(90.0, -350.0)).unwrap(),
            origin: p(0.0, 0.0),
            orientation: BuildingOrientation::from_radians(0.37).unwrap(),
        };
        let region = FloorRegion::from_architectural(&source, projection).unwrap();
        let expected = ScenePlanPolygon::from_architectural(&source, projection).unwrap();
        assert_eq!(region.outline(), &expected);
        let json = serde_json::to_value(&region).unwrap();
        assert_eq!(
            serde_json::from_value::<FloorRegion>(json.clone()).unwrap(),
            region
        );
        assert_eq!(
            postcard::from_bytes::<FloorRegion>(&postcard::to_allocvec(&region).unwrap()).unwrap(),
            region
        );
        let mut invalid = json;
        invalid["Architectural"]["vertices"] =
            serde_json::json!([[0.0, 0.0], [1.0, 0.0], [0.0, 0.0]]);
        assert!(serde_json::from_value::<FloorRegion>(invalid).is_err());
        let invalid = FloorRegionWire::Scene {
            vertices: vec![ScenePlanPoint::try_from(Vec2::ZERO).unwrap(); 3],
        };
        assert!(
            postcard::from_bytes::<FloorRegion>(&postcard::to_allocvec(&invalid).unwrap()).is_err()
        );
    }
}
