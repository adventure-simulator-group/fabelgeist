# Encrypted privacy-scan originals

This authenticated archive preserves the omitted scan-registration files,
scanner sources and exact first scan outputs. They can contain local temporary
paths or their digests. The same separately held 32-byte private key used by the
preceding checkpoint is required for plaintext recovery; key custody remains on
the same machine and ciphertext is opaque to public scans.

Run `python3 VERIFY.py` here to check ciphertext identity. Add
`--key /path/to/private-key` to authenticate in memory. Add
`--restore-root /new/destination` only for a new, nonexistent destination. The
archive preserves file bytes and modes, but normalizes owner and timestamps.
Never restore over the existing extracted evidence. These are evidence checks,
not product validation or audit closure.
