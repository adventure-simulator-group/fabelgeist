#!/usr/bin/env python3
"""Verify audit evidence bytes and provenance only; never execute product code."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import tarfile

P = Path(__file__).resolve().parent

def load(name):
    return json.loads((P / name).read_text())

def sha(data):
    return hashlib.sha256(data).hexdigest()

def oid(kind, data):
    return hashlib.sha1(f'{kind} {len(data)}\0'.encode() + data).hexdigest()

def parse_tree(data):
    offset = 0
    entries = {}
    while offset < len(data):
        space = data.index(b' ', offset)
        nul = data.index(b'\0', space + 1)
        name = data[space + 1:nul].decode()
        entries[name] = data[nul + 1:nul + 21].hex()
        offset = nul + 21
    return entries

def entry_at(raw, targets):
    text = raw.decode()
    pos = re.search(r'"entries"\s*:\s*\[', text).end()
    decoder = json.JSONDecoder()
    found = {}
    for i in range(max(targets) + 1):
        while text[pos] in ' \n\r\t,':
            pos += 1
        start = pos
        _, pos = decoder.raw_decode(text, pos)
        if i in targets:
            found[i] = (text[start:pos].encode(),
                        len(text[:start].encode()), len(text[:pos].encode()))
    return found

def pointer(obj, ptr):
    for part in ptr.strip('/').split('/') if ptr != '/' else []:
        obj = obj[int(part)] if isinstance(obj, list) else obj[part]
    return obj

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--anchors-dir', type=Path, required=True)
    ap.add_argument('--archives-dir', type=Path, required=True)
    ap.add_argument('--original-queue', type=Path, required=True)
    ap.add_argument('--reconstructed-queue', type=Path, required=True)
    ap.add_argument('--original-registration', type=Path, required=True)
    ap.add_argument('--skip-manifest', action='store_true')
    a = ap.parse_args()
    if not a.skip_manifest:
        manifest = load('MANIFEST.json')
        actual = {str(f.relative_to(P)) for f in P.rglob('*')
                  if f.is_file() and f.name != 'MANIFEST.json'}
        assert actual == set(manifest['files']), 'manifest file set differs'
        for name, info in manifest['files'].items():
            data = (P / name).read_bytes()
            assert len(data) == info['bytes'] and sha(data) == info['sha256'], name
    qual = load('PUBLIC-SUCCESSOR-QUALIFICATION.json')
    assert sha(a.original_registration.read_bytes()) == qual['original_registration_sha256']
    assert sha((P / 'METADATA-REGISTRATION.json').read_bytes()) == qual['public_successor_sha256']

    # Check both complete queue inputs but retain only selected exact entries.
    extraction = load('QUEUE-EXTRACTION.json')
    original = a.original_queue.read_bytes()
    reconstructed = a.reconstructed_queue.read_bytes()
    assert sha(original) == extraction['ORIGINAL']['whole_sha256']
    assert sha(reconstructed) == extraction['RECONSTRUCTION']['whole_sha256']
    old = entry_at(original, {2307, 2312, 2316})
    public = entry_at(reconstructed, {2312})
    for label, result in [('ORIGINAL', old[2312]), ('RECONSTRUCTION', public[2312])]:
        data, start, end = result
        assert data == (P / f'QUEUE-ENTRY-{label}.json').read_bytes()
        assert (start, end) == (extraction[label]['byte_start'], extraction[label]['byte_end_exclusive'])
        assert sha(data) == extraction[label]['entry_sha256']
    assert old[2312][0] == public[2312][0] and extraction['byte_equal']
    for n in (2307, 2316):
        assert old[n][0] == (P / f'RELATED-q{n}-ORIGINAL.json').read_bytes()
        assert sha(old[n][0]) == load('RELATED-VIEW.json')['entries'][str(n)]['sha256']
    for n in (2307, 2316):
        prior = (P / f'PRIOR-q{n}-QUEUE-ENTRY-ORIGINAL.json').read_bytes()
        prior_manifest = load(f'PRIOR-q{n}-MANIFEST.json')
        record = prior_manifest['files']['QUEUE-ENTRY-ORIGINAL.json']
        assert prior == old[n][0] and sha(prior) == record['sha256']
        assert len(prior) == record['bytes']
    historical_lines = old[2312][0].decode().splitlines()
    for label, start, end in [('A', 1, 70), ('B', 71, 140), ('C', 141, 210), ('D', 211, 255)]:
        text = ''.join(f'{i:4} {historical_lines[i-1]}\n' for i in range(start, end + 1))
        assert text == (P / f'HISTORY-DISPLAY-{label}.txt').read_text()

    # Verify the exact historical records referenced by the recovered entry.
    file_reg = load('HISTORY-FILE-REGISTRATION.json')['files']
    for name, info in file_reg.items():
        data = (P / 'historical-originals' / name).read_bytes()
        assert sha(data) == info['expected_sha256'], name
    projected = load('HISTORY-PROJECTION.json')
    preg = load('HISTORY-PROJECTION-REGISTRATION.json')
    for name, points in projected.items():
        obj = json.loads((P / 'historical-originals' / name).read_text())
        for ptr, projection in points.items():
            value = pointer(obj, ptr)
            category = ('reader_fields' if '/READ-17' in name else
                        'proposal_fields' if 'PROPOSAL' in name else
                        'verification_fields' if 'VERIFY-' in name else 'adoption_fields')
            assert projection == {k: value[k] for k in preg[category] if k in value}

    # Verify raw commit -> tree path -> captured current blob identities.
    proof = load('SOURCE-IDENTITIES.json')
    revisions = ('e2755a732c84e67296b5e3441a4259a84f461b2f',
                 '95e63628f9d70b8dd6e6fede64334696c6995009')
    for rev in revisions:
        data = (a.anchors_dir / (rev + '.commit')).read_bytes()
        assert oid('commit', data) == rev and sha(data) == proof[rev]['commit_sha256']
        root = data.split(b'\n', 1)[0].decode().split()[1]
        assert root == proof[rev]['root_tree']
        for name, info in proof[rev]['paths'].items():
            tree = root
            for part in name.split('/'):
                data = (P / 'git-trees' / (tree + '.tree')).read_bytes()
                assert oid('tree', data) == tree
                tree = parse_tree(data)[part]
            assert tree == info['tree_blob_oid']
            captured = (P / 'sources' / name).read_bytes()
            assert oid('blob', captured) == info['captured_current_blob_oid']
            assert (oid('blob', captured) == tree) == info['captured_current_matches_revision']
            assert sha(captured) == info['sha256_of_captured_current']
            if rev == revisions[1]:
                assert oid('blob', captured) == tree
    for name in proof[revisions[0]]['paths']:
        old_id = proof[revisions[0]]['paths'][name]['tree_blob_oid']
        new_id = proof[revisions[1]]['paths'][name]['tree_blob_oid']
        assert (old_id != new_id) == name.endswith('/Cargo.toml')

    # Verify locked native source identity, without processing nested fixtures.
    package = (a.archives_dir / 'rawzip-0.5.2.crate').read_bytes()
    package_sha = 'ac9999039f74cd73c03158cddfe4362991703c74efe5ce5c8073026f80a5e48c'
    assert sha(package) == package_sha
    lock = (P / 'sources/crates/fabelgeist-numpy-storage/Cargo.lock').read_text()
    assert re.search(r'(?ms)^\[\[package\]\]\nname = "rawzip"\nversion = "0\.5\.2"\n.*?^checksum = "' + package_sha + r'"$', lock)
    with tarfile.open(a.archives_dir / 'rawzip-0.5.2.crate', 'r:gz') as tar:
        for name in ('rawzip-0.5.2/src/archive.rs', 'rawzip-0.5.2/src/locator.rs'):
            assert tar.extractfile(name).read() == (P / 'sources' / name).read_bytes()

    # Each actual displayed source interval must have been registered first.
    views = {}
    for receipt in sorted(P.glob('SOURCE*-RECEIPT-*.json')):
        r = json.loads(receipt.read_text())
        ext = receipt.name.startswith('SOURCE-EXT-')
        registration = load('SOURCE-REGISTRATION-EXT.json' if ext else 'SOURCE-REGISTRATION.json')
        assert datetime.datetime.fromisoformat(r['displayed_at_utc']) >= datetime.datetime.fromisoformat(registration['registered_at_utc'])
        start, end = r['range']
        assert any(start >= x and end <= y for x, y in registration['windows'][r['source']])
        data = (P / 'sources' / r['source']).read_bytes()
        lines = data.decode().splitlines()
        assert sha(data) == r['source_sha256'] and end <= len(lines)
        text = f"SOURCE {r['source']}:{start}-{end}\n" + ''.join(f'{i:4} {lines[i-1]}\n' for i in range(start, end + 1))
        assert sha(text.encode()) == r['display_sha256']
        assert text == (P / r['display']).read_text()
        views.setdefault(r['source'], set()).update(range(start, end + 1))
    limits = load('READ-LIMITS.json')
    for name, info in limits.items():
        n = len((P / 'sources' / name).read_text().splitlines())
        listed = set()
        for start, end in info['personally_displayed']:
            listed.update(range(start, end + 1))
        complement = set()
        for start, end in info['not_personally_displayed']:
            complement.update(range(start, end + 1))
        assert listed == views[name] and listed | complement == set(range(1, n + 1))
        assert not listed & complement and n == info['actual_eof']
    print('Evidence verification passed: queues, historical records, Git paths, locked native source, registered displays, and range complements.')
    print('Only supplied source-package members were read; no embedded ZIP/JAR fixtures or product code were executed.')

if __name__ == '__main__':
    main()
