//! Pure street clipping and retained upload-ready city-ground batches.
use super::*;
use std::sync::OnceLock;

type InstalledGround = Vec<(traffic::TrafficTile, CityGroundKind, Handle<Mesh>, usize)>;

#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::presentation) struct PreparedCityGround {
    batches: Vec<(traffic::TrafficTile, CityGroundKind, mesh::SurfaceVertices)>,
    #[serde(skip)]
    installed: OnceLock<InstalledGround>,
    #[serde(skip)]
    pub(super) traffic: streaming::RetainedTrafficMasks,
}

impl PreparedCityGround {
    pub(super) fn new(
        streets: &[CityStreetPatch],
        yards: &[CityYardPatch],
        groups: &[FurnitureGroup],
        support: &GroundSupport,
    ) -> Self {
        let mut builders: [CitySurfaceMeshBuilder; 5] = Default::default();
        let beds = yards
            .iter()
            .filter(|yard| yard.surface == CityYardSurface::KitchenGarden)
            .map(|yard| yard.corners_metres)
            .collect::<Vec<_>>();
        let bed_index = partition::SpatialIndex::new(&beds, |bed| partition::bounds(*bed));
        let group_index = partition::SpatialIndex::new(groups, |group| {
            partition::bounds(group.footprint.corners())
        });
        for yard in yards.iter().copied() {
            builders[CityGroundKind::from(yard.surface).index()].append_yard(
                yard,
                &bed_index,
                support,
                &group_index,
            );
        }
        for street in streets.iter().copied() {
            builders[CityGroundKind::from(street.surface()).index()].append_street(
                street,
                support,
                &group_index,
            );
        }
        Self {
            batches: builders
                .into_iter()
                .zip(CityGroundKind::ALL)
                .flat_map(|(builder, kind)| {
                    builder
                        .into_vertices()
                        .map(move |(tile, vertices)| (tile, kind, vertices))
                })
                .collect(),
            installed: OnceLock::new(),
            traffic: Default::default(),
        }
    }

    pub(in crate::presentation) fn from_scene(
        input: &TacticalSceneInput,
        terrain: &SceneTerrain,
        groups: &[FurnitureGroup],
        maximum_lods: usize,
    ) -> Self {
        let mut support = GroundSupport::default();
        support.add_mesh(
            &crate::presentation::terrain::urban_playable_mesh(terrain, input.landform.as_ref()),
            Vec3::ZERO,
        );
        let environment = input.environment_snapshot(input.digest().expect("validated scene"));
        let lods = input
            .vista
            .lods
            .iter()
            .take(maximum_lods)
            .collect::<Vec<_>>();
        let mut inner = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        for (index, lod) in lods.iter().copied().enumerate() {
            let origin = Vec3::new(
                lod.origin_east_metres as f32,
                0.0,
                lod.origin_north_metres as f32,
            );
            for mesh in super::super::vista_lod_meshes_with_morph(
                lod,
                inner,
                lods.get(index + 1).copied(),
                (index == 0).then_some(terrain),
                (index == 0).then_some(&environment),
                environment.weather,
            ) {
                support.add_mesh(&mesh, origin);
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        Self::new(&input.streets, &input.yards, groups, &support)
    }

    pub(super) fn meshes(&self, meshes: &mut Assets<Mesh>) -> &InstalledGround {
        self.installed.get_or_init(|| {
            self.batches
                .iter()
                .map(|(tile, kind, vertices)| {
                    let mesh = vertices.clone().build();
                    let triangles = mesh_triangle_count(&mesh);
                    (*tile, *kind, meshes.add(mesh), triangles)
                })
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_transport_preserves_geometry_and_return_visit_reuses_mesh_handles() {
        let terrain = SceneTerrain::from_heightmap(
            3,
            3,
            8.0,
            vec![0.0, 1.0, 2.0, 1.0, 2.0, 3.0, 2.0, 3.0, 4.0],
        )
        .unwrap();
        let mut support = GroundSupport::default();
        support.add_mesh(
            &crate::presentation::terrain::urban_playable_mesh(&terrain, None),
            Vec3::ZERO,
        );
        let street = CityStreetPatch::Corridor {
            start_metres: Vec2::new(-6.0, 0.0),
            end_metres: Vec2::new(6.0, 0.0),
            half_width_metres: 2.0,
            surface: CityStreetSurface::Fieldstone,
        };
        let prepared = PreparedCityGround::new(&[street], &[], &[], &support);
        let mut bytes = Vec::new();
        ciborium::into_writer(&prepared, &mut bytes).unwrap();
        let restored: PreparedCityGround = ciborium::from_reader(bytes.as_slice()).unwrap();
        let mut assets = Assets::<Mesh>::default();
        let original = prepared.meshes(&mut assets);
        let decoded = restored.meshes(&mut assets);
        assert!(!original.is_empty());
        assert_eq!(original.len(), decoded.len());
        for (a, b) in original.iter().zip(decoded) {
            let a = assets.get(&a.2).unwrap();
            let b = assets.get(&b.2).unwrap();
            for attribute in [
                Mesh::ATTRIBUTE_POSITION,
                Mesh::ATTRIBUTE_NORMAL,
                Mesh::ATTRIBUTE_UV_0,
                Mesh::ATTRIBUTE_UV_1,
                Mesh::ATTRIBUTE_COLOR,
            ] {
                assert_eq!(
                    a.attribute(attribute).unwrap().get_bytes(),
                    b.attribute(attribute).unwrap().get_bytes()
                );
            }
        }
        let handles = decoded.iter().map(|b| b.2.clone()).collect::<Vec<_>>();
        let count = assets.len();
        assert_eq!(
            restored
                .meshes(&mut assets)
                .iter()
                .map(|b| b.2.clone())
                .collect::<Vec<_>>(),
            handles
        );
        assert_eq!(assets.len(), count, "return creates no new ground meshes");
    }
}
