# Replaying the central evidence application

`APPLICATION-EVIDENCE.tar.gz` contains exact predecessor and successor images,
the reviewed v5 draft, a separately identified adopted-report candidate and
the point-in-time main observation. The transform applies nine partial queue
assessments and appends nine planning relationships; it closes no item or lens
and adds no unique-defect count.

The restricted v4 preview retains a copy of the v3 bootstrap, which does not
generate v4's added navigation-review binding. The captured v4 transform and
its separately preserved correction identify the actual successor.

From the successor checkpoint checkout, verify the evidence and current root:

```sh
python3 -B delegated/postb111-followups/application/VERIFY.py --publication .
```

The old snapshot-specific verifiers assert b111's root files and counts. To
replay them, provide a clean separate checkout at the exact b111 predecessor:

```sh
python3 -B delegated/postb111-followups/application/VERIFY.py \
  --publication . --predecessor /path/to/clean-b111-checkout
```

The verifier checks that checkout's commit and tree before evidence-only
replay. A key is not needed for this guard-only replay; full authenticated
restoration needs the separately held same-machine key and documented earlier
anchors and Git objects. Neither check establishes independent custody, missing
checkpoint 3821 recovery, source semantics or product behavior.

The application verifier above checks the transform, candidate images and
review identities. To include the exact preceding remote-recovery receipt and
the other checkpoint guards, use the combined successor entry point:

```sh
python3 -B verify_postb111_checkpoint.py --predecessor /path/to/clean-b111-checkout
```

Omit `--predecessor` only when the narrower replay is intended. The receipt
is same-machine recovery evidence; publication and independent custody are
separate. See `CHECKPOINT-QUALIFICATIONS.md` for archive and status limits.
