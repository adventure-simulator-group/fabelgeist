use super::*;

impl Package {
    pub(super) fn native_road_lines(&self) -> Vec<adventuresim_terrain::road_pack::NativeRoadLine> {
        self.routing_roads
            .iter()
            .map(|line| adventuresim_terrain::road_pack::NativeRoadLine {
                kind: line.kind,
                points: line.points.iter().map(|point| point.0).collect(),
            })
            .collect()
    }

    /// Native source-kernel handoff: full longitude/latitude degree pairs in
    /// canonical road order, shared by the road mask and vector package.
    pub(super) fn native_road_geometry(&self) -> Vec<Vec<[f64; 2]>> {
        self.routing_roads
            .iter()
            .map(|line| line.points.iter().map(|point| point.0).collect())
            .collect()
    }
}

pub(super) fn build(
    package: &Package,
    wetlands: Vec<Vec<Vec<[f64; 2]>>>,
    wetland_source_sha256: String,
) -> adventuresim_terrain::builder::Features {
    adventuresim_terrain::builder::Features {
        roads: package.native_road_geometry(),
        water: package
            .water
            .iter()
            .map(|polygon| {
                polygon
                    .rings
                    .iter()
                    .map(|ring| ring.iter().map(|point| point.0).collect())
                    .collect()
            })
            .collect(),
        wetlands,
        wetland_source_sha256,
        cultivated: Vec::new(),
        cultivation_source_sha256: format!("{:x}", Sha256::digest(b"no-cultivation")),
        cultivation_rules_version:
            adventuresim_world_import::cultivation::CULTIVATION_RULES_VERSION,
        terrain_features: Vec::new(),
    }
}

pub(super) fn finalize(
    package: &Package,
    wetlands: Vec<Vec<Vec<[f64; 2]>>>,
    wetland_source_sha256: String,
    cultivated: CultivatedLand,
    world: &CompiledWorld,
) -> adventuresim_terrain::builder::Features {
    let mut features = build(package, wetlands, wetland_source_sha256);
    features.terrain_features = world.terrain_features.clone();
    features.cultivated = cultivated.polygons;
    features.cultivation_source_sha256 = cultivated.source_sha256;
    features
}
