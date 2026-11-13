# Evidence-only application

`APPLICATION-EVIDENCE.tar.gz` retains the candidate's four predecessor images,
four prospective after-images, two new root records, the transform and its
candidate receipt. The archive normalizes tar metadata. Constructing it does
not install the candidate; the adopted wording in the prospective report takes
effect only after a later exact installation.

Before installation, an evidence checkout can replay against the separate
candidate directory:

```sh
python3 -B delegated/post76fe-followups/application/VERIFY.py \
  --publication . --candidate /path/to/candidate
```

After installation, omit `--candidate`. An optional `--predecessor` points to
an exact clean 76fe checkout. That predecessor's guards need the separately
held historical and pinned-source Git objects, supplied through a suitable
object store or a temporary alternate. The seven-object subset and original
queue have their own restoration and key dependencies. A successful replay
checks evidence identities and deterministic application, not source
semantics, complete history, checkpoint 3821, independent custody, or product
behavior.
