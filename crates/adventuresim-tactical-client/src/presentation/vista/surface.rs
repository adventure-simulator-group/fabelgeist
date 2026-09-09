//! Retained vista heights and the shared playable-to-vista surface boundary.

use super::*;

/// Keeps the closest vista rings for camera-local terrain refinement and city
/// ground seating. Every consumer queries the same stitched surface.
#[derive(Resource, Default, Clone)]
pub(in crate::presentation) struct ActiveVistaSurface {
    revision: u64,
    scene_digest: String,
    playable_half_extent: Vec2,
    lods: Vec<VistaLod>,
    urban: bool,
    ground_support: streets::GroundSupport,
}

impl ActiveVistaSurface {
    pub(in crate::presentation) fn update(&mut self, bundle: &SceneVistaBundle) {
        self.revision = self.revision.wrapping_add(1);
        self.scene_digest.clone_from(&bundle.scene_digest);
        self.playable_half_extent = bundle.playable_half_extent_metres;
        self.lods = bundle.lods.iter().take(2).cloned().collect();
        self.urban = !bundle.streets.is_empty() || !bundle.yards.is_empty();
        self.ground_support = default();
    }

    pub(in crate::presentation) fn is_urban_scene(&self, digest: &str) -> bool {
        self.urban && self.scene_digest == digest
    }

    pub(super) fn ground_support(&self) -> &streets::GroundSupport {
        &self.ground_support
    }

    pub(super) fn retain_playable(
        &mut self,
        terrain: &SceneTerrain,
        landform: Option<&TerrainLandformRecipe>,
    ) {
        if self.urban {
            self.ground_support.add_mesh(
                &super::super::terrain::urban_playable_mesh(terrain, landform),
                Vec3::ZERO,
            );
        }
    }

    pub(super) fn present_chunk(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        material: Handle<TacticalVistaMaterial>,
        mesh: Mesh,
        lod: &VistaLod,
        chunk: usize,
    ) {
        let origin = Vec3::new(
            lod.origin_east_metres as f32,
            0.0,
            lod.origin_north_metres as f32,
        );
        if self.urban {
            self.ground_support.add_mesh(&mesh, origin);
        }
        let triangle_count = mesh_triangle_count(&mesh);
        commands.spawn((
            Name::new(format!("Tactical vista LOD {} chunk {chunk}", lod.level)),
            VistaTerrain(lod.level),
            VistaTerrainMesh(lod.level),
            TerrainTriangleCount(triangle_count),
            NotShadowCaster,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(origin),
        ));
    }

    pub(in crate::presentation) fn revision(&self) -> u64 {
        self.revision
    }

    pub(in crate::presentation) fn presented_height_at(
        &self,
        scene_digest: &str,
        terrain: &SceneTerrain,
        local: Vec2,
    ) -> Option<f32> {
        if let Some(height) = terrain.height_at(local) {
            return Some(height);
        }
        if self.scene_digest != scene_digest {
            return None;
        }
        let lod = self.lods.first()?;
        let world = local
            + Vec2::new(
                lod.origin_east_metres as f32,
                lod.origin_north_metres as f32,
            );
        let vista_height = presented_height_at(lod, world, self.lods.get(1))?;
        Some(stitch_vista_height_to_playable_edge(
            terrain,
            local,
            self.playable_half_extent,
            lod.spacing_metres,
            vista_height,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_near_vista_surface_continues_detail_patch_across_playable_bounds() {
        let terrain =
            SceneTerrain::from_heightmap(3, 3, 2.0, vec![10.0; 9]).expect("playable terrain");
        let lod = VistaLod {
            level: 0,
            spacing_metres: 2.0,
            width: 5,
            depth: 5,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![20.0; 25],
            environment: vec![EnvironmentalSample::default(); 25],
        };
        let retained = ActiveVistaSurface {
            revision: 1,
            scene_digest: "boundary".into(),
            playable_half_extent: Vec2::splat(2.0),
            lods: vec![lod],
            urban: false,
            ground_support: default(),
        };
        assert_eq!(
            retained.presented_height_at("boundary", &terrain, Vec2::new(2.0, 0.0)),
            Some(10.0)
        );
        let outside = retained
            .presented_height_at("boundary", &terrain, Vec2::new(3.0, 0.0))
            .unwrap();
        assert!((outside - 15.0).abs() < 0.0001, "{outside}");
        assert_eq!(
            retained.presented_height_at("different", &terrain, Vec2::new(3.0, 0.0)),
            None
        );
    }
}
