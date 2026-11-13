# Fixed post-8b evidence receipt

This receipt records a same-machine authenticated restore and comparison of
44 selected roots, one stage-builder snapshot, and one pre-capture control.
Run `python3 -B VERIFY-RECEIPT.py` for receipt identities. Add `--ciphertexts
DIRECTORY` to check bound ciphertexts without a key; `--prior-inventory
INVENTORY` additionally checks 772 supplied prior nonce values. These checks
do not repeat authentication or prove completeness of the prior inventory.

The key, prior ciphertexts, originals and Git objects remain separate on the
same machine. The 8b auto-packing notice and unknown effect on unreachable
objects remain qualified. No independent custody, full Git closure, missing
checkpoint 3821, product validation, repair or lens closure is established.
