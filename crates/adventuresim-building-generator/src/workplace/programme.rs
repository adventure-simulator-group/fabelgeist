use super::*;
use crate::{Footprint, RoomKind, RoomRequirement, StoreyProgram, WallStyle};

impl WorkplaceKind {
    pub(crate) const fn yard_width_metres(self) -> f32 {
        match self {
            Self::Stable | Self::Barn | Self::Malthouse | Self::Dyer => 4.5,
            Self::Smithy | Self::Bakehouse | Self::Brewery | Self::Tannery => 6.0,
            Self::Warehouse => 8.0,
            _ => 0.0,
        }
    }
}

impl BuildingProgram {
    pub(crate) fn validate_workplace_edits(
        &self,
        edits: &[crate::BuildingEdit],
    ) -> Result<(), crate::GenerationError> {
        if self.archetype == crate::BuildingArchetype::Workplace
            && (self.workplace_kind().is_none() || self.workplace_size.is_none())
        {
            return Err(crate::GenerationError::InvalidWorkplaceProgram);
        }
        if self.workplace_kind().is_some() && !edits.is_empty() {
            return Err(crate::GenerationError::UnsupportedEdit(
                "working-building bays must be edited through their structural programme"
                    .to_owned(),
            ));
        }
        Ok(())
    }

    pub(crate) fn workplace_roofs(&self) -> Option<Vec<crate::RoofPiece>> {
        use crate::{GableProfile, RidgeAxis, RoofKind, RoofPiece};
        let kind = self.workplace_kind()?;
        let (width, depth) = self.footprint.dimensions();
        let size = Vec2::new(f32::from(width), f32::from(depth)) * crate::CELL_SIZE_METRES;
        let mut roofs = vec![RoofPiece {
            kind: RoofKind::Gable,
            centre: size * 0.5,
            size,
            base_height_metres: self.storey_height_metres,
            pitch_degrees: self.roof_pitch_degrees,
            ridge_axis: RidgeAxis::Z,
            eave_metres: 0.35,
            gable_profile: GableProfile::Plain,
        }];
        if matches!(kind, WorkplaceKind::Barn | WorkplaceKind::Stable) {
            roofs.push(RoofPiece {
                kind: RoofKind::Gable,
                centre: Vec2::new(size.x + 3.0, size.y - 2.4),
                size: Vec2::new(2.4, 3.6),
                base_height_metres: 2.2,
                pitch_degrees: 35.0,
                ridge_axis: RidgeAxis::Z,
                eave_metres: 0.15,
                gable_profile: GableProfile::Plain,
            });
        }
        if kind == WorkplaceKind::Brewery {
            roofs.push(super::brewing::service_roof(size));
        }
        if kind == WorkplaceKind::Warehouse {
            roofs.push(super::warehouse::loading_roof(size));
        }
        if kind == WorkplaceKind::Tannery {
            roofs.push(super::wet::service_roof(size));
        }
        Some(roofs)
    }

    /// Select the physical programme before reserving its plot or compiling any representation.
    pub fn with_workplace_size(mut self, size: WorkplaceSize) -> Self {
        let Some(kind) = self.workplace_kind() else {
            return self;
        };
        self.archetype = crate::BuildingArchetype::Workplace;
        self.workplace_size = Some(size);
        let (width, depth, height, pitch, room) = match kind {
            WorkplaceKind::Barn => (8, 10, 4.2, 48.0, RoomKind::Storage),
            WorkplaceKind::Stable => (7, 10, 3.3, 40.0, RoomKind::Stalls),
            WorkplaceKind::Granary => (6, 7, 6.4, 53.0, RoomKind::Storage),
            WorkplaceKind::Smithy => (6, 6, 3.2, 42.0, RoomKind::Workshop),
            WorkplaceKind::Bakehouse => (5, 6, 3.1, 48.0, RoomKind::KilnRoom),
            WorkplaceKind::MarketHall => (8, 10, 4.0, 45.0, RoomKind::GreatHall),
            WorkplaceKind::Brewery => (7, 8, 3.6, 43.0, RoomKind::VatRoom),
            WorkplaceKind::Malthouse => (8, 10, 3.1, 36.0, RoomKind::KilnRoom),
            WorkplaceKind::TimberYard => (8, 10, 3.7, 32.0, RoomKind::Storage),
            WorkplaceKind::Carpenter => (7, 7, 3.7, 47.0, RoomKind::Workshop),
            WorkplaceKind::Warehouse => (7, 14, 6.4, 47.0, RoomKind::Storage),
            WorkplaceKind::Dyer => (6, 8, 3.2, 40.0, RoomKind::VatRoom),
            WorkplaceKind::Tannery => (6, 8, 2.8, 32.0, RoomKind::VatRoom),
        };
        let depth = depth + size.extra_bays() * 2;
        self.footprint = Footprint::Rectangle { width, depth };
        self.storey_height_metres = height;
        self.roof_pitch_degrees = pitch + (self.seed % 5) as f32;
        self.storeys = vec![StoreyProgram {
            rooms: vec![RoomRequirement::new(room, width * depth)],
        }];
        self.vertical_connections.clear();
        self.timber_frame_style = None;
        self.upper_storey_projection_metres = 0.0;
        self.wall_style = if matches!(
            kind,
            WorkplaceKind::Smithy
                | WorkplaceKind::Bakehouse
                | WorkplaceKind::Granary
                | WorkplaceKind::Brewery
                | WorkplaceKind::Malthouse
                | WorkplaceKind::Warehouse
                | WorkplaceKind::Dyer
        ) {
            WallStyle::Stone
        } else {
            WallStyle::TimberFrame
        };
        self
    }
}
