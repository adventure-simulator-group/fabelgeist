//! Exact GPU membership, geometry/material joins and three-way tree batches.
use super::*;
use bevy::{
    math::{Affine3, Affine3Ext},
    pbr::MeshUniform,
};
use bindings::RenderMember;
use commands::{Dispatch, Receipt};
use decode::PhaseData;
use gpu_commands::{GpuCommand, WorkBudget};
use members::MemberRole;

// Native MeshUniform packs the material slot into its lower sixteen bits.
const MATERIAL_SLOT_MASK: u32 = u16::MAX as u32;

struct Audit<'a> {
    members: &'a [RenderMember],
    expected: &'a [TaggedMember],
    seen: Vec<MeshTag>,
    tree_batches: Vec<serde_json::Value>,
    commands: Vec<serde_json::Value>,
    submitted_commands: usize,
    submitted_instances: u64,
    nonempty_commands: usize,
    budget: WorkBudget,
}

pub(super) fn verify(
    pose: &Pose,
    submission: &proof::Submission,
    phases: &[PhaseData],
) -> Result<serde_json::Value> {
    let mut audit = Audit {
        members: &submission.members,
        expected: &pose.expected,
        seen: Vec::new(),
        tree_batches: Vec::new(),
        commands: Vec::new(),
        submitted_commands: 0,
        submitted_instances: 0,
        nonempty_commands: 0,
        budget: WorkBudget::new(),
    };
    for receipt in &submission.commands {
        ensure!(
            receipt.frame == submission.frame && receipt.camera == pose.camera,
            "stale GPU command owner"
        );
        if !matches!(receipt.dispatch, Dispatch::Indirect { .. }) {
            ensure!(
                !submission
                    .members
                    .iter()
                    .any(|member| receipt.compatible(member)
                        && receipt.binding.geometry == member.binding.geometry),
                "owned member used a nonindirect dispatch"
            );
            continue;
        }
        let data = phases
            .iter()
            .find(|data| data.phase == receipt.phase)
            .context("submitted phase snapshot missing")?;
        for command in GpuCommand::read(receipt, data, &mut audit.budget)? {
            audit.command(receipt, command, data)?;
        }
    }
    ensure!(
        audit.seen.len() == pose.expected.len()
            && pose
                .expected
                .iter()
                .all(|member| audit.seen.contains(&member.tag)),
        "missing expected GPU member"
    );
    ensure!(
        !audit.tree_batches.is_empty(),
        "no actual three-way tree command"
    );
    Ok(
        serde_json::json!({"role":pose.role, "view":pose.view, "frame":submission.frame,
        "camera_entity_bits":pose.camera_bits, "gpu_membership_verified":true,
        "expected_membership":pose.expected.iter().map(TaggedMember::report).collect::<Vec<_>>(),
        "render_members":submission.members.iter().map(RenderMember::report).collect::<Vec<_>>(),
        "api_color_dispatches":submission.commands.len(),
        "submitted_indirect_commands":audit.submitted_commands,
        "nonempty_indirect_commands":audit.nonempty_commands,
        "submitted_instances_including_unowned":audit.submitted_instances,
        "owned_instances":audit.seen.len(), "tree_batches":audit.tree_batches,
        "actual_commands":audit.commands,
        "terrain_background_complete":true,
        "scope":"actual main-color indirect dispatches and decoded compacted GPU instance membership; held camera/entity/material bindings; no exact screenshot-frame timestamp, prepass/shadow, browser, adaptive, arrival, vertex/texture-byte or performance acceptance"}),
    )
}

impl Audit<'_> {
    fn command(&mut self, receipt: &Receipt, command: GpuCommand, data: &PhaseData) -> Result<()> {
        self.submitted_commands = self
            .submitted_commands
            .checked_add(1)
            .context("command counter overflow")?;
        self.budget.instances(command.instance_count)?;
        self.submitted_instances = self
            .submitted_instances
            .checked_add(u64::from(command.instance_count))
            .context("instance counter overflow")?;
        let end = command
            .first_instance
            .checked_add(command.instance_count)
            .context("compacted output range overflow")?;
        let range = usize::try_from(command.first_instance)?..usize::try_from(end)?;
        let instances = data
            .meshes
            .get(range)
            .context("compacted output range exceeds logical shader bound")?;
        if command.instance_count > 0 {
            self.nonempty_commands += 1;
        }
        let mut owned = Vec::new();
        for instance in instances {
            // Admit the primitive at the native MeshUniform-to-MeshTag boundary.
            let tag = MeshTag(instance.tag);
            if tag.0 == 0 {
                continue;
            }
            let member = self
                .members
                .iter()
                .find(|member| member.member.tag == tag)
                .context("unexpected submitted nonzero mesh tag")?;
            ensure!(
                !self.seen.contains(&tag),
                "duplicate submitted owned mesh tag"
            );
            verify_member(receipt, &command, instance, member)?;
            self.seen.push(tag);
            owned.push(member);
        }
        let tree_members: Vec<_> = owned
            .iter()
            .copied()
            .filter(|member| matches!(member.member.role, MemberRole::Tree { .. }))
            .collect();
        if !tree_members.is_empty() {
            self.tree_batch(receipt, &command, instances, &tree_members)?;
        }
        self.commands.push(serde_json::json!({"phase":receipt.phase, "parameter":command.parameter,
            "geometry":command.geometry.report(), "first_instance":command.first_instance,
            "instance_count":command.instance_count,
            "instances":instances.iter().map(|instance| serde_json::json!({"tag":instance.tag,
                "world_from_local":instance.world_from_local.map(|column|column.to_array()),
                "first_vertex_index":instance.first_vertex_index,
                "material_and_lightmap_slot":instance.material_and_lightmap_bind_group_slot})).collect::<Vec<_>>()}));
        Ok(())
    }

    fn tree_batch(
        &mut self,
        receipt: &Receipt,
        command: &GpuCommand,
        instances: &[MeshUniform],
        owned: &[&RenderMember],
    ) -> Result<()> {
        let Some(first) = owned.first() else {
            anyhow::bail!("empty owned tree command");
        };
        let MemberRole::Tree { part, .. } = first.member.role else {
            anyhow::bail!("non-tree batch classification");
        };
        let expected: Vec<_> = self
            .expected
            .iter()
            .filter(|member| {
                matches!(member.role,
            MemberRole::Tree { part:expected, .. } if expected == part)
            })
            .collect();
        let required = layout::TreeLayout::Shared.root_count();
        ensure!(
            expected.len() == required,
            "expected tree part is not one per root"
        );
        ensure!(
            usize::try_from(command.instance_count)? == required && owned.len() == required,
            "tree part was split or mixed instead of one shared three-instance command"
        );
        let mut expected_tags: Vec<_> = expected.iter().map(|member| member.tag.clone()).collect();
        let mut actual_tags: Vec<_> = instances
            .iter()
            .map(|instance| MeshTag(instance.tag))
            .collect();
        expected_tags.sort_by_key(|tag| tag.0);
        actual_tags.sort_by_key(|tag| tag.0);
        ensure!(
            actual_tags == expected_tags,
            "actual tree command tag multiset differs"
        );
        let mut roots = Vec::new();
        for member in expected {
            let MemberRole::Tree { root, .. } = member.role else {
                anyhow::bail!("tree part role changed");
            };
            ensure!(
                !roots.contains(&root),
                "duplicate root within shared tree part"
            );
            roots.push(root);
        }
        self.tree_batches
            .push(serde_json::json!({"part":part, "phase":receipt.phase,
            "parameter":command.parameter, "instance_count":command.instance_count,
            "tags":actual_tags.iter().map(|tag|tag.0).collect::<Vec<_>>(),
            "root_bits":roots.iter().map(|root|root.to_bits()).collect::<Vec<_>>()}));
        Ok(())
    }
}

fn verify_member(
    receipt: &Receipt,
    command: &GpuCommand,
    actual: &MeshUniform,
    member: &RenderMember,
) -> Result<()> {
    ensure!(
        receipt.compatible(member),
        "actual command pipeline/material-type/group/slab differs from owned member"
    );
    ensure!(
        command.geometry == member.binding.geometry,
        "actual command geometry allocation differs from owned mesh"
    );
    ensure!(
        actual.first_vertex_index == member.binding.geometry.first_vertex(),
        "GPU instance first vertex differs"
    );
    ensure!(
        actual.material_and_lightmap_bind_group_slot & MATERIAL_SLOT_MASK == member.material_slot,
        "GPU instance material slot differs"
    );
    let expected = Affine3::from_mat4(member.member.transform).to_transpose();
    ensure!(
        actual
            .world_from_local
            .iter()
            .all(|column| column.is_finite())
            && actual.world_from_local == expected,
        "GPU instance world transform differs"
    );
    Ok(())
}
