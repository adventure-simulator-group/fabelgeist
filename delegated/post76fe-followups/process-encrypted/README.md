# Post-76fe process custody

This group contains encrypted process records and a custody-control collection.
The staged-capture copied tree is deliberately omitted: its 258 files matched
the existing 76fe Git commit at capture; exact local modes are recorded in the
encrypted control. The key, earlier ciphertexts, Git objects, history anchors
and nested keys are separately held. Original failure and later
alternate-dependent replay are distinct same-machine records.

`VERIFY.py` checks ciphertext identity without a key. Supply the separately
held key and a new restore destination for authenticated restoration. This
does not establish independent custody, historical completeness, missing
checkpoint 3821 recovery, semantic closure or product behavior.
