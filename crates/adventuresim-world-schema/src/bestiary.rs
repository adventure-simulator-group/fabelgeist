//! Bestiary study categories and bounded direct and transferred knowledge.
use serde::{Deserialize, Serialize};

/// Practical ceiling for direct or transferred Bestiary study.
///
/// The shared skill curve remains asymptotic, but this bound prevents several
/// correlated fields from adding up to knowledge beyond the authored mastery
/// range.
pub const BESTIARY_MASTERY_HOURS: f32 = 5_000.0;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(crate = spacetimedb_lib))]
#[serde(rename_all = "snake_case")]
pub enum BestiaryCategory {
    Beast,
    Undead,
    Human,
    Werekin,
    Elf,
    Dwarf,
    Fey,
    Spirit,
    Greenskin,
    Insectoid,
    Draconid,
    Construct,
    Wildmen,
}

impl BestiaryCategory {
    pub const ALL: [Self; 13] = [
        Self::Beast,
        Self::Undead,
        Self::Human,
        Self::Werekin,
        Self::Elf,
        Self::Dwarf,
        Self::Fey,
        Self::Spirit,
        Self::Greenskin,
        Self::Insectoid,
        Self::Draconid,
        Self::Construct,
        Self::Wildmen,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Beast => "beast",
            Self::Undead => "undead",
            Self::Human => "human",
            Self::Werekin => "werekin",
            Self::Elf => "elf",
            Self::Dwarf => "dwarf",
            Self::Fey => "fey",
            Self::Spirit => "spirit",
            Self::Greenskin => "greenskin",
            Self::Insectoid => "insectoid",
            Self::Draconid => "draconid",
            Self::Construct => "construct",
            Self::Wildmen => "wildmen",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Beast => "Beast",
            Self::Undead => "Undead",
            Self::Human => "Human",
            Self::Werekin => "Werekin",
            Self::Elf => "Elf",
            Self::Dwarf => "Dwarf",
            Self::Fey => "Fey",
            Self::Spirit => "Spirit",
            Self::Greenskin => "Greenskin",
            Self::Insectoid => "Insectoid",
            Self::Draconid => "Draconid",
            Self::Construct => "Construct",
            Self::Wildmen => "Wildmen",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|category| category.id() == id)
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Beast => 0,
            Self::Undead => 1,
            Self::Human => 2,
            Self::Werekin => 3,
            Self::Elf => 4,
            Self::Dwarf => 5,
            Self::Fey => 6,
            Self::Spirit => 7,
            Self::Greenskin => 8,
            Self::Insectoid => 9,
            Self::Draconid => 10,
            Self::Construct => 11,
            Self::Wildmen => 12,
        }
    }

    /// Transfer represents shared diagnostic knowledge, not possible tag
    /// combinations. The matrix is deliberately conservative and symmetric.
    pub const fn correlation(self, other: Self) -> f32 {
        const C: [[f32; 13]; 13] = [
            [
                1.0, 0.05, 0.10, 0.70, 0.10, 0.05, 0.20, 0.15, 0.10, 0.15, 0.30, 0.02, 0.35,
            ],
            [
                0.05, 1.0, 0.35, 0.15, 0.30, 0.25, 0.15, 0.55, 0.25, 0.10, 0.10, 0.10, 0.20,
            ],
            [
                0.10, 0.35, 1.0, 0.35, 0.40, 0.40, 0.15, 0.20, 0.30, 0.05, 0.05, 0.15, 0.65,
            ],
            [
                0.70, 0.15, 0.35, 1.0, 0.25, 0.20, 0.30, 0.15, 0.20, 0.05, 0.10, 0.02, 0.40,
            ],
            [
                0.10, 0.30, 0.40, 0.25, 1.0, 0.35, 0.35, 0.20, 0.25, 0.05, 0.05, 0.15, 0.20,
            ],
            [
                0.05, 0.25, 0.40, 0.20, 0.35, 1.0, 0.15, 0.10, 0.25, 0.05, 0.05, 0.30, 0.15,
            ],
            [
                0.20, 0.15, 0.15, 0.30, 0.35, 0.15, 1.0, 0.45, 0.15, 0.25, 0.20, 0.10, 0.30,
            ],
            [
                0.15, 0.55, 0.20, 0.15, 0.20, 0.10, 0.45, 1.0, 0.10, 0.10, 0.10, 0.20, 0.20,
            ],
            [
                0.10, 0.25, 0.30, 0.20, 0.25, 0.25, 0.15, 0.10, 1.0, 0.05, 0.10, 0.15, 0.25,
            ],
            [
                0.15, 0.10, 0.05, 0.05, 0.05, 0.05, 0.25, 0.10, 0.05, 1.0, 0.15, 0.10, 0.10,
            ],
            [
                0.30, 0.10, 0.05, 0.10, 0.05, 0.05, 0.20, 0.10, 0.10, 0.15, 1.0, 0.10, 0.10,
            ],
            [
                0.02, 0.10, 0.15, 0.02, 0.15, 0.30, 0.10, 0.20, 0.15, 0.10, 0.10, 1.0, 0.05,
            ],
            [
                0.35, 0.20, 0.65, 0.40, 0.20, 0.15, 0.30, 0.20, 0.25, 0.10, 0.10, 0.05, 1.0,
            ],
        ];
        C[self.index()][other.index()]
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(crate = spacetimedb_lib))]
pub struct BestiaryHours {
    pub beast: f32,
    pub undead: f32,
    pub human: f32,
    pub werekin: f32,
    pub elf: f32,
    pub dwarf: f32,
    pub fey: f32,
    pub spirit: f32,
    pub greenskin: f32,
    pub insectoid: f32,
    pub draconid: f32,
    pub construct: f32,
    pub wildmen: f32,
}

impl BestiaryHours {
    pub fn direct_values(self) -> impl Iterator<Item = (BestiaryCategory, f32)> {
        BestiaryCategory::ALL
            .into_iter()
            .map(move |category| (category, self.direct(category)))
    }

    pub fn direct_fields_valid(self, maximum: f32) -> bool {
        maximum.is_finite()
            && maximum >= 0.0
            && self
                .direct_values()
                .all(|(_, hours)| hours.is_finite() && (0.0..=maximum).contains(&hours))
    }

    pub fn direct(self, category: BestiaryCategory) -> f32 {
        match category {
            BestiaryCategory::Beast => self.beast,
            BestiaryCategory::Undead => self.undead,
            BestiaryCategory::Human => self.human,
            BestiaryCategory::Werekin => self.werekin,
            BestiaryCategory::Elf => self.elf,
            BestiaryCategory::Dwarf => self.dwarf,
            BestiaryCategory::Fey => self.fey,
            BestiaryCategory::Spirit => self.spirit,
            BestiaryCategory::Greenskin => self.greenskin,
            BestiaryCategory::Insectoid => self.insectoid,
            BestiaryCategory::Draconid => self.draconid,
            BestiaryCategory::Construct => self.construct,
            BestiaryCategory::Wildmen => self.wildmen,
        }
    }

    pub fn direct_mut(&mut self, category: BestiaryCategory) -> &mut f32 {
        match category {
            BestiaryCategory::Beast => &mut self.beast,
            BestiaryCategory::Undead => &mut self.undead,
            BestiaryCategory::Human => &mut self.human,
            BestiaryCategory::Werekin => &mut self.werekin,
            BestiaryCategory::Elf => &mut self.elf,
            BestiaryCategory::Dwarf => &mut self.dwarf,
            BestiaryCategory::Fey => &mut self.fey,
            BestiaryCategory::Spirit => &mut self.spirit,
            BestiaryCategory::Greenskin => &mut self.greenskin,
            BestiaryCategory::Insectoid => &mut self.insectoid,
            BestiaryCategory::Draconid => &mut self.draconid,
            BestiaryCategory::Construct => &mut self.construct,
            BestiaryCategory::Wildmen => &mut self.wildmen,
        }
    }

    pub fn effective(self, category: BestiaryCategory) -> f32 {
        // Bestiary leaves are trained: correlation cannot bootstrap a target
        // category which has never received formal/direct study.
        if self.direct(category) <= 0.0 {
            return 0.0;
        }
        BestiaryCategory::ALL
            .into_iter()
            .map(|studied| {
                let direct = self.direct(studied);
                (if direct.is_finite() {
                    direct.max(0.0)
                } else {
                    0.0
                }) * category.correlation(studied)
            })
            .sum::<f32>()
    }

    pub fn maximum_effective(self) -> f32 {
        BestiaryCategory::ALL
            .into_iter()
            .map(|category| self.effective(category))
            .fold(0.0, f32::max)
    }

    /// Average correlated coverage across the complete Bestiary family.
    pub fn aggregate_effective(self) -> f32 {
        BestiaryCategory::ALL
            .into_iter()
            .map(|category| self.effective(category))
            .sum::<f32>()
            / BestiaryCategory::ALL.len() as f32
    }

    pub fn total_direct(self) -> f32 {
        self.direct_values()
            .map(|(_, hours)| {
                if hours.is_finite() {
                    hours.max(0.0)
                } else {
                    0.0
                }
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bestiary_correlations_are_symmetric_nonrecursive_and_direct_gated() {
        for left in BestiaryCategory::ALL {
            assert_eq!(left.correlation(left), 1.0);
            for right in BestiaryCategory::ALL {
                assert_eq!(left.correlation(right), right.correlation(left));
            }
        }
        assert!(
            BestiaryCategory::Wildmen.correlation(BestiaryCategory::Human)
                > BestiaryCategory::Wildmen.correlation(BestiaryCategory::Fey)
        );
        let hours = BestiaryHours {
            human: 1_000.0,
            ..Default::default()
        };
        assert_eq!(hours.effective(BestiaryCategory::Wildmen), 0.0);
        assert_eq!(hours.effective(BestiaryCategory::Human), 1_000.0);
        assert!(hours.aggregate_effective() > 0.0);
        assert!(hours.aggregate_effective() < hours.maximum_effective());

        let mastered = BestiaryHours {
            beast: super::BESTIARY_MASTERY_HOURS,
            werekin: super::BESTIARY_MASTERY_HOURS,
            wildmen: super::BESTIARY_MASTERY_HOURS,
            ..Default::default()
        };
        assert!(mastered.effective(BestiaryCategory::Beast) >= super::BESTIARY_MASTERY_HOURS);
    }

    #[test]
    fn bestiary_direct_fields_reject_nonfinite_negative_or_overmastery_values() {
        assert!(
            BestiaryHours {
                wildmen: 100.0,
                ..Default::default()
            }
            .direct_fields_valid(super::BESTIARY_MASTERY_HOURS)
        );
        for invalid in [
            -1.0,
            f32::NAN,
            f32::INFINITY,
            super::BESTIARY_MASTERY_HOURS + 1.0,
        ] {
            assert!(
                !BestiaryHours {
                    wildmen: invalid,
                    ..Default::default()
                }
                .direct_fields_valid(super::BESTIARY_MASTERY_HOURS)
            );
        }
    }
}
