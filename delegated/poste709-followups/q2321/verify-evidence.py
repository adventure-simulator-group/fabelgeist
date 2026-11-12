#!/usr/bin/env python3
"""Verify recorded audit evidence only; do not run product code."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re

P = Path(__file__).resolve().parent
CURRENT = '95e63628f9d70b8dd6e6fede64334696c6995009'
OLD = '6533e444fa37552927070cc8d8190907fd3b413e'

def load(name):
    return json.loads((P / name).read_text())

def sha(data):
    return hashlib.sha256(data).hexdigest()

def oid(kind, data):
    return hashlib.sha1(f'{kind} {len(data)}\0'.encode() + data).hexdigest()

def parse_tree(data):
    offset, entries = 0, {}
    while offset < len(data):
        space = data.index(b' ', offset)
        nul = data.index(b'\0', space + 1)
        entries[data[space + 1:nul].decode()] = (data[offset:space].decode(), data[nul + 1:nul + 21].hex())
        offset = nul + 21
    return entries

def entry_at(raw, wanted):
    text = raw.decode()
    pos = re.search(r'"entries"\s*:\s*\[', text).end()
    decoder = json.JSONDecoder()
    for _ in range(wanted + 1):
        while text[pos] in ' \n\r\t,':
            pos += 1
        start = pos
        _, pos = decoder.raw_decode(text, pos)
    return text[start:pos].encode(), len(text[:start].encode()), len(text[:pos].encode())

def pointer(obj, ptr):
    for part in ptr.strip('/').split('/') if ptr != '/' else []:
        obj = obj[int(part)] if isinstance(obj, list) else obj[part]
    return obj

def verify_chain(root, name, chain, source, expected):
    tree = root
    parts = name.split('/')
    assert len(parts) == len(chain)
    for part, item in zip(parts, chain):
        data = (P / 'git-trees' / (tree + '.tree')).read_bytes()
        assert oid('tree', data) == tree
        mode, child = parse_tree(data)[part]
        assert item == {'tree': tree, 'entry': part, 'mode': mode, 'oid': child}
        tree = child
    assert oid('blob', source) == tree == expected['blob_oid']
    assert sha(source) == expected['sha256']

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--current-commit', type=Path, required=True)
    ap.add_argument('--historical-commit', type=Path, required=True)
    ap.add_argument('--original-queue', type=Path, required=True)
    ap.add_argument('--reconstructed-queue', type=Path, required=True)
    ap.add_argument('--skip-manifest', action='store_true')
    args = ap.parse_args()
    if not args.skip_manifest:
        manifest = load('MANIFEST.json')
        actual = {str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name != 'MANIFEST.json'}
        assert actual == set(manifest['files']), 'manifest file set differs'
        for name, info in manifest['files'].items():
            data = (P / name).read_bytes()
            assert len(data) == info['bytes'] and sha(data) == info['sha256'], name

    extraction = load('QUEUE-EXTRACTION.json')
    selected = {}
    for label, filename in [('ORIGINAL', args.original_queue), ('RECONSTRUCTION', args.reconstructed_queue)]:
        raw = filename.read_bytes()
        assert sha(raw) == extraction[label]['whole_sha256']
        data, start, end = entry_at(raw, 2321)
        assert data == (P / f'QUEUE-ENTRY-{label}.json').read_bytes()
        assert (start, end) == (extraction[label]['byte_start'], extraction[label]['byte_end_exclusive'])
        assert sha(data) == extraction[label]['entry_sha256']
        selected[label] = data
    assert selected['ORIGINAL'] == selected['RECONSTRUCTION'] and extraction['byte_equal']
    lines = selected['ORIGINAL'].decode().splitlines()
    assert len(lines) == 634
    for n, start in enumerate(range(1, len(lines) + 1, 65)):
        end = min(start + 64, len(lines))
        assert ''.join(f'{i:4} {lines[i-1]}\n' for i in range(start, end + 1)) == (P / f'HISTORY-DISPLAY-{chr(65+n)}.txt').read_text()

    for row in load('HISTORY-FILES-RECEIPT.json'):
        assert row['expected_match'] and sha((P / 'history' / row['path']).read_bytes()) == row['expected_sha256']
    assert len(load('HISTORY-FILES-RECEIPT.json')) == len(load('HISTORY-REFERENCE-INVENTORY.json')) == 26
    for row in load('HISTORY-EXT-REGISTRATION.json')['files']:
        assert sha((P / 'history' / row['path']).read_bytes()) == row['expected_sha256']
    proof_reg = load('HISTORY-PROOF-REGISTRATION.json')
    assert sha((P / 'history' / proof_reg['path']).read_bytes()) == proof_reg['expected_sha256']
    for item in load('HISTORY-OBJECTS-REGISTRATION.json')['objects']:
        data = (P / 'historical-objects' / item['sha256']).read_bytes()
        assert sha(data) == item['sha256']
        assert data == (P / 'sources' / OLD / item['source_id'].removeprefix('pr-working:')).read_bytes()
    hreg = load('HISTORY-POINTER-REGISTRATION.json')
    for name, views in load('HISTORY-POINTER-DISPLAY.json').items():
        obj = json.loads((P / 'history' / name).read_text())
        for ptr, value in views.items():
            assert ptr in hreg['exact_pointers'][name] and value == pointer(obj, ptr)
    dreg = load('DISCOVERY-DISPLAY-REGISTRATION.json')
    obj = json.loads((P / 'history' / dreg['path']).read_text())
    assert load('DISCOVERY-DISPLAY.json') == {k: obj[k] for k in dreg['fields']}
    qentry = json.loads(selected['ORIGINAL'])
    dptr = load('HISTORY-DISCOVERY-POINTER-REGISTRATION.json')['pointer']
    assert pointer(qentry, dptr) == load('HISTORY-DISCOVERY-POINTER-VIEW.json')
    assert load('HISTORY-DISCOVERY-POINTER-VIEW.json')['sha256'] == sha((P / 'history' / dreg['path']).read_bytes())

    proof = load('FINAL-GIT-PROOF.json')
    assert not proof['errors']
    roots = {}
    for rev, filename in [(CURRENT, args.current_commit), (OLD, args.historical_commit)]:
        raw = filename.read_bytes()
        assert oid('commit', raw) == rev and sha(raw) == proof['commits'][rev]['commit_sha256']
        root = raw.split(b'\n', 1)[0].decode().split()[1]
        assert root == proof['commits'][rev]['root_tree']
        roots[rev] = root
    checked = set()
    for key, info in proof['paths'].items():
        rev, name = key.split(':', 1)
        source = (P / 'sources' / rev / name).read_bytes()
        verify_chain(roots[rev], name, info['chain'], source, info)
        checked.add(key)
    frozen = load('FROZEN-OBJECT-PROOF.json')
    assert not frozen['errors'] and frozen['root_tree'] == roots[OLD]
    assert frozen['commit_sha256'] == sha(args.historical_commit.read_bytes())
    for name, info in frozen['paths'].items():
        source = (P / 'sources' / OLD / name).read_bytes()
        verify_chain(roots[OLD], name, info['chain'], source, info)
        checked.add(OLD + ':' + name)
    router = load('ROUTER-GIT-PROOF.json')
    assert router['ref'] == CURRENT and router['root_tree'] == roots[CURRENT]
    verify_chain(roots[CURRENT], router['path'], router['chain'], (P / 'sources' / CURRENT / router['path']).read_bytes(), router)
    checked.add(CURRENT + ':' + router['path'])
    all_sources = {rev + ':' + str(f.relative_to(P / 'sources' / rev)) for rev in (CURRENT, OLD) for f in (P / 'sources' / rev).rglob('*') if f.is_file()}
    assert checked == all_sources and len(checked) == 27
    assert roots[CURRENT] != roots[OLD]
    for row in load('FILE-COMPARISON-VIEW.json'):
        a = (P / 'sources' / OLD / row['path']).read_bytes()
        b = (P / 'sources' / CURRENT / row['path']).read_bytes()
        assert sha(a) == row['old_sha256'] and sha(b) == row['current_sha256'] and (a == b) == row['byte_equal']
    for i in range(1, 6):
        reg = load(f'TREE-CONNECTOR-REGISTRATION-{i}.json')
        outer = load(f'TREE-CONNECTOR-{i}.json')['structuredContent']
        obj = outer.get('structuredContent')
        if not isinstance(obj, dict) or 'tree' not in obj:
            obj = json.loads(outer['content'])
        assert not obj.get('truncated', False)
        body = b''.join((e['mode'].lstrip('0') + ' ' + e['path']).encode() + b'\0' + bytes.fromhex(e['sha']) for e in obj['tree'])
        assert oid('tree', body) == obj['sha'] == reg['tree_oid']
        assert body == (P / 'git-trees' / (obj['sha'] + '.tree')).read_bytes()

    views = {}
    receipts = sorted(P.glob('SOURCE*-RECEIPT-*.json'))
    for filename in receipts:
        rec = json.loads(filename.read_text())
        m = re.fullmatch(r'SOURCE(?:-(.+?))?-RECEIPT-(.+)\.json', filename.name)
        assert m
        reg = load('SOURCE-REGISTRATION' + ('-' + m.group(1) if m.group(1) else '') + '.json')
        assert datetime.datetime.fromisoformat(rec['displayed_at_utc']) >= datetime.datetime.fromisoformat(reg['registered_at_utc'])
        start, end = rec['range']
        ref, name = rec['ref'], rec['source']
        assert any(start >= a and end <= b for a, b in reg['windows'][ref][name])
        data = (P / 'sources' / ref / name).read_bytes()
        source_lines = data.decode().splitlines()
        assert sha(data) == rec['source_sha256'] and 1 <= start <= end <= len(source_lines)
        text = f'SOURCE {ref}:{name}:{start}-{end}\n' + ''.join(f'{i:4} {source_lines[i-1]}\n' for i in range(start, end + 1))
        assert sha(text.encode()) == rec['display_sha256'] and text == (P / rec['display']).read_text()
        views.setdefault((ref, name), set()).update(range(start, end + 1))
    limits = load('READ-LIMITS.json')
    assert not limits['errors'] and len(receipts) == limits['receipt_count'] == 35
    described = {(rev, name) for rev, paths in limits['personally_displayed_union_by_revision'].items() for name in paths}
    assert set(views) == described
    for rev, paths in limits['personally_displayed_union_by_revision'].items():
        for name, ranges in paths.items():
            assert {i for a, b in ranges for i in range(a, b + 1)} == views[(rev, name)]

    ledger = (P / 'ledger-snapshot/BASELINE-95E-20261110.json').read_bytes()
    view = load('LEDGER-VIEW.json')
    assert sha(ledger) == view['snapshot_sha256']
    entries = json.loads(ledger)['newly_assessed_ledger']['entries']
    assert len(entries) == view['entry_count']
    for terms in (view['terms'], load('LEDGER-EXT-VIEW.json')):
        for term, result in terms.items():
            hits = [{'index': i, 'id': row.get('id')} for i, row in enumerate(entries) if term.lower() in json.dumps(row, ensure_ascii=False).lower()]
            assert len(hits) == result['count'] and hits[:30] == result['first_30']
            assert result['truncated'] == (len(hits) > 30)
    for ptr, row in load('LEDGER-CANDIDATE-DISPLAY.json').items():
        assert ptr in load('LEDGER-CANDIDATE-REGISTRATION.json')['pointers'] and row == pointer(json.loads(ledger), ptr)
    rel = load('RELATED-Q2320-REPORT-REGISTRATION.json')
    assert sha((P / 'RELATED-Q2320-MANIFEST.json').read_bytes()) == rel['source_manifest_sha256']
    assert sha((P / 'RELATED-Q2320-ASSESSMENT.md').read_bytes()) == rel['expected_sha256']
    assert any(x['path'] == 'ASSESSMENT.md' and x['sha256'] == rel['expected_sha256'] for x in load('RELATED-Q2320-MANIFEST.json')['files'])
    print('Evidence verification passed: queue entries, historical records and objects, 27 Git-bound paths, 35 source views, related report, and ledger metadata.')
    print('No product code, product parser, build, test, fixture, or runtime probe was executed.')

if __name__ == '__main__':
    main()
