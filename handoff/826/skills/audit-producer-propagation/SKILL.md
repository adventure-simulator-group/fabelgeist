---
name: audit-producer-propagation
description: "Trace unfinished bespoke-type producer migrations through internal APIs and retained state. Use for this focused lens or a bespoke-audit assignment."
---

# Producer propagation

Follow each migrated quantity from its original producer through storage,
operations and consumers to its actual wire, framework or numerical boundary.

- Inventory primitive fields/returns, repeated admission, `.into()`, primitive
  `From`, casts, getters and encodings. Follow every materially distinct path;
  finding a checked constructor does not finish the trace.
- Identify producers that still emit raw values and consumers that immediately
  reconstruct the same owner. Carry the guarantee across those application APIs.
- For every primitive escape, locate the receiving operation. Document its
  actual native requirement; a private function or explanatory comment alone
  does not make a boundary.
- Preserve units, clock/authority distinctions, rounding, saturation, wrapping
  and legitimate authored or fixture admission. Do not flag conversion syntax
  by itself.

For each gap, name the earliest unfinished producer and the connected changes
needed to close that family. Deduplicate symptoms without dropping paths.

## Primitive-escape canaries

Inspect accessors as well as explicit conversions. Examples include:

- Time: `.seconds()`, `.as_secs_f32()`.
- Distance and speed: `.metres()`, `.metres_per_second()`.
- Angles and cycles: `.radians()`, `.degrees()`, `.cycles()`.
- Rates: `.hertz()`, `.per_second()`, `.phase_units_per_second()`.
- Fractions and coefficients: `.fraction()`, `.phase_units()`, `.factor()`,
  `.gain()`, `.as_unit_f32()`.
- Counts and identities: `.encoded()`, `.encoded_sequence()`, `.samples()`,
  `.events()`, `.count()`, `.to_u64()`.
- Coordinates and arrays: `.metres()` returning a vector, `.as_vec2()`,
  `.as_vec3()`, `.to_array()`, `.as_array()`.
- Text identities: `.as_str()`, `.into_string()`, `.to_string()`.
- Generic accessors: `.get()`, `.value()`, public tuple fields such as `.0`.

This list is illustrative, not exhaustive. Check the actual return type:
getters such as `.elapsed()` or `.speed()` can return another checked type and
preserve the guarantee. Include `.into()`, primitive `From` conversions and
`as` casts in the same investigation.

Trace extracted values through intermediate locals, returned values, retained
fields, callbacks and trait implementations to the receiving operation. Strong
signals are an internal primitive parameter, later reconstruction of the same
bespoke type, primitive application storage, or identity text driving logic.
Inspect callers outside the changed-file list too.

A final arithmetic, rendering, serialization or logging operation may need the
primitive. Establish that concrete need rather than treating every accessor as
a defect or accepting a private helper as a boundary without reading its body.

Compilation exposes incompatible signatures. A getter feeding an unchanged
primitive parameter still compiles, so green CI and a changed-file inventory
do not establish complete producer-to-consumer coverage. Record untraced paths
as unfinished coverage rather than a family-level completion verdict.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
