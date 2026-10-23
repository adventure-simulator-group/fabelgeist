# Retained NumPy format reader

This directory contains the published `npyz` 0.9.1 source, with local corrections
to header arithmetic. Fabelgeist uses its NPY header parser and primitive scalar
readers. Its optional NPZ adapter is disabled; ZIP ownership stays in
`fabelgeist-numpy-storage`.

- Upstream: <https://github.com/ExpHP/npyz>.
- Release revision: `822a3a34564ba5b9c00fcfb81ca081246f4fe3ef`.
- Published package SHA-256:
  `0a01a88a1d2f89d362475ad16f34de3994f7c86b12259236f5d33537b040bcd4`.
- License: MIT. The upstream README retains Michael Lamparski's and Pavel
  Potoček's notices. The published package has no separate license file;
  `LICENSE-MIT` supplies the MIT text with those notices.
- The normalized manifest, original manifest, lockfile, source, examples,
  benchmarks, existing tests and fixture data are retained. Registry completion
  metadata and upstream automation are omitted. The root workspace excludes
  this package; NumPy storage selects it through a path dependency with default
  features disabled.

The local `rustfmt.toml` preserves published formatting inside this directory.
`cargo fmt --all` also traverses local path dependencies, so workspace exclusion
alone does not prevent rewriting this third-party source. First-party format
checks remain unchanged. The pinned nightly formatter supports this local
ignore configuration.

## Header arithmetic

The original header constructor multiplied dimensions and strides without
checking overflow. A malformed or enormous shape could panic in a checked build
or wrap in an unchecked build before the caller could validate it.

`src/read.rs` now checks element-count, payload-byte and stride products and
returns `InvalidData` on overflow. A scalar still contains one element. A zero
axis gives zero elements, but every stored stride must remain representable;
an enormous empty array with an unrepresentable stride is rejected. Both C and
Fortran ordering use the same checked prefix-product calculation.

`src/header.rs` checks the conversion of the versioned 32-bit header length to
`usize`. This does not impose an allocation budget or certify arbitrary-input
resource safety. Random-access seek arithmetic and unrelated upstream APIs are
outside this patch.

The existing four stride observations only unwrap the newly fallible helper;
their inputs and expected results are unchanged. No test, case, helper or
assertion was added. Fabelgeist's existing metadata tests also exercise header
admission, including enormous empty shapes and overflow. The application adopts
upstream header validation before its supported-type and ordering policy.

Keep future updates tied to a published source revision and package hash.
Reconcile this arithmetic patch against upstream changes before replacing the
vendor tree. Do not preserve a second handwritten application header parser or
fallback path.
