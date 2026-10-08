# Declared ZIP directory member count

The ZIP reader retains the selected directory cardinality as a private bespoke
`ArchiveMemberCount`. Native ordinary `u16` and ZIP64 `u64` fields admit every
value without a new resource limit. A private named `CentralDirectory` record
carries that count alongside the existing native byte offset; storage
addressing remains a separate responsibility.

The ordinary count's saturation value still requests ZIP64 substitution. An
offset saturation can also select ZIP64 fields, even when the ordinary count
is zero. Matching locator and end records replace both fields together; the
existing mismatch, bounds and fallback conditions remain unchanged. The reader
continues to use each format's total member count rather than its per-disk
count. These fields describe a declaration, not validated directory size.

## Native representation ports

The count owner projects only a capped count into `Vec`'s native allocation
capacity. The inherited `min(4096)` and cast are unchanged. This cap limits the
initial reservation; a directory with more actual members still grows the
vector and retains its members.

The iteration port returns the original native `Range<u64>` from zero to the
selected declared count. The range end is exclusive. No conversion to `usize`
narrows iteration on a smaller platform, and the loop's ignored native ordinal
has no semantic consumer. No unused ordinal owner or public count metadata is
introduced.

A selected zero count produces no members. Other selected counts iterate until
the declaration is exhausted or the existing central-header check stops them.
A count larger than the available directory still permits a partial result;
count/header discrepancies and duplicate first-match behavior are unchanged.
Stored members borrow and deflated members decode into owned bytes as before.

## Remaining format responsibilities

Record signatures, member names, compression, flags and checksums are separate
from cardinality. Native byte positions, widths, lengths, spans, casts and
whole-record framing also retain their existing contracts. A malformed central
name length can still panic when a nonzero count reaches that member. This
change does not add a framing validation or error policy.

Focused tests cover ordinary and ZIP64 count selection, zero and huge values,
per-disk disagreements, saturation and fallback, native full-width range and
allocation behavior, real members beyond the initial cap, duplicate selection,
owned/mapped ZIP and NumPy values, errors and the inherited framing panic.
