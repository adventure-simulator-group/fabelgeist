# Central evidence verifier versions

`VERIFY.py` is the preserved initial verifier. It failed because it required
one wording for a complete, untruncated receipt; the original receipt used an
equivalent punctuation variant. Its exact failure was recorded privately.
`VERIFY-v2.py` accepted both wordings and passed against the pre-review reports.
`VERIFY-v3.py` passed the first independently reviewed successor.
`VERIFY-v4.py` is the current successor; it additionally verifies the recorded
spot-check correction distinguishing two historical adoption scopes. Earlier
verifiers and drafts remain as historical evidence.

The verifier establishes byte identity and recorded relationships, not source
semantics, native behavior, independent custody or complete privacy.
