# ZIP record-signature interpretation

The ZIP reader classifies native little-endian signatures into a private
`ZipRecordKind` at its byte-decoding boundary. Five recognized roles select
end-directory, central-member, local-member, ZIP64 end and ZIP64 locator
records. An opaque `ZipSignature` retains every unknown code without rejecting
it. The public `ZipArchive` and `Npz` APIs expose member and array metadata,
so record kinds remain an internal format responsibility.

Each existing caller consumes the named kind. Unknown codes keep that caller's
mismatch behavior: end-directory scanning continues backward, a mismatched
ZIP64 locator or end retains the ordinary directory fields, a central-member
mismatch stops iteration, and a local-member mismatch reports a corrupt entry.
Recognized codes in the wrong position have the same mismatch behavior.

The first end-directory signature encountered while scanning backward still
selects the directory. Scan bounds, ZIP64 trigger conditions and fallback,
validation order, duplicate member selection, errors and decoding are unchanged.
Stored bytes still borrow from the mapped or owned archive buffer; deflated
members still decode into owned bytes. NumPy `.npz` arrays use this same public
archive reader.

## Remaining byte framing

Record classification decodes only the signature; it does not validate an
entire record. Existing scalar decoders and their byte offsets, widths, counts,
lengths, casts and slice bounds remain separate responsibilities. In particular,
a malformed central name length can still panic during slicing. Signature
interpretation does not add a new malformed-frame policy. Flags, checksums,
member names and compression have separate responsibilities as well.

Focused tests cover native classification, unknown codes, scan selection and
window bounds, ZIP64 fallbacks, local errors, owned/mapped member storage,
duplicate and disagreeing headers, NumPy values and the inherited frame panic.
