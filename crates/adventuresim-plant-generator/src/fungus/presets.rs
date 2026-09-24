use super::{FertileSurface, FungusParameters};
use crate::Pigment;
use serde::{Deserialize, Serialize};

/// Representative native fruiting forms; modern range supports the period inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FungusSpecies {
    FlyAgaric,
    Porcini,
    Chanterelle,
    CommonPuffball,
}
impl FungusSpecies {
    pub const ALL: [Self; 4] = [
        Self::FlyAgaric,
        Self::Porcini,
        Self::Chanterelle,
        Self::CommonPuffball,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::FlyAgaric => "Fly agaric · Amanita muscaria",
            Self::Porcini => "Porcini · Boletus edulis",
            Self::Chanterelle => "Chanterelle · Cantharellus cibarius",
            Self::CommonPuffball => "Common puffball · Lycoperdon perlatum",
        }
    }
    pub fn parameters(self) -> FungusParameters {
        let mut p = FungusParameters {
            cap_elevation_m: 0.13,
            stipe_radius_m: 0.009,
            stipe_base_ratio: 1.7,
            stipe_bend: 0.07,
            cap_radius_m: 0.07,
            cap_rise_ratio: 0.35,
            cap_thickness_ratio: 0.08,
            cap_depression_ratio: 0.0,
            rim_wave: 0.018,
            rim_lobes: 5,
            asymmetry: 0.04,
            fertile_surface: FertileSurface::Gills,
            fold_count: 88,
            fold_depth_m: 0.003,
            decurrent_m: 0.0,
            ring_radius_ratio: 2.5,
            ring_height_fraction: 0.7,
            ornament_count: 75,
            ornament_radius_m: 0.003,
            ornament_height_m: 0.0015,
            cap: Pigment([184, 35, 22]),
            underside: Pigment([231, 221, 193]),
            stipe: Pigment([230, 218, 191]),
            ornament: Pigment([243, 230, 199]),
        };
        match self {
            Self::FlyAgaric => {}
            Self::Porcini => {
                p.cap_elevation_m = 0.095;
                p.stipe_radius_m = 0.025;
                p.stipe_base_ratio = 1.25;
                p.cap_radius_m = 0.075;
                p.cap_rise_ratio = 0.58;
                p.cap_thickness_ratio = 0.17;
                p.fertile_surface = FertileSurface::Pores;
                p.ring_radius_ratio = 0.0;
                p.ornament_count = 0;
                p.cap = Pigment([113, 68, 35]);
                p.underside = Pigment([199, 193, 117]);
                p.stipe = Pigment([212, 192, 147]);
            }
            Self::Chanterelle => {
                p.cap_elevation_m = 0.065;
                p.stipe_radius_m = 0.008;
                p.stipe_base_ratio = 0.7;
                p.cap_radius_m = 0.042;
                p.cap_rise_ratio = 0.08;
                p.cap_thickness_ratio = 0.05;
                p.cap_depression_ratio = 0.42;
                p.rim_wave = 0.12;
                p.rim_lobes = 6;
                p.asymmetry = 0.12;
                p.fertile_surface = FertileSurface::Ridges;
                p.fold_count = 30;
                p.fold_depth_m = 0.0014;
                p.decurrent_m = 0.03;
                p.ring_radius_ratio = 0.0;
                p.ornament_count = 0;
                p.cap = Pigment([224, 162, 44]);
                p.underside = Pigment([224, 177, 68]);
                p.stipe = Pigment([221, 172, 62]);
            }
            Self::CommonPuffball => {
                p.cap_elevation_m = 0.028;
                p.stipe_radius_m = 0.012;
                p.stipe_base_ratio = 0.65;
                p.cap_radius_m = 0.026;
                p.cap_rise_ratio = 1.05;
                p.cap_thickness_ratio = 0.72;
                p.fertile_surface = FertileSurface::Enclosed;
                p.ring_radius_ratio = 0.0;
                p.rim_wave = 0.015;
                p.ornament_count = 180;
                p.ornament_radius_m = 0.0012;
                p.ornament_height_m = 0.002;
                p.cap = Pigment([220, 210, 183]);
                p.underside = p.cap;
                p.stipe = p.cap;
                p.ornament = Pigment([232, 221, 194]);
            }
        }
        p
    }
}
