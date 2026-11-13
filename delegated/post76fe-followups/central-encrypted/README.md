# Post-76fe central review custody

This encrypted group retains the selected v4 draft and correction, separately
attributed v3/v4 and projection reviews, their predecessors and replays, and
the fixed source-group post-build control and scan chain. It also preserves
the earlier process-binding correction chain. The earlier source and process
ciphertext groups, original history Git objects, source transports and the
same-machine key remain separately held. The v3/v4 generic public replay outputs
are byte-identical and alone do not bind their respective supplied packages.

`VERIFY.py` checks ciphertext identities without a key; authenticated restore
requires the separately held key and a new destination. This is evidence
custody, not independent custody, checkpoint 3821 recovery, semantic closure
or product behavior validation.
