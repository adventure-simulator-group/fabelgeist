#!/usr/bin/env python3
"""Verify audit evidence bytes and provenance; do not run product code."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re

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
    for i in range(wanted + 1):
        while text[pos] in ' \n\r\t,':
            pos += 1
        start = pos
        _, pos = decoder.raw_decode(text, pos)
    return (text[start:pos].encode(), len(text[:start].encode()), len(text[:pos].encode()))

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
    a = ap.parse_args()
    if not a.skip_manifest:
        manifest = load('MANIFEST.json')
        actual = {str(f.relative_to(P)) for f in P.rglob('*')
                  if f.is_file() and f.name != 'MANIFEST.json'}
        assert actual == set(manifest['files']), 'manifest file set differs'
        for name, info in manifest['files'].items():
            data = (P / name).read_bytes()
            assert len(data) == info['bytes'] and sha(data) == info['sha256'], name

    extraction = load('QUEUE-EXTRACTION.json')
    originals = {}
    for label, filename in [('ORIGINAL', a.original_queue), ('RECONSTRUCTION', a.reconstructed_queue)]:
        raw = filename.read_bytes()
        assert sha(raw) == extraction[label]['whole_sha256']
        data, start, end = entry_at(raw, 2308)
        assert data == (P / f'QUEUE-ENTRY-{label}.json').read_bytes()
        assert (start, end) == (extraction[label]['byte_start'], extraction[label]['byte_end_exclusive'])
        assert sha(data) == extraction[label]['entry_sha256']
        originals[label] = (raw, data)
    assert originals['ORIGINAL'][1] == originals['RECONSTRUCTION'][1] and extraction['byte_equal']
    lines = originals['ORIGINAL'][1].decode().splitlines()
    for label, start, end in [('A', 1, 70), ('B', 71, 140), ('C', 141, 210), ('D', 211, 280), ('E', 281, 317)]:
        text = ''.join(f'{i:4} {lines[i-1]}\n' for i in range(start, end + 1))
        assert text == (P / f'HISTORY-DISPLAY-{label}.txt').read_text()

    for name, info in load('HISTORY-FILE-REGISTRATION.json')['files'].items():
        assert sha((P / 'historical-originals' / name).read_bytes()) == info['expected_sha256'], name
    for name, expected in load('HISTORY-QUALIFICATION-REGISTRATION.json')['files'].items():
        assert sha((P / 'historical-originals' / name).read_bytes()) == expected, name
    projection = load('HISTORY-PROJECTION.json')
    preg = load('HISTORY-PROJECTION-REGISTRATION.json')
    for name, points in projection.items():
        obj = json.loads((P / 'historical-originals' / name).read_text())
        for ptr, view in points.items():
            val = pointer(obj, ptr)
            cat = ('proposal_fields' if ptr.startswith('/contract_updates') else
                   'reader_fields' if ptr.startswith('/rows') else
                   'verification_fields' if 'VERIFY-' in name else 'adoption_fields')
            assert view == {k: val[k] for k in preg[cat] if k in val}
    fields = load('HISTORY-QUALIFICATION-PROJECTION-REGISTRATION.json')['fields']
    for name, view in load('HISTORY-QUALIFICATION-PROJECTION.json').items():
        obj = json.loads((P / 'historical-originals' / name).read_text())
        assert view == {k: obj[k] for k in fields if k in obj}

    proof = load('FINAL-TREE-PROOF.json')
    current = '95e63628f9d70b8dd6e6fede64334696c6995009'
    historical = '6533e444fa37552927070cc8d8190907fd3b413e'
    for rev, filename in [(current, a.current_commit), (historical, a.historical_commit)]:
        raw = filename.read_bytes()
        assert oid('commit', raw) == rev and sha(raw) == proof[rev]['commit_sha256']
        root = raw.split(b'\n', 1)[0].decode().split()[1]
        assert root == proof[rev]['root_tree']
        for name, info in proof[rev]['paths'].items():
            tree = root
            for component in name.split('/'):
                data = (P / 'git-trees' / (tree + '.tree')).read_bytes()
                assert oid('tree', data) == tree
                tree = parse_tree(data)[component]
            assert tree == info['blob']
            source = (P / 'sources' / current / name).read_bytes()
            assert oid('blob', source) == tree and sha(source) == info['sha256']
    sample = next(iter(proof[current]['paths']))
    assert proof[current]['root_tree'] != proof[historical]['root_tree']
    assert proof[current]['paths'][sample]['chain'][1]['oid'] == proof[historical]['paths'][sample]['chain'][1]['oid'] == '6b3a8309ed98a1e8f473e74176dccd632d531c1b'
    requests = load('CONNECTOR-REGISTRATION.json')
    for i, rev in enumerate(requests['refs']):
        for j, name in enumerate(requests['paths']):
            obj = load(f'CONNECTOR-{i}-{j}.json')['structuredContent']
            source = obj['content'].encode()
            assert obj['encoding'] == 'utf-8' and oid('blob', source) == obj['sha']
            assert source == (P / 'sources' / rev / name).read_bytes()
            assert source == (P / 'sources' / current / name).read_bytes()

    # Reconstruct each captured current tree response and verify the tree ID.
    for i in range(1, 5):
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
    for filename in sorted(P.glob('SOURCE*-RECEIPT-*.json')):
        r = json.loads(filename.read_text())
        match = re.match(r'SOURCE-(EXT\d*)-RECEIPT-', filename.name)
        registration = load('SOURCE-REGISTRATION-' + match.group(1) + '.json' if match else 'SOURCE-REGISTRATION.json')
        assert datetime.datetime.fromisoformat(r['displayed_at_utc']) >= datetime.datetime.fromisoformat(registration['registered_at_utc'])
        start, end = r['range']
        assert any(start >= x and end <= y for x, y in registration['windows'][r['source']])
        data = (P / 'sources' / current / r['source']).read_bytes()
        source_lines = data.decode().splitlines()
        assert sha(data) == r['source_sha256'] and end <= len(source_lines)
        text = f"SOURCE {r['source']}:{start}-{end}\n" + ''.join(f'{i:4} {source_lines[i-1]}\n' for i in range(start, end + 1))
        assert sha(text.encode()) == r['display_sha256'] and text == (P / r['display']).read_text()
        views.setdefault(r['source'], set()).update(range(start, end + 1))
    for name, info in load('READ-LIMITS.json').items():
        n = len((P / 'sources' / current / name).read_text().splitlines())
        shown = {i for start, end in info['personally_displayed'] for i in range(start, end + 1)}
        unread = {i for start, end in info['not_personally_displayed'] for i in range(start, end + 1)}
        assert shown == views[name] and shown | unread == set(range(1, n + 1))
        assert not shown & unread and n == info['actual_eof']

    # Replay bounded literal metadata searches over the captured exact snapshots.
    ledger = (P / 'ledger-snapshot/BASELINE-95E-20261110.json').read_bytes()
    ledger_view = load('LEDGER-SEARCH-VIEW.json')
    assert sha(ledger) == ledger_view['ledger_sha256']
    entries = json.loads(ledger)['newly_assessed_ledger']['entries']
    assert len(entries) == ledger_view['entry_count']
    for term, result in ledger_view['terms'].items():
        hits = [{'index': i, 'identifiers': {k: row[k] for k in ('contract_id', 'id', 'source_id', 'lens') if k in row and isinstance(row[k], (str, int))}}
                for i, row in enumerate(entries) if term in json.dumps(row, ensure_ascii=False)]
        assert len(hits) == result['count'] and hits[:20] == result['first_20']
    hist_view = load('RELATED-HISTORY-VIEW.json')
    queue_entries = json.loads(originals['ORIGINAL'][0])['entries']
    assert len(queue_entries) == hist_view['entry_count']
    for term, result in hist_view['terms'].items():
        hits = [{'index': i, 'contract_id': row.get('contract_id'), 'lens': row.get('lens')}
                for i, row in enumerate(queue_entries) if term in json.dumps(row, ensure_ascii=False)]
        assert len(hits) == result['count'] and hits[:20] == result['first_20']
    print('Evidence verification passed: exact queues, historical records, both Git trees, source displays, and bounded metadata searches.')
    print('No product code, parser, build, test, fixture, or runtime probe was executed.')

if __name__ == '__main__':
    main()
