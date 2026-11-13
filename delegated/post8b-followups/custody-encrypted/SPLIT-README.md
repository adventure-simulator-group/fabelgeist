# Split ciphertext transport, destination-safe successor

This transport preserves the original 46-object ciphertext packet while
keeping each piece at or below 64 MiB. Two large ciphertexts are split into
ordered pieces; 44 smaller objects are copied intact. The split manifest binds
the original manifest, object lengths and hashes, ordered piece hashes, and
the observed original local root and file modes.

Run `python3 -B VERIFY-SPLIT.py` for bounded-memory, no-key verification of
all 46 ciphertext streams across 48 pieces and within-group nonce uniqueness.
The default checks owner-executable state as recorded by Git, not full local
modes. Add `--strict-modes` to check the full recorded transport file modes.
Pin the externally recorded split-manifest and verifier SHA-256 identities.
Prior nonce inventory is not rechecked; no key or AES-GCM authentication is used.

Run `python3 -B RESTORE-SPLIT.py NEW_ABSENT_DIRECTORY` to verify and restore
the complete original ciphertext packet, applying the recorded original full
local root and file modes. Add `--strict-modes` to enforce strict transport
modes. The destination must be outside the transport: its normalized and
resolved path must be neither the transport nor beneath it, and its existing
ancestors must not identify the source root. Symlinked path components, extra
or missing members, changed bytes and existing destinations are rejected.

Restoration is sequential, not transactional; failure can leave a partial
directory that is not overwritten or automatically deleted. Path checks are
same-machine checks: unusual mount aliases, namespace changes or concurrent
adversarial path swaps are not universally excluded. Metadata beyond modes,
independent custody and plaintext recovery remain outside the claim.

The finite public metadata/control scan does not inspect hidden plaintext.
Ciphertext is scanned only for selected key forms within each original stream,
including across piece boundaries; no general ciphertext-pattern, cross-object
or cross-file privacy claim follows. Ciphertext metadata reveals lengths.
Git, original evidence, key and prior inventories remain separate dependencies.
No product code, build, test, repair, runtime probe or validation is performed.
