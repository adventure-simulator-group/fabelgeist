//! Anatomical coverage unions for one item, independent of its material.

use serde::{Deserialize, Serialize};

use crate::body::{BodyPart, BodySide};
use crate::item_catalog_schema::{
    EquipmentAnatomicalRegion, EquipmentPlacement, MAX_EQUIPMENT_SURFACE_SEGMENTS, SurfaceAnchor,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArmorCoverageSpan {
    pub start: f32,
    pub end: f32,
}

impl ArmorCoverageSpan {
    pub fn centered(coverage: f32) -> Self {
        let coverage = coverage.clamp(0.0, 1.0);
        Self {
            start: (1.0 - coverage) * 0.5,
            end: (1.0 + coverage) * 0.5,
        }
    }

    pub fn contains(self, coordinate: f32) -> bool {
        let coordinate = coordinate.clamp(0.0, 1.0 - f32::EPSILON);
        coordinate >= self.start && coordinate < self.end
    }
}

/// One region's retained interval. Overlap never creates an additional layer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArmorCoverageSegment {
    pub span: ArmorCoverageSpan,
    pub region: Option<EquipmentAnatomicalRegion>,
    pub anchor: SurfaceAnchor,
    pub laterality: Option<BodySide>,
    pub surface_index: Option<usize>,
}

/// All disconnected coverage of one item over one coarse combat body part.
/// Serialization emits only occupied entries and rejects oversized input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "Vec<ArmorCoverageSegment>",
    into = "Vec<ArmorCoverageSegment>"
)]
pub struct AuthoredArmorCoverage {
    segments: [Option<ArmorCoverageSegment>; MAX_EQUIPMENT_SURFACE_SEGMENTS],
}

impl AuthoredArmorCoverage {
    /// Explicit interval for innate protection or a synthetic combat fixture.
    pub fn from_span(span: ArmorCoverageSpan) -> Self {
        let mut result = Self::default();
        result.segments[0] = Some(ArmorCoverageSegment {
            span,
            region: None,
            anchor: SurfaceAnchor::Center,
            laterality: None,
            surface_index: None,
        });
        result
    }

    /// Projects every authored region; absent regions give no protection.
    pub fn from_placement(placement: &EquipmentPlacement, part: BodyPart) -> Self {
        let mut segments = Vec::new();
        for (surface_index, surface) in placement.surface.iter().enumerate() {
            let count = surface.regions.len().max(1) as f32;
            let coverage = surface.coverage.clamp(0.0, 1.0);
            let retained = match surface.anchor {
                SurfaceAnchor::Proximal => (0.0, coverage),
                SurfaceAnchor::Distal => (1.0 - coverage, 1.0),
                SurfaceAnchor::Center => ((1.0 - coverage) * 0.5, (1.0 + coverage) * 0.5),
            };
            for (index, region) in surface.regions.iter().copied().enumerate() {
                let (region_part, body_start, body_end, laterality) = region_body_interval(region);
                if region_part != part {
                    continue;
                }
                let start = retained.0.max(index as f32 / count);
                let end = retained.1.min((index as f32 + 1.0) / count);
                if start >= end {
                    continue;
                }
                let local_start = start * count - index as f32;
                let local_end = end * count - index as f32;
                segments.push(ArmorCoverageSegment {
                    span: ArmorCoverageSpan {
                        start: body_start + (body_end - body_start) * local_start,
                        end: body_start + (body_end - body_start) * local_end,
                    },
                    region: Some(region),
                    anchor: surface.anchor,
                    laterality,
                    surface_index: Some(surface_index),
                });
            }
        }
        Self::try_from(segments).expect("catalog validates anatomical segment capacity")
    }

    pub fn segments(&self) -> impl Iterator<Item = &ArmorCoverageSegment> {
        self.segments.iter().flatten()
    }

    pub fn contains(self, coordinate: f32) -> bool {
        self.segments()
            .any(|segment| segment.span.contains(coordinate))
    }
}

impl TryFrom<Vec<ArmorCoverageSegment>> for AuthoredArmorCoverage {
    type Error = &'static str;

    fn try_from(segments: Vec<ArmorCoverageSegment>) -> Result<Self, Self::Error> {
        if segments.len() > MAX_EQUIPMENT_SURFACE_SEGMENTS {
            return Err("too many anatomical coverage segments");
        }
        if segments.iter().any(|segment| {
            !segment.span.start.is_finite()
                || !segment.span.end.is_finite()
                || segment.span.start < 0.0
                || segment.span.end > 1.0
                || segment.span.start > segment.span.end
        }) {
            return Err("invalid anatomical coverage interval");
        }
        let mut result = Self::default();
        for (entry, segment) in result.segments.iter_mut().zip(segments) {
            *entry = Some(segment);
        }
        Ok(result)
    }
}

impl From<AuthoredArmorCoverage> for Vec<ArmorCoverageSegment> {
    fn from(coverage: AuthoredArmorCoverage) -> Self {
        coverage.segments().copied().collect()
    }
}

fn region_body_interval(
    region: EquipmentAnatomicalRegion,
) -> (BodyPart, f32, f32, Option<BodySide>) {
    use super::targeting::{ABDOMEN_END, CHEST_AXILLA_END, CHEST_LOWER_EDGE_END};
    use EquipmentAnatomicalRegion as Region;
    let axilla_middle = (CHEST_LOWER_EDGE_END + CHEST_AXILLA_END) * 0.5;
    match region {
        Region::Chest => (BodyPart::Chest, 0.0, CHEST_LOWER_EDGE_END, None),
        Region::LeftAxilla => (
            BodyPart::Chest,
            CHEST_LOWER_EDGE_END,
            axilla_middle,
            Some(BodySide::Left),
        ),
        Region::RightAxilla => (
            BodyPart::Chest,
            axilla_middle,
            CHEST_AXILLA_END,
            Some(BodySide::Right),
        ),
        Region::Stomach => (BodyPart::Stomach, 0.0, ABDOMEN_END, None),
        Region::Groin => (BodyPart::Stomach, ABDOMEN_END, 1.0, None),
        Region::LeftUpperArm => (BodyPart::LeftArm, 0.0, 0.5, Some(BodySide::Left)),
        Region::LeftForearm => (BodyPart::LeftArm, 0.5, 1.0, Some(BodySide::Left)),
        Region::RightUpperArm => (BodyPart::RightArm, 0.0, 0.5, Some(BodySide::Right)),
        Region::RightForearm => (BodyPart::RightArm, 0.5, 1.0, Some(BodySide::Right)),
        Region::LeftThigh => (BodyPart::LeftLeg, 0.0, 0.5, Some(BodySide::Left)),
        Region::LeftLowerLeg => (BodyPart::LeftLeg, 0.5, 1.0, Some(BodySide::Left)),
        Region::RightThigh => (BodyPart::RightLeg, 0.0, 0.5, Some(BodySide::Right)),
        Region::RightLowerLeg => (BodyPart::RightLeg, 0.5, 1.0, Some(BodySide::Right)),
        Region::Neck => (BodyPart::Head, 0.0, 0.25, None),
        Region::Head => (BodyPart::Head, 0.25, 1.0, None),
    }
}

/// Selects one physical layer even if several of its segments contain the point.
pub fn layered_armor_contact_index(
    sample: f32,
    layers: impl IntoIterator<Item = AuthoredArmorCoverage>,
) -> Option<usize> {
    layers
        .into_iter()
        .position(|coverage| coverage.contains(sample))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coverage(item: &str, placement: usize, part: BodyPart) -> AuthoredArmorCoverage {
        let definition = crate::item_catalog::definition(item).unwrap();
        AuthoredArmorCoverage::from_placement(
            &definition.equipment.as_ref().unwrap().placements[placement],
            part,
        )
    }

    #[test]
    fn doublet_covers_plate_axillary_openings_without_filling_neckline() {
        let plate = coverage("breastplate", 0, BodyPart::Chest);
        let cloth = coverage("arming_doublet", 0, BodyPart::Chest);
        for sample in [0.10, 0.50, 0.70] {
            assert!(cloth.contains(sample));
            assert_eq!(layered_armor_contact_index(sample, [plate, cloth]), Some(0));
        }
        for sample in [0.87, 0.90] {
            assert!(!plate.contains(sample));
            assert!(cloth.contains(sample));
            assert_eq!(layered_armor_contact_index(sample, [plate, cloth]), Some(1));
        }
        assert_eq!(layered_armor_contact_index(0.97, [plate, cloth]), None);
    }

    #[test]
    fn voiders_cover_both_axillae_without_protecting_the_chest_plate_surface() {
        let mail = coverage("mail_voiders", 0, BodyPart::Chest);
        assert_eq!(mail.segments().count(), 2);
        assert_eq!(
            mail.segments().map(|s| s.laterality).collect::<Vec<_>>(),
            vec![Some(BodySide::Left), Some(BodySide::Right)]
        );
        for sample in [0.87, 0.90] {
            assert!(mail.contains(sample));
        }
        for sample in [0.1, 0.5, 0.8, 0.97] {
            assert!(!mail.contains(sample));
        }
    }

    #[test]
    fn disconnected_joint_mail_leaves_cloth_between_patches_on_both_arms() {
        for part in [BodyPart::LeftArm, BodyPart::RightArm] {
            let mail = coverage("mail_voiders", 0, part);
            let cloth = coverage("arming_doublet", 0, part);
            assert_eq!(mail.segments().count(), 3);
            for sample in [0.05, 0.40, 0.50, 0.60] {
                assert!(mail.contains(sample));
                assert_eq!(layered_armor_contact_index(sample, [mail, cloth]), Some(0));
            }
            for sample in [0.20, 0.30, 0.70, 0.95] {
                assert!(!mail.contains(sample));
                assert_eq!(layered_armor_contact_index(sample, [mail, cloth]), Some(1));
            }
            let encoded = serde_json::to_string(&mail).unwrap();
            let decoded: AuthoredArmorCoverage = serde_json::from_str(&encoded).unwrap();
            assert_eq!(
                decoded, mail,
                "telemetry retains every disconnected segment"
            );
        }
    }

    #[test]
    fn overlapping_patches_are_one_layer_and_unauthored_parts_are_empty() {
        let span = ArmorCoverageSpan::centered(0.5);
        let single = AuthoredArmorCoverage::from_span(span);
        let segment = *single.segments().next().unwrap();
        let overlap = AuthoredArmorCoverage::try_from(vec![segment, segment]).unwrap();
        assert_eq!(layered_armor_contact_index(0.5, [overlap, single]), Some(0));
        assert!(!coverage("mail_voiders", 0, BodyPart::Stomach).contains(0.5));
    }

    #[test]
    fn garment_chains_retain_both_limb_regions() {
        for (placement, part) in [(0, BodyPart::LeftLeg), (1, BodyPart::RightLeg)] {
            let hose = coverage("padded_chausses", placement, part);
            assert!(hose.contains(0.1));
            assert!(hose.contains(0.9));
            assert_eq!(hose.segments().count(), 2);
        }
        let left = coverage("vambrace", 0, BodyPart::LeftArm);
        let right = coverage("vambrace", 1, BodyPart::RightArm);
        for sample in [0.1, 0.5, 0.9] {
            assert_eq!(left.contains(sample), right.contains(sample));
        }
        assert!(!left.contains(0.5));
        assert!(left.contains(0.9));
    }

    #[test]
    fn oversized_or_invalid_serialized_coverage_is_rejected() {
        let coverage = AuthoredArmorCoverage::from_span(ArmorCoverageSpan::centered(1.0));
        let segment = *coverage.segments().next().unwrap();
        let oversized = vec![segment; MAX_EQUIPMENT_SURFACE_SEGMENTS + 1];
        assert!(AuthoredArmorCoverage::try_from(oversized.clone()).is_err());
        assert!(
            serde_json::from_value::<AuthoredArmorCoverage>(serde_json::json!(oversized)).is_err()
        );
        let invalid = ArmorCoverageSegment {
            span: ArmorCoverageSpan {
                start: 0.8,
                end: 0.2,
            },
            ..segment
        };
        assert!(AuthoredArmorCoverage::try_from(vec![invalid]).is_err());
    }

    #[test]
    fn brayette_protects_groin_without_granting_abdominal_or_full_thigh_mail() {
        let groin = coverage("mail_brayette", 0, BodyPart::Stomach);
        assert!(groin.contains(0.95));
        assert!(!groin.contains(0.5));
        assert_eq!(
            super::super::anatomical_subregion(BodyPart::Stomach, 0.95),
            super::super::AnatomicalSubregion::Groin
        );
        for item in ["arming_doublet", "cuirass", "fauld"] {
            assert!(
                !coverage(item, 0, BodyPart::Stomach).contains(0.95),
                "{item} must not imply crotch protection"
            );
        }
        for part in [BodyPart::LeftLeg, BodyPart::RightLeg] {
            let shorts = coverage("mail_brayette", 0, part);
            assert!(shorts.contains(0.05));
            assert!(shorts.contains(0.14));
            assert!(!shorts.contains(0.2));
            assert!(!shorts.contains(0.5));
        }
    }

    #[test]
    fn knee_and_neck_mail_leave_adjacent_anatomy_to_other_layers() {
        for (placement, part, opposite) in [
            (0, BodyPart::LeftLeg, BodyPart::RightLeg),
            (1, BodyPart::RightLeg, BodyPart::LeftLeg),
        ] {
            let knee = coverage("mail_knee_voider", placement, part);
            let cloth = coverage("padded_chausses", placement, part);
            for point in [0.45, 0.55] {
                assert_eq!(layered_armor_contact_index(point, [knee, cloth]), Some(0));
            }
            for point in [0.2, 0.8] {
                assert_eq!(layered_armor_contact_index(point, [knee, cloth]), Some(1));
            }
            assert!(!coverage("mail_knee_voider", placement, opposite).contains(0.5));
        }
        let collar = coverage("mail_standard", 0, BodyPart::Head);
        assert!(collar.contains(0.1));
        assert!(!collar.contains(0.5));
        assert!(!coverage("mail_standard", 0, BodyPart::Chest).contains(0.5));
    }
}
