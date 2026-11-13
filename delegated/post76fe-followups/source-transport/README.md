# After-b111 evidence transport proposal

This is an unpublished, evidence-only proposal. Its 21 deterministic archives
retain the source public packets and selected path-safe receipts byte-for-byte
with their member modes. Tar timestamps and ownership metadata are normalized.
`ARCHIVE-MAP.json` lists whole-file identities and every archive member.

Run `python3 -B VERIFY.py` to verify the root manifest, archive digests, tar
member bytes and modes, and each nested source manifest. The verifier does not
execute product code or inspect the separately retained restricted originals.

The index preserves reviewer, revision and historical status distinctions.
Private source evidence, complete original and reconstructed history streams,
raw scans and some replay dependencies require separate restricted custody.
Same-machine replay or a same-machine copy is not independent custody.
Encryption and nonce allocation are pending coordination. No queue status,
ledger, source code, repair, build, test or runtime behavior is changed.
