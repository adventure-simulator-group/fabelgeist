# Receiving result ownership review

Pinned Root `ccb1b6944d674c0bf29d10e6820c0ac4c55092b7`; tree
`4cc54cebccefd4eb249900fb22d160cdaba53485`. The complete focused source account and
all13 contracts are in the JSON. No source edits or project execution occurred.

RS-R1 (P3): `TextureValue::try_from` at `texture_value.rs:32` hides the trait
associated error behind `TextureValueResult<Self>`. The explicit exception at
`crates/AGENTS.md:118–122` requires trait error parameters to remain visible.
Use `std::result::Result<Self, Self::Error>` there. Retain the owning alias for
nontrait fetch/store consumers and preserve its actual error and method body.

Root selected RS-S1 as an accepted focused cleanup: use one `StagingResult<T>`
for the two `StagedResults` decoding methods. The same rule explicitly permits standard
results where errors differ, and this module also contains facade APIs returning
`GenerateError`; therefore repeated explicit results do not independently prove
a must-fix violation. Keep the facade family separate. Never widen errors or
replace source causes merely to shorten signatures.

Texture admission retains its private scalar/lane relationship and typed wrong-
type error. Staging retains exact session/ticket membership, checked byte extent,
zero-sized rejection, fallible allocation, zero-prefix behavior and native
readback/source chains. Those bodies and native shader tokens need no change for
the proposed result spelling.

Current old body/part/bracer/breastplate/Creator callers still require the agreed
borrowed fallible ticket API and canonical grouped Readback receipt. The Root
Readback API currently differs from the facade's expected canonical API. These
are required #780 dependencies, not repaired or cleared by alias spelling.

No tests, cases, helpers, assertions, doctests, native loops, waits, traits,
error variants or compatibility APIs were authored. Both focused owners are
small; alias belongs before structs. Relevant final hosted baseline reductions,
complete connected integration and final same-head reviews/CI remain required.
