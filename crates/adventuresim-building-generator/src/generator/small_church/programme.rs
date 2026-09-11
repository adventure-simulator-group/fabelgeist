use super::*;

impl BuildingProgram {
    pub(crate) fn configure_small_church_size(&mut self, size: ServiceBuildingSize) {
        self.service_size = Some(size);
        let dimensions = Dimensions::from_program(self)
            .expect("small church requires an explicit chapel or parish use");
        self.footprint = Footprint::Rectangle {
            width: dimensions.width_cells,
            depth: dimensions.nave_cells + dimensions.chancel_cells,
        };
        self.storey_height_metres = dimensions.nave_eave;
        self.roof_pitch_degrees = match dimensions.kind {
            SmallChurchKind::Chapel => 48.0,
            SmallChurchKind::Parish => 52.0,
        };
        self.storeys = vec![crate::StoreyProgram {
            rooms: vec![
                RoomRequirement::new(
                    RoomKind::Nave,
                    dimensions.width_cells * dimensions.nave_cells,
                )
                .exterior(),
            ],
        }];
        if dimensions.chancel_cells > 0 {
            self.storeys[0].rooms.push(
                RoomRequirement::new(
                    RoomKind::Chancel,
                    (dimensions.width_cells - 2) * dimensions.chancel_cells,
                )
                .exterior(),
            );
        }
        self.vertical_connections.clear();
        self.timber_frame_style = None;
        self.upper_storey_projection_metres = 0.0;
        self.church_program = None;
        self.wall_style = WallStyle::Stone;
    }
}

impl Dimensions {
    pub(super) fn roofs(self, pitch: f32) -> Vec<RoofPiece> {
        let mut roofs = vec![RoofPiece {
            kind: RoofKind::Gable,
            centre: Vec2::new(self.width() * 0.5, self.nave_depth() * 0.5),
            size: Vec2::new(self.width(), self.nave_depth()),
            base_height_metres: self.nave_eave,
            pitch_degrees: pitch,
            ridge_axis: RidgeAxis::Z,
            eave_metres: 0.35,
            gable_profile: GableProfile::Plain,
        }];
        if self.chancel_cells > 0 {
            let depth = self.depth() - self.nave_depth();
            // The front verge ends inside the nave's rear masonry, rather
            // than overhanging freely into its occupied interior.
            let front_inset = roofs[0].eave_metres;
            roofs.push(RoofPiece {
                centre: Vec2::new(
                    self.width() * 0.5,
                    self.nave_depth() + (depth + front_inset) * 0.5,
                ),
                size: Vec2::new(self.width() - 2.0 * CELL_SIZE_METRES, depth - front_inset),
                base_height_metres: self.chancel_eave,
                pitch_degrees: pitch - 4.0,
                ..roofs[0]
            });
        }
        roofs.push(RoofPiece {
            kind: RoofKind::Pavilion,
            centre: self.bell_centre(),
            size: Vec2::splat(2.1),
            base_height_metres: self.bell_floor(pitch) + 1.65,
            pitch_degrees: 55.0,
            ridge_axis: RidgeAxis::Z,
            eave_metres: 0.12,
            gable_profile: GableProfile::Plain,
        });
        roofs
    }
}
