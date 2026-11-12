# Encrypted originals

These archives preserve selected exact restricted review, capture, replay and
Git-object evidence. They require the separately held same-machine primary
key. Delegate keys are wrapped inside the authenticated private collections.
Run `python3 -B VERIFY.py` for public identity checks, add `--key KEY_PATH`
for authentication, and optionally `--restore-root NEW_DIRECTORY` to restore
into a new, nonexistent directory. Never restore over existing evidence.

The selected Git store is incomplete, ciphertext is opaque to public scans,
and same-machine key custody is not independent recovery. No product behavior
or missing checkpoint 3821 records are established.
