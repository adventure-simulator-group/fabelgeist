#!/usr/bin/env python3
"""Bind an unregistered navigation reconstruction and interrupted check."""
import hashlib, json
from pathlib import Path, PurePosixPath
root = Path(__file__).resolve().parent
packet = root / 'delegated/coordinator-navigation-qualification-13u'
h = lambda b: hashlib.sha256(b).hexdigest()
manifest = (packet / 'MANIFEST.json').read_bytes()
assert h(manifest) == '5793bcb1fd7846e96ab7d48f562bdd24448132279fcde67817f15135c05f7eba'
seen = set()
for row in json.loads(manifest)['files']:
    p = PurePosixPath(row['path'])
    assert not p.is_absolute() and '..' not in p.parts and p.as_posix() not in seen
    seen.add(p.as_posix())
    b = (packet / p).read_bytes()
    assert (len(b), h(b)) == (row['bytes'], row['sha256'])
assert {p.relative_to(packet).as_posix() for p in packet.rglob('*') if p.is_file()} == seen | {'MANIFEST.json'}
q = json.loads((packet / 'QUALIFICATION.json').read_bytes())
assert q['observer'] == '/root' and len(q['events']) == 3
b = (root / 'baseline-059/queue.json').read_bytes()
assert h(b) == q['events'][0]['source_sha256']
entries = json.loads(b)['entries']
reconstructed = ''.join(f'{i} {json.dumps(entries[i], ensure_ascii=True)[:1000]}\n' for i in range(2297, 2308)).encode()
assert reconstructed == (packet / 'QUEUE-PREFIX-RECONSTRUCTION.txt').read_bytes()
assert h(reconstructed) == q['events'][0]['reconstructed_display_sha256']
assert h((root / 'delegated/capture-terrain-q2301/REPORT.json').read_bytes()) == q['events'][1]['source_sha256']
r = json.loads((packet / 'ABORTED-RUN.json').read_bytes())
log = (packet / 'ABORTED-EVIDENCE.log').read_bytes()
assert r['exit_observed'] == 130 and (len(log), len(log.splitlines()), h(log)) == (r['transcript_bytes'], r['transcript_lines'], r['transcript_sha256'])
assert log.decode(errors='replace').splitlines()[-1] == r['last_output_line']
scan = (root / 'delegated/coordinator-navigation-qualification-13u-scan.json').read_bytes()
assert h(scan) == '7b2211b70fd37e191f58a83dade12bb1909d700d45f935c19b8ff882329f985c'
s = json.loads(scan)
assert s['forbidden_digest_total'] == 0 and not s['other_candidate_counts'] and not s['exceptions']
print('Verified unregistered navigation reconstruction and interrupted evidence transcript; neither supplies registered source credit or a completed first check.')
