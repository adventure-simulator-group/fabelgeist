//! Actual entity/part/terrain identity joined to an explicit diagnostic tag.
use super::*;
use bevy::asset::UntypedAssetId;
use inventory::PartIdentity;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TaggedMember {
    pub tag: MeshTag,
    pub entity: Entity,
    pub role: MemberRole,
    pub mesh: AssetId<Mesh>,
    pub material: UntypedAssetId,
    pub transform: Mat4,
    pub native_visibility: NativeVisibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum NativeVisibility {
    Visible,
    NotVisible,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MemberRole {
    Tree { root: Entity, part: PartIdentity },
    Terrain(background::Role),
}

impl TaggedMember {
    pub(super) fn collect(
        current: &inventory::CurrentInventory,
        tags: &HashMap<Entity, MeshTag>,
    ) -> Result<Vec<Self>> {
        let mut members = Vec::new();
        for member in &current.members {
            members.push(Self {
                tag: tags.get(&member.entity).context("owned tree tag")?.clone(),
                entity: member.entity,
                role: MemberRole::Tree {
                    root: member.root,
                    part: member.part,
                },
                mesh: member.mesh,
                material: member.material,
                transform: member.transform,
                native_visibility: if current.visible.contains(&member.entity) {
                    NativeVisibility::Visible
                } else {
                    NativeVisibility::NotVisible
                },
            });
        }
        for member in &current.background {
            members.push(Self {
                tag: tags
                    .get(&member.entity)
                    .context("owned terrain tag")?
                    .clone(),
                entity: member.entity,
                role: MemberRole::Terrain(member.role),
                mesh: member.mesh,
                material: member.material,
                transform: member.transform,
                native_visibility: NativeVisibility::Visible,
            });
        }
        ensure!(
            members.len() <= MAX_FIXTURE_MEMBERS,
            "GPU expected-member fixture budget"
        );
        members.sort_by_key(|member| member.entity);
        for (index, member) in members.iter().enumerate() {
            ensure!(
                member.tag.0 != 0
                    && members[..index]
                        .iter()
                        .all(|earlier| earlier.tag.0 != member.tag.0
                            && earlier.entity != member.entity),
                "GPU tag/member bijection"
            );
        }
        Ok(members)
    }

    pub(super) fn report(&self) -> serde_json::Value {
        let role = match self.role {
            MemberRole::Tree { root, part } => {
                serde_json::json!({"kind":"tree", "root_bits":root.to_bits(), "part":part})
            }
            MemberRole::Terrain(role) => serde_json::json!({"kind":"terrain", "role":role}),
        };
        serde_json::json!({"tag":self.tag.0, "entity_bits":self.entity.to_bits(), "role":role,
            "mesh":format!("{:?}",self.mesh), "material":format!("{:?}",self.material),
            "world_from_mesh":self.transform.to_cols_array(),
            "native_visibility":self.native_visibility})
    }
}
