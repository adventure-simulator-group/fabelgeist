# Retarget reports

A rig profile describes how to recognize a skeleton's joints. A source-to-target
retargeting recipe resolves two such profiles into a joint mapping. Its rendered
diagnostics are a `RetargetReport`, shared by `ResolvedProfile::report` and
`Retargeter::report`, rather than an arbitrary string or a profile label.

```rust
use fabelgeist_animation::animation::retarget::RetargetReport;

let report: RetargetReport = resolved.report(&source, &target);
println!("{report}");
let complete: RetargetReport = retargeter.report();
println!("{complete}");
```

Keep the artifact typed through storage, parameters and return values. `Clone`
and report-to-report equality are available. Use `Display` or `to_string()` at
printing, encoding or text assertion boundaries. `Debug` retains the inner
String's quoting and escaping, including Unicode, newlines and NUL. There is no
arbitrary-text constructor, extraction API, default report or serde contract.

The mapping report retains the recipe and rig labels, joint counts and original
joint spellings. Roles follow the humanoid vocabulary's order, with absent
counterparts shown as `<unmapped>`. Roles absent on both sides are omitted.
Declared chains present on both sides are printed only when their lengths
differ, retaining joint order and ` -> ` separators. Optional missing-role lists
retain their stored order. Unmapped skeleton joint indices remain separate data
in `ResolvedRig::unmapped`; the renderer does not enumerate them.

The retargeter appends the scale with four fractional digits and, when present,
end effectors in their existing order. An end effector is a hand, foot or head
used as the end of a mapped limb. No report rendering operation changes profile
resolution, error precedence, clip motion or sampled animation values.

The lowest shared owner is `animation::retarget::report`, reexported at
`animation::retarget`. It owns the completed artifact and mapping renderer; the
retargeter retains the footer because it owns the skeletons and measured scale.
A report neither recognizes names nor acquires exact joint identity. Canonical
joint-name integration remains a separate prerequisite for the broader joint
and track migration. Indices, chain lengths and scale remain unchanged here.
