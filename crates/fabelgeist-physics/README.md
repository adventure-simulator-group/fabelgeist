# Collider list counts and allocation

Analytic collider lists retain a full host count, distinct from their native
allocation target. Private bespoke types keep that difference through growth,
update admission, diagnostics, and shader parameter encoding. These values are
owned by the physics contact module; they do not expose new public query APIs.

Initial allocation holds sixteen collider records. Growth compares the existing
unsigned-word count with capacity, rounds the full host length to a power of
two, then narrows. Smaller and empty lists retain capacity. Empty replacements
leave uploaded words untouched, while ordinary recording reads the current list
count. Update admission compares full host lengths before uploading or changing
the held list. Mismatches retain their existing diagnostic.

Capacity assignment still precedes buffer allocation. A selected capacity is
therefore an allocation target, not proof of live buffer extent after a failed
allocation. Native narrowing and host overflow behavior are unchanged; these
types do not validate device limits. Generic construction, upload and recording
errors remain separate migration work.

Collider geometry, packing and shaders are unchanged. Tests cover native record
upload, retained tails, growth, empty replacement, and rejected-update
nonmutation. Frozen capacity cases retain host/native boundary behavior.
Floating-point collision output can vary between GPU providers, so one
provider's captured words are not a universal numerical contract.
