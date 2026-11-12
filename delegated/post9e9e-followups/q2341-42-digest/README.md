# Post-seal privacy and removed-path digest receipt

The keyed, evidence-only verifier authenticated the protected originals and
searched both the public source packet and the separately sealed scan receipt
for raw, lower-hex and upper-hex SHA-256, SHA-1 and MD5 of the exact removed
values and directory prefixes. Counts and qualification are in
`SEAL-DIGEST.stdout`; no individual removed value or digest is published.
The key is separately held on the same machine, a recovery dependency rather
than independent public recoverability. This receipt is itself separately
scanned; replay the packet verifier with both receipts as `--digest-scope` to
repeat the private-input check. Scope excludes unknown values and transforms.
