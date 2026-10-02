use super::{Corolla, FlowerParameters, LeafArrangement};
use crate::Pigment;
use serde::{Deserialize, Serialize};

/// Native or long-established Central European species; see SOURCES.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowerSpecies {
    Daisy,
    MeadowButtercup,
    WoodAnemone,
    CornPoppy,
    NettleLeavedBellflower,
}
impl FlowerSpecies {
    pub const ALL: [Self; 5] = [
        Self::Daisy,
        Self::MeadowButtercup,
        Self::WoodAnemone,
        Self::CornPoppy,
        Self::NettleLeavedBellflower,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Daisy => "Common daisy · Bellis perennis",
            Self::MeadowButtercup => "Meadow buttercup · Ranunculus acris",
            Self::WoodAnemone => "Wood anemone · Anemone nemorosa",
            Self::CornPoppy => "Corn poppy · Papaver rhoeas",
            Self::NettleLeavedBellflower => "Nettle-leaved bellflower · Campanula trachelium",
        }
    }
    pub fn parameters(self) -> FlowerParameters {
        let mut p = FlowerParameters {
            height_m: 0.10,
            stem_radius_m: 0.0008,
            stem_lean: 0.08,
            heads: 1,
            corolla: Corolla::RayAndDisk,
            petals: 28,
            petal_length_m: 0.008,
            petal_width_ratio: 0.17,
            petal_cup: 0.03,
            petal_notch: 0.08,
            petal_ripple: 0.025,
            center_radius_m: 0.004,
            center_height_ratio: 0.5,
            stamens: 0,
            head_tilt: 0.12,
            leaf_arrangement: LeafArrangement::BasalRosette,
            leaves: 7,
            leaf_length_m: 0.035,
            leaf_width_ratio: 0.35,
            leaf_lobes: 0,
            leaf_lobe_depth: 0.0,
            leaflets: 1,
            leaf_fan_radians: 0.0,
            petal: Pigment([237, 233, 220]),
            petal_base: Pigment([237, 233, 220]),
            base_fraction: 0.0,
            center: Pigment([224, 173, 28]),
            anther: Pigment([224, 173, 28]),
            green: Pigment([59, 101, 39]),
        };
        match self {
            Self::Daisy => {}
            Self::MeadowButtercup => {
                p.height_m = 0.45;
                p.heads = 3;
                p.stem_radius_m = 0.0013;
                p.corolla = Corolla::FreePetals;
                p.petals = 5;
                p.petal_length_m = 0.011;
                p.petal_width_ratio = 0.72;
                p.petal_cup = 0.3;
                p.center_radius_m = 0.0025;
                p.petal = Pigment([238, 188, 22]);
                p.petal_base = p.petal;
                p.stamens = 24;
                p.leaf_arrangement = LeafArrangement::Alternate;
                p.leaves = 4;
                p.leaf_length_m = 0.06;
                p.leaf_width_ratio = 0.8;
                p.leaf_lobes = 3;
                p.leaf_lobe_depth = 0.55;
                p.leaflets = 5;
                p.leaf_fan_radians = 2.8;
                p.leaf_width_ratio = 0.35;
            }
            Self::WoodAnemone => {
                p.height_m = 0.18;
                p.corolla = Corolla::FreePetals;
                p.petals = 6;
                p.petal_length_m = 0.016;
                p.petal_width_ratio = 0.5;
                p.petal_cup = 0.22;
                p.center_radius_m = 0.002;
                p.stamens = 30;
                p.leaf_arrangement = LeafArrangement::Whorl;
                p.leaves = 3;
                p.leaf_length_m = 0.045;
                p.leaf_width_ratio = 0.85;
                p.leaf_lobes = 3;
                p.leaf_lobe_depth = 0.5;
                p.leaflets = 3;
                p.leaf_fan_radians = 1.7;
                p.leaf_width_ratio = 0.55;
            }
            Self::CornPoppy => {
                p.height_m = 0.55;
                p.stem_radius_m = 0.0015;
                p.corolla = Corolla::FreePetals;
                p.petals = 4;
                p.petal_length_m = 0.037;
                p.petal_width_ratio = 0.62;
                p.petal_cup = 0.35;
                p.petal_ripple = 0.07;
                p.center_radius_m = 0.004;
                p.center_height_ratio = 0.9;
                p.stamens = 36;
                p.petal = Pigment([189, 37, 28]);
                p.petal_base = Pigment([38, 29, 31]);
                p.base_fraction = 0.3;
                p.center = Pigment([59, 69, 40]);
                p.anther = Pigment([38, 29, 31]);
                p.leaf_arrangement = LeafArrangement::Alternate;
                p.leaves = 5;
                p.leaf_length_m = 0.085;
                p.leaf_width_ratio = 0.4;
                p.leaf_lobes = 5;
                p.leaf_lobe_depth = 0.72;
            }
            Self::NettleLeavedBellflower => {
                p.height_m = 0.65;
                p.stem_radius_m = 0.002;
                p.heads = 5;
                p.corolla = Corolla::FusedBell;
                p.petals = 5;
                p.petal_length_m = 0.032;
                p.petal_width_ratio = 0.45;
                p.petal_cup = 0.75;
                p.center_radius_m = 0.002;
                p.head_tilt = 0.6;
                p.stamens = 5;
                p.petal = Pigment([99, 78, 159]);
                p.petal_base = p.petal;
                p.leaf_arrangement = LeafArrangement::Alternate;
                p.leaves = 6;
                p.leaf_length_m = 0.085;
                p.leaf_width_ratio = 0.5;
                p.leaf_lobes = 8;
                p.leaf_lobe_depth = 0.16;
            }
        }
        p
    }
}
