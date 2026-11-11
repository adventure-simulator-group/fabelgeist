#!/usr/bin/env python3
"""Verify viewer-read and native-source evidence, without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(k, b):
    return hashlib.sha1(k.encode() + b' ' + str(len(b)).encode() + b'\0' + b).hexdigest()
def unz(p):
    return gzip.decompress((root / p).read_bytes())

a = load('ASSESSMENT-VIEWER-METRIC-OWNERSHIP-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1699-COMPLETE.json')
p = load('ASSESSMENT-L01-HISTORICAL-1699-PARTIAL-001.json')
w = load('COVERAGE-L01-ROLES.json')['roles'][1699]
oldranges = p['new_personal_read_ranges'] + h['new_historical_personal_read_ranges']
current = h['current_comparison']['current_personal_read_ranges']
for ranges in (oldranges, current):
    assert ranges[0][0] == 1 and ranges[-1][-1] == 4477
    assert all(x[1] + 1 == y[0] for x, y in zip(ranges, ranges[1:]))
assert p['next_unread_line'] == 1601 and p['new_personal_read_ranges'][-1][1] == 1600
assert 'no additional current semantic' in p['current_comparison']['qualification']
assert h['earlier_partial'] == 'ASSESSMENT-L01-HISTORICAL-1699-PARTIAL-001.json'
assert h['historical_personal_read_complete'] and h['lines'] == 4477
assert w['status'] == 'full-historical-body-trace-open' and w['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1699-COMPLETE.json'
assert any(x['assessment'] == h['earlier_partial'] for x in w['historical_partial_observations'])
assert any(x['assessment'] == 'ASSESSMENT-L1-TREE-RESIDENCY.json' for x in w['current_observations'])
assert w['current_95e_full_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1699-COMPLETE.json'
assert h['current_sources'][0]['ranges'] == current and h['current_sources'][0]['revision'] == a['source_revision']
with tarfile.open(root / h['evidence_archive'], 'r:gz') as archive:
    historical = archive.extractfile(h['historical_sha256']).read()
body = unz(h['current_comparison']['preserved_bytes'])
assert historical == body and sha(body) == h['historical_sha256'] == w['historical_sha256']
assert oid('blob', body) == h['git_blob'] == h['current_comparison']['git_blob']
assert len(body.splitlines()) == 4477
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert tree[a['current_source']['path']] == oid('blob', body)
assert b'.filter(|value| value.is_finite())' in body
assert b'.entry(path.to_owned())' in body and b'.push(value);' in body
assert b'.filter(|(path, _)| path.ends_with(suffix))' in body
assert b'summed_render_metric(&render_diagnostics, "elapsed_gpu", |metric| metric.p95)' in body
assert b'host.architecture == "aarch64"' in body and b'chip.contains("Apple M5")' in body
assert b'gpu.contains("Apple M5")' in body and b'gib >= 16' in body

lock = unz('main-advance-95e/tree-stream-evidence/current-cargo-lock.source.gz')
assert oid('blob', lock) == tree['Cargo.lock']
assert b'name = "bevy_diagnostic"\nversion = "0.19.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "07a3a736f15486650c4c6866798a78d625c7063c991a3cc7b13f1577d89c82ae"' in lock
assert b'name = "bevy_render"\nversion = "0.19.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "fc86a32b2eef9859e1e7d323c6d1a9150fe77aa17c30fd05abebc12f645929dc"' in lock
members = {}
for package in ('bevy_diagnostic-0.19.1', 'bevy_render-0.19.1'):
    d = load('native-evidence/' + package + '/MANIFEST.json')
    raw = (root / 'native-evidence' / package / d['archive']).read_bytes()
    assert len(raw) == d['bytes'] and sha(raw) == d['sha256']
    with tarfile.open(root / 'native-evidence' / package / d['archive'], 'r:gz') as archive:
        for entry in d['inspected_members']:
            data = archive.extractfile(entry['path']).read()
            assert len(data) == entry['bytes'] and sha(data) == entry['sha256']
            members[entry['path']] = data
native = members['bevy_diagnostic-0.19.1/src/diagnostic.rs']
assert b'self.history.back()' in native and b'self.measurement().map(|measurement| measurement.value)' in native
render = members['bevy_render-0.19.1/src/diagnostic/internal.rs']
assert b'let parent = self.open_spans.iter().rfind(|v| v.thread_id == thread_id)' in render
assert b'(end - begin) * (timestamp_period_ns as f64) / 1e6' in render
assert b'path: self.diagnostic_path(&span.path_range, "elapsed_gpu")' in render
assert b'let Some(diagnostics) = mutex.0.lock().ok().and_then(|mut v| v.take())' in render
assert b'time,\n                value: diagnostic.value' in render
# Independent symbolic arithmetic, not a product test or benchmark.
index = (19 * 95 + 50) // 100
assert index == 18 and index < 19 and index >= 18

receipt = unz(a['earlier_receipt']['preserved_bytes'])
assert sha(receipt) == a['earlier_receipt']['sha256']
manifest = json.loads(unz('recovered/original-transport-manifest.json.gz'))
assert any(x['path'] == a['earlier_receipt']['original_path'] and x['sha256'] == sha(receipt) and x['git_blob'] == oid('blob', receipt) for x in manifest['entries'])
assert load('ASSESSMENT-TREE-STREAM-DIAGNOSTICS-95E.json')['earlier_historical_source_identities'][0]['new_personal_historical_full_read'] is False
assert len(load('ASSESSMENT-STRATEGIC-REPLACEMENT-ERROR-95E.json')['recovered_historical_records']) == 39
ids = [f['id'] for f in a['existing_finding_connections']]
assert ids == ['ADLER-L11-PERFORMANCE-DIAGNOSTIC-AGGREGATE-AUTHORITY', 'ADLER-L11-PERFORMANCE-TARGET-IDENTITY-ADMISSION']
queue = load('RECONCILIATION-059.json')['entries']
assert all(queue[i]['current_reconciliation'].startswith('partial ') and 'VIEWER-METRIC' in queue[i]['current_reconciliation'] for i in (93, 94))
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert all(sum(e['id'] == f and e['record'] == 'ASSESSMENT-VIEWER-METRIC-OWNERSHIP-95E.json' for e in ledger) == 1 for f in ids)
assert all(f['accepted_historical_unique_count_change'] == 0 and f['repair'] == 'not started' and f['validation'] == 'not performed' for f in a['existing_finding_connections'])
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == a['source_revision']
print('Verified complete viewer role, prior qualifications, native source and two partial reconciliations; no product execution.')
