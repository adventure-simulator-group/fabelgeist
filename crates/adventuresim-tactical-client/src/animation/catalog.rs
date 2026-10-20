use adventuresim_tactical_core::animation::{AnimationPackId, SkeletonFamily};
use std::collections::BTreeMap;

#[cfg(test)]
use std::path::{Path, PathBuf};

use super::catalog_id::{
    AnimationAssetDirectory, AnimationAssetPath, AuthoredAnimationFrame, MotionId,
};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

mod builtin;

/// The two contact switches in the authored 24-frame combat cycle.
pub(super) const COMBAT_FIRST_CONTACT_PHASE: f32 = 6.0 / 24.0;
pub(super) const COMBAT_SECOND_CONTACT_PHASE: f32 = 18.0 / 24.0;

/// Explicit code-owned file/frame catalog. Semantic ownership never depends on
/// glTF animation names or scene contents.
#[derive(Resource, Debug)]
pub struct AnimationPackCatalog {
    pub(super) packs: BTreeMap<AnimationPackId, PackCatalog>,
}

#[derive(Debug, Clone)]
pub(super) struct PackCatalog {
    pub(super) skeleton_family: SkeletonFamily,
    pub(super) fallback: Option<AnimationPackId>,
    pub(super) motions: BTreeMap<MotionId, MotionSource>,
    pub(super) poses: BTreeMap<SemanticPose, PoseAnchor>,
    pub(super) references: BTreeMap<MotionId, Vec<ReferenceAnchor>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MotionSource {
    pub(super) path: AnimationAssetPath,
    pub(super) required_last_frame: AuthoredAnimationFrame,
    pub(super) last_frame: AuthoredAnimationFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PoseAnchor {
    pub(super) motion: MotionId,
    pub(super) frame: AuthoredAnimationFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReferenceAnchor {
    pub(super) pose: SemanticPose,
    pub(super) frame: AuthoredAnimationFrame,
}

pub(super) struct PackBuilder {
    id: AnimationPackId,
    asset_directory: AnimationAssetDirectory,
    pack: PackCatalog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CatalogError {
    InvalidAssetDirectory(String),
    InvalidMotionStem(String),
    DuplicatePose(SemanticPose),
    UnknownMotion(String),
    InvalidPack(PackValidationError),
}

impl Default for AnimationPackCatalog {
    fn default() -> Self {
        match Self::biped_root() {
            Ok(catalog) => catalog,
            Err(error) => panic!("built-in biped animation catalog must be valid: {error:?}"),
        }
    }
}

impl PackBuilder {
    pub(super) fn new(
        id: &AnimationPackId,
        skeleton_family: &SkeletonFamily,
        fallback: Option<&AnimationPackId>,
        asset_directory: AnimationAssetDirectory,
    ) -> Self {
        Self {
            id: id.to_owned(),
            asset_directory,
            pack: PackCatalog {
                skeleton_family: skeleton_family.to_owned(),
                fallback: fallback.cloned(),
                motions: BTreeMap::new(),
                poses: BTreeMap::new(),
                references: BTreeMap::new(),
            },
        }
    }

    pub(super) fn motion(&mut self, id: &MotionId, last_frame: AuthoredAnimationFrame) {
        self.variable_motion(id, last_frame, last_frame);
    }

    pub(super) fn variable_motion(
        &mut self,
        id: &MotionId,
        required_last_frame: AuthoredAnimationFrame,
        last_frame: AuthoredAnimationFrame,
    ) {
        self.pack.motions.insert(
            id.clone(),
            MotionSource {
                path: self.asset_directory.motion_path(id),
                required_last_frame,
                last_frame,
            },
        );
    }

    pub(super) fn pose(
        &mut self,
        motion: &MotionId,
        frame: AuthoredAnimationFrame,
        pose: SemanticPose,
    ) -> Result<(), CatalogError> {
        if !self.pack.motions.contains_key(motion) {
            return Err(CatalogError::UnknownMotion(motion.to_string()));
        }
        if self
            .pack
            .poses
            .insert(
                pose,
                PoseAnchor {
                    motion: motion.clone(),
                    frame,
                },
            )
            .is_some()
        {
            return Err(CatalogError::DuplicatePose(pose));
        }
        Ok(())
    }

    pub(super) fn reference(
        &mut self,
        motion: &MotionId,
        frame: AuthoredAnimationFrame,
        pose: SemanticPose,
    ) -> Result<(), CatalogError> {
        if !self.pack.motions.contains_key(motion) {
            return Err(CatalogError::UnknownMotion(motion.to_string()));
        }
        self.pack
            .references
            .entry(motion.clone())
            .or_default()
            .push(ReferenceAnchor { pose, frame });
        Ok(())
    }

    pub(super) fn finish(self) -> (AnimationPackId, PackCatalog) {
        (self.id, self.pack)
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditorRouteResolution {
    pub route: &'static str,
    pub requested: SemanticPose,
    pub resolved_pack: AnimationPackId,
    pub resolved_pose: SemanticPose,
    pub mirrored: bool,
    pub motion_path: PathBuf,
    pub frame: AuthoredAnimationFrame,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct EditorValidationReport {
    pub pack_count: usize,
    pub motion_count: usize,
    pub missing_motion_count: usize,
    pub warnings: Vec<String>,
    pub route_resolutions: Vec<EditorRouteResolution>,
}

/// Validate the code-owned semantic catalog against an editor asset root and
/// resolve the same deterministic ordinary and raised/attack samples used by
/// the capture viewer. Errors are returned together so the native tool can
/// print a complete actionable preflight instead of failing at the first file.
#[cfg(test)]
pub(crate) fn validate_editor_asset_root(
    asset_root: &Path,
) -> Result<EditorValidationReport, Vec<String>> {
    let catalog = AnimationPackCatalog::default();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut motion_count = 0;
    let mut missing_motion_count = 0;

    for (pack_id, pack) in &catalog.packs {
        motion_count += pack.motions.len();
        for (motion_id, motion) in &pack.motions {
            let full_path = asset_root.join(motion.path.as_ref());
            if !full_path.is_file() {
                missing_motion_count += 1;
                warnings.push(format!(
                    "pack {pack_id} motion {motion_id} is missing {}",
                    full_path.display()
                ));
            }
        }
        for (pose, anchor) in &pack.poses {
            match pack.motions.get(&anchor.motion) {
                Some(motion) if anchor.frame <= motion.last_frame => {}
                Some(motion) => errors.push(format!(
                    "pack {pack_id} pose {} frame {} exceeds {} frame {}",
                    pose.as_str(),
                    anchor.frame,
                    anchor.motion,
                    motion.last_frame
                )),
                None => errors.push(format!(
                    "pack {pack_id} pose {} references unknown motion {}",
                    pose.as_str(),
                    anchor.motion
                )),
            }
        }
    }

    let ordinary = SkeletonState::default()
        .with_local_velocity(Vec3::NEG_Z * 2.0)
        .with_world_velocity(Vec3::NEG_Z * 2.0);
    let mut raised_attack = SkeletonState::default();
    raised_attack
        .begin_attack(
            AttackSpec::default(),
            adventuresim_tactical_core::animation::AnimationSampleTick::from(10u64),
            adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64),
        )
        .expect("catalog fixture starts from idle");
    raised_attack
        .advance_action(adventuresim_tactical_core::animation::AnimationSampleTick::from(20u64));

    let mut route_resolutions = Vec::new();
    for (route, state) in [
        ("ordinary_locomotion", ordinary),
        ("raised_guard_attack", raised_attack),
    ] {
        let evaluation = AnimationEvaluation::from_skeleton(&state).unwrap();
        let samples = if evaluation.action.is_empty() {
            &evaluation.base
        } else {
            &evaluation.action
        };
        if samples.is_empty() {
            errors.push(format!("route {route} produced no semantic samples"));
            continue;
        }
        for sample in samples {
            match resolve_editor_pose(
                route,
                &catalog,
                &AnimationPackId::HUMANOID_UNARMED,
                sample.pose,
                asset_root,
            ) {
                Some(resolution) => {
                    if !resolution.motion_path.is_file() {
                        errors.push(format!(
                            "required route {route} resolves {} to missing {}",
                            sample.pose.as_str(),
                            resolution.motion_path.display()
                        ));
                    }
                    route_resolutions.push(resolution);
                }
                None => errors.push(format!(
                    "route {route} cannot resolve semantic pose {} from pack humanoid_unarmed",
                    sample.pose.as_str()
                )),
            }
            if let PoseSampling::Span { end, .. } = sample.sampling {
                match resolve_editor_pose(
                    route,
                    &catalog,
                    &AnimationPackId::HUMANOID_UNARMED,
                    end,
                    asset_root,
                ) {
                    Some(resolution) => {
                        if !resolution.motion_path.is_file() {
                            errors.push(format!(
                                "required route {route} resolves span endpoint {} to missing {}",
                                end.as_str(),
                                resolution.motion_path.display()
                            ));
                        }
                        route_resolutions.push(resolution);
                    }
                    None => errors.push(format!(
                        "route {route} cannot resolve span endpoint {} from pack humanoid_unarmed",
                        end.as_str()
                    )),
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(EditorValidationReport {
            pack_count: catalog.packs.len(),
            motion_count,
            missing_motion_count,
            warnings,
            route_resolutions,
        })
    } else {
        Err(errors)
    }
}

#[cfg(test)]
fn resolve_editor_pose(
    route: &'static str,
    catalog: &AnimationPackCatalog,
    requested_pack: &AnimationPackId,
    requested_pose: SemanticPose,
    asset_root: &Path,
) -> Option<EditorRouteResolution> {
    let mut pack_id = requested_pack;
    loop {
        let pack = catalog.packs.get(pack_id)?;
        let resolved = pack
            .poses
            .get(&requested_pose)
            .map(|anchor| (requested_pose, anchor, false))
            .or_else(|| {
                let counterpart = requested_pose.mirrored_counterpart()?;
                pack.poses
                    .get(&counterpart)
                    .map(|anchor| (counterpart, anchor, true))
            });
        if let Some((resolved_pose, anchor, mirrored)) = resolved {
            let motion = pack.motions.get(&anchor.motion)?;
            return Some(EditorRouteResolution {
                route,
                requested: requested_pose,
                resolved_pack: pack_id.to_owned(),
                resolved_pose,
                mirrored,
                motion_path: asset_root.join(motion.path.as_ref()),
                frame: anchor.frame,
            });
        }
        pack_id = pack.fallback.as_ref()?;
    }
}
