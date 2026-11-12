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
        data, start, end = entry_at(raw, 2322)
        assert data == (P / f'QUEUE-ENTRY-{label}.json').read_bytes()
        assert (start, end) == (extraction[label]['byte_start'], extraction[label]['byte_end_exclusive'])
        assert sha(data) == extraction[label]['entry_sha256']
        selected[label] = data
    assert selected['ORIGINAL'] == selected['RECONSTRUCTION'] and extraction['byte_equal']
    lines = selected['ORIGINAL'].decode().splitlines()
    for label, start, end in [('A', 1, 65), ('B', 66, 130), ('C', 131, 143)]:
        assert ''.join(f'{i:4} {lines[i-1]}\n' for i in range(start, end + 1)) == (P / f'HISTORY-DISPLAY-{label}.txt').read_text()

    for name in ('HISTORY-FILES-REGISTRATION.json', 'HISTORY-FILES-EXT-REGISTRATION.json', 'HISTORY-PARTIAL-REGISTRATION.json'):
        for info in load(name)['files']:
            assert sha((P / 'history' / info['relative_path']).read_bytes()) == info['expected_sha256']
    for view_name, reg_name in [('HISTORY-POINTER-DISPLAY.json', 'HISTORY-POINTER-REGISTRATION.json'), ('HISTORY-ATTRIBUTION-DISPLAY.json', 'HISTORY-ATTRIBUTION-REGISTRATION.json')]:
        for name, views in load(view_name).items():
            obj = json.loads((P / 'history' / name).read_text())
            for ptr, value in views.items():
                assert value == pointer(obj, ptr)
                assert ptr in load(reg_name)['exact_pointers'][name]

    proof = load('FINAL-GIT-PROOF.json')
    assert not proof['errors']
    for rev, filename in [(CURRENT, args.current_commit), (OLD, args.historical_commit)]:
        raw = filename.read_bytes()
        assert oid('commit', raw) == rev and sha(raw) == proof['commits'][rev]['commit_sha256']
        root = raw.split(b'\n', 1)[0].decode().split()[1]
        assert root == proof['commits'][rev]['root_tree']
        for key, info in proof['paths'].items():
            if not key.startswith(rev + ':'):
                continue
            name = key[len(rev) + 1:]
            tree = root
            for component, item in zip(name.split('/'), info['chain']):
                data = (P / 'git-trees' / (tree + '.tree')).read_bytes()
                assert oid('tree', data) == tree
                mode, child = parse_tree(data)[component]
                assert item == {'tree': tree, 'entry': component, 'mode': mode, 'oid': child}
                tree = child
            assert len(name.split('/')) == len(info['chain'])
            source = (P / 'sources' / rev / name).read_bytes()
            assert oid('blob', source) == tree == info['blob_oid'] and sha(source) == info['sha256']
    assert set(proof['paths']) == {rev + ':' + str(path.relative_to(P / 'sources' / rev)) for rev in (CURRENT, OLD) for path in (P / 'sources' / rev).rglob('*') if path.is_file()}
    for path, digest in [('crates/adventuresim-stdb-module/src/foraging.rs', 'be82a5ef7905b05d68bba4d61d5173cba6011c51ee8504da85d697bb2d80df84'), ('crates/adventuresim-stdb-module/src/foraging/timing.rs', '30193f2454edfb59bb1c74ca3f63056accf4a4540bc7fd16260bdfc8ed5b7cbc')]:
        old_source = (P / 'sources' / OLD / path).read_bytes()
        assert old_source == (P / 'sources' / CURRENT / path).read_bytes()
        assert old_source == (P / 'history/parallel-l13-20261029/objects' / digest).read_bytes()
    core = 'crates/adventuresim-core/src/foraging.rs'
    assert (P / 'sources' / OLD / core).read_bytes() == (P / 'sources' / CURRENT / core).read_bytes()
    assert proof['commits'][OLD]['root_tree'] != proof['commits'][CURRENT]['root_tree']
    for i in range(1, 3):
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
        r = json.loads(filename.read_text())
        m = re.fullmatch(r'SOURCE(?:-(.+?))?-RECEIPT-(.+)\.json', filename.name)
        assert m
        reg = load('SOURCE-REGISTRATION' + ('-' + m.group(1) if m.group(1) else '') + '.json')
        assert datetime.datetime.fromisoformat(r['displayed_at_utc']) >= datetime.datetime.fromisoformat(reg['registered_at_utc'])
        start, end = r['range']
        assert reg['baseline'] == CURRENT and any(start >= x and end <= y for x, y in reg['windows'][r['source']])
        data = (P / 'sources' / CURRENT / r['source']).read_bytes()
        source_lines = data.decode().splitlines()
        assert sha(data) == r['source_sha256'] and 1 <= start <= end <= len(source_lines)
        text = f"SOURCE {r['source']}:{start}-{end}\n" + ''.join(f'{i:4} {source_lines[i-1]}\n' for i in range(start, end + 1))
        assert sha(text.encode()) == r['display_sha256'] and text == (P / r['display']).read_text()
        views.setdefault(r['source'], set()).update(range(start, end + 1))
    limits = load('READ-LIMITS.json')
    assert not limits['errors'] and len(receipts) == limits['receipt_count']
    assert set(views) == set(limits['current_personal_displayed_union'])
    for name, ranges in limits['current_personal_displayed_union'].items():
        assert {i for a, b in ranges for i in range(a, b + 1)} == views[name]

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
    print('Evidence verification passed: selected queues, historical records, both Git commits and trees, 42 source views, and bounded ledger metadata.')
    print('No product code, product parser, build, test, fixture, or runtime probe was executed.')

if __name__ == '__main__':
    main()
