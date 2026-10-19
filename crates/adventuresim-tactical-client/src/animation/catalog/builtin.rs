//! Authored built-in motion catalogues and their complete semantic validation.

use super::*;

impl AnimationPackCatalog {
    pub(in crate::animation) fn biped_root() -> Result<Self, CatalogError> {
        let (id, pack) = PackCatalog::unarmed()?;
        let mut two_handed = PackBuilder::new(
            &AnimationPackId::HUMANOID_2H_CLOSE,
            &SkeletonFamily::HUMANOID,
            Some(&AnimationPackId::HUMANOID_UNARMED),
            AnimationAssetDirectory::HUMANOID_2H_CLOSE,
        );
        add_attack_motions(&mut two_handed)?;
        let (two_handed_id, two_handed_pack) = two_handed.finish();
        let mut library = AnimationPackLibrary::default();
        library
            .insert(AnimationPack {
                id: id.clone(),
                skeleton_family: pack.skeleton_family.clone(),
                fallback: pack.fallback.clone(),
                clips: pack.poses.keys().copied().collect(),
            })
            .map_err(CatalogError::InvalidPack)?;
        library
            .insert(AnimationPack {
                id: two_handed_id.clone(),
                skeleton_family: two_handed_pack.skeleton_family.clone(),
                fallback: two_handed_pack.fallback.clone(),
                clips: two_handed_pack.poses.keys().copied().collect(),
            })
            .map_err(CatalogError::InvalidPack)?;
        library
            .validate_complete(&id)
            .map_err(CatalogError::InvalidPack)?;
        Ok(Self {
            packs: BTreeMap::from([(id, pack), (two_handed_id, two_handed_pack)]),
        })
    }
}

impl PackCatalog {
    fn unarmed() -> Result<(AnimationPackId, Self), CatalogError> {
        let mut builder = PackBuilder::new(
            &AnimationPackId::HUMANOID_UNARMED,
            &SkeletonFamily::HUMANOID,
            None,
            AnimationAssetDirectory::HUMANOID_UNARMED,
        );
        for (motion, pose) in [
            (&MotionId::IDLE_RELAXED, SemanticPose::IdleRelaxed),
            (&MotionId::PRONE_IDLE, SemanticPose::ProneIdle),
            (&MotionId::SUPINE_IDLE, SemanticPose::SupineIdle),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        add_locomotion_motions(&mut builder)?;
        for (motion, takeoff, contact) in [
            (
                &MotionId::QUICKSTEP_FORWARD,
                SemanticPose::QuickstepForwardTakeoff,
                SemanticPose::QuickstepForwardContact,
            ),
            (
                &MotionId::QUICKSTEP_RIGHT,
                SemanticPose::QuickstepRightTakeoff,
                SemanticPose::QuickstepRightContact,
            ),
            (
                &MotionId::QUICKSTEP_LEFT,
                SemanticPose::QuickstepLeftTakeoff,
                SemanticPose::QuickstepLeftContact,
            ),
            (
                &MotionId::QUICKSTEP_BACK,
                SemanticPose::QuickstepBackTakeoff,
                SemanticPose::QuickstepBackContact,
            ),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(12u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), takeoff)?;
            builder.pose(motion, AuthoredAnimationFrame::from(12u16), contact)?;
        }
        for (motion, pose) in [
            (&MotionId::PRONE_CRAWL, SemanticPose::ProneCrawlContact),
            (
                &MotionId::SUPINE_SCAMPER,
                SemanticPose::SupineScamperContact,
            ),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        // Pre-reflected copies of the sparse gait anchors are selected only
        // as mirrored blend endpoints; they own no independent semantics.
        builder.motion(
            &MotionId::WALK_MIRRORED,
            AuthoredAnimationFrame::from(64u16),
        );
        builder.motion(&MotionId::RUN_MIRRORED, AuthoredAnimationFrame::from(64u16));
        builder.motion(
            &MotionId::PRONE_CRAWL_MIRRORED,
            AuthoredAnimationFrame::from(0u16),
        );
        builder.motion(
            &MotionId::SUPINE_SCAMPER_MIRRORED,
            AuthoredAnimationFrame::from(0u16),
        );
        builder.motion(&MotionId::DIVE, AuthoredAnimationFrame::from(0u16));
        for pose in [
            SemanticPose::DiveForward,
            SemanticPose::DiveBackward,
            SemanticPose::DiveLeft,
            SemanticPose::DiveRight,
        ] {
            builder.pose(&MotionId::DIVE, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        for (motion, pose) in [
            (&MotionId::AIRBORNE_CENTER, SemanticPose::AirborneCenter),
            (&MotionId::AIRBORNE_TRAVEL, SemanticPose::AirborneTravel),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        add_attack_motions(&mut builder)?;
        for (motion, pose) in [
            (&MotionId::PRONE_TRANSITION, SemanticPose::ProneTransition),
            (
                &MotionId::PRONE_SUPINE_ROLL_LEFT,
                SemanticPose::ProneSupineRollLeft,
            ),
            (
                &MotionId::PRONE_SUPINE_ROLL_RIGHT,
                SemanticPose::ProneSupineRollRight,
            ),
            (&MotionId::SUPINE_TRANSITION, SemanticPose::SupineTransition),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        Ok(builder.finish())
    }
}

fn add_attack_motions(builder: &mut PackBuilder) -> Result<(), CatalogError> {
    builder.variable_motion(
        &MotionId::SWING,
        AuthoredAnimationFrame::from(4u16),
        AuthoredAnimationFrame::from(12u16),
    );
    for (frame, pose) in [
        (AuthoredAnimationFrame::from(0u16), SemanticPose::GuardSwing),
        (
            AuthoredAnimationFrame::from(4u16),
            SemanticPose::AttackSwing,
        ),
        (
            AuthoredAnimationFrame::from(8u16),
            SemanticPose::RecoverSwing,
        ),
        (
            AuthoredAnimationFrame::from(12u16),
            SemanticPose::ContinueSwing,
        ),
    ] {
        builder.pose(&MotionId::SWING, frame, pose)?;
    }
    builder.variable_motion(
        &MotionId::THRUST,
        AuthoredAnimationFrame::from(4u16),
        AuthoredAnimationFrame::from(12u16),
    );
    for (frame, pose) in [
        (
            AuthoredAnimationFrame::from(0u16),
            SemanticPose::GuardThrust,
        ),
        (
            AuthoredAnimationFrame::from(4u16),
            SemanticPose::AttackThrust,
        ),
        (
            AuthoredAnimationFrame::from(8u16),
            SemanticPose::RecoverThrust,
        ),
        (
            AuthoredAnimationFrame::from(12u16),
            SemanticPose::ContinueThrust,
        ),
    ] {
        builder.pose(&MotionId::THRUST, frame, pose)?;
    }
    builder.variable_motion(
        &MotionId::OFFHAND,
        AuthoredAnimationFrame::from(0u16),
        AuthoredAnimationFrame::from(4u16),
    );
    builder.pose(
        &MotionId::OFFHAND,
        AuthoredAnimationFrame::from(0u16),
        SemanticPose::GuardOffhand,
    )?;
    builder.pose(
        &MotionId::OFFHAND,
        AuthoredAnimationFrame::from(0u16),
        SemanticPose::AttackOffhand,
    )?;
    builder.pose(
        &MotionId::OFFHAND,
        AuthoredAnimationFrame::from(4u16),
        SemanticPose::AttackOffhandPrepared,
    )?;
    Ok(())
}

fn add_locomotion_motions(builder: &mut PackBuilder) -> Result<(), CatalogError> {
    for (motion, last_frame, anchors) in [
        (
            &MotionId::WALK,
            AuthoredAnimationFrame::from(64u16),
            [
                (
                    AuthoredAnimationFrame::from(0u16),
                    SemanticPose::WalkContact,
                ),
                (
                    AuthoredAnimationFrame::from(16u16),
                    SemanticPose::WalkPassing,
                ),
            ],
        ),
        (
            &MotionId::RUN,
            AuthoredAnimationFrame::from(64u16),
            [
                (AuthoredAnimationFrame::from(0u16), SemanticPose::RunContact),
                (AuthoredAnimationFrame::from(16u16), SemanticPose::RunFlight),
            ],
        ),
    ] {
        builder.motion(motion, last_frame);
        for (frame, pose) in anchors {
            builder.pose(motion, frame, pose)?;
        }
    }
    builder.motion(&MotionId::COMBAT_STANCE, AuthoredAnimationFrame::from(0u16));
    builder.pose(
        &MotionId::COMBAT_STANCE,
        AuthoredAnimationFrame::from(0u16),
        SemanticPose::CombatStance,
    )?;
    // The authored combat locomotion files expose keys at 0/6/12/18. The
    // publication pipeline closes each cycle by copying frame 0 to frame
    // 24, so frames 6/18 remain the two authoritative contacts.
    for (motion, pose) in [
        (&MotionId::STRAFE, SemanticPose::StrafeCycle),
        (&MotionId::SKIP, SemanticPose::SkipCycle),
    ] {
        builder.motion(motion, AuthoredAnimationFrame::from(24u16));
        builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
    }
    Ok(())
}
