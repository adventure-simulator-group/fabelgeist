# Selected history and report bindings

`SOURCE-MAP.json` binds 17 public report members to their separate review
archives. Six older original transports remain in the parent checkpoint.
`HISTORY-SELECTION.json` records only 21 selected original/reconstructed 059
relationships and the later wrapper projections. An absent original `status`
member is distinct from a wrapper that supplies `null`; the complete original
and reconstructed queue streams differ.

From the evidence checkout, run:

```sh
python3 -B delegated/post76fe-followups/reconciliation/VERIFY.py \
  --publication .
```

The exact historical commit and path trees, plus pinned source objects needed
by earlier guards, may not be in a fresh clone. A separately preserved minimal
object subset and the authenticated original queue can supply a temporary Git
alternate; the separately held key and earlier ciphertext are dependencies.
This does not reconstruct complete Git history or checkpoint 3821, establish
independent custody or validate product behavior. Each reviewer and lens keeps
its original attribution and scope.
