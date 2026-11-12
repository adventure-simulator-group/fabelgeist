# Encrypted exact evidence

These authenticated ciphertexts preserve the listed original private packets,
selected raw anchors and earlier recovery keys. Their contents can include local
paths or other private metadata. The public scans check ciphertext bytes, not
its opaque plaintext. The separate 32-byte key is held privately on the same
machine; this is not independent custody, and loss of that key prevents
recovery. Preserve the key separately from the branch.

From this directory, verify ciphertext identities without the key:

```sh
python3 VERIFY.py
```

With the separately held key, verify all plaintext archives in memory:

```sh
python3 VERIFY.py --key /path/to/private-key
```

To recover the exact file bytes and recorded modes, use a new destination that
must not already exist:

```sh
python3 VERIFY.py --key /path/to/private-key --restore-root /new/destination
```

The archive normalizes owner and timestamps; it does not preserve those
filesystem metadata. Do not restore over the existing extracted evidence. These
checks concern evidence custody, not product behavior or audit completion.
