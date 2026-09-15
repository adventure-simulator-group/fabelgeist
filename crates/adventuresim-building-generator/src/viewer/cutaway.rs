//! A horizontal section of canonical ground-floor wall and frame meshes.
use super::*;
use adventuresim_building_generator::BuildingLodMaterial;

pub(super) const CUT_HEIGHT_METRES: f32 = 1.25;

pub(super) fn spawn(world: &mut World, palette: &RenderPalette, plan: &BuildingPlan, origin: Vec2) {
    let mut bounds = sample_polygon::SampleBounds {
        min: Vec3::splat(f32::INFINITY),
        max: Vec3::splat(f32::NEG_INFINITY),
    };
    for batch in adventuresim_building_generator::compile_building_detail(plan).meshes {
        let material = match batch.material {
            BuildingLodMaterial::Wall(
                adventuresim_building_generator::WallMaterialClass::TimberInfill,
            )
            | BuildingLodMaterial::InteriorPlaster => &palette.plaster,
            BuildingLodMaterial::Wall(_) => &palette.stone,
            BuildingLodMaterial::Timber
            | BuildingLodMaterial::InteriorTimber
            | BuildingLodMaterial::TimberEndGrain => &palette.timber,
            BuildingLodMaterial::Glass => &palette.glass,
            BuildingLodMaterial::Iron | BuildingLodMaterial::LeadAlloy => &palette.roof_secondary,
            _ => continue,
        };
        let faces = batch
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .filter_map(|indices| {
                let polygon = clip(indices.map(|index| batch.vertices[index as usize].position));
                (polygon.len() >= 3).then_some(polygon)
            })
            .collect::<Vec<_>>();
        if faces.is_empty() {
            continue;
        }
        for point in faces.iter().flatten() {
            let world_point = *point + Vec3::new(origin.x, 0.0, origin.y);
            bounds.min = bounds.min.min(world_point);
            bounds.max = bounds.max.max(world_point);
        }
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(flat_face_mesh(&faces));
        world.spawn((
            Name::new("canonical ground-floor section"),
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(origin.x, 0.0, origin.y),
            EditorBuildingEntity,
        ));
    }
    spawn_caps(world, palette, plan, origin);
    world.insert_resource(bounds);
    world.spawn((
        Name::new("architectural section authority annotation"),
        Text::new(format!("Horizontal section at {CUT_HEIGHT_METRES:.2} m | canonical walls and frame | colors identify rooms")),
        TextFont { font_size: bevy::text::FontSize::Px(16.0), ..default() },
        TextColor(Color::BLACK),
        Node { position_type: PositionType::Absolute, left: px(24), bottom: px(24), ..default() },
        EditorBuildingEntity,
    ));
}

fn spawn_caps(world: &mut World, palette: &RenderPalette, plan: &BuildingPlan, origin: Vec2) {
    use adventuresim_building_generator::ResolvedSolidShape;
    for solid in &plan.resolved_geometry.solids {
        if !matches!(
            solid.shape,
            ResolvedSolidShape::Cuboid | ResolvedSolidShape::TimberPanelPrism { .. }
        ) {
            continue;
        }
        let detail = adventuresim_building_generator::compile_solid_detail(plan, solid);
        let mut section = Vec::<Vec3>::new();
        for batch in &detail.meshes {
            for indices in batch.indices.as_chunks::<3>().0 {
                let triangle = indices.map(|index| batch.vertices[index as usize].position);
                for edge in 0..3 {
                    let a = triangle[edge];
                    let b = triangle[(edge + 1) % 3];
                    if (a.y < CUT_HEIGHT_METRES) != (b.y < CUT_HEIGHT_METRES) {
                        let point = a.lerp(b, (CUT_HEIGHT_METRES - a.y) / (b.y - a.y));
                        if !section
                            .iter()
                            .any(|old| old.distance_squared(point) < 0.000_001)
                        {
                            section.push(point);
                        }
                    }
                }
            }
        }
        if section.len() < 3 {
            continue;
        }
        let centre = section.iter().sum::<Vec3>() / section.len() as f32;
        section.sort_by(|a, b| {
            (a.z - centre.z)
                .atan2(a.x - centre.x)
                .total_cmp(&(b.z - centre.z).atan2(b.x - centre.x))
        });
        section.reverse();
        let timber = detail.meshes.iter().any(|batch| {
            matches!(
                batch.material,
                BuildingLodMaterial::Timber | BuildingLodMaterial::InteriorTimber
            )
        });
        let material = if timber {
            &palette.timber
        } else {
            &palette.stone
        };
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(flat_face_mesh(&[section]));
        world.spawn((
            Name::new("architectural section cap"),
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(origin.x, 0.0, origin.y),
            EditorBuildingEntity,
        ));
    }
}

fn clip(triangle: [Vec3; 3]) -> Vec<Vec3> {
    let mut result = Vec::new();
    for edge in 0..3 {
        let a = triangle[edge];
        let b = triangle[(edge + 1) % 3];
        let inside = a.y <= CUT_HEIGHT_METRES;
        if inside {
            result.push(a);
        }
        if inside != (b.y <= CUT_HEIGHT_METRES) {
            result.push(a.lerp(b, (CUT_HEIGHT_METRES - a.y) / (b.y - a.y)));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_section_retains_lower_faces_and_cuts_at_one_height() {
        let vertices = clip([Vec3::ZERO, Vec3::X, Vec3::new(1.0, 3.0, 0.0)]);
        assert_eq!(vertices.len(), 4);
        assert!(vertices.iter().all(|p| p.y <= CUT_HEIGHT_METRES));
        assert_eq!(
            vertices.iter().filter(|p| p.y == CUT_HEIGHT_METRES).count(),
            2
        );
        assert!(
            clip([
                Vec3::Y * 3.0,
                Vec3::new(1.0, 3.0, 0.0),
                Vec3::new(0.0, 4.0, 0.0)
            ])
            .is_empty()
        );
    }
}
