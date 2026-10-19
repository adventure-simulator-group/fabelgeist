//! Per-root transform, role and complete selected-child bijection.
use super::*;

pub(super) struct RootInventory {
    pub(super) members: Vec<Member>,
    pub(super) visible: Vec<Entity>,
}

impl Inspector<'_, '_> {
    pub(super) fn check_root(&self, owned: &OwnedTrees, root: &OwnedRoot) -> Result<RootInventory> {
        self.cache.verify(&root.cache, self.review.representation)?;
        let (local, global, visibility, children, presented) = self.roots.get(root.cache.root())?;
        ensure!(
            *local == root.transform
                && global.to_matrix() == local.to_matrix()
                && *visibility == Visibility::Inherited
                && presented,
            "owned root transform/visibility/capability drift"
        );
        let children = children.context("owned specimen children")?;
        ensure!(
            children.len() == owned.bindings.len(),
            "incomplete or extra owned child inventory"
        );
        let mut members = Vec::with_capacity(children.len());
        let mut visible = Vec::new();
        for child in children.iter() {
            let SpecimenPartItem {
                entity,
                mesh,
                local,
                global,
                visibility,
                in_view,
                bark,
                leaf,
                aggregate,
                card,
                standard,
                interior,
                original,
                parent,
                descendants,
                unbatchable,
            } = self.parts.get(child)?;
            ensure!(
                unbatchable
                    && parent.parent() == root.cache.root()
                    && descendants.is_none_or(|children| children.is_empty()),
                "unexpected nested or reparented specimen member"
            );
            let binding = owned
                .bindings
                .iter()
                .find(|binding| binding.mesh == mesh.0.id())
                .context("unadmitted owned mesh")?;
            let (authored, effective) =
                match (bark, leaf, aggregate, card, standard, interior, original) {
                    (Some(material), None, None, None, None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, Some(material), None, None, None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, None, Some(material), None, None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, None, None, Some(material), None, None, None) => {
                        (material.0.id().untyped(), material.0.id().untyped())
                    }
                    (None, None, None, None, None, Some(material), Some(original)) => {
                        (original.0.id().untyped(), material.0.id().untyped())
                    }
                    _ => anyhow::bail!("owned material absent, ambiguous or not converted"),
                };
            self.roles
                .check(entity, binding.part, &root.cache, &self.cache)?;
            let expected_visibility = self.review.representation.visibility(binding.part);
            ensure!(
                authored == binding.material
                    && *local == Transform::IDENTITY
                    && global.to_matrix() == root.transform.to_matrix()
                    && *visibility == expected_visibility,
                "owned binding/transform/selection drift"
            );
            if in_view.get() {
                visible.push(entity);
            }
            members.push(Member {
                entity,
                root: root.cache.root(),
                part: binding.part,
                mesh: mesh.0.id(),
                material: effective,
                authored_material: authored,
                transform: global.to_matrix(),
                visibility: *visibility,
            });
        }
        for binding in &owned.bindings {
            ensure!(
                members
                    .iter()
                    .filter(|member| member.mesh == binding.mesh)
                    .count()
                    == 1,
                "owned inventory is not a bijection with admitted parts"
            );
        }
        Ok(RootInventory { members, visible })
    }
}
