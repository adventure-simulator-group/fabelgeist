# Bespoke-type contract review

Use this mode to assess the guarantees of affected semantic families across
their actual data flow. A correct wrapper or passing compiler does not establish
a complete migration.

## Establish the scope and claims

Identify the reviewed revision and comparison base, the families the change
claims to migrate, and the user's accepted deferrals. Inspect the complete
current source for those responsibilities, not only changed lines or the latest
repair. Include related unchanged producers and consumers without expanding into
an unsolicited whole-codebase audit.

Start the architectural assessment from the source, requested behavior and
accepted constraints. Implementation plans, dispositions and previous verdicts
are claims to check afterward, not evidence of correctness or completeness. Do
not independently turn a requested current family into future work. Preserve
explicitly accepted deferrals instead of silently reopening them.

A repair, baseline-only check or publication audit may retain earlier results,
but its conclusion must identify that limited scope. Repeated narrow passes do
not become a fresh architectural review. This mode does not itself authorize
additional agents, source changes, new tests, external posts or broader tools.

## Trace each family

For every family claimed complete, follow the actual path and retain the source
locations that establish its guarantees:

    owner -> admission -> producers -> retained state -> operations
          -> consumers -> serialization/framework boundary

Establish all of the following, grouping fields that share a real contract:

- Meaning: units, coordinate frame, identity or authority, signedness, valid
  zero, bounds, and any supported extrapolation, wrapping or overflow policy.
- Canonical owner: where that meaning belongs and whether an existing type
  actually matches. Equal underlying numbers or physical units do not establish
  equal roles or valid ranges. Move a shared owner when needed; avoid parallel
  equivalent types.
- Admission: every constructor, conversion, decoder, deserializer, reflection
  path and default that can create the value. A checked constructor is
  insufficient when another construction path bypasses it.
- Producers and storage: fields, parameters, return values and meaningful
  collections carry the guarantee. Find upstream raw values, repeated internal
  admission and premature primitive extraction. Name the actual native kernel or
  external interface when a primitive handoff is justified.
- Operations: accumulation, interpolation, scaling and replacement preserve the
  contract or return an appropriate failure. Finite operands can overflow;
  infallible arithmetic traits are unsuitable when failure must be reported.
  Check whether a failure can leave partially updated state.
- Relationships: ordered bounds, positive denominators, correlated enum
  payloads, matching collection lengths/order, receiving-instance membership and
  cached classifications stay consistent. Independently valid fields do not
  prove a valid record. Check visibility and mutation after admission.
- Boundary behavior: encodings and diagnostics retain the intended contract.
  Distinguish deliberate API/schema changes from accidental changes to zero,
  tiny values, sign, clamping, normalization, lookup precedence or error order.

Inspect every materially distinct path in the family. If coverage is incomplete,
name the unexamined path and leave completeness unverified. A few constructor
checks cannot stand in for a complete trace.

## Challenge the guarantees

Try to construct a concrete misuse that the claimed contract should prevent:
wrong-role values, malformed admission, a bypassing deserialize/derive,
arithmetic that produces an invalid replacement, mismatched columns or
independent mutation that invalidates cached or relational state. Trace existing
call sites rather than relying solely on hypothetical signatures.

A static counterexample can establish an API or completeness gap without proving
a current runtime failure. Label that distinction. Do not invent bounds,
behavior or requirements to manufacture findings; establish the intended
contract from source, usage, tests and applicable guidance. If material policy
is unresolved, state the ambiguity and its consequence.

Run relevant existing checks within the task's execution and test constraints.
Report which behavior they exercise. Compilation, lint success, execution counts
and previous reviewer approval do not prove unexercised boundary or runtime
behavior. Do not author tests or probes beyond the user's authorization.

## Calibrate the abstraction

Do not equate more wrappers with stronger design. Open identifiers may preserve
arbitrary text; closed roles may need enums; ordinary predicates may return
bool. Authored literals, fixtures, format tokens and actual native numerical
kernels can legitimately use primitives. A named field alone does not enforce
its unit or bound, and a file split alone does not establish ownership.

A contribution that can exceed one is not a bounded influence. An ordinal is not
proof of membership in a particular collection. An elapsed interval allowing
zero is not by itself proof of a positive recovery duration. Close the relevant
contract without imposing unrelated stricter policies or elaborate branding.

Derives are appropriate when they preserve admission and representation. For
example, derived deserialization through validated TryFrom can retain a checked
scalar contract; ordinary transparent deserialization may bypass it. Do not ban
or require a derive merely because the handwritten implementation looks short.

Check overlapping migrations for duplicate owners or incompatible behavior when
evidence of overlap is available. Honor repository numerical-comparison policy;
renaming a magic value or replacing machine epsilon with an arbitrary constant
is not a semantic remedy.

## Report evidence and completion

Keep the visible review concise and actionable. Separate demonstrated defects,
incomplete migrations, useful proposed strengthening and accepted follow-up
work. Cite applicable repository rules for actual violations; do not present a
design suggestion as a requirement merely because this skill considers it.

For each reviewed family, give a compact source-backed trace in prose or a small
table: identify its canonical owner, meaningful producer/consumer locations,
verified admission and operation guarantees, the actual boundary, and remaining
limits. This is required even when there are no findings; an unchecked checklist
or a list of type names is insufficient. Avoid a ceremonial per-field ledger.

State correctness findings and migration completeness separately. Describe a
family as complete, partial, an accepted follow-up, or unverified, according to
the evidence. A sound increment can have accepted follow-ups; it must not claim
the broader migration is finished. If only some families were reviewed, identify
the rest as unverified rather than granting an overall pass.

For an issue, show the concrete scenario or representable invalid state, exact
location, material consequence and smallest coherent correction. Prefer closing
one producer-to-consumer family over unrelated cleanup. Preserve accepted error
or other migration deferrals and distinguish source inspection from executed
validation and production acceptance.
