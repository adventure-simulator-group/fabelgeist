# FBX property-table keys

`FbxPropertyName` identifies a key in an FBX (Filmbox) `Properties70` block,
such as `RotationOrder` or `Lcl Translation`. Node lookup, animation defaults
and time spans, and the Momentum Human Rig (MHR) character importer use this
shared bespoke type.

Keys borrow exact bytes from string or raw-byte properties. Lookup performs no
UTF-8 decoding, normalization or case folding. Unknown names, embedded
separators and undecodable bytes remain distinct. Authored text enters through
an explicit conversion; known importer keys have shared constants.

Lookup selects the first `Properties70` block and its first child with a
matching string or raw-byte key. The child record name is not checked. Numeric
values start at property index 4. A missing or nonnumeric scalar uses the
caller's default; a missing, short or partly nonnumeric vector uses the entire
default vector. Selected nodes borrow the tree and can outlive temporary keys.

A property-table key does not classify an animation connection or establish the
type of its value. Record and subclass labels, connection IDs and labels,
numeric payloads, parser errors and rig membership retain their existing owners
and behavior.
