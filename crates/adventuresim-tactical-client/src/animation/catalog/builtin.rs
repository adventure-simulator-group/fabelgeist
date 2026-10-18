//! Authored built-in motion catalogues and their complete semantic validation.

use super::*;

impl AnimationPackCatalog {
    pub(in crate::animation) fn biped_root() -> Result<Self, CatalogError> {
        let (id, pack) = PackCatalog::unarmed()?;
        let mut two_handed = PackBuilder::new(
            &AnimationPackId::HUMANOID_2H_CLOSE,
            "humanoid",
            Some(&AnimationPackId::HUMANOID_UNARMED),
            "animations/biped/2h_close",
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
            "humanoid",
            None,
            "animations/biped/unarmed",
        );
        for (motion, pose) in [
            ("idle_relaxed", SemanticPose::IdleRelaxed),
            ("prone_idle", SemanticPose::ProneIdle),
            ("supine_idle", SemanticPose::SupineIdle),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        add_locomotion_motions(&mut builder)?;
        for (motion, takeoff, contact) in [
            (
                "quickstep_forward",
                SemanticPose::QuickstepForwardTakeoff,
                SemanticPose::QuickstepForwardContact,
            ),
            (
                "quickstep_right",
                SemanticPose::QuickstepRightTakeoff,
                SemanticPose::QuickstepRightContact,
            ),
            (
                "quickstep_left",
                SemanticPose::QuickstepLeftTakeoff,
                SemanticPose::QuickstepLeftContact,
            ),
            (
                "quickstep_back",
                SemanticPose::QuickstepBackTakeoff,
                SemanticPose::QuickstepBackContact,
            ),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(12u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), takeoff)?;
            builder.pose(motion, AuthoredAnimationFrame::from(12u16), contact)?;
        }
        for (motion, pose) in [
            ("prone_crawl", SemanticPose::ProneCrawlContact),
            ("supine_scamper", SemanticPose::SupineScamperContact),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        // Pre-reflected copies of the sparse gait anchors are selected only
        // as mirrored blend endpoints; they own no independent semantics.
        builder.motion("walk_mirrored", AuthoredAnimationFrame::from(64u16));
        builder.motion("run_mirrored", AuthoredAnimationFrame::from(64u16));
        builder.motion("prone_crawl_mirrored", AuthoredAnimationFrame::from(0u16));
        builder.motion(
            "supine_scamper_mirrored",
            AuthoredAnimationFrame::from(0u16),
        );
        builder.motion("dive", AuthoredAnimationFrame::from(0u16));
        for pose in [
            SemanticPose::DiveForward,
            SemanticPose::DiveBackward,
            SemanticPose::DiveLeft,
            SemanticPose::DiveRight,
        ] {
            builder.pose("dive", AuthoredAnimationFrame::from(0u16), pose)?;
        }
        for (motion, pose) in [
            ("airborne_center", SemanticPose::AirborneCenter),
            ("airborne_travel", SemanticPose::AirborneTravel),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        add_attack_motions(&mut builder)?;
        for (motion, pose) in [
            ("prone_transition", SemanticPose::ProneTransition),
            ("prone_supine_roll_left", SemanticPose::ProneSupineRollLeft),
            (
                "prone_supine_roll_right",
                SemanticPose::ProneSupineRollRight,
            ),
            ("supine_transition", SemanticPose::SupineTransition),
        ] {
            builder.motion(motion, AuthoredAnimationFrame::from(0u16));
            builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
        }
        Ok(builder.finish())
    }
}

fn add_attack_motions(builder: &mut PackBuilder) -> Result<(), CatalogError> {
    builder.variable_motion(
        "swing",
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
        builder.pose("swing", frame, pose)?;
    }
    builder.variable_motion(
        "thrust",
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
        builder.pose("thrust", frame, pose)?;
    }
    builder.variable_motion(
        "offhand",
        AuthoredAnimationFrame::from(0u16),
        AuthoredAnimationFrame::from(4u16),
    );
    builder.pose(
        "offhand",
        AuthoredAnimationFrame::from(0u16),
        SemanticPose::GuardOffhand,
    )?;
    builder.pose(
        "offhand",
        AuthoredAnimationFrame::from(0u16),
        SemanticPose::AttackOffhand,
    )?;
    builder.pose(
        "offhand",
        AuthoredAnimationFrame::from(4u16),
        SemanticPose::AttackOffhandPrepared,
    )?;
    Ok(())
}

fn add_locomotion_motions(builder: &mut PackBuilder) -> Result<(), CatalogError> {
    for (motion, last_frame, anchors) in [
        (
            "walk",
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
            "run",
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
    builder.motion("combat_stance", AuthoredAnimationFrame::from(0u16));
    builder.pose(
        "combat_stance",
        AuthoredAnimationFrame::from(0u16),
        SemanticPose::CombatStance,
    )?;
    // The authored combat locomotion files expose keys at 0/6/12/18. The
    // publication pipeline closes each cycle by copying frame 0 to frame
    // 24, so frames 6/18 remain the two authoritative contacts.
    for (motion, pose) in [
        ("strafe", SemanticPose::StrafeCycle),
        ("skip", SemanticPose::SkipCycle),
    ] {
        builder.motion(motion, AuthoredAnimationFrame::from(24u16));
        builder.pose(motion, AuthoredAnimationFrame::from(0u16), pose)?;
    }
    Ok(())
}
