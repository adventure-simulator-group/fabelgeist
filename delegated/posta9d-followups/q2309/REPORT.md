# Independent review of historical q2309

Reader: `/root/verifier_scope`. This is a bounded source and evidence-identity
review of recovered 059 q2309 against pinned current source
`95e63628f9d70b8dd6e6fede64334696c6995009`. It does not decide a new queue
closure, create a defect count, edit the ledger, or validate the product.

## Current source and recorded dependency identity

- `crates/fabelgeist-numpy-storage/Cargo.toml:8-20` pins `rawzip = "=0.5.2"`
  and declares its own workspace. The root manifest explicitly excludes that
  crate (`Cargo.toml:51-60`). In the 95e child lockfile the selected
  `fabelgeist-numpy-storage` package lists `rawzip` as a dependency and the
  sole selected rawzip package is registry version 0.5.2 with checksum
  `ac9999039f74cd73c03158cddfe4362991703c74efe5ce5c8073026f80a5e48c`
  (`Cargo.lock:4304-4308` and the registered structured lock selection).
  Registered structured searches found no rawzip package in either the e275
  or 95e root lock. This does not make the root lock authoritative for the
  excluded child workspace.
- The child's entire e275 and 95e lockfile bytes match: SHA-256
  `33fe471032dbeeb221003b375f903a389269e9e7cfcf37d1118e2035870e3d89`.
  Their manifests differ: the 95e version adds a development-profile block,
  while the displayed direct dependency declarations match. Matching the
  child lock is a recorded package identity, not a performed Cargo resolution,
  feature-unification, build, signature or native-runtime check.
- The current owned ZIP adapter imports rawzip and uses its locator and
  directory/member APIs; it uses flate2 for deflate
  (`crates/fabelgeist-numpy-storage/src/zip.rs:1-12,86-109,160-181`). That
  source trace does not establish supported archive inputs, parser outcomes,
  native correctness, allocation limits or a complete NPZ audit.
- The independently recovered 260,248-byte rawzip package hashes to the
  checksum in the pinned child lock. Its exact `rawzip-0.5.2/src/locator.rs`
  member matches the separately recovered 17,883-byte historical member,
  SHA-256 `334cc9c41217a207e621e6b0177bed39871365cc97ba178fee3be804293604ed`.
  The same package and member independently match members of the preexisting
  published `delegated/crate-metadata-evidence.tar.gz`. This proves identity
  relative to the recorded lock and archived bytes, not a registry signature
  or independent remote provenance. No native source line was personally
  displayed or executed by this reviewer.

## Historical and current relationships

Original 059 q2309 records
`REVIEW-L11-NPZ-OWN-CRATE-NATIVE-LOCK-AUTHORITY` as
`rejected-suspicion-source-qualified`, with the named historical scope marked
closed, no all-lens closure, no population credit and no runtime pass. Its
original queue entry byte slice is equal to the separately path-redacted public
reconstruction slice, but the two full queue streams have different identities;
those slices are not original standalone queues.

All five named historical PLAN-192, READ-172, root READ-L11-708,
VERIFY-L11-708 and proposal records were copied without modification from the
read-only user extraction and matched their q2309 hashes. Their hashes, sizes
and relative identities also match the published original-transport inventory;
that is not independent off-machine custody. READ-172 attributes the e275
manifest, child lock and native locator reads to the original Lens 11 agent
and says root adoption pending. The separately dated root mapping adopts
PL11-1109, PL11-1110 and PL11-1111 without a root reread. Their chronology and
personal read attribution are preserved.

The separately attributed committed checkpoint-5 assessment by
`/root/rig_identity` already connected this same historical scoped nonfinding
to 95e and independently matched a package and five archive members. This
review independently matched one member; it does not claim that delegate's
other native reads or transfer their attribution. Original 059 q2306 concerns
the related NPY framing/owned-array admission boundary and explicitly leaves
its own exact-scope discovery incomplete. It is not silently merged with
q2309. Missing checkpoint 3821 queue and ledger are not reconstructed or
inferred.

## Remaining questions and custody

Other native members, NPZ/ZIP consumers, zip64 and deflate behavior, package
feature resolution, resource limits, provenance and any actual product/native
outcome remain separate open scopes. Their status is not settled by the
historical lock identity or this bounded review.

A preliminary scan of the exact upstream package found three email candidates
and could not inspect 30 nested ZIP assets; its first printed summary was
truncated, and complete machine-readable results were retained separately.
The exact unchanged package remains under restricted custody and already
exists in the identified prior publication archive. This successor excludes
the package itself, retains its separately verified locator member, and binds
both recovery routes and their checksums in `PRIVACY-BOUNDARY.json`. The prior
published archive and upstream package are not cleared by scanning this
successor. One earlier source-range preparation failed on an over-EOF request
before any body was displayed; the failure and corrected registration are
preserved in `LOCATOR-QUALIFICATION.json`.

All verification here was evidence-only. No Cargo resolution, product code,
parser, native package, fixture, build, test, macro or runtime probe was run;
no protected extraction, publication or ledger was modified.
