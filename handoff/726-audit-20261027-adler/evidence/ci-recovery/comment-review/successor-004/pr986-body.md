CI stops at incomplete equipment fixtures and stale procedural snapshots,
while several material and cloud tests repeatedly generate data unrelated to
their assertions. Leaf sampling also changes across native builds because shoot
identifiers are reconstructed from floating-point positions.

This change restores the existing fixture contracts and carries stable,
seed-derived identifiers from leaf-bearing shoots through leaf generation,
sampling, inspection and prepared-tree fingerprints.

- Complete the existing one-joint garment rig and equipment image resources.
- Reserve runtime texture handles in material fixtures and inspect committed
  sampler containers directly. Share the production cloud-ray integrator with
  the existing bounded ray and complete-seam checks.
- Keep native wood geometry unchanged while propagating shoot identity. Use the
  existing 0.98 near-card width calibration for beech, hazel geometry revision 3,
  and tree-bake revision 18; refresh their measured snapshots and dependent
  scene, city, furniture and cloud assets with the existing generators.
- Derive the closed-door sightline fixture from current generated door geometry
  and initialize the studio-cache fixture's actual asset resources.
- Add source-tagged hosted regeneration and focused Windows execution of ten
  existing foliage/prepared-asset cases alongside ordinary Linux quality checks.
  Retain the existing source-size ceilings and lower the two skeleton ceilings
  justified by the extraction.

## Validation

The [complete hosted fixture refresh](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40246244270)
passed the hazel exporter, all three selected scene generators, the paired city
export, cloud regeneration and identity recheck, the six-bake tree exporter,
seven understory cases and five leaf cases. Generated outputs were checked
against the recorded source revision before integration.

The [hosted baseline export](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40248503551)
provided the strict skeleton reductions. A source-equivalent nine-word hash loop
keeps the existing tree-impostor ceiling without losing identifier bits.

Full ordinary Linux and focused Windows verification is pending on this PR's
final head. The PR remains draft until those checks and final output review pass.

## Implementation context

Shoot identity uses the producer's deterministic seed rather than rounded or
quantized positions. This intentionally changes shoot-salted leaf realization;
existing cases use current-generator fixtures and regenerated snapshot values.
The woodland and closed-door cases derive their specimens from current geometry
instead of obsolete fixed captures. Production cloud arithmetic, pixel order and
filtering are preserved. The bounded cloud cases do not claim repeated
full-image equality.

Independent Code, Rule, Comment and scoped thirteen-lens source reviews have
been performed. Final generated-output reconciliation and hosted acceptance
remain separate gates. The Windows cases check the same committed snapshots
and prepared geometry fingerprints; no platform-specific expectations are added.
