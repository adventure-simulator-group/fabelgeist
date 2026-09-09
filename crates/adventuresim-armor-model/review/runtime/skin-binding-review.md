# Runtime skin binding review

Disposition: no concrete P1/P2 identified in this bounded read-only review of the equipment skin/layout initialization correction. This is a code-review disposition, not native runtime or visual acceptance.

## Reviewed lifecycle

- `equipment/render_binding.rs`: worn parts remain hidden while skin binding is absent. The original skinned mesh is retained in `EquipmentMeshSources.bound`; dropping creates and caches a separate mesh without joint-index/weight attributes. The shared source mesh is unchanged. Re-equipping restores the original handle, and later drops reuse the joint-free clone.
- `equipment/skin.rs`: all named wearer joints must resolve before `SkinnedMesh` is inserted. An unavailable/removed wearer or scene item removes the binding. Transfers to another wearer replace the joints.
- `equipment/visuals.rs`: the chained skin, morph, render-binding, and placeholder systems apply deferred mutations between dependent operations. This prevents a rendered frame with a dropped joint-bearing mesh or a re-equipped joint-free mesh under the wrong skin layout.
- `equipment/morphs.rs`: cloning the mesh preserves named morph metadata; the same names let weights update before either cached handle is selected. Scene items reset weights; wearer transfers calculate the new wearer's weights.
- `equipment.rs`: resolved mesh parts start hidden; placeholder roots retain existing scene/world placement and wearer-rig reparenting. Part visibility becomes inherited, so it does not override a root hidden while the rig scene is unavailable.

## Evidence and limits

Inspected `target/armor-review/occupancy/render-binding-test.log`: the focused lifecycle test records 1 passed, 0 failed. The test checks initial hiding, bound visibility, joint-free dropping, source immutability, retained morph names, hidden re-equipping while skin is absent, original-handle restoration, and dropped-mesh cache reuse. The log also contains a preceding network connection warning; it does not negate the recorded test pass.

No tests or builds were run by this reviewer. Native capture validation after the layout correction remains owned by the coordinator/implementation agent. This review does not certify all-body armor fitting, asset promotion, or visual quality. The earlier tasset-flare finding was separately resolved.

Final extraction reinspection: `ProceduralEquipmentPart::render_bundle` now owns initial hidden visibility, mesh/material, identity transform, no-frustum-culling and outline components. `resolve_procedural_equipment_models` calls that bundle as the same root child. This is equivalent to the reviewed initialization and introduces no additional finding.
