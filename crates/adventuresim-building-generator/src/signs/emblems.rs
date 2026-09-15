//! Authored pictograms use documented period tools, without invented civic heraldry.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeEmblem {
    Hammer,
    BreadPeel,
    Shuttle,
}

impl TradeEmblem {
    pub(super) const fn for_use(usage: BuildingUse) -> Option<Self> {
        match usage {
            BuildingUse::Smithy | BuildingUse::Weaponsmith | BuildingUse::Armorer => {
                Some(Self::Hammer)
            }
            BuildingUse::Bakehouse => Some(Self::BreadPeel),
            BuildingUse::Weaver => Some(Self::Shuttle),
            _ => None,
        }
    }

    #[cfg(feature = "sign-render")]
    pub(super) fn covers(self, p: Vec2) -> bool {
        match self {
            Self::Hammer => {
                segment(p, Vec2::new(-0.35, 0.65), Vec2::new(0.2, -0.35), 0.09)
                    || polygon(
                        p,
                        &[[0.0, -0.6], [0.64, -0.25], [0.49, 0.02], [-0.15, -0.34]],
                    )
            }
            Self::BreadPeel => {
                segment(p, Vec2::new(0.0, 0.82), Vec2::new(0.0, -0.05), 0.095)
                    || polygon(
                        p,
                        &[
                            [-0.22, 0.1],
                            [-0.48, -0.18],
                            [-0.48, -0.67],
                            [-0.3, -0.8],
                            [0.3, -0.8],
                            [0.48, -0.67],
                            [0.48, -0.18],
                            [0.22, 0.1],
                        ],
                    )
            }
            Self::Shuttle => {
                polygon(
                    p,
                    &[
                        [-0.8, 0.0],
                        [-0.46, -0.22],
                        [0.46, -0.22],
                        [0.8, 0.0],
                        [0.46, 0.22],
                        [-0.46, 0.22],
                    ],
                ) && !(p.x.abs() < 0.35 && p.y.abs() < 0.12)
                    || segment(p, Vec2::new(-0.3, 0.0), Vec2::new(0.3, 0.0), 0.045)
            }
        }
    }
}

#[cfg(feature = "sign-render")]
fn segment(p: Vec2, a: Vec2, b: Vec2, radius: f32) -> bool {
    let delta = b - a;
    let t = ((p - a).dot(delta) / delta.length_squared()).clamp(0.0, 1.0);
    p.distance_squared(a + delta * t) <= radius * radius
}

#[cfg(feature = "sign-render")]
fn polygon(p: Vec2, vertices: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for index in 0..vertices.len() {
        let a = Vec2::from_array(vertices[index]);
        let b = Vec2::from_array(vertices[(index + 1) % vertices.len()]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
    }
    inside
}
