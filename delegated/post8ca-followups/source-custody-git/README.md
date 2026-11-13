# Source custody identity in a Git checkout

The original `source-custody-receipt/` is preserved byte-for-byte. Its
original verifier compares full local file modes, which were `0664` at the
recorded cutoff. Git preserves executable bits but does not preserve those
other permission bits; a checkout using `0644` can therefore fail that
original mode comparison even when its bytes match. Reproduce the original
full-mode check only in a separately restored copy with its recorded modes.

From this directory, the additive checker verifies receipt and ciphertext
identities, recorded restore/scan results and Git-compatible executable bits:

```sh
python3 -B VERIFY.py --receipt ../source-custody-receipt \
  --ciphertexts ../source-encrypted --final-scan ../SOURCE-CUSTODY-FINAL-SCAN.json
```

An optional separately held prior inventory checks the four new nonce values
against the supplied 646 values; that comparison alone does not prove the
inventory's provenance. Authenticated restoration still requires the
separately held key and historical evidence. Same-machine custody is not an
independent backup, and neither checker establishes product behavior or audit
closure.
