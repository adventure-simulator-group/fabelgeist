# Rust agent guide

These instructions apply to hand-written Rust source and Cargo manifests under
`crates/`. Files in `adventuresim-stdb-client/src/` are generated and are
governed by the repository-root generation rule instead.

## Style and design sources

- Use the default [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
  through `rustfmt`; do not maintain a competing hand-formatted style.
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
  unless a repository-specific contract is more precise.
- For agent-authored Rust, also apply Microsoft's
  [AI Guidelines](https://microsoft.github.io/rust-guidelines/guidelines/ai/):
  prefer idiomatic APIs, strong domain types, testable boundaries, behavioral
  tests, and documentation of the resulting design rather than the design
  process.
- Prefer direct, readable code over cleverness, speculative abstractions,
  compatibility scaffolding, or duplicated paths.

## Semantic values and types

- Treat a literal as magic when it encodes a domain unit, scale, bound,
  probability, tuning decision, protocol or status value, route, query, or
  error classification whose meaning is not intrinsic at the use site.
  Repetition is evidence of a shared concept, but a one-use literal can still
  be magic.
- Put a semantic value in the module that owns its meaning and give it a domain
  name. Reuse an existing owner before creating another constant, type, or
  helper.
- Default to bespoke domain types for domain values, including identifiers,
  units, quantities, states, validated values, and meaningful collections. Use
  enums, newtypes, and validated structs so invalid combinations are rejected
  by the compiler. Keep raw strings, numbers, and containers at explicit system
  boundaries or where the value is genuinely primitive.
- Repeated primitive-to-bespoke conversions at internal call sites indicate an
  incomplete producer migration. Carry the bespoke type through producer
  fields, return types and parameters; convert at the actual literal, decoding,
  storage or framework boundary. Avoid extracting a primitive only to admit it
  again in the next application operation. Numeric fixtures and required native
  adapters remain valid boundaries; conversion count is a review signal, not a
  blanket prohibition on `From` or `TryFrom`.
- Do not carry a naked primitive through domain logic when a bespoke type can
  express its unit, invariant, authority, or allowed state more precisely.
- Represent domain records with small structs and named fields. Avoid positional
  primitive tuples in parameters and return values, including tuples nested in
  containers. Prefer named records for local domain tables and recipe entries
  as well.
- Give each record field its semantic domain type, reusing existing owners and
  enforcing relevant invariants. Naming a field does not justify leaving its
  domain unit, bound, or state encoded in a naked primitive.
- Unit-named primitive leaf constructors may accept native numbers or vectors
  at literal, decoding and framework boundaries. Composite domain APIs accept
  existing checked leaves. Carry those leaves between operations rather than
  repeatedly extracting and admitting native values. Document the frame, unit
  and reason for each native numerical-kernel handoff; being private does not
  alone justify a primitive API.
- Use enums for closed domain roles and states. Distinguish an ordinal index
  from the role selected by that index: use a domain index type when meaningful
  and classify it into a role at the owning boundary. Do not replace arbitrary
  indices with enums that merely enumerate numbers.
- Use named enums for domain roles or states carried between APIs, including
  two-state roles when the variants communicate meaning. Keep `bool` for
  ordinary named predicates and genuinely primitive flags at explicit adapters.
  A Boolean wire representation does not determine the domain type.
- Unit `()` and tuples required by external or framework APIs are valid. Keep
  required positional representations at those boundaries and convert domain
  records through explicit adapters.
- Define each domain concept once in its lowest shared owning module. Import and
  reuse that type, its constants, and its conversions; never create equivalent
  bespoke types independently in multiple files.
- Do not create constants whose names merely restate their values. Keep
  intrinsically clear `0` and `1`, small indexes and dimensions, authored
  catalog data, isolated test fixtures, and local format tokens inline when a
  name would only add indirection.
- Inline authored geometry must make its component role, units and axis/frame
  interpretation clear at construction. Shared dimensional relationships,
  acceptance thresholds and tolerances have a named owner. An authored-catalog
  registry scope does not exempt policy logic embedded in the same file.
- Give semantically distinct pair members named fields, or destructure a fixed
  homogeneous pair into meaningful names. Use checked access for externally
  supplied or otherwise unproven dynamic indices. Direct indexing is valid for
  fixed arrays and bounds established by construction or control flow; do not
  invent missing-element outcomes for guaranteed elements.
- Do not branch on mutable human-facing prose. Return a stable typed error or
  code from the authoritative boundary and map it to presentation text once.

## Modules and files

- After module documentation and inner attributes, order file-level items as:
  `use` imports and re-exports; `mod` declarations; constants, statics and type
  aliases; structs; enums; traits; impl blocks; then functions, public before
  private. Put the `#[cfg(test)]` module last, whether inline or declared in
  another file. Keep documentation and attributes attached to their items.
- Split modules by responsibility, not to satisfy a mechanical line limit.
  Roughly 400 lines of production code is a healthy target and 500 lines is a
  design smell; exclude unit tests from that judgment.
- Before materially expanding a production file that is already near or above
  that range, extract the responsibility being added into a focused module.
- Split a shorter file when it owns multiple distinct concerns. A longer file
  may remain intact when it is genuinely cohesive and splitting would obscure
  the design.
- Small helper modules, including double-digit-line files, are encouraged when
  they own a clear concern. Do not merge them merely to avoid small files. If a
  helper grows large, check whether it has accumulated multiple concerns.
- Extract stable responsibilities into real modules and move their tests and
  module or type documentation with them. Do not simulate modularity with
  section-banner comments.
- Avoid pass-through modules, services, traits, and wrappers that add no
  policy, invariant, boundary, or reusable abstraction.

## Functions, methods, and constructors

- Use descriptive names for domain parameters and locals, such as `assembly`,
  `centre`, `dimensions` or `clearance`. Reserve short names for obvious
  mathematical axes and tightly scoped iterator/index variables; a visible type
  does not by itself justify abbreviating a domain role.
- In a cohesive module whose fallible APIs share one error type, use or reuse
  an owning `Result<T>` alias. Where error types differ, preserve the
  distinctions with descriptive aliases or explicit standard results. Do not
  widen an error solely to shorten signatures. Framework and trait error
  parameters remain explicit.
- Keep control flow flat with pattern matching (`if let`, `while let`, and
  `let ... else`) and early returns. Prefer `?` for error propagation rather
  than nesting success and failure branches.
- Never use `unwrap` or `expect` outside tests. Propagate a typed error or handle
  the relevant cases explicitly.
- When an operation has a clear receiver and its main argument is a local
  struct, define it as an inherent method on that struct.
- Define functions that construct a local type as associated constructors.
  Use `new` for the primary construction path, a domain verb when that is
  clearer, and `from_*` for construction from a particular representation.
  Prefer `From` or `TryFrom` when the source type fully determines the
  conversion.
- Keep a free function when an operation is symmetric across multiple types,
  belongs to the module rather than one type, or must satisfy a trait or
  callback signature.
- Prefer self-documenting parameter types over positional booleans, numbers,
  or ambiguous `Option` values.

## Lints, tests, and optimization

- The `fabelgeist-rust-quality check` command enforces the semantic-value
  registry, raw-string branching rules, Cargo lint inheritance, and the
  500-line file and 100-line function no-growth ceilings. `just lint` runs it
  automatically.
- Keep legitimate fixtures, authored catalogs, shader source, and external
  boundary adapters in `rust-quality.toml`. Every scope or exception must be
  narrow and reasoned; the checker rejects entries that no longer match.
- Use the checker's `census` subcommand to review repeated values. Its output is
  advisory: repetition suggests a concept worth reviewing but does not make an
  otherwise ordinary value illegal.
- After splitting an oversized module or migrating a semantic family, run the
  `baseline` subcommand and apply only the reductions relevant to that change.
  Never raise a ceiling to accommodate new production debt.
- Use `#[expect(..., reason = "...")]` for a localized lint exception. Reserve
  `#[allow(...)]` for generated code and macro expansions where `#[expect]`
  cannot be used reliably.
- Never write unit tests for an object unless the user explicitly requests
  them.
- When writing tests, cover observable behavior, failure paths, and domain
  invariants. Do not add tautological tests that merely repeat constants or
  mirror the implementation.
- Optimization is not a prototyping goal. Do not optimize without a measured
  problem, an acceptance constraint, or an explicit user request. A
  readability-neutral lint fix is ordinary maintenance, not permission for a
  performance redesign.
- Run `just fmt-check` and `just lint` for Rust changes. Use narrower package
  checks while iterating, then run the applicable final gates.
