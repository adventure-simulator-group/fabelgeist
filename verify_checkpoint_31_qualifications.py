#!/usr/bin/env python3
"""Verify a qualified historical pin mismatch and unregistered view."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
R = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
cp = json.loads((R / 'DELEGATED-EVIDENCE-CHECKPOINT-31.json').read_text())
b = next(x for x in cp['bundles'] if x['bundle_id'] == 'saphyr-2573-2576-reconcile')
with tarfile.open(R / b['archive']) as t:
    q = json.load(t.extractfile('history-pin-qualification.json'))
    v = json.load(t.extractfile('unregistered-view-qualification.json'))
    r = json.loads(gzip.decompress(t.extractfile('history-774/admission03-scope-reconciliation-046.json.gz').read()))
    registration = gzip.decompress(t.extractfile('history-774/admission03/source-range-registration.jsonl.gz').read())
    qualification = gzip.decompress(t.extractfile('history-774/admission03/source-qualifications.jsonl.gz').read())
    reg_prefix = t.extractfile('history-774/reconstructed-first-18-registration.jsonl').read()
    qual_prefix = t.extractfile('history-774/reconstructed-first-10-qualification.jsonl').read()
    assert reg_prefix == b''.join(registration.splitlines(keepends=True)[:18])
    assert qual_prefix == b''.join(qualification.splitlines(keepends=True)[:10])
    assert sha(registration) == q['later_f82_registration_whole_sha256'] and sha(qualification) == q['later_f82_qualification_whole_sha256']
    assert sha(reg_prefix) == q['later_registration_first_18_sha256'] != q['earlier_046_registration_pin']
    assert sha(qual_prefix) == q['later_qualification_first_10_sha256'] == q['earlier_046_qualification_pin']
    assert [x['sha256'] for x in r['input_pins'][:2]] == [q['earlier_046_registration_pin'], q['earlier_046_qualification_pin']]
    assert r['role_qualifications'] and 'not a global absence claim' in q['qualification']
    assert v['displayed_ranges'] == [[1, 5]] and 'no assigned, support or lens credit' in v['credit']
    assert 'not personally displayed under the prospective registration' in v['credit']
print('Verified distinct historical registration pins and bounded search qualification, plus unregistered incidental viewing with no credit.')
