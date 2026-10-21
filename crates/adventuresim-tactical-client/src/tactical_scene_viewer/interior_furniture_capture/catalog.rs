//! Production catalogue staging and envelope framing for the viewer.
use super::*;

pub(in crate::tactical_scene_viewer) fn setup_catalog(
    commands: &mut Commands,
    layout: &mut FurnitureLayout,
    terrain: &SceneTerrain,
    profile: &str,
    output: &std::path::Path,
) -> Result<Option<Vec<BuildingReviewCamera>>> {
    if profile != PROFILE {
        return Ok(None);
    }
    if !layout.instances.is_empty() {
        return Err(CatalogAdmissionError::AlreadyPlaced.into());
    }
    commands.insert_resource(crate::presentation::InteriorFurnitureExhibition);
    let mut cameras = Vec::new();
    for (index, kind) in FurnitureKind::INTERIOR.into_iter().enumerate() {
        let bay = Vec2::new(
            (index % CATALOG_COLUMNS) as f32 - 2.0,
            (index / CATALOG_COLUMNS) as f32 - 3.0,
        ) * CATALOG_BAY_SPACING_METRES;
        let keys = FurnitureVariant::ALL.map(|variant| FurnitureKey::natural(kind, variant));
        let [compact, broad] = keys.map(|key| {
            key.interior_spec()
                .map(|spec| spec.size_metres.metres())
        });
        let sizes = [compact?, broad?];
        let pair_width = sizes[0].x + sizes[1].x + catalog_pair_gap(kind);
        for (variant, key) in keys.into_iter().enumerate() {
            let x = if variant == 0 {
                -pair_width * 0.5 + sizes[0].x * 0.5
            } else {
                pair_width * 0.5 - sizes[1].x * 0.5
            };
            let point = bay + Vec2::X * x;
            layout.instances.push(GeneratedFurniture {
                scene: SceneFurniture {
                    id: FurnitureInstanceId(
                        CATALOG_INSTANCE_ID_BASE + (index * 2 + variant) as u64,
                    ),
                    key,
                    // Specimens have no real building, but use the same interior location payload.
                    location: FurnitureLocation::Interior {
                        building_id: (0).into(),
                        room_id: adventuresim_building_generator::RoomIndex::from_serialized(
                            index as u16,
                        ),
                        storey: adventuresim_building_generator::StoreyIndex::from_serialized(0),
                    },
                },
                position_metres:
                    adventuresim_building_generator::spatial_geometry::Position::from_metres(
                        Vec3::new(
                            point.x,
                            terrain
                                .required_height_at(ScenePlanPoint::try_from(point)?)?
                                .metres(),
                            point.y,
                        ),
                    )?,
                orientation: BuildingOrientation::from_radians(std::f32::consts::PI)
                    .ok_or(CatalogAdmissionError::InvalidOrientation)?,
            });
        }
        let envelope = Vec3::new(
            pair_width,
            sizes[0].y.max(sizes[1].y),
            sizes[0].z.max(sizes[1].z),
        );
        let ground = Vec3::new(
            bay.x,
            terrain
                .required_height_at(ScenePlanPoint::try_from(bay)?)?
                .metres(),
            bay.y,
        );
        cameras.push(catalog_camera(kind, envelope, ground));
    }
    cameras.push(joined_counters(layout, terrain)?);
    cameras.extend(finishes::stage(layout, terrain)?);
    expect_furniture(commands, layout)?;
    std::fs::write(
        output.join("interior-catalog.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({ "left_variant": "Compact", "right_variant": "Broad", "finish_order": ["natural", "handled", "repaired", "painted"],
            "instances": layout.instances, "note": "Production furniture models staged as catalog specimens; building_id zero is reserved for these captures." }),
        )?,
    )?;
    Ok(Some(cameras))
}

pub(super) fn catalog_camera(
    kind: FurnitureKind,
    size: Vec3,
    ground: Vec3,
) -> BuildingReviewCamera {
    let side = if kind == FurnitureKind::BunkBed {
        1.1
    } else {
        0.24
    };
    let outward = Vec3::new(side, 0.5, 1.0).normalize();
    let right = Vec3::Y.cross(outward).normalize();
    let up = outward.cross(right);
    let tangent_y = (CATALOG_FOV_DEGREES.to_radians() * 0.5).tan();
    let tangent_x = tangent_y * super::super::capture_resolution::CAPTURE_ASPECT_RATIO;
    let target = ground + Vec3::Y * size.y * 0.5;
    // Fit every envelope corner in the actual perspective frustum, including tall beds' feet.
    let distance = envelope_corners(size)
        .into_iter()
        .map(|corner| {
            let point = corner - Vec3::Y * size.y * 0.5;
            point.dot(outward)
                + CATALOG_FRAMING_MARGIN
                    * (point.dot(right).abs() / tangent_x).max(point.dot(up).abs() / tangent_y)
        })
        .fold(0.0_f32, f32::max);
    BuildingReviewCamera {
        position: target + outward * distance,
        target,
        plaster_raking_light: None,
    }
}

pub(super) fn catalog_pair_gap(kind: FurnitureKind) -> f32 {
    // The side-on ladder proof needs room between the deep bed frames in projection.
    if kind == FurnitureKind::BunkBed {
        2.4
    } else {
        CATALOG_PAIR_GAP_METRES
    }
}

fn joined_counters(
    layout: &mut FurnitureLayout,
    terrain: &SceneTerrain,
) -> Result<BuildingReviewCamera> {
    use FurnitureKind::*;
    let variant = FurnitureVariant::Broad;
    let size = FurnitureKey::natural(Counter, variant)
        .interior_spec()?
        .size_metres
        .metres();
    let corner = FurnitureKey::natural(CounterCorner, variant)
        .interior_spec()?
        .size_metres
        .metres();
    let first = (corner.x + size.x) * 0.5;
    let second = first + size.x;
    let bay = Vec2::new(80.0, 60.0);
    let assembly = [
        (CounterCorner, Vec2::ZERO, 0.0),
        (Counter, Vec2::new(first, 0.0), 0.0),
        (CounterRightEnd, Vec2::new(second, 0.0), 0.0),
        (Counter, Vec2::new(0.0, first), std::f32::consts::FRAC_PI_2),
        (
            CounterLeftEnd,
            Vec2::new(0.0, second),
            std::f32::consts::FRAC_PI_2,
        ),
    ];
    let ground = terrain.required_height_at(ScenePlanPoint::try_from(bay)?)?;
    for (index, (kind, local, yaw)) in assembly.into_iter().enumerate() {
        let point = bay - local;
        layout.instances.push(GeneratedFurniture {
            scene: SceneFurniture {
                id: FurnitureInstanceId(
                    CATALOG_INSTANCE_ID_BASE + (CATALOG.len() * 2 + index) as u64,
                ),
                key: FurnitureKey::natural(kind, variant),
                location: FurnitureLocation::Interior {
                    building_id: (0).into(),
                    room_id: adventuresim_building_generator::RoomIndex::from_serialized(
                        CATALOG.len() as u16,
                    ),
                    storey: adventuresim_building_generator::StoreyIndex::from_serialized(0),
                },
            },
            position_metres:
                adventuresim_building_generator::spatial_geometry::Position::from_metres(
                    Vec3::new(point.x, ground.metres(), point.y),
                )?,
            orientation: BuildingOrientation::from_radians(yaw + std::f32::consts::PI)
                .ok_or(CatalogAdmissionError::InvalidOrientation)?,
        });
    }
    let target = Vec3::new(bay.x - first, ground.metres() + size.y * 0.5, bay.y - first);
    Ok(BuildingReviewCamera {
        position: target + Vec3::new(5.5, 4.5, 6.0),
        target,
        plaster_raking_light: None,
    })
}
