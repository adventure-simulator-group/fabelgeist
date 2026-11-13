# Post-8ca source evidence custody

This group contains exactly four encrypted evidence roots: pending source
intake, q3100 author intake, q3100 peer intake, and q185 peer intake. Their
separate source readers, revision qualifications and correction records remain
inside those roots. No prior encrypted group or source checkout is included.

`VERIFY.py` checks ciphertext identities without a key. With the separately
held key and a new restore destination, it authenticates and restores the
archives; a separately held comparison verifies original bytes and modes.
Original complete queue streams and the key are separately held dependencies.
This is same-machine custody, not independent durability, full Git closure,
checkpoint 3821 recovery, product validation, repair or lens closure.
