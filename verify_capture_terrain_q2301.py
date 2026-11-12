#!/usr/bin/env python3
"""Verify portable q2301 evidence and separately qualified local replay."""
import argparse, hashlib, json, subprocess, sys, tarfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
h = lambda b: hashlib.sha256(b).hexdigest()
packet = root / 'delegated/capture-terrain-q2301'
manifest = (packet / 'MANIFEST.json').read_bytes()
assert h(manifest) == 'b8f69a8b91eee14e30cd8c165f56daa7071ccd537b8eedd1f707be79feb46511'
rows = json.loads(manifest)['files']
assert len(rows) == 71
seen = set()
for row in rows:
    p = PurePosixPath(row['path'])
    assert not p.is_absolute() and '..' not in p.parts and p.as_posix() not in seen
    seen.add(p.as_posix())
    path = packet / p
    assert path.is_file() and not path.is_symlink()
    b = path.read_bytes()
    assert (len(b), h(b)) == (row['bytes'], row['sha256'])
assert {p.relative_to(packet).as_posix() for p in packet.rglob('*') if p.is_file()} == seen | {'MANIFEST.json'}
archive = root / 'delegated/capture-terrain-q2301-archive.tar.gz'
archive_hash = '66c013d7995d41bc546c47de80f8342708c360c2e176c22beef3876ba2911cf3'
assert h(archive.read_bytes()) == archive_hash
with tarfile.open(archive, 'r:gz') as t:
    members = t.getmembers()
    assert len(members) == 72
    archived = set()
    for member in members:
        p = PurePosixPath(member.name)
        assert member.isfile() and not p.is_absolute() and p.parts[0] == 'bundle' and '..' not in p.parts
        rel = PurePosixPath(*p.parts[1:])
        assert rel.as_posix() not in archived
        archived.add(rel.as_posix())
        assert t.extractfile(member).read() == (packet / rel).read_bytes()
    assert archived == seen | {'MANIFEST.json'}
scan = root / 'delegated/capture-terrain-q2301-scan'
receipt = (scan / 'RECEIPT.json').read_bytes()
assert h(receipt) == 'd7f9ddb3202b42b84caff52a49f11289371c22ee87c6d9ab5da7d9a7ade3ae30'
s = json.loads(receipt)
assert s['archive_sha256'] == archive_hash
for name, digest in s['scans'].items():
    b = (scan / name).read_bytes()
    assert h(b) == digest
    x = json.loads(b)
    assert x['forbidden_digest_total'] == 0 and not x['other_candidate_counts'] and not x['exceptions']
replay = root / 'delegated/capture-terrain-q2301-replay'
r = json.loads((replay / 'RECORD.json').read_bytes())
log = (replay / 'VERIFICATION.log').read_bytes()
assert h(log) == r['transcript_sha256'] and len(log) == r['transcript_bytes']
assert r['packet_manifest_sha256'] == h(manifest)
assert r['private_capture_manifest_sha256'] == h((packet / 'PRIVATE-CAPTURE-MANIFEST.json').read_bytes())
report = json.loads((root / 'ASSESSMENT-CENTRAL-CAPTURE-TERRAIN-13U-95E.json').read_bytes())
assert report['source_report'] == 'delegated/capture-terrain-q2301/REPORT.json'
assert report['historical_queue'] == 'baseline-059/queue.json#/entries/2301'
agent = json.loads((packet / 'REPORT.json').read_bytes())
assert agent['author'] == '/root/mhr_character' and agent['historical_queue_pointer'] == report['historical_queue']
a = argparse.ArgumentParser()
a.add_argument('--publication', type=Path)
a.add_argument('--private-capture', type=Path)
args = a.parse_args()
cmd = [sys.executable, '-B', str(packet / 'verify-evidence.py')]
if args.publication:
    cmd += ['--publication', str(args.publication.resolve())]
if args.private_capture:
    cmd += ['--private-capture', str(args.private_capture.resolve())]
subprocess.run(cmd, cwd=root, check=True)
print('Verified exact terrain packet, portable archive, scan and qualified same-machine replay receipts; no product behavior or private custody implied.')
