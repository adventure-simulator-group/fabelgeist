#!/usr/bin/env python3
"""Verify source-only audit evidence; do not run product or fixture code."""
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
        entries[data[space + 1:nul].decode()] = data[nul + 1:nul + 21].hex()
        offset = nul + 21
    return entries

def entry_at(raw, wanted):
    text = raw.decode()
    pos = re.search(r'"entries"\s*:\s*\[', text).end()
    decoder = json.JSONDecoder()
    found = {}
    for i in range(max(wanted) + 1):
        while text[pos] in ' \n\r\t,':
            pos += 1
        start = pos
        _, pos = decoder.raw_decode(text, pos)
        if i in wanted:
            found[i] = (text[start:pos].encode(), len(text[:start].encode()), len(text[:pos].encode()))
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
    ap.add_argument('--skip-manifest', action='store_true')
    a = ap.parse_args()
    if not a.skip_manifest:
        manifest = load('MANIFEST.json')
        actual = {str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name != 'MANIFEST.json'}
        assert actual == set(manifest['files']), 'manifest file set differs'
        for name, info in manifest['files'].items():
            data = (P / name).read_bytes()
            assert len(data) == info['bytes'] and sha(data) == info['sha256'], name
    extract = load('QUEUE-EXTRACTION.json')
    original = a.original_queue.read_bytes()
    reconstructed = a.reconstructed_queue.read_bytes()
    assert sha(original) == extract['ORIGINAL']['whole_sha256']
    assert sha(reconstructed) == extract['RECONSTRUCTION']['whole_sha256']
    selected = entry_at(original, {2307, 2313, 2315, 2316})
    public = entry_at(reconstructed, {2315})
    for label, item in [('ORIGINAL', selected[2315]), ('RECONSTRUCTION', public[2315])]:
        data, start, end = item
        assert data == (P / f'QUEUE-ENTRY-{label}.json').read_bytes()
        assert (start, end) == (extract[label]['byte_start'], extract[label]['byte_end_exclusive'])
        assert sha(data) == extract[label]['entry_sha256']
    assert selected[2315][0] == public[2315][0] and extract['byte_equal']
    history_lines = selected[2315][0].decode().splitlines()
    for label, start, end in [('A', 1, 55), ('B', 56, 110), ('C', 111, 148)]:
        text = ''.join(f'{i:4} {history_lines[i-1]}\n' for i in range(start, end + 1))
        assert text == (P / f'HISTORY-DISPLAY-{label}.txt').read_text()
    related = load('RELATED-HISTORY-VIEW.json')
    related_reg = load('RELATED-HISTORY-REGISTRATION.json')
    for i in (2307, 2313, 2316):
        body = selected[i][0]
        prior = (P / f'PRIOR-q{i}-QUEUE-ENTRY-ORIGINAL.json').read_bytes()
        manifest_raw = (P / f'PRIOR-q{i}-MANIFEST.json').read_bytes()
        manifest = json.loads(manifest_raw)
        assert body == prior == (P / f'RELATED-q{i}-ORIGINAL.json').read_bytes()
        assert sha(prior) == manifest['files']['QUEUE-ENTRY-ORIGINAL.json']['sha256']
        assert len(prior) == manifest['files']['QUEUE-ENTRY-ORIGINAL.json']['bytes']
        assert sha(manifest_raw) == related[str(i)]['prior_manifest_sha256']
        obj = json.loads(body)
        assert related[str(i)]['projection'] == {k: obj.get(k) for k in related_reg['allowed_fields']}
    for name, info in load('HISTORY-FILE-REGISTRATION.json')['files'].items():
        assert sha((P / 'historical-originals' / name).read_bytes()) == info['expected_sha256'], name
    projection = load('HISTORY-PROJECTION.json')
    preg = load('HISTORY-PROJECTION-REGISTRATION.json')
    for name, points in projection.items():
        obj = json.loads((P / 'historical-originals' / name).read_text())
        for ptr, view in points.items():
            val = pointer(obj, ptr)
            cat = ('reader_fields' if '/READ-17' in name else
                   'proposal_fields' if 'PROPOSAL' in name else
                   'verification_fields' if 'VERIFY-' in name else 'adoption_fields')
            assert view == {k: val[k] for k in preg[cat] if k in val}

    proof = load('GIT-PROOF.json')
    for rev, info in proof.items():
        data = (a.anchors_dir / (rev + '.commit')).read_bytes()
        assert oid('commit', data) == rev and sha(data) == info['commit_sha256']
        root = data.split(b'\n', 1)[0].decode().split()[1]
        assert root == info['root_tree']
        for name, detail in info['paths'].items():
            tree = root
            for part in name.split('/'):
                body = (P / 'git-trees' / (tree + '.tree')).read_bytes()
                assert oid('tree', body) == tree
                tree = parse_tree(body)[part]
            assert tree == detail['blob']
            captured = (P / 'sources' / name).read_bytes()
            assert oid('blob', captured) == detail['current_captured_blob']
            assert (oid('blob', captured) == tree) == detail['captured_matches']
            assert sha(captured) == detail['sha256_of_current_capture']
    current = '95e63628f9d70b8dd6e6fede64334696c6995009'
    old = 'e2755a732c84e67296b5e3441a4259a84f461b2f'
    for name in proof[current]['paths']:
        assert proof[current]['paths'][name]['captured_matches']
        assert (proof[current]['paths'][name]['blob'] != proof[old]['paths'][name]['blob']) == name.endswith('/Cargo.toml')
    sample = 'crates/fabelgeist-numpy-storage/src/zip/error.rs'
    assert proof[current]['paths'][sample]['chain'][2]['oid'] == proof[old]['paths'][sample]['chain'][2]['oid'] == 'c17733266a6817b5313f94bd38edb5a45a35caa5'

    archive_path = a.archives_dir / 'rawzip-0.5.2.crate'
    archive_sha = 'ac9999039f74cd73c03158cddfe4362991703c74efe5ce5c8073026f80a5e48c'
    assert sha(archive_path.read_bytes()) == archive_sha
    lock = (P / 'sources/crates/fabelgeist-numpy-storage/Cargo.lock').read_text()
    assert re.search(r'(?ms)^\[\[package\]\]\nname = "rawzip"\nversion = "0\.5\.2"\n.*?^checksum = "' + archive_sha + r'"$', lock)
    native_data = {}
    with tarfile.open(archive_path, 'r:gz') as tar:
        for member in ('rawzip-0.5.2/src/errors.rs', 'rawzip-0.5.2/src/locator.rs',
                       'rawzip-0.5.2/src/archive.rs', 'rawzip-0.5.2/Cargo.toml'):
            native_data[member] = tar.extractfile(member).read()
            if member != 'rawzip-0.5.2/Cargo.toml':
                assert native_data[member] == (P / 'sources' / member).read_bytes()
    excluded = load('PRIVATE-EXCLUSION-QUALIFICATION.json')
    assert sha(native_data['rawzip-0.5.2/Cargo.toml']) == excluded['member_sha256']
    assert excluded['source_archive_sha256'] == archive_sha

    def source_bytes(name):
        return native_data[name] if name == 'rawzip-0.5.2/Cargo.toml' else (P / 'sources' / name).read_bytes()

    views = {}
    for filename in sorted(P.glob('SOURCE*-RECEIPT-*.json')):
        r = json.loads(filename.read_text())
        reg = load('SOURCE-REGISTRATION-EXT.json' if filename.name.startswith('SOURCE-EXT-') else 'SOURCE-REGISTRATION.json')
        assert datetime.datetime.fromisoformat(r['displayed_at_utc']) >= datetime.datetime.fromisoformat(reg['registered_at_utc'])
        start, end = r['range']
        assert any(start >= x and end <= y for x, y in reg['windows'][r['source']])
        data = source_bytes(r['source'])
        lines = data.decode().splitlines()
        assert sha(data) == r['source_sha256'] and end <= len(lines)
        text = f"SOURCE {r['source']}:{start}-{end}\n" + ''.join(f'{i:4} {lines[i-1]}\n' for i in range(start, end + 1))
        assert sha(text.encode()) == r['display_sha256'] and text == (P / r['display']).read_text()
        views.setdefault(r['source'], set()).update(range(start, end + 1))
    for name, info in load('READ-LIMITS.json').items():
        n = len(source_bytes(name).decode().splitlines())
        shown = {i for start, end in info['personally_displayed'] for i in range(start, end + 1)}
        unread = {i for start, end in info['not_personally_displayed'] for i in range(start, end + 1)}
        assert shown == views[name] and shown | unread == set(range(1, n + 1))
        assert not shown & unread and n == info['actual_eof']

    ledger = (P / 'ledger-snapshot/BASELINE-95E-20261110.json').read_bytes()
    ledger_view = load('LEDGER-SEARCH-VIEW.json')
    assert sha(ledger) == ledger_view['snapshot_sha256']
    rows = json.loads(ledger)['newly_assessed_ledger']['entries']
    assert len(rows) == ledger_view['entry_count']
    for term, result in ledger_view['terms'].items():
        hits = [{'index': i, 'identifiers': {k: row[k] for k in ('contract_id', 'id', 'source_id', 'lens') if k in row and isinstance(row[k], (str, int))}}
                for i, row in enumerate(rows) if term in json.dumps(row, ensure_ascii=False)]
        assert len(hits) == result['count'] and hits[:20] == result['first_20']
    print('Evidence verification passed: queues, historical records, Git source paths, locked rawzip source, registered displays and literal ledger search.')
    print('Only named source and manifest members were read from the source package; no embedded archive fixtures or product code were executed.')

if __name__ == '__main__':
    main()
