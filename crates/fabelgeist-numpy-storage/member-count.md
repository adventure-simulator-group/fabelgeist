# Declared ZIP directory member count

`rawzip` selects ordinary or ZIP64 directory metadata and bounds each serialized
record. At this external decoding boundary, `ArchiveMemberCount` admits the
selected `u64` declaration without narrowing it. The adapter retains this
bespoke type as an application policy: reserve at most 4096 entry slots, then
request at most the declared number of records through rawzip's iterator.
The cap controls initial reservation, not the number of supported members.

A selected zero count requests no records. A large declaration stops when the
selected directory span is exhausted. It does not require that the declared
count equal the actual number of entries. Errors encountered while parsing
selected records reject the whole constructor; no partial directory is exposed.
Members beyond the declared upper bound are not selected. The underlying
rawzip iterator alone would ignore the declaration, so this bound belongs to
our adapter.

The end-record search window remains 66,000 bytes. Rawzip selects the last
signature in that window and validates its complete record and comment; it does
not automatically retry an earlier signature after a selected candidate fails.
It probes a preceding ZIP64 locator even without saturated ordinary fields.
ZIP64 selection requires consistent nonsaturated ordinary metadata and complete
record framing; an unsuitable candidate falls back to ordinary metadata.
Rawzip may correct classic offsets for an undeclared archive prelude.

These structural policies reject some formerly accepted malformed archives,
including inconsistent ZIP64 records and corrupted partial directories.
Malformed name frames return parser errors. Existing fixture inputs retain
observations of these outcomes; valid stored/Deflate and lookup observations
continue through ZIP, NPZ and the corrective importer.

Names retain directory order and duplicates. ZIP exact lookup and NPZ's
first matching full-name-or-one-suffix-stripped lookup remain application
contracts. Count declarations do not certify payload integrity or impose a
resource limit. See the [size contract](zip-extents.md) and
[error contract](zip-reader-errors.md).
