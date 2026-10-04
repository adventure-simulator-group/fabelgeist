# fabelgeist-storage

Checked byte positions, lengths, spans, and borrowed serialized views shared by
binary readers. This crate uses the standard library and has no dependencies.
FBX framing and NumPy/ZIP admission use this owner without coupling their format
policies or pulling a tensor runtime into an FBX asset pipeline.

`StorageByteOffset` is an address; `StorageByteLength` is a quantity. Advancing
a position or adding lengths rejects overflow. Distances reject backward order.
`StorageView::portion` and `tail_at` check the actual borrowed file extent
before indexing, including empty views exactly at the end. Bounds errors retain
the requested span and available length.

Little-endian field decoders are representation conversions. Their callers
immediately admit words into format-specific tags, counts, or quantities. Native
byte access belongs at direct standard-library or external codec calls. The
`From<StorageByteLength> for u64` conversion supplies native IO limits;
arithmetic and handwritten framing interfaces retain the nominal length.

These types admit every representable unsigned wire address or length. They do
not claim that an address lies within a particular file; checked view operations
establish that fact. Array shapes, archive identity, FBX node bounds, and format
versions belong to their respective format owners.
