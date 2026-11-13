# Replaying the central application

`APPLICATION-EVIDENCE.tar.gz` preserves exact predecessor and successor images,
the reconstructed public proposals, the separately identified adopted
assessment, and the point-in-time main observation. The transform applies only
six partial queue assessments and appends three planning relationships. No queue
item or lens is semantically closed, and no unique-defect count is added.

From the successor publication checkout, verify the evidence and current root:

```sh
python3 -B delegated/post5f6a-followups/application/VERIFY.py --publication .
```

The old checkpoint verifiers assert the old root files and counts. To replay
them, provide a clean, separately recovered checkout at the exact predecessor
commit `5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0`:

```sh
python3 -B delegated/post5f6a-followups/application/VERIFY.py \
  --publication . --predecessor /path/to/clean-predecessor-checkout
```

The verifier binds the supplied checkout to the predecessor commit and tree
before invoking its evidence-only checks. Its Git object store, the separately
held same-machine key, earlier q2812 anchors and named nested keys remain
recovery dependencies. A same-machine replay does not establish independent
custody, missing checkpoint 3821 recovery, product behavior or audit closure.
