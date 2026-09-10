//! Furniture placement envelopes are authored independently of mesh compilation.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FurnitureAccessFace {
    Front,
    Back,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct InteriorFurnitureSpec {
    pub size_metres: Vec3,
    pub required_faces: &'static [FurnitureAccessFace],
}

impl InteriorFurnitureSpec {
    pub const ACCESS_DEPTH_METRES: f32 = 0.75;
    pub fn access_bounds(self, face: FurnitureAccessFace) -> CollisionBounds {
        let half = self.size_metres * 0.5;
        let depth = Self::ACCESS_DEPTH_METRES;
        let (min, max) = match face {
            FurnitureAccessFace::Front => (
                Vec3::new(-half.x, 0.0, -half.z - depth),
                Vec3::new(half.x, 1.9, -half.z),
            ),
            FurnitureAccessFace::Back => (
                Vec3::new(-half.x, 0.0, half.z),
                Vec3::new(half.x, 1.9, half.z + depth),
            ),
            FurnitureAccessFace::Left => (
                Vec3::new(-half.x - depth, 0.0, -half.z),
                Vec3::new(-half.x, 1.9, half.z),
            ),
            FurnitureAccessFace::Right => (
                Vec3::new(half.x, 0.0, -half.z),
                Vec3::new(half.x + depth, 1.9, half.z),
            ),
        };
        CollisionBounds { min, max }
    }
}

impl FurnitureKey {
    pub fn interior_spec(self) -> Option<InteriorFurnitureSpec> {
        use FurnitureAccessFace::*;
        use FurnitureKind::*;
        let (small, broad, required_faces): (_, _, &[FurnitureAccessFace]) = match self.kind {
            DiningTable => ([1.5, 0.78, 0.8], [2.2, 0.78, 0.9], &[Left]),
            Bench => ([1.2, 0.48, 0.4], [1.8, 0.48, 0.4], &[Front]),
            Chair => ([0.5, 0.95, 0.52], [0.6, 1.05, 0.58], &[Front]),
            Stool => ([0.4, 0.46, 0.4], [0.48, 0.48, 0.48], &[Front]),
            Bed => ([1.0, 0.85, 2.0], [1.5, 0.95, 2.1], &[Left]),
            BunkBed => ([1.0, 1.75, 2.0], [1.2, 1.9, 2.1], &[Left]),
            StorageChest => ([0.9, 0.55, 0.5], [1.3, 0.65, 0.6], &[Front]),
            Cupboard => ([0.9, 1.65, 0.5], [1.4, 1.85, 0.6], &[Front]),
            Shelving => ([1.0, 1.65, 0.4], [1.6, 1.85, 0.5], &[Front]),
            WritingDesk => ([1.1, 0.8, 0.65], [1.6, 0.8, 0.75], &[Front]),
            Lectern => ([0.65, 1.2, 0.6], [0.85, 1.3, 0.7], &[Front]),
            ChurchBench => ([1.5, 0.95, 0.55], [2.4, 1.0, 0.6], &[Front]),
            Altar => ([1.5, 1.0, 0.8], [2.2, 1.05, 1.0], &[Front]),
            WardBed => ([0.9, 0.85, 2.0], [1.1, 0.9, 2.1], &[Left, Right]),
            BathTub => ([1.0, 0.7, 1.6], [1.2, 0.8, 1.9], &[Front]),
            WashStand => ([0.65, 0.85, 0.5], [0.9, 0.85, 0.6], &[Front]),
            Workbench => ([1.5, 0.85, 0.7], [2.2, 0.9, 0.85], &[Front]),
            CuttingTable => ([1.8, 0.85, 1.0], [2.4, 0.85, 1.2], &[Front, Back]),
            ToolRack => ([1.0, 1.5, 0.35], [1.6, 1.7, 0.4], &[Front]),
            WeaponRack => ([1.2, 1.7, 0.5], [1.8, 1.8, 0.6], &[Front]),
            ArmourStand => ([0.55, 1.65, 0.55], [0.65, 1.8, 0.65], &[Front]),
            GrainBin => ([1.0, 0.85, 0.8], [1.5, 1.0, 1.0], &[Front]),
            StorageCrate => ([0.7, 0.65, 0.6], [1.0, 0.8, 0.8], &[Front]),
            Counter | CounterLeftEnd | CounterRightEnd => {
                ([1.2, 1.05, 0.7], [1.8, 1.05, 0.7], &[Front, Back])
            }
            CounterCorner => ([0.7, 1.05, 0.7], [0.7, 1.05, 0.7], &[Front, Left]),
            DisplayCounter => ([1.2, 0.95, 0.65], [1.8, 0.95, 0.75], &[Front, Back]),
            DryingRack => ([1.2, 1.5, 0.55], [1.8, 1.7, 0.65], &[Front]),
            KneadingTrough => ([1.2, 0.85, 0.65], [1.8, 0.85, 0.75], &[Front]),
            ButchersBlock => ([0.8, 0.85, 0.7], [1.1, 0.9, 0.9], &[Front]),
            CaskRack => ([1.2, 1.4, 0.75], [1.8, 1.6, 0.85], &[Front]),
            HayRack => ([1.2, 1.2, 0.55], [1.8, 1.3, 0.65], &[Front]),
            FeedTrough => ([1.2, 0.55, 0.55], [1.8, 0.6, 0.6], &[Front]),
            Barrel | CargoStack | TableBenchSet | CanvasStall | HitchingTrough => return None,
        };
        Some(InteriorFurnitureSpec {
            size_metres: Vec3::from_array(match self.variant {
                FurnitureVariant::Compact => small,
                FurnitureVariant::Broad => broad,
            }),
            required_faces,
        })
    }
}
