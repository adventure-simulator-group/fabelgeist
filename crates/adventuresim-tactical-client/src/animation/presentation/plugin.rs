use super::*;

impl Plugin for TacticalAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationPackCatalog>()
            .init_resource::<pose_buffer::PoseBufferMetrics>()
            .init_resource::<pose_buffer::RigDefinitions>()
            .init_resource::<pose_buffer::BakedClipBank>()
            .init_resource::<AuthoredLocomotionStrides>()
            .init_resource::<AnimationRuntime>()
            .init_resource::<semantic_route::SemanticRouteTelemetry>()
            .init_resource::<TerrainIkEnabled>()
            .init_resource::<ProceduralAnimationClock>()
            .init_resource::<procedural::FixedTickPoseCache>()
            .init_resource::<secondary_physics::SecondaryPhysicsTelemetry>()
            .register_required_components::<procedural::HumanoidBone, secondary_physics::SecondaryBoneDynamics>()
            .add_message::<LocomotionPresentationEvent>()
            .add_systems(Startup, request_animation_packs)
            .add_systems(Update, (
                super::super::skeletal_proportions::load_skeletal_bases,
                super::super::skeletal_proportions::sync_skeletal_proportions,
            ).chain().after(capture_authored_bind_transforms).before(pose_buffer::update_pose_buffers))
            .add_observer(on_successful_attack)
            .add_systems(
                Update,
                (
                    collect_loaded_packs,
                    attach_loaded_rig_scenes,
                    super::super::identity_morphs::sync_character_morphs,
                    update_presented_skeletons,
                    establish_animation_targets,
                    procedural::bind_humanoid_bones,
                    procedural::cache_humanoid_rigs,
                    full_ragdoll::sync_full_ragdolls,
                    full_ragdoll::resolve_ragdoll_terrain_contacts,
                    capture_authored_bind_transforms,
                    procedural::capture_humanoid_rig_axes,
                    semantic_route::evaluate_semantic_route_paths,
                    evaluate_skeletons,
                    tick_impact_reactions,
                    pose_buffer::update_pose_buffers,
                    pose_buffer::calibrate_authored_locomotion_strides,
                    pose_buffer::calibrate_character_strides,
                    update_rig_visibility,
                    emit_locomotion_presentation_events,
                    trace_locomotion_presentation_events,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    procedural::restore_procedural_look_base,
                    pose_buffer::apply_pose_buffers,
                    restore_authored_bind_pose,
                    procedural::apply_pose_mirroring,
                    procedural::apply_procedural_dive_lower_body,
                    procedural::apply_locomotion_height,
                    procedural::orient_guarded_run_lower_body,
                    procedural::apply_landing_leg_compression,
                    procedural::apply_locomotion_body_response,
                    procedural::apply_jump_anticipation,
                    procedural::apply_head_and_torso_look,
                    secondary_physics::apply_secondary_bone_physics,
                    procedural::apply_terrain_leg_ik,
                    procedural::enforce_anatomical_knee_yaw,
                    procedural::apply_arm_and_weapon_constraints,
                    full_ragdoll::apply_full_ragdoll_pose,
                    procedural::stabilize_repeated_fixed_tick_pose,
                )
                    .chain()
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                (
                    procedural::refresh_raised_support_after_propagation,
                    // Diagnostics must observe the final global transforms
                    // that the renderer receives, including procedural IK.
                    log_animation_diagnostics,
                )
                    .chain()
                    .after(TransformSystems::Propagate),
            )
            .add_systems(Update, super::super::diagnostics::report_system_spikes);
    }
}
