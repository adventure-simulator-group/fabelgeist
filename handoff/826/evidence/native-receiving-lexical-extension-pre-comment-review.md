# Lexical source extension: pre-repair Comment discovery

Root `19a6b62687713b9ecfa7aa137003f11bd662baf9`, tree
`f8c67de65206ca90efe21e54c6c9e2e9bafa9cc3`.

The connected source-selection weakness reaches all five generated signature
parsers, native Scan/Reduce binary projection, Min/Max comparison renaming and
Gather/Scatter/Stream/Stencil rewriting. Current selection and replacement run
separate regexes on unmasked text. This is the same G4/shared-source family,
not a new catalog of wording findings.

Twenty whole source files were freshly read, two binary-consumer files have
explicit bounded reads, and six same-pin complete native source reads carry
with exact hashes and current contract reconciliation. The JSON gives every
extent and hash. Proposed new owner bodies receive no review credit.

## Required documentation contracts

- `FunctionSource` is a borrowed lexical view of one uniquely selected named
  function, retaining original source and byte-addressed spans. Its
  comment-masked text keeps identical UTF-8 byte positions. It is not a
  complete WGSL parser, type validator or compiler.
- Actual Naga 29.0.4 source uses nested block comments. Line comments end at
  U+000A through U+000D, U+0085, U+2028 or U+2029. The new mask must match those
  rules and preserve original slicing positions, including multibyte text.
  No earlier complete Naga parse or wider acceptance policy follows.
- NativeParameter classifies visible name/type fields from masked text while
  SourceParameter keeps the original trimmed declaration. Do not claim
  original-source preservation while re-emitting masked comments or replacing
  authored alias/wrapper spelling with reconstructed types.
- Rewriters use the same selected spans as admission. G4 changes only the
  actual comparison declaration identifier. Generated header projection may
  intentionally remove metadata parameters or lower reserved neighborhood
  syntax; document that edit accurately. Preserve retained declarations and
  source outside the intended spans, including body/comments/globals. Stencil
  neighborhood lowering must avoid comments and incidental substrings.
- Binary and comparison adapters project the **first argument type**. They do
  not establish agreement of operand/return types or validity of the body.
  `ComparisonSignature::parse/value_type/renamed_source` consumes the common
  owner; remove the old comparison parser rather than add a forwarding API.
- Duplicate visible declarations must be rejected as ambiguous before renaming
  one could make invalid source valid. Do not describe selecting the first
  textual match as admitting the unique intended declaration.
- Public mutable authored code remains reparsed and keyed as current source.
  Complete generated-program Naga/native admission still happens after role
  and neighborhood lowering. Lexical success proves no compiled pipeline,
  native acceptance, GPU completion, rollback or later device health.

The SDK hash and exact comment/line-end excerpts are in the JSON. This is source
inspection only: no edits, execution, check, build, scanner, generation, test,
new case/helper/assertion/probe, push or CI. External fixture bodies were not
freshly reread in this bounded extension; post-repair preservation evidence and
agreed caller adapters remain required.

Ownership/API agreements, complete Root pre-repair inventory, actual
implementation and full post-repair reviews remain necessary. Wider
Reduce/Scan/descriptor/stride/producer/caller/lock and final same-pushed-SHA
all13/all3/hosted gates are open. No family or global clearance is granted.
