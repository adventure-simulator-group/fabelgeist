#!/usr/bin/env python3
"""Verify the qualified same-machine recovery record and exact transcript."""
import argparse, hashlib, json, subprocess
from pathlib import Path, PurePosixPath
root = Path(__file__).resolve().parent
packet = root / 'delegated/checkpoint26e-verification'
h = lambda b: hashlib.sha256(b).hexdigest()
manifest = (packet/'MANIFEST.json').read_bytes()
assert h(manifest) == '17f0866396f0259aea7e14981785662d7c411f58780de82709c9b387e6a0d3c1'
seen = set()
for row in json.loads(manifest)['files']:
    p = PurePosixPath(row['path'])
    assert not p.is_absolute() and '..' not in p.parts and p.as_posix() not in seen
    seen.add(p.as_posix())
    b = (packet/p).read_bytes()
    assert (len(b),h(b)) == (row['bytes'],row['sha256'])
assert {p.relative_to(packet).as_posix() for p in packet.rglob('*') if p.is_file()} == seen | {'MANIFEST.json'}
r = json.loads((packet/'RECOVERY-RECORD.json').read_bytes())
a = (packet/'PREPUSH-EVIDENCE.log').read_bytes()
b = (packet/'RECOVERY-EVIDENCE.log').read_bytes()
assert a == b and (len(a),len(a.splitlines()),h(a)) == (r['transcript_bytes'],r['transcript_lines'],r['prepush_and_recovery_transcript_sha256'])
before = json.loads((packet/'BEFORE.json').read_bytes())
after = json.loads((packet/'AFTER.json').read_bytes())
assert before['expected_fetch_head'] == before['observed_checkout_head'] == r['published_commit'] and before['status_porcelain_empty']
assert after['head'] == r['published_commit'] and after['status_porcelain_empty'] and after['exit'] == 0
assert (after['transcript_bytes'],after['transcript_lines'],after['transcript_sha256']) == (len(b),len(b.splitlines()),h(b))
s = json.loads((packet/'STAGED-DIGEST-SCAN.json').read_bytes())
assert s['stats']['top_files'] == 353 and s['forbidden_digest_total'] == 0 and not s['other_candidate_counts'] and not s['exceptions']
assert b'issues 0' in (packet/'STAGED-PATTERN-SCAN.log').read_bytes()
main = json.loads((packet/'MAIN-POSTCHECK.json').read_bytes())
assert main['observed_revision'] == r['source_revision']
a = argparse.ArgumentParser()
a.add_argument('--git-repo',type=Path)
args = a.parse_args()
if args.git_repo:
    got = subprocess.check_output(['git','-C',str(args.git_repo),'show','-s','--format=%H %P',r['published_commit']],text=True).strip().split()
    assert got == [r['published_commit'],r['parent_commit']]
    print('Specified local Git repository contains the recorded commit-parent relationship.')
else:
    print('Commit-parent comparison requires --git-repo.')
print('Verified byte-identical evidence transcripts and bounded scan receipts; checkout identity and independent custody remain separately qualified.')
