# Native receiving signature: pre-repair Code discovery

Root `19a6b62687713b9ecfa7aa137003f11bd662baf9` (tree
`f8c67de65206ca90efe21e54c6c9e2e9bafa9cc3`), with exact peer caller context
`ead46619dec5ef45993f63202cb289bd3986b018`. No source edits or project execution
occurred.

Two source-backed lexical defects remain open: RECEIVE-G4’s admitted whitespace
is not renamed by Min/Max, and the same shared fixed-arity source parsers can
select commented declarations. The proposed ComparisonSignature owner is
appropriate once it owns actual source identity and its private byte span, with
the acceptance below. D2’s narrow Error-only derive is valid by pinned macro
source reasoning.

## Actionable findings

### RECEIVE-G4 (P2)

A valid declaration with whitespace between the function name and parenthesis is
admitted by the regex but not renamed by exact String::replace. The wrapper
calls min_cmp/max_cmp, which is absent at native compilation. Whitespace after
fn, tabs and line breaks have the same mismatch.

Source: crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs:213-227 at
19a6b62687713b9ecfa7aa137003f11bd662baf9;
crates/fabelgeist-compute/src/reduce/min.rs:15-28 at
ead46619dec5ef45993f63202cb289bd3986b018;
crates/fabelgeist-compute/src/reduce/max.rs:15-28 at
ead46619dec5ef45993f63202cb289bd3986b018.

An immutable ComparisonSignature source owner must retain the actual admitted
declaration-name span and ResourceBaseType; Min and Max consume its value_type
and renamed_source. Remove the old free comparison parser after both exact
callers migrate; no compatibility forwarder or second parser.

### G4-COMMENT-SELECTION (P2)

Both fixed-arity parsers run captures on original unfiltered source. A leading
line-comment declaration with f32 parameters before a real u32
comparison/scan/reduce declaration selects f32. Generated native array/wrapper
types then conflict with the actual real declaration. This is a representable
public-source API defect, not an observed current fixture failure.

Source: crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs:195-209,213-227
at 19a6b62687713b9ecfa7aa137003f11bd662baf9;
crates/fabelgeist-compute/src/scan.rs:76-90 at
ead46619dec5ef45993f63202cb289bd3986b018;
crates/fabelgeist-compute/src/reduce/mod.rs:51-68 at
ead46619dec5ef45993f63202cb289bd3986b018;
crates/fabelgeist-compute/src/reduce/execution.rs:101-114 at
ead46619dec5ef45993f63202cb289bd3986b018.

Use one canonical comment-aware declaration selection responsibility for both
binary and comparison forms. Ignore native line comments and nested block
comments without changing original offsets or text; preserve actual local/native
error boundaries. No independent general WGSL parser or earlier full native
compilation.

## Single-span admission edge

Two real fn min declarations remain duplicate min_cmp declarations under current
replacement, so native validation rejects them. Renaming only the first can
produce distinct fn min_cmp and fn min, silently admitting formerly duplicate
declarations.

Do not silently admit ambiguous real declaration sets. Record an explicit typed
admission disposition; keep one-span text preservation and avoid full standalone
native parse that could reject context-dependent authored functions.

Native Naga 29.0.4 consumes nested block comments and line comments. Its line
endings include U+000A through U+000D, U+0085, U+2028 and U+2029; blankspace
additionally includes U+200E/U+200F. Ignoring only LF-ended comments is
insufficient for the native lexical contract. A filtered selection view must
preserve original UTF-8 byte positions. The native lexer is private to Naga’s
WGSL frontend; no direct public lexer API is available.

## Error-only derive proof

GeneratedProgramError has no error/display/from/source/backtrace attributes, no
source-named field and no Backtrace field. In thiserror-impl 2.0.20, prop.rs
therefore makes has_display, has_source and has_backtrace false. valid.rs does
not require #[error] when has_display is false. expand.rs::impl_enum emits only
an empty Error implementation, preserving the exact existing manual Display,
default source(None) and auto traits. No From/provide or extra field Error
bounds are generated.

The owning Compute manifest already depends on thiserror. Derive only
thiserror::Error, remove the empty manual Error impl and its unused import, and
leave every Display byte/variant/payload unchanged. GPU
NativeOperatorSignatureError has no existing thiserror dependency and is not
part of that derive change.

## Acceptance and dependencies

- ComparisonSignature borrows the immutable original source; its
  declaration-name byte span and projected ResourceBaseType are private and
  admitted together. Native byte ranges use source offsets, not GPU
  BufferByteOffset/length owners.

- Renamed source changes exactly that admitted real declaration-name span. All
  other bytes, including comments, whitespace, struct/global identifiers, other
  declarations, function bodies and existing shader literals, remain exact. No
  global String::replace remains in either caller.

- Every previously accepted whitespace spelling remains coherent. Native comment
  trivia between syntax tokens, nested block comments and all native
  line-end/blankspace forms must be treated consistently where selection accepts
  them. Preserve UTF-8 boundaries and original byte offsets.

- The single-span design must not silently turn duplicate real target
  declarations into distinct valid functions. Record an explicit ambiguity
  admission disposition before repair; rejecting ambiguous selection with the
  local typed Comparison owner is a reasonable minimum. Full standalone parse of
  authored code is unsuitable when generated globals provide context.

- Keep first-type projection/Custom/native-validation responsibility as
  currently defined; do not add unrelated type equality, identifier, operator
  numeric, resource, workgroup or full WGSL validation policy merely for this
  rename.

- Preserve regex construction errors as actual errors when regex remains, and
  preserve NativeOperatorSignatureError comparison/binary typed roots and manual
  Display. Actual native shader/pipeline causes and success-only cache
  publication still belong to existing owners.

- Both Min/Max callers must migrate to the same final owner in the agreed
  connected family before source acceptance, and the obsolete free
  parse_compare_signature must be removed. GPU export/reexport paths must expose
  the canonical owner; no alias/shim/parallel parser.

- D2 may add only thiserror::Error to GeneratedProgramError derives, remove its
  empty manual Error impl and unused Error import, retain the exact manual
  Display and all variants/payloads. No #[error], #[from], #[source] or
  #[backtrace] attributes are needed or allowed as an incidental change.

- D2 must preserve source(None), no generated Display/From/provide
  implementation, existing auto traits and no new generic/field Error bounds.
  Current Compute dependency already owns thiserror; do not add a GPU dependency
  to derive NativeOperatorSignatureError.

- Existing fixture cases/helpers/operands/oracles and WGSL bodies stay
  unchanged; no tests/probes/assertions are added. Existing custom Item fixture
  uses exact fn min( and therefore does not exercise whitespace or
  comment-selection defects.

- No local project execution; final actual caller composition, independent
  reviews and required hosted gates still apply on final pushed SHA.

The actual existing custom Item fixture has the exact no-space fn min( spelling;
scalar Min/Max fixtures use no custom source. Existing Scan/Reduce literals are
also ordinary exact spellings. They do not establish the lexical
counterexamples; no additional fixture is authorized.

Whole-function/API agreement with the peer caller owners remains pending. Both
callers and removal of the old free comparison API must form one connected
migration. Broader native/generated/count/stride/Reduce composition and final
same-pushed-SHA reviews/checks remain required. This receipt grants no global,
compiled, numerical or hosted acceptance.

## Source accounts

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh complete source; SHA-256
  `1c9dc99c0f186c95668f0b42fffcbc51b94bd822a907476d449f51e38570db01`.

-
  `crates/fabelgeist-gpu/src/data/gpu/compute/signature/native_operator_error.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh complete source; SHA-256
  `c21f0a10ae14401b8f7b364c14f98de4e628de2dc5605d82e205fec09c9af99e`.

- `crates/fabelgeist-compute/src/source_error.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh complete source; SHA-256
  `5be0e86d52971e11461b1148c710cbc98a8a47628f798c51f68ed6136d17e832`.

- `crates/fabelgeist-compute/Cargo.toml`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh complete source; SHA-256
  `f7da2a134b0c98ad462cb8a5ac6a9e422ef5a49f9406993beb83a950f5991560`.

- `crates/fabelgeist-gpu/Cargo.toml` (19a6b62687713b9ecfa7aa137003f11bd662baf9):
  fresh complete source; SHA-256
  `707cc7a7a86fd855e779b3233b4dd97d3fd2be4496aa550a4388a1dec80d8a58`.

- `crates/fabelgeist-compute/src/reduce/min.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `7ea0c2ce5604bb08c70b246318a2e6f3a0b3dcc0197fbabcc29fa868d60b905f`.

- `crates/fabelgeist-compute/src/reduce/max.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `4e16a6ef11f3d5f562eeed220aa6bef85607d229b552901baa020ca24dc1c043`.

- `crates/fabelgeist-compute/src/reduce/mod.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `e97b56c90018e2382c2715260d0608d334eff6d141bc7dd8bb8fe6f708ed20ce`.

- `crates/fabelgeist-compute/src/reduce/execution.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `fdf743edb8382f0bae0750c6255d89a22855f9be09f50179981ffd8f26964435`.

- `crates/fabelgeist-compute/src/reduce/layout.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `19f555f68f33ff087db6fb06d6a5f37bb4ca784708e74739098c7c99b2c0de15`.

- `crates/fabelgeist-compute/src/reduce/tests/mod.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `d0d3dffc4a91c467d5fdab631a26961745163417f8b7818ae760a3702ea24d8f`.

- `crates/fabelgeist-compute/src/scan.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `f8ec75e11bdf6145f32018251f300ccedd7c73067bc549ebd35ace53e5745d79`.

- `crates/fabelgeist-compute/src/scan/wgsl.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh complete source; lexical,
  error, native assembly/cache and dispatch consequences assessed; not full
  numerical acceptance; SHA-256
  `803e777cf527e7c130b233a52edf330827c91ab7de8b2d9206bdd7d5bf56f9ec`.

- `crates/fabelgeist-compute/src/marching_cubes/mod.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh lines 38-85, complete
  selected Scan source operand and constructor context; SHA-256
  `20e8ff76b8ffe77cd50150082f3acf9cdfb56166be89945786e94c147151d31b`. Remaining
  extraction body not reviewed or cleared here.

- `crates/fabelgeist-compute/src/dual_contouring/mod.rs`
  (ead46619dec5ef45993f63202cb289bd3986b018): fresh lines 25-70, complete
  selected Scan source operand and constructor context; SHA-256
  `22625a943934af4f763dc3466fc3da94d759ce5e1e77637076a88633cb48238b`. Remaining
  extraction body not reviewed or cleared here.

- `crates/fabelgeist-gpu/src/data/gpu/mod.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh lines 1-70 (complete file);
  SHA-256 `ba5bd199bd66da1a3060421b34e228b33f7a8d86f5c4c8611c3a30cf6305c45d`.

- `crates/fabelgeist-gpu/src/prelude.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh lines 1-90 (complete file);
  SHA-256 `1d909746b1d9fbd8135c06cb27bf2193f6f9141c1b11c26a6d3d856e1dcf0666`.

- `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh lines 1-70, complete
  signature module/export declaration; remainder not cleared; SHA-256
  `c3a5b335a373a7b4cf5fd605e527fe3efe8fa25bf0d1c7ef1d5ae840555f72f4`.

- `crates/fabelgeist-compute/src/lib.rs`
  (19a6b62687713b9ecfa7aa137003f11bd662baf9): fresh lines 1-110, complete
  current module/export declarations; unreceived source_error composition
  remains known required; SHA-256
  `17d52c28a3489b266ec2a514084215b192e6bdea87d3326ab518ea5001f15771`.

-
  `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/expand.rs`
  (pinned dependency source): fresh complete source lines 1-650 (file complete);
  enum branch 222-506 governs D2; SHA-256
  `76285acc08ccb8a0a93a3cbd58c3e12cbcde61720ad676be181fff7b1f278df3`.

-
  `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/prop.rs`
  (pinned dependency source): fresh complete source;
  has_display/has_source/backtrace/from detection; SHA-256
  `9f350183bae7cb3b82e5cfe893212572c386fa52337cacf9976e0856312a468f`.

-
  `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/valid.rs`
  (pinned dependency source): fresh complete source; no display attribute
  requirement when has_display is false; SHA-256
  `700ac9fd6ecfbefe39b943ecd7c1f99dff8f1412b9aafef3b7c9feace4145c1b`.

-
  `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/ast.rs`
  (pinned dependency source): fresh lines 1-130; enum/variant attribute
  inheritance; remaining field conversion not independently needed; SHA-256
  `c717fffeb2923b2840d58fd02923a1926a48fa8336039f57a9e94f9275ae7ec6`.

-
  `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/naga-29.0.4/src/front/wgsl/parse/lexer.rs`
  (pinned dependency source): fresh lines 180-380,395-446, plus exact visibility
  references 486-504 from source search; complete native comment branch and
  line-end/blankspace definitions; remaining lexer not cleared; SHA-256
  `7208b9d96e850b285a45fb114b8a40eb74ce2cc143c528a1e967626d8395fcc0`.

