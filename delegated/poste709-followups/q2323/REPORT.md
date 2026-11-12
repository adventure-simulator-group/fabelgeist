# Bounded review of historical 059 q2323

## Historical record and evidence

Original 059 queue `/entries/2323` is
`REVIEW-L11-CREATOR-PRIVATE-BODY-FIT-PAIRING`. It records a
`rejected-suspicion-source-qualified` disposition and discovery closed only
within the exact named private-creator scope. It does not close all lenses or
its coarse parent, complete the global alias inventory, or pass repair or
runtime validation. The original queue revision
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e` and public path-redacted
reconstruction `26e360920b7b2ec32d5c76238623eaf6d6617d7e` have different
full-stream hashes, but their exact q2323 byte slices match. This historical
state is not recovered checkpoint 3821.

All 18 proposal, plan, delegated-read, root-adoption and verification files
cited by q2323 were recovered by their recorded hashes from the protected
extraction. A published transport inventory has a matching digest, byte count
and relative suffix for each; this does not prove independent off-machine
custody. The four separate root adoption records preserve the original Lens 11
reviewer's personal read attribution, explicitly do not create a root reread
or other-lens credit, and record no runtime pass. The original reader's
then-pending root-adoption wording and the later adoption are separate events.

The 11 q2323 source files at historical
`e2755a732c84e67296b5e3441a4259a84f461b2f` match their recorded hashes and
are byte-identical to their counterparts at pinned
`95e63628f9d70b8dd6e6fede64334696c6995009`. Six additional selected
support files, including `studio_cache.rs`, `device_body.rs` and MHR model
construction, also match between those revisions. This compares only those
files, not whole revisions, dependency versions, model assets or behavior.
Historical personal full-body reads remain the original reviewer's work. This
reviewer's own prospective source displays and unread complements are in
`SOURCE-READ-RECEIPT.json`; file hashing and capture are not personal reading.

## Pinned source assessment

1. **The ordinary single-regeneration data flow is bounded.**
   `studio_generation.rs:21-67,86-91` obtains one `GeneratedCharacter` from
   `StudioCache::body` and passes that value alongside the current `BodyModel`
   to clothing, armor, drape and preview work. `device_equipment.rs:43-75,
   78-111` derives faces, UV projection, skin rows and joint names from that
   model and positions, normals, joints and device cache from the supplied
   generated realization. `underlayer_equipment.rs:58-91` and
   `fitted_existing.rs:2-55` show the analogous explicit split. This supports
   the historical same-call-site observation when the cached body corresponds
   to the model used; it is not a universal model-instance proof. Armor fit
   cache hits may reuse an earlier fit instead of executing a new fit.

2. **A supported same-LOD reload qualifies the association claim.**
   `main.rs:408-440` reloads and replaces `BodyModel` when the corrective flag
   changes, even at the same LOD, and marks the studio dirty. The body key in
   `studio_cache.rs:30-55` includes LOD and recipe proportions, identity and
   expression, but not the model instance or corrective flag. If those key
   values remain equal, the next `regenerate_mesh` can receive the previous
   generated Arc while also passing the reloaded model. The model loader reads
   the same LOD-named FBX and conditionally loads corrective data
   (`fabelgeist-mhr/src/model.rs:158-189`); the asset bytes and network values
   were not inspected, and no asset replacement or changed output is asserted.

   This is already the historical q2327
   `REVIEW-L11-CREATOR-CORRECTIVE-MODEL-CACHE-DEPENDENCY-POLICY`, which
   records a conditional policy disposition with discovery closed only in
   its named scope. The separate current
   `REVIEW-CREATOR-CORRECTIVE-CACHE-POLICY-001` assessment preserves that
   history and leaves output impact, model identity policy and aliases open.
   The present review connects that existing question to q2323; it does not
   overwrite the old scoped rejection, claim a new unique defect or show a
   mismatched supported rig or visible fit.

3. **Counts establish bounds, not a shared rig instance.**
   `fabelgeist-rig/src/table.rs:8-10` and `skin.rs:6-16,112-124` explicitly
   distinguish rig bounds from rig-instance identity; `MeshVertexTable` makes
   the corresponding vertex distinction (`character/vertices.rs:6-19,49-60`).
   The FBX skin path constructs per-vertex index and weight rows together
   (`character/skin.rs:58-65,97-150`). The public creator crate exposes
   `device_fit` (`lib.rs:61`), whose `FitBody` and `Realization` accept separate
   inputs (`device_fit.rs:27-46`). `Realization::upload` checks the skin/name
   joint count and constructs `BodySurface`; a fresh `GpuBody::new` checks
   nonempty positions, matching lengths, a matching joint bound and face
   indices (`device_fit.rs:49-69`; `fabelgeist-armor/src/gpu/body.rs:13-77`).
   These checks do not certify common source-instance or joint order.

   A previously populated `DeviceBody` returns its cached upload before the
   upload closure runs (`device_body.rs:8-25`), so the fresh-upload checks
   should not be described as rechecking every call. The separately
   attributed `REVIEW-RUNTIME-BODY-DEVICE-CACHE-IDENTITY-001` concerns public
   mutable runtime body/cache reuse, with an inspected tactical reset
   safeguard and no executed mismatch. The current rig delegate's
   `DELEGATE-RIG-EQUAL-BOUND-INSTANCE-QUESTION` addresses separately supplied
   equal-bound public rig data. Both are related policy questions, not proof
   of a private creator failure or automatically the same first contract.

4. **Checked UV indexing has a cardinality boundary.**
   `Character::from_fbx_bytes` triangulates faces, parses UVs, and obtains UV
   triangles independently (`character.rs:118-145`). Missing UV data returns
   empty vectors and mismatched UV-index and polygon counts produce an empty
   UV-face vector (`character.rs:207-257`). The selected model constructor
   (`model.rs:158-189,222-243`; `model/upload.rs:33-123`) does not show an
   additional UV-face cardinality check. `vertex_texcoords` starts a
   vertex-sized zero table and zips faces with UV faces; its checked vertex
   and coordinate lookups run for visited pairs, so an empty UV-face list
   visits none and returns the zero-filled table (`device_equipment.rs:13-40`).
   Its length can satisfy the fresh `GpuBody::new` length check when the other
   stated preconditions also hold. This is a source-derived conditional
   pathway, not an actual accepted MHR asset, completed model, device result,
   rendered garment or demonstrated user-visible error. Same-vertex UV seams
   and any intended missing-UV policy also remain unassessed.

   The original historical source receipt and later topology records already
   identified silently empty UV triangles and left producer policy open; the
   e275 Lens 11 fitting-consumer disposition specifically qualified the zip
   cardinality and possible last-face-corner UV choice. The separately
   attributed `REVIEW-SURFACE-CUT-UV-FACE-ADMISSION-001` concerns a distinct
   underlayer cut consumer and a symbolic array-bound case; it is related but
   is not automatically this projection's first contract. Do not add an
   unverified unique historical finding or conflate the two consumers.

## Additional historical recovery and limits

The original 059 ledger's topology record and three related dispositions are
stored as **JSON reconstructions of selected semantic objects**, not exact
original ledger byte slices. The packet binds their selectors to the original
ledger stream hash; the optional Git check reparses and compares them. They
retain their original frozen, `7a9d92ad7357779e9280146c521bd8588b86fcdb`,
and e275 source qualifications, respectively. The earlier 7a9 root adoption
and verification match their cited hashes and remain separately attributed.

The ledger also cites `adler/READ-L11-371.json` without an expected hash in
that citation. Two located older local copies have identical bytes and match
published transport digest, size and path-suffix metadata; the candidate is
retained with that qualification, not presented as independently proven
custody or a checkpoint 3821 record. Its source role attributes the older
full-file read to the original reviewer and says mismatched UV counts silently
empty the UV faces; it explicitly reports no validation or current
reconciliation.

For the companion q2324 review, the selected ordinary generation path calls
`Mhr::forward` before later fitting (`generation.rs:21-32`); the forward pass
constructs normals using `vertex_normals`, whose tensor select and
select-assign consume the retained face corners (`model.rs:357-369,396-442`).
The earlier upload checked conversion to i32 width, not membership in the
vertex table (`model/upload.rs:75-82`). This identifies an earlier native
index operation without asserting how its backend handles an invalid index,
what valid assets contain, or whether a later probe would execute.

Outstanding questions include actual supported public cross-model callers and
ordering policy, producer/model identity during reload, real asset UV and
corrective invariants, all downstream consumers and global first-operation
aliases. Registered locators are capped; the broad rig-joint search retained
100 of 240 matches. One historical pointer search initially examined only
dictionary scalar strings and missed list strings; a separately registered
corrected search is retained. A display request extending past DeviceBody EOF
was clamped and recorded. Bounded scans do not clear hidden originals,
unexamined Git history, unknown secrets or unsupported formats. Hashes and
recorded chronology do not prove comprehension, semantic truth, runtime or
independent custody. No product code, assets, fixtures, graphics, builds,
tests, probes, repairs, ledger or publication changes were executed.
