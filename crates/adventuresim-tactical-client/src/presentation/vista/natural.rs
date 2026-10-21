//! Existing sampled-ring presentation for unoccupied geographic scenes.
use super::*;
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use adventuresim_tactical_core::scene::TerrainSupportError;

struct SampledVistaMesher<'a> {
    lod: &'a VistaLod,
    inner_half_extent: Vec2,
    coarser_lod: Option<&'a VistaLod>,
    playable_terrain: Option<&'a SceneTerrain>,
    colors: VistaVertexColors<'a>,
    normal_sample_distance: PositiveLength,
}

impl SampledVistaMesher<'_> {
    fn chunk(
        &self,
        chunk_x: usize,
        chunk_z: usize,
        chunk_cells: usize,
    ) -> VistaMeshResult<Option<Mesh>> {
        let width = usize::from(self.lod.width);
        let depth = usize::from(self.lod.depth);
        let center_x = (width - 1) as f32 * 0.5;
        let center_z = (depth - 1) as f32 * 0.5;
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        for z in chunk_z..(chunk_z + chunk_cells).min(depth - 1) {
            for x in chunk_x..(chunk_x + chunk_cells).min(width - 1) {
                let cell_min = Vec2::new(
                    (x as f32 - center_x) * self.lod.spacing_metres,
                    (z as f32 - center_z) * self.lod.spacing_metres,
                );
                let cell_max = cell_min + Vec2::splat(self.lod.spacing_metres);
                for rectangle in cell_rectangles_outside_inner_rectangle(
                    cell_min,
                    cell_max,
                    self.inner_half_extent,
                ) {
                    for [minimum_x, maximum_x, minimum_z, maximum_z] in
                        subdivide_playable_boundary_rectangle(
                            rectangle,
                            self.inner_half_extent,
                            self.playable_terrain,
                        )
                    {
                        let base = positions.len() as u32;
                        let vertices = [
                            self.vertex(PlanDisplacement::try_from(Vec2::new(
                                minimum_x, minimum_z,
                            ))?)?,
                            self.vertex(PlanDisplacement::try_from(Vec2::new(
                                maximum_x, minimum_z,
                            ))?)?,
                            self.vertex(PlanDisplacement::try_from(Vec2::new(
                                maximum_x, maximum_z,
                            ))?)?,
                            self.vertex(PlanDisplacement::try_from(Vec2::new(
                                minimum_x, maximum_z,
                            ))?)?,
                        ];
                        positions.extend(vertices.map(|vertex| vertex.0));
                        normals.extend(vertices.map(|vertex| vertex.1));
                        colors.extend(vertices.map(|vertex| vertex.2));
                        indices.extend_from_slice(&[
                            base,
                            base + 2,
                            base + 1,
                            base,
                            base + 3,
                            base + 2,
                        ]);
                    }
                }
            }
        }
        if positions.is_empty() {
            return Ok(None);
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        Ok(Some(mesh))
    }
    // The positional arrays are Bevy's native vertex attribute encodings.
    fn vertex(&self, local: PlanDisplacement) -> VistaMeshResult<([f32; 3], [f32; 3], [f32; 4])> {
        let point = self.lod.scene_point(local)?;
        let geometry = VistaVertexGeometry::sample(
            self.lod,
            self.coarser_lod,
            self.playable_terrain,
            local,
            self.inner_half_extent,
            self.normal_sample_distance,
        )
        .ok_or(TerrainSupportError::MissingSupport { point })?;
        let tangent_x = geometry.east_tangent().metres();
        let tangent_z = geometry.north_tangent().metres();
        let normal = tangent_z
            .cross(tangent_x)
            .try_normalize()
            .ok_or(VistaMeshError::DegenerateNormal { point })?;
        let color = self
            .colors
            .at(local, self.inner_half_extent)
            .ok_or(VistaMeshError::MissingColor { point })?;
        let local = local.metres();
        Ok((
            [local.x, geometry.elevation().metres(), local.y],
            normal.to_array(),
            color,
        ))
    }
}

pub(super) fn sampled_vista_lod_meshes_with_morph(
    lod: &VistaLod,
    inner_half_extent: Vec2,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: Option<&SceneTerrain>,
    playable_environment: Option<&SceneEnvironment>,
    weather: WeatherSnapshot,
) -> VistaMeshResult<Vec<Mesh>> {
    validate_source(lod)?;
    if let Some(coarser) = coarser_lod {
        validate_source(coarser)?;
    }
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    let mesher = SampledVistaMesher {
        lod,
        inner_half_extent,
        coarser_lod,
        playable_terrain,
        colors: VistaVertexColors::new(lod, coarser_lod, playable_environment, weather),
        normal_sample_distance: PositiveLength::from_metres(lod.spacing_metres.min(100.0))?,
    };
    // Small rings keep at least two chunks across the longer axis for culling.
    let chunk_cells = VISTA_CHUNK_CELLS.min((width.max(depth) - 1).div_ceil(2).max(1));
    let mut meshes = Vec::new();
    for z in (0..depth - 1).step_by(chunk_cells) {
        for x in (0..width - 1).step_by(chunk_cells) {
            if let Some(mesh) = mesher.chunk(x, z, chunk_cells)? {
                meshes.push(mesh);
            }
        }
    }
    Ok(meshes)
}

pub(super) fn validate_source(lod: &VistaLod) -> VistaMeshResult<()> {
    if !lod.has_height_grid() || lod.environment.len() != lod.heights_metres.len() {
        return Err(VistaMeshError::InvalidLod { level: lod.level });
    }
    Ok(())
}
