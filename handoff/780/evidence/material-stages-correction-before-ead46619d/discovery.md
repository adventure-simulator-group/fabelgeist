# Stage fixture Ratio and source order: pre-repair all13 extension

Pinned HEAD `ead46619dec5ef45993f63202cb289bd3986b018`. Full current stages.rs
read: 1–352.

This narrow discovery is complete before repair. The reviewer made no source
change or project execution. It does not clear stage units/errors or broader
material work.

At line308 replace only `start(&|i| i.selection.mail.roughness = 0.9)` with
`start(&|i| i.selection.mail.roughness =
fabelgeist_math::units::Ratio::from_ratio(0.9))`. Ratio::from_ratio is the
existing lower owner and preserves the same f32 literal. The existing
mail-appearance assertion remains the same case.

Move complete blocks, retaining docs/attrs/cfg/vis and original blank
separators. Within StageSettings impl retain header/associated constants, move
full apply method with doc before full validate method; no function body edit.
Reverse exact permutation and one RHS constructor must reproduce whole original
source.

Final order: prefix; StageSettings declaration with docs attrs; DrapeSettings
declaration with docs attrs; DrapeCheckpoints declaration with docs attrs;
DrapeStart enum with docs; StageSettings impl (associated constants, apply+doc,
validate); DrapeSettings impl; DrapeCheckpoints impl; DrapeInput impl; test
module last.

All original production bodies, fixture helpers, cases, assertions, strings,
documentation and attributes remain exact after reversing the declared
permutation and literal constructor. No tests, cases, helpers, assertions or
doctests may be added.

1. **producer-propagation**: Existing fixture is a native numeric literal
producer for MailWeave.roughness, which is canonical Ratio; insert exact
unit-named leaf at literal source, no primitive adapter.

2. **existing-type-owners**: Math units::Ratio is already the actual canonical
owner; reuse it and no alternate Roughness wrapper/dependency.

3. **admission-contracts**: Ratio::from_ratio preserves f32 exactly and adds no
finite/range policy; 0.9 remains valid MailWeave roughness. Existing tests
intentionally use existing data structures, no API bypass is introduced.

4. **invariant-operations**: Only assignment RHS changes. The closure still
mutates only mail roughness, not any simulation field; no arithmetic or
operation order changes.

5. **aggregate-states**: DrapeStart::Finish current same_setup excludes mail
appearance. Existing assertion remains same contract/case. Source
setup/sewn/settled transition relations remain exact.

6. **type-ownership**: No owner move: lower Math Ratio and existing MailWeave
remain canonical. Source order repair stays in Root-owned Creator file and
changes no protected native function.

7. **redundant-authority**: No new state/default/derived copy is introduced.
Existing body/checkpoint fields and source comparisons remain byte-exact.

8. **behavior-preservation**: All three existing stage tests and all
assertions/helpers remain byte-for-byte except one existing literal
construction. Production bodies stay byte-exact after block permutation
reversal; native programs untouched.

9. **enum-shapes**: Existing StageSettings/DrapeSettings/DrapeCheckpoints
records and DrapeStart payload enum remain unchanged. No new role/flag or state
transition.

10. **derivability**: Existing derives remain exact and attached to the same
struct; no manual trait implementation added. Reordering is mechanical, not
behavioral.

11. **type-granularity**: Ratio distinguishes dimensionless authored roughness;
stage counts, gravity and damping remain current types and explicitly outside
this correction. Do not infer raw stage-unit clearance.

12. **magic-strings**: All strings/docs/errors/native host API names in moved
blocks are byte-preserved. No prose classification or API text changes.

13. **magic-numbers**: Only source literal0.9 gains the existing Ratio unit
constructor; all stage constants, zero/empty policies, defaults and other
fixture numeric values remain exact.

Source hashes/read extents and exact block hashes are in the JSON ledger. Before
source is /tmp/pr780-stage-literal-before.rs.

The actual checkpoint source corrects the earlier MP-6 hypothesis: completed
mail-only edits select DrapeStart::Finish; this extension does not assert a
solver rerun.

