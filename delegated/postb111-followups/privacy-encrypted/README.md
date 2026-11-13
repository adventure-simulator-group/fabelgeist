# Encrypted privacy-correction custody evidence

Run `python3 -B VERIFY.py` to check encrypted payload identities. A separately
held key authenticates plaintext; supply a new `--restore-root` to restore.
The encrypted custody-control object contains the private source inventory,
prior nonce audit and scripts. Only selected final-control members are captured.

The earlier ciphertext collections, Git objects and key remain separate
same-machine dependencies. Source bytes and root, directory and file modes are
preserved; timestamps, ownership and ACLs are not. Ciphertext is opaque to
public scans. No independent custody, product validation, historical checkpoint
3821 recovery or audit closure is asserted.
