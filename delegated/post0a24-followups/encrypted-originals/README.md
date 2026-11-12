# Encrypted exact audit evidence

These authenticated ciphertexts preserve the private delegated packets,
selected replay inputs, independent reviews and control records listed in the
manifest. Plaintext can contain local paths and private Git metadata; public
scans inspect ciphertext bytes only. The separate 32-byte key is the same
private key used by the preceding checkpoint and remains on the same machine.
Preserve it separately from the branch: losing it prevents plaintext recovery.
This is not independent custody.

Verify ciphertext identities without the key:

```sh
python3 VERIFY.py
```

Authenticate the plaintext archives in memory with the separately held key:

```sh
python3 VERIFY.py --key /path/to/private-key
```

To restore exact file bytes and recorded modes, choose a new destination that
does not exist:

```sh
python3 VERIFY.py --key /path/to/private-key --restore-root /new/destination
```

Never restore over the protected extracted evidence. Ownership and timestamps
are normalized; packet-specific dependencies and omissions remain in the
replay-dependency record. These checks do not establish product behavior,
complete privacy, independent custody or recovery of checkpoint 3821.
