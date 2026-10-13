# NumPy decoding and file-read errors

NPY, NumPy's binary array format, stores a header dictionary followed by an
element payload. `NpyArray::from_bytes` admits those bytes with
`DecodeResult<NpyArray>`. `NpyArray::read` reads a standalone file and returns
`ReadResult<NpyArray>`.

## Owners and error paths

`NpyDecodeError` distinguishes the existing magic, header length, header
encoding, required-field, unsupported element type, Fortran order, shape-token
and checked-layout rejections. `NpyHeaderField` identifies a missing dtype or
shape field. `NpyDecodeError::Layout` retains the existing `NpyLayoutError`,
including its checked dimensions, element count, word width or payload lengths.
The shape and decoded-value owners remain authoritative; the error does not
admit a second metadata or value representation.

Match the `Layout` variant to obtain that concrete layout context. Tensor
conversion still returns `NpyLayoutError` directly. The transparent decoder
variant forwards the layout's existing presentation and underlying source; it
does not promise that the wrapper's source is the layout object itself.

`NpyReadError::Read` retains the failed operating-system path and `io::Error`.
`NpyReadError::Decode` retains that path and the concrete decoder error. Header
UTF-8 and shape-integer errors retain their original native causes. Rejected
descriptor and shape-token strings keep their serialized spelling only; they are
diagnostic provenance, not admitted array values. Paths retain the original
operating-system spelling rather than becoming array or archive names.

Ordinary display preserves the original context message. Alternate display adds
each native cause in the same order as the previous generic error chain. The
types derive their error traits and simple display messages; focused formatting
adapters retain alternate context/cause presentation. Structured Debug output
exposes the new error classification.

## Admission order and native boundaries

Magic and the original minimum buffer length are checked first. The original
version branch selects a two-byte header length for version one and a four-byte
length otherwise. The non-v1 branch now rejects ten- or eleven-byte buffers as
`TruncatedHeader` before reading those four bytes, instead of panicking. Earlier
magic rejection remains earlier; this guard introduces no new supported-version
or resource policy.

Header extent and UTF-8 admission precede dtype selection, Fortran rejection,
shape parsing, checked element and payload lengths, then payload slicing.
Optional header fields, descriptor spellings, whitespace trimming, scalar and
empty shapes, trailing-byte policy and native little-endian value decoding
remain unchanged. Numeric casts, not-a-number (NaN) bits, signed zero, element
order and native tensor rank conversion are owned by the existing metadata/value
APIs.

Native dictionary keys, byte lengths, offsets, rejected input spelling and
filesystem paths stay at their actual serialized or operating-system ports. The
decoder does not add a full Python dictionary parser, checksum validation, an
allocation policy or a new header-version owner.

## Composition and validation limits

`Npz::array` retains its existing general archive-error boundary and adds
array-name context to the concrete decoder cause. Model loading continues to
compose errors through `?`, without converting the error into a string. ZIP
errors and the wider model-loader error API are separate families.

The existing package suite and documentation examples remain the validation
suite. Three existing downcast expressions now project the typed decoder
variant; their test inputs, expected values and assertion strength remain
unchanged. No test, fixture, case or documentation example is added. The short
non-v1 header guard and new error/source variants have no newly authored direct
cases; source review and existing-suite success must not be described as
complete malformed-input coverage.

Native extent arithmetic on 32-bit hosts, arbitrary-input/resource safety, full
character-creator execution, all compute kernels and calibrated rendering are
not established by this error-family change.
