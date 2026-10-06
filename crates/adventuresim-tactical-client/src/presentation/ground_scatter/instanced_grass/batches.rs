//! Construct retained GPU instance batches from the procedural tuft lattice.
use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
struct PackedTufts(
    #[serde(with = "adventuresim_tactical_core::geometry_transport::binary")] Vec<InstanceData>,
);

#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::presentation) struct PreparedTufts(
    [[PackedTufts; GrassSpecies::ALL.len()]; TIERS.len()],
);

impl From<TierSpeciesBatches> for PreparedTufts {
    fn from(batches: TierSpeciesBatches) -> Self {
        Self(batches.map(|tier| tier.map(PackedTufts)))
    }
}

impl PreparedTufts {
    pub(in crate::presentation) fn to_batches(&self) -> TierSpeciesBatches {
        self.0
            .each_ref()
            .map(|tier| tier.each_ref().map(|batch| batch.0.clone()))
    }
}

/// Turns filled instance batches into one entity per (tier, species), each
/// carrying `marker` on top of the shared instanced-draw components.
pub(in crate::presentation) fn spawn_tuft_batches(
    world: GrassWorld<'_, '_, '_>,
    batches: &mut TierSpeciesBatches,
    label: &str,
    marker: impl Bundle + Clone,
    base_seed: u64,
    pigment: TuftPigment,
    grass: &crate::presentation::config::GrassConfig,
) {
    let GrassWorld {
        commands,
        meshes,
        materials,
    } = world;
    for lod in TIERS {
        let material = materials.add(diagnostics::material(
            lod,
            grass,
            pigment.density,
            pigment.dryness,
            pigment.wind_scale,
        ));
        for species in GrassSpecies::ALL {
            let instances = std::mem::take(&mut batches[lod.tier_index()][species.index()]);
            if instances.is_empty() {
                continue;
            }
            let (mesh, triangle_count) = diagnostics::add_mesh(
                meshes,
                grass_tuft_mesh(
                    pigment.color,
                    lod,
                    pigment.density,
                    species,
                    streams::TUFT_MESH
                        .seed(
                            base_seed.into(),
                            &[species.index() as u64, lod.tier_index() as u64],
                        )
                        .to_u64(),
                    grass,
                ),
            );
            let metadata = culling::metadata(
                meshes.get(&mesh).expect("generated tuft"),
                lod.width_compensation(pigment.density),
                pigment.wind_scale,
                grass.interaction.maximum_push,
            );
            let mut entity = commands.spawn((
                Name::new(format!(
                    "{label} {species:?} {lod:?} tufts ({})",
                    instances.len()
                )),
                GroundScatterLayer::Grass,
                triangle_count,
                GpuCullCompute,
                // Batches span the whole scene, so CPU frustum culling can
                // only ever hide them wholesale - and worse, a culled frame
                // drops the batch from `RenderMeshInstances`, which makes
                // eidolon free and re-upload the retained instance buffers
                // every time the camera pitch crosses the horizon. Culling
                // belongs solely to the GPU compute pass.
                NoFrustumCulling,
                Mesh3d(mesh),
                InstancedMeshMaterial(material.clone()),
                fitted_batch_aabb(&instances, configured_tuft_footprint_metres(lod, grass)),
                InstanceMaterialData {
                    instances: Arc::new(instances),
                    color: metadata,
                    visibility_range: configured_tier_visibility_range(lod, grass),
                },
                Transform::default(),
                Visibility::Inherited,
            ));
            entity.insert(marker.clone());
            if !diagnostics::casts_shadows(lod, grass) {
                entity.insert(NotShadowCaster);
            }
        }
    }
}
