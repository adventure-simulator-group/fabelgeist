# Closing evidence snapshot

`CAPTURE.json` binds the staged evidence files before this closing directory
was added. `SCAN.json` records their separate finite scan. From the checkout
root:

```sh
python3 -B delegated/post76fe-followups/closing/VERIFY.py
```

The four closing files were created after that capture. Their separate bounded
scan, final staged capture and later publication and recovery process records
are retained locally for the next custody cutoff. Ciphertext contents, unknown
private values and encodings, and Git history are outside the finite scans.
These records do not establish independent custody, semantic closure or
product behavior.
