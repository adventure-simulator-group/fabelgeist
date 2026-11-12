# Encrypted exact evidence

These authenticated ciphertexts preserve the listed private delegated packets,
a ledger anchor and the minimal original inputs for the bounded 40-digest
scanner. Their contents can include local paths or other private metadata. The
public scans check ciphertext bytes, not their opaque plaintext. The separate
32-byte key is the same private key used by the preceding checkpoint and is held
on the same machine. This is not independent custody, and loss of that key
prevents recovery. Preserve it separately from the branch.

Verify ciphertext identities without the key:

```sh
python3 VERIFY.py
```

With the separately held key, authenticate the plaintext archives in memory:

```sh
python3 VERIFY.py --key /path/to/private-key
```

To recover exact file bytes and recorded modes, choose a new destination that
must not already exist:

```sh
python3 VERIFY.py --key /path/to/private-key --restore-root /new/destination
```

Owner and timestamp metadata are normalized. Never restore over the existing
extracted evidence. Some packet replays also need earlier authenticated
anchors and their separately recorded dependencies. The digest scanner uses
local account context, so its result is not an independent custody proof.
These checks concern evidence, not product behavior or audit completion.
