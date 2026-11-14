# Party enrollment validation update

This additive update follows the [first slice checkpoint](INDEX.md) at
`f148b3e8982209f43ef4a1553787d4f80d64e589`. The earlier checkpoint correctly
records the lint retry as still running at its cutoff. The network-enabled
`just lint` retry subsequently passed with exit 0. The earlier offline failure
and its cause remain preserved in the preceding checkpoint.

The new [ledger successor](LEDGER-v4.json) binds the final lint transcript to
repair commit `d276139b62e62bcbd1ed766433146c686765aab7`. The repair is in
[PR #1005](https://github.com/adventure-simulator-group/fabelgeist/pull/1005),
which was draft and unmerged at this cutoff. The local regression, mission
suite and formatting results are unchanged. A GitHub Rust quality workflow was
in progress; this update does not claim its result.

The prior checkpoint's branch head and both encrypted blob contents were
independently read from GitHub and matched their staged identities. This
validation update preserves the complete final lint log, earlier checkpoint
verification and later process controls in `validation-custody/`. Its key is
separately held on the same machine. Authenticated restore and independent
source-byte, directory and mode comparison succeeded. The ciphertext manifest
and verifier are bound by the following SHA-256 values:

- Manifest: `87ec2fe33eabd83b3a3964962a03be5c5adc9b6dbd1a949944cbfc8cb93f0a8f`
- Verifier: `fb38cbf18961daed2de7bc6d434753f22b632f5d89720fc04391c66a44c61e0c`

The recovery limits, missing checkpoint 3821 ledger and queue, original reader
attributions, open relationships, and lack of independent backup described in
the first checkpoint remain in force. New WeatherSnapshot source reviews and
later publication and PR events are outside this cutoff. See the original
checkpoint for restore conditions and the distinction between no-key identity
checks and authenticated restoration.
