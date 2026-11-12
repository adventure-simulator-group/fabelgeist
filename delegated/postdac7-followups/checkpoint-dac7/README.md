# Checkpoint dac7 recovery evidence

This bundle retains the exact point-in-time staged capture, evidence-only replay,
bounded scan receipts, precommit comparison, later main observation and detached
recovery replay for checkpoint `dac7d0055716053f245139971eb67ddfd87fa462`.

The two evidence transcripts are byte-identical. Both checkouts and the
separately held encryption key are on the same machine. This is local recovery
evidence, not independent custody, complete privacy, nested-child assertion
proof, product validation, or recovery of the missing checkpoint 3821 records.
Raw precommit/capture command output containing local paths is preserved in the
encrypted checkpoint-control collection rather than published in plaintext.
Run `python3 -B VERIFY.py` to check the retained public evidence bytes.
