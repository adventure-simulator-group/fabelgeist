# Lexical source extension: pre-repair Code discovery

Root `19a6b62687713b9ecfa7aa137003f11bd662baf9` (tree
`f8c67de65206ca90efe21e54c6c9e2e9bafa9cc3`). This completes source discovery for
the named bounded extension: all five generated signatures,
NativeParameter/source association, all four authored argument rewrites and
Stencil Neighbors lowering, with actual definition/native/cache/fixture
consumers. Repairs remain open. No production edits or project execution
occurred.

The shared GPU FunctionSource proposal is appropriate when it owns one original
borrow, one same-byte comment mask and private admitted source spans. It must
supply one canonical named declaration to role parsing and rewriting; no
independent whole-source fn regex or second comment scanner remains.

## Current actionable defects

### LEX-G1 (P2)

All five regexes apply captures directly to original source. A leading complete
commented signature can provide different parameters/output than the real
following declaration; role bindings, call arguments, output types and native
resource declarations are then generated from the comment. Missing declarations
can also be admitted from comments alone.

Source: map/signature.rs:24-51; gather/signature.rs:24-70;
scatter/signature.rs:23-73; stream/signature.rs:24-78;
stencil/signature.rs:26-93.

All five consume the single GPU FunctionSource selection/header/argument/return
authority; no private schema regex selects fn declarations from whole source
afterward.

### LEX-G2 (P2)

A leading comment containing only fn gather(decoy) is ignored by the full
signature regex (no body brace), but selected by the shorter rewrite regex. A
following real gather with a Resource parameter is correctly parsed but never
stripped, so generated WGSL retains an invalid metadata type. Scatter/Stream and
non-neighborhood Stencil have the same independent shorter-selector structure.

Source: gather/program.rs:16-24; scatter/program.rs:16-24;
stream/program.rs:13-21; stencil/bindings.rs:112-117.

Rewrite only the admitted argument span from that same FunctionSource owner. Do
not independently search the original source again, and preserve all bytes
outside intentional header/type substitutions.

### LEX-G3 (P2)

The raw argument splitter treats punctuation inside comments as angle depth,
argument commas and name/type colons. For example, a valid native comment
containing a colon between the parameter identifier and real colon becomes
InvalidParameter rather than the actual index/value role. Inline type comments
also interfere with exact coordinate/resource/offset classification.

Source: source_parameter.rs:57-105; map/signature.rs:31-48;
gather/signature.rs:31-66; scatter/signature.rs:30-67;
stream/signature.rs:31-75; stencil/signature.rs:33-83.

Consume paired original and same-byte visible argument slices from the lexical
owner; split/classify using visible tokens and derive each original
declaration/type/name range from the same offsets. Preserve original diagnostics
and declarations rather than substituting the masked source into shaders.

## Required design preservation

- LEX-A1: A retained declaration ending in // must keep a real line terminator
  before the comma or closing parenthesis. Current trim and join(", ") can
  discard the only newline after a line comment. Once visible-token role lexing
  admits such valid source, copying a trimmed raw declaration can comment out
  generated delimiters. Preserve sufficient original trivia or use safe newline
  separators, without mutating body/comment text outside the admitted span.

- LEX-A2: Retain successful real Neighbors type uses beyond the selected stencil
  header. Current replace_all rewrites every real matching NeighborsND<T>
  occurrence when a neighborhood is present. Helper function parameters/returns
  and local type annotations using the admitted neighborhood type can therefore
  succeed today. Replacing only the selected argument type would leave
  unsupported pseudo-generic types in those same sources. Use the same
  comment-aware source/span authority for intentional real type substitutions;
  preserve comments and unrelated expressions/globals. Do not invent a narrower
  source restriction without an existing authoritative contract.

- LEX-A3: Real duplicate target names must remain rejected. Renaming only one
  declaration or stripping only one duplicate header can turn formerly invalid
  duplicate declarations into valid separate declarations. The FunctionSource
  selection must explicitly reject an ambiguous real named declaration set;
  never choose silently. Same native/comment source identity applies to all
  seven named protocols.

- LEX-A4: Mask and original source must have identical byte lengths with
  admitted UTF-8 slice boundaries. Native WGSL consumes nested block comments
  and line endings U+000A..000D, U+0085, U+2028/U+2029. Pattern blankspace
  additionally includes U+200E/U+200F. One canonical lexical owner handles
  these; no second per-algorithm scanner. Original source remains the only
  emitted authored text.

The Stencil type-lowering point qualifies “preserve the body”: authored text
outside intentional argument/name edits and real reserved neighborhood type
envelopes stays byte-exact. Existing successful reserved-type uses in helper
signatures, local/global type annotations and constructors must keep that
lowering. Restricting replacements to only the selected parameter would be an
unintended behavior restriction.

## Thirteen source-backed dispositions

- producer propagation: The GPU lexical owner must reach binary/comparison
  helpers, all five schema parsers, NativeParameter and every argument/type
  rewrite, while public definitions/execute/build always use the current public
  code field. Disposition: repair required. Source: GPU signature;
  source_parameter; Map/Gather/Scatter/Stream/Stencil entrypoints and private
  programs.

- magic numbers: Byte spans and fixed grammar punctuation are native source
  metadata. std byte ranges are appropriate privately; do not reuse GPU buffer
  offsets or add a competing count/stride owner. Existing NeighborhoodDimension
  owns 1..4 dimensions. Disposition: intentional source boundary. Source:
  FunctionSource proposal; NativeParameter char_indices;
  NeighborhoodParameter/Dimension.

- magic strings: One selected native name is authoritative; GeneratedFunction
  names retain actual algorithm protocol/error messages. Reserved
  index/size/offset/center/val/resource names retain their semantic role
  precedence. Disposition: preserve with centralized source selection. Source:
  all five signature role matches, GeneratedFunction::name and prior native
  operator error.

- existing type owners: Keep ResourceBaseType, ShaderParameter,
  CoordinateParameter/Shape, NeighborhoodParameter and typed
  GeneratedProgramError. NativeParameter/SourceParameter should consume lexical
  slices, not introduce a second type grammar or erase original declarations.
  Disposition: reuse required. Source: complete source_parameter, signature and
  neighborhood owners.

- aggregate states: Original borrow, same-length visible mask, unique
  name/header/argument/return spans and selected declaration form one private
  immutable admitted source. Each NativeParameter pairs original declaration
  with the role derived from its visible slice. Disposition: proposed owner
  appropriate; pairing required. Source: source_parameter.rs and all five
  SourceParameter::new sites.

- admission contracts: Real function token/declaration selection ignores
  complete native comments, rejects ambiguous names and preserves
  MissingFunction/MissingReturn/InvalidParameter/coordinate/native error
  ownership. No earlier full Naga parsing of pseudo-types or generated-context
  globals. Disposition: LEX-G1/G3 and A3/A4 open. Source: fresh parsers and
  pinned Naga lexer.

- invariant operations: Rewrites consume admitted spans, preserve UTF-8
  boundaries and line-comment terminators, and apply multiple edits without
  offset drift or overlapping replacements. Emit original retained declarations
  and intended real neighborhood aliases only. Disposition: LEX-G2 and A1/A2/A4
  open. Source: four rewrites, NativeParameter trim, stencil replace_all.

- redundant authority: No raw whole-source fn regex remains in schemas or
  programs. A build operation selects once and routes that authority to role
  parsing and transformation; contextual bindings stay derived from current
  source/resource recipe. Definition admission and later builds may reselect
  current mutable code at their separate existing boundaries. Disposition:
  repair required. Source: definition/build/cache/execute bodies, signature
  parsers and independent rewrite regexes.

- enum shapes: Keep ordered Index/Size/Value/Resource and fieldless
  Other/Offset/Center cases with existing real payloads. Do not collapse source
  association into semantic booleans or reintroduce independently mutable
  declarations/roles. Disposition: preserve settled GS-STATE1. Source: five
  private role enums and SourceParameter.

- type granularity: FunctionSource is a lexical relationship owner, not an
  all-purpose WGSL validator. Keep algorithm semantic schemas owned by Compute
  and native value/coordinate grammar at existing owners; do not add unrelated
  supported features. Disposition: bounded shared abstraction appropriate.
  Source: actual parse/role/native assembly split.

- type ownership: GPU owns one native lexical selection/mask responsibility
  usable by Compute without a dependency cycle. Compute owns semantic role
  admission/rewrite policy. Parent reports peer API agreement received; reviewer
  makes no source edit. Disposition: agreed ownership; implementation pending.
  Source: GPU/Compute module boundaries and actual source error owners.

- derivability: Existing derived Eq/Hash on owned schemas and canonical payloads
  should remain semantically equivalent if storage changes. Prior D2 proof
  remains valid for Error-only derive, with exact manual Display/source(None)
  and no new GPU dependency. Disposition: preserve; D2 separate pending repair.
  Source: current schema derives; prior pinned thiserror proof.

- behavior preservation: Preserve source-order arguments, explicit-index
  priority, Stream optional input/offset behavior, Stencil center/val/fallback,
  Custom types, exact shader math/resources/workgroups/signed-neighborhood
  bounds, success-only caches and current error order. Do not expand the
  unsupported neighbor+secondary Buffer path or introduce a new feature promise.
  Disposition: acceptance required; no behavior execution. Source: fresh private
  programs/callers plus exact prior neighborhood/layout/fixture source.

## Acceptance

- One GPU FunctionSource lexical owner borrows original source and owns exactly
  one same-byte visible comment mask; all admitted source spans are private,
  tied to that source and immutable. No second comment scanner/parser in
  Compute.

- Canonical selection recognizes actual fn/name tokens, native comment trivia
  and unique real named declaration, not search hits in comments or longer
  identifiers. Binary and comparison helpers consume it; ComparisonSignature
  parse/value_type/renamed_source remains the agreed public comparison
  relationship.

- All five signatures parse that owner’s argument/header/optional-return views.
  NativeParameter consumes original/visible paired views, classifies only
  visible tokens, preserves original declaration and typed original diagnostic
  payloads, and retains source-order roles.

- Gather/Scatter/Stream/non-neighborhood Stencil rewrite the same admitted
  argument span, never perform another raw fn regex search, and preserve every
  authored byte outside intentional replacements. Retained line comments cannot
  consume generated separators/closing syntax.

- Stencil neighborhood lowering reuses the same lexical/source association for
  real matching type uses, including existing successful helper signatures/local
  type annotations. Comment text and unrelated body/global text remain exact;
  generated alias spelling comes from the admitted neighborhood/type owner, not
  a competing raw regex projection.

- Duplicate real target names reject explicitly; singular edits cannot silently
  convert invalid duplicate declarations into valid differently named functions.
  Multiple intended source edits must avoid offset drift and overlap.

- No full Naga parse moves ahead of generated metadata lowering or context
  binding, because Resource/Neighbors pseudo-types and generated globals make
  standalone source parsing inappropriate. Native shader/pipeline errors retain
  actual causes and occur at existing actual admission boundaries.

- Preserve MissingFunction/MissingReturn/InvalidParameter/DuplicateParameter and
  CoordinateParameter native causes, NativeOperatorSignatureError roles/Display,
  malformed/unsupported neighborhood policy and settled unit role cases. Do not
  broaden numeric/source grammar merely to close lexical selection.

- All public definitions retain current-code recipe/cache identity and
  success-only publication. Input/output/workgroup/packing/count/stride/native
  progress behavior stays unchanged; remaining owner integration cannot be
  called cleared.

- Existing five-family fixtures, literals, values, cases, helpers and assertions
  remain unchanged. No new tests/cases/helpers/assertions/probes; do not claim
  coverage for absent Scatter or lexical-comment fixtures.

- Read-only pre-repair inventory only. Root complete all13/three independent
  reviews, connected owner integration and final same-pushed-SHA hosted/baseline
  gates remain required.

## Coverage and evidence

All 51 sources from this reviewer’s complete prior generated wave were compared
with the exact current Git pin: 50 are byte-identical. The sole different
TextureValue file exactly matches this reviewer’s separate full result-style
successor review. The JSON distinguishes fresh full current reads from exact
qualified full-body carries and carries no old lexical verdict.

Fresh complete reads cover all five role schemas,
NativeParameter/SourceParameter, actual definitions/build/cache/execute methods,
Gather/Scatter/Stream programs, Stream argument emission, Stencil
bindings/program/neighborhood owner, Map bindings/program, coordinate causes,
relevant native grid/fetch consumers and actual existing fixture source. Current
numerical neighborhood/layout/recipe bodies with exact prior hashes are
qualified carries.

Existing Map, Gather, Stream and Stencil fixture modules and existing helper
bodies were read, with their unchanged source literals/operands/oracles. There
is no scoped Scatter fixture module. The preceding focused review read
Min/Max/Reduce fixtures and Scan operands. Existing fixtures do not establish
the new lexical edge scenarios; none was created or run.

MapBindings fetch replacements and NeighborhoodFetch placeholder substitutions
operate on framework-generated snippets, so they are intentional/non-applicable
to the authored declaration-selection repair. Fluid ResourceParameters remains
source-independent.

No local build, check, test, lint, scanner, generator, parser/regex/macro probe,
numerical execution or hosted run occurred. Root canonical
count/stride/ResourcePipeline/Scan/Reduce/caller/export/lock composition remains
required. Complete final all13, all three reviews and hosted/baseline gates on
the same final pushed revision remain open.

## Per-file source accounts

- `crates/fabelgeist-compute/src/advect/mod.rs`: qualified complete-body carry
  from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `d8daf7ae1365b035842b1731b92c4df1a903f9f714f0c20f8add90f402d1b9a9`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/advect/program.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `9b8fdcd03904db882c1cf5a470a8cf2bf0c6b18906d32868053736bb7245c583`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/divergence/mod.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `5a78332aaf68f30b99277c9b4b05e62ba8d9e3d1d596235622bd1011b2dd5a94`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/divergence/program.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `3a1bdc53478c9050161e2367f4c1ffb5de5b4320acbe8ddedda44e9617489e1a`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/gather/mod.rs`: fresh complete current source;
  all actual functions and existing fixtures read; SHA-256
  `fc7d06ae42f46f8cb79b83eb27b82fe175df972668d5762f4e411d7b6481cb11`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/gather/program.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `bacc25932f7cc1a95d6831c870111186644a9581603d94da93aa932fe8895589`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/gather/recipe.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `f9f1191683c0890a728199fbca3a598436d356ce1857b8ab96ccdafd354bebf8`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/gather/signature.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `e638381ad701a4b95e91bc4e3128d8c84526fa81cab3e535895aba1af3c3f975`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/gather/tests/mod.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `dbe8d4a8b46fe877571190636697e5562371b4803d7959ea74802f4701ddf928`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/gradient/mod.rs`: qualified complete-body carry
  from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `c8aeb37ef3b15273143fa486bf2fe0112b61b680d97960b6a3096583457ca2d0`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/gradient/program.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `abe90e298f9e600275b4fb939eda8668752e73c3f509ecb101372800226c4af7`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/map/bindings.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `9eb07ca6eda0d217c4c6a598bfc21e8e0eed0617dd501040a1101ca7ae7d0d76`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/map/mod.rs`: fresh complete current source; all
  actual functions and existing fixtures read; SHA-256
  `65ca55becef0b0c833137a2a1fb84babb0c0b0b1557709e1f7b6d6e96340154e`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/map/program.rs`: fresh complete current source;
  all actual functions and existing fixtures read; SHA-256
  `a31b1d79247334aa2e945b5ca80ca18e7078eee9cf021b4b9984d67e37a1b511`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/map/recipe.rs`: qualified complete-body carry
  from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `2b74f16ffced873fe963fa9e529ae410a73f41140d10334818b26c8663bef420`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/map/signature.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `b6b44fdecaee32315772c9f8c3891382e8a1c8afe63adc32d1f47f9fd81a259b`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/map/tests/helper.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `9f52c94d50003be0a234ae69eaa07605cf90a82bf1edeea6dd2ce82ebcd4debc`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/map/tests/mod.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `b5a49e81728272b5bb04064a40395b62442969f9e4fcf10dbb0d13e1149d30e9`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/resource_parameters.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `7adb4ee8a1c9700e34b85246e60d890c44e34504fec74e312424726759107364`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/resource_pipeline.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `be1947333e9c7e412076674fe073ff1e17261f02a4d757ad4b295ada422649ff`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/resource_recipe.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `620d3f2477548bfff9c9c15e5ba64e534a9ee2e2a756d1b1dbeeff64dd792ac2`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/scatter.rs`: fresh complete current source; all
  actual functions and existing fixtures read; SHA-256
  `ddc7310cd5ed6f1777cab41bafed49dbac575d7b582cfceb42136452a85656a6`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/scatter/program.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `8b9abcab3f6b7682c61d65c1b1382ea7aae18b2a556ca94a7420190322cd4d3c`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/scatter/signature.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `6fbe78a21b4e9f9bdab5486d9320f3c90923f8923cd75088fdc35a51a834a4d7`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/source_error.rs`: qualified complete-body carry
  from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `5be0e86d52971e11461b1148c710cbc98a8a47628f798c51f68ed6136d17e832`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/source_parameter.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `7b896ef0e0884eea5e583405df72784da6578f7d10941cd42f042c007668bebf`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil.rs`: fresh complete current source; all
  actual functions and existing fixtures read; SHA-256
  `30b16468bef923b7a1642f35e3868ecaf34c61f0bcf1d7cdaabd84aa57655b56`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/bindings.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `bce3a66580252ce1353e2ae500dc11ea596deea17d46155a2f4fd9584365846d`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/neighborhood.rs`: fresh complete
  current source; all actual functions and existing fixtures read; SHA-256
  `0294ad8985b692be2f40fba2d36a8a37d28970fb538f654e56ba3b94dfa0353e`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/neighborhood/boundary.rs`: qualified
  complete-body carry from own generated-stencil-layout-fixed-code-review: exact
  source hash independently revalidated at current pin; SHA-256
  `b0909c76e5621b9fd5cc3d9d7a76f3a91b6d59d30e8b83b00432b081a93e6401`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/stencil/neighborhood/coordinates.rs`: qualified
  complete-body carry from own generated-stencil-layout-fixed-code-review: exact
  source hash independently revalidated at current pin; SHA-256
  `22738c1730f5893cd5f0e7aee4110b20366d320afdbad736dd5ec8dfb248f112`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/stencil/neighborhood/fetch.rs`: fresh complete
  current source; all actual functions and existing fixtures read; SHA-256
  `84b1d84fb4043e8dc16961272f54856e19e49edb85dd9a9ece666fe7cf02dac3`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/neighborhood/grid.rs`: fresh complete
  current source; all actual functions and existing fixtures read; SHA-256
  `2b4589d47565d626f74b5287fb8645ff636ea39129a28f952435f982007a4913`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/neighborhood/layout.rs`: qualified
  complete-body carry from own generated-stencil-layout-fixed-code-review: exact
  source hash independently revalidated at current pin; SHA-256
  `818c4f3b02ca461da41b87e7249fe143cefde84345e4540d2e9e1f68137c4a7d`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/stencil/neighborhood/layout/error.rs`:
  qualified complete-body carry from own
  generated-stencil-layout-fixed-code-review: exact source hash independently
  revalidated at current pin; SHA-256
  `b0a8b0dc7dc823fb4f24d86350eed7184344e713b65734d7424a3fd3c6f7465b`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/stencil/neighborhood/neighbors.rs`: qualified
  complete-body carry from own generated-stencil-layout-fixed-code-review: exact
  source hash independently revalidated at current pin; SHA-256
  `27389a70b30c76f8555661c3b26cf8cf950880a4507daea8d9774f218078ba82`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/stencil/program.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `3f96a9dcc7cf5fd005c41d4304f0da8494133c870c449395892e902dc4fbc674`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/recipe.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `a30f08a0e95afb9c0573bd1633f846099d18d5664fecd4107b98ef3927a1015a`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-compute/src/stencil/signature.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `04af568622f691ab9b1b50eea9d3fc38ca514e55fd2ad81ea50caaf23ee6586a`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stencil/tests.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `10cdf168bb0f690e5bf589b22daa446b60329c4bcf5b651aff1026f2dd8a552a`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stream/call.rs`: fresh complete current source;
  all actual functions and existing fixtures read; SHA-256
  `bdebde0792dd5c21e33d8acc172f6bc001f5470ed65beccfaab044a33f4999f8`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stream/mod.rs`: fresh complete current source;
  all actual functions and existing fixtures read; SHA-256
  `50a97cb78ec18e7d46a9e6bc6cc6c4825fe07a60113da7023fa1834fc6cc8a6c`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stream/program.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `345b4ec7ccbd94c59c299e41d3f2324bbc58605246250b5f57b11b747e5b529a`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stream/signature.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `cc7727ee7caed96373fcf49f63b22e670f24458cd447d715492fbe1b80805796`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-compute/src/stream/tests/mod.rs`: fresh complete current
  source; all actual functions and existing fixtures read; SHA-256
  `282793cfda3d949ab6e7c11539331d845f0911b5b89c63b34fc264f940819ce1`. selected
  lexical/source/role/native emission relationships; no executed numerical
  acceptance

- `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs`: qualified complete-body
  carry from own generated-stencil-layout-fixed-code-review: exact source hash
  independently revalidated at current pin; SHA-256
  `c3a5b335a373a7b4cf5fd605e527fe3efe8fa25bf0d1c7ef1d5ae840555f72f4`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs`: qualified
  complete-body carry from own generated-stencil-layout-fixed-code-review: exact
  source hash independently revalidated at current pin; SHA-256
  `59b58c24ca9681cbb2a48ec68854ff407ea61398fb50d85d83e1d58c34868330`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs`: qualified
  complete-body carry from own generated-stencil-layout-fixed-code-review: exact
  source hash independently revalidated at current pin; SHA-256
  `1c9dc99c0f186c95668f0b42fffcbc51b94bd822a907476d449f51e38570db01`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs`: fresh
  complete current source; all actual functions and existing fixtures read;
  SHA-256 `709f96bf90a39b588f07d8dcee684d59de96937a1499cbd2f06d2c235a323ea5`.
  selected lexical/source/role/native emission relationships; no executed
  numerical acceptance

-
  `crates/fabelgeist-gpu/src/data/gpu/compute/signature/native_operator_error.rs`:
  qualified complete-body carry from own
  generated-stencil-layout-fixed-code-review: exact source hash independently
  revalidated at current pin; SHA-256
  `c21f0a10ae14401b8f7b364c14f98de4e628de2dc5605d82e205fec09c9af99e`. current
  relying contracts revalidated against fresh signature/native assembly reads;
  no automatic prior pass or new full-body reread

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs`:
  qualified carry from own complete receiving-result-style post review; exact
  current SHA checked below; SHA-256
  `d840e177522b7d59f70ae3373c25707b3cc0d1ac06c93939da1ca6cb126b261f`. unchanged
  texture-value native type receiver, differing only previously reviewed
  explicit trait Result signature/formatter delta from older generated snapshot

