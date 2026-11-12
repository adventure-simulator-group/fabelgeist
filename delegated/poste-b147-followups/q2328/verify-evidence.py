#!/usr/bin/env python3
"""Recheck q2328 evidence bytes and source object links, without product execution."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re

P = Path(__file__).resolve().parent
CURRENT = '95e63628f9d70b8dd6e6fede64334696c6995009'
OLD = 'e2755a732c84e67296b5e3441a4259a84f461b2f'

def load(name):
    return json.loads((P / name).read_text())

def sha(data):
    return hashlib.sha256(data).hexdigest()

def oid(kind, data):
    return hashlib.sha1(f'{kind} {len(data)}\0'.encode() + data).hexdigest()

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
        part = part.replace('~1', '/').replace('~0', '~')
        obj = obj[int(part)] if isinstance(obj, list) else obj[part]
    return obj

def parse_tree(data):
    entries, off = {}, 0
    while off < len(data):
        space = data.index(b' ', off)
        nul = data.index(b'\0', space + 1)
        entries[data[space+1:nul].decode()] = (data[off:space].decode(), data[nul+1:nul+21].hex())
        off = nul + 21
    return entries

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--current-commit', type=Path, required=True)
    ap.add_argument('--historical-commit', type=Path, required=True)
    ap.add_argument('--original-queue', type=Path, required=True)
    ap.add_argument('--reconstructed-queue', type=Path, required=True)
    ap.add_argument('--ledger-snapshot', type=Path, required=True)
    ap.add_argument('--skip-manifest', action='store_true')
    a = ap.parse_args()
    if not a.skip_manifest:
        m = load('MANIFEST.json')
        files = {str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f.name != 'MANIFEST.json'}
        assert files == set(m['files']), 'manifest file set differs'
        for name, item in m['files'].items():
            raw = (P/name).read_bytes()
            assert len(raw) == item['bytes'] and sha(raw) == item['sha256'], name

    ext = load('QUEUE-EXTRACTION.json')
    entries = {}
    for label, filename in [('ORIGINAL', a.original_queue), ('RECONSTRUCTION', a.reconstructed_queue)]:
        raw = filename.read_bytes()
        assert sha(raw) == ext[label]['whole_sha256']
        body, start, end = entry_at(raw, 2328)
        assert body == (P/f'QUEUE-ENTRY-{label}.json').read_bytes()
        assert (start, end) == (ext[label]['byte_start'], ext[label]['byte_end_exclusive'])
        assert sha(body) == ext[label]['entry_sha256']
        entries[label] = body
    assert entries['ORIGINAL'] == entries['RECONSTRUCTION'] and ext['byte_equal']
    lines = entries['ORIGINAL'].decode().splitlines()
    assert len(lines) == 187
    for n, start in enumerate(range(1, len(lines)+1, 60), 1):
        end = min(start+59, len(lines))
        view = ''.join(f'{i:4} {lines[i-1]}\n' for i in range(start, end+1))
        assert view == (P/f'HISTORY-DISPLAY-{n:02}.txt').read_text()

    inventory = {r['path'].removeprefix('target/726-audit-adler-20261027/'):r['sha256'] for r in load('HISTORY-REFERENCE-INVENTORY.json')}
    receipts = load('HISTORY-FILES-RECEIPT.json')
    assert len(inventory) == len(receipts) == 13
    for rec in receipts:
        raw = (P/'history'/rec['path']).read_bytes()
        assert rec['expected_match'] and len(raw) == rec['bytes']
        assert sha(raw) == rec['sha256'] == rec['expected_sha256'] == inventory[rec['path']]
    reg = load('HISTORY-POINTER-REGISTRATION.json')
    for name, views in load('HISTORY-POINTER-CAPTURE.json').items():
        src = json.loads((P/'history'/name).read_text())
        for ptr, value in views.items():
            assert ptr in reg['exact_pointers'][name] and pointer(src, ptr) == value
    sr = load('HISTORY-ROOT-SUMMARY-REGISTRATION.json')
    for name, summary in load('HISTORY-ROOT-SUMMARY-VIEW.json').items():
        assert name in sr['files']
        src = json.loads((P/'history'/name).read_text())
        assert {k:src.get(k) for k in sr['fields']} == {k:summary[k] for k in sr['fields']}
        assert summary['finding_ids'] == [{'index':i,'id':x['id']} for i,x in enumerate(src.get('finding_connections_and_scope_dispositions', []))]
    rr = load('HISTORY-ROOT-SCOPE-REGISTRATION.json')
    obj = pointer(json.loads((P/'history'/rr['path']).read_text()), rr['pointer'])
    assert {k:obj[k] for k in rr['fields']} == load('HISTORY-ROOT-SCOPE-VIEW.json')

    roots = {}
    for rev, filename in [(CURRENT,a.current_commit),(OLD,a.historical_commit)]:
        raw = filename.read_bytes()
        assert oid('commit', raw) == rev
        roots[rev] = (raw.split(b'\n',1)[0].split()[1].decode(), sha(raw), len(raw))
    proofs = load('FINAL-GIT-PROOF.json')
    checked, used = set(), set()
    assert len(proofs) == 11
    for item in proofs:
        rev, name = item['revision'], item['path']
        root, commit_hash, commit_len = roots[rev]
        assert (item['root_tree'],item['commit_sha256'],item['commit_bytes']) == (root,commit_hash,commit_len)
        cur = root
        parts = name.split('/')
        assert len(parts) == len(item['chain'])
        for part, link in zip(parts, item['chain']):
            raw = (P/'git-trees'/(cur+'.tree')).read_bytes()
            assert oid('tree', raw) == cur
            used.add(cur)
            mode, nxt = parse_tree(raw)[part]
            assert link == {'tree':cur,'component':part,'mode':mode,'object':nxt}
            cur = nxt
        src = (P/'sources'/rev/name).read_bytes()
        assert oid('blob',src) == cur and sha(src) == item['source_sha256'] and len(src) == item['source_bytes']
        checked.add(rev+':'+name)
    sources = {rev+':'+str(f.relative_to(P/'sources'/rev)) for rev in (CURRENT,OLD) for f in (P/'sources'/rev).rglob('*') if f.is_file()}
    assert checked == sources and used == set(load('TREE-USED.json'))
    for item in load('FILE-COMPARISON-VIEW.json'):
        old = (P/'sources'/OLD/item['path']).read_bytes()
        cur = (P/'sources'/CURRENT/item['path']).read_bytes()
        assert sha(old) == item['old_sha256'] and sha(cur) == item['current_sha256'] and (old == cur) == item['byte_equal']
    assert len(load('FILE-COMPARISON-VIEW.json')) == 4
    for filename in sorted(P.glob('TREE-CONNECTOR-[123]-*.json')):
        root = json.loads(filename.read_text())
        assert not root.get('isError')
        outer = root['structuredContent']
        obj = outer.get('structuredContent')
        if not isinstance(obj,dict) or 'tree' not in obj:
            obj = json.loads(outer['content'])
        assert not obj.get('truncated',False)
        body = b''.join((e['mode'].lstrip('0')+' '+e['path']).encode()+b'\0'+bytes.fromhex(e['sha']) for e in obj['tree'])
        assert oid('tree',body) == obj['sha'] and body == (P/'git-trees'/(obj['sha']+'.tree')).read_bytes()

    seen = []
    for filename in sorted(P.glob('SOURCE*-RECEIPT-*.json')):
        rec = json.loads(filename.read_text())
        m = re.fullmatch(r'SOURCE(?:-(.+?))?-RECEIPT-(.+)\.json',filename.name)
        assert m
        regname = 'SOURCE-REGISTRATION'+('-'+m.group(1) if m.group(1) else '')+'.json'
        reg = load(regname)
        assert datetime.datetime.fromisoformat(rec['displayed_at_utc']) >= datetime.datetime.fromisoformat(reg['registered_at_utc'])
        start,end = rec['range']; name = rec['source']
        assert any(start>=x and end<=y for x,y in reg['windows'][name])
        raw = (P/'sources'/CURRENT/name).read_bytes(); lines = raw.decode().splitlines()
        assert sha(raw) == rec['source_sha256'] and 1<=start<=end<=len(lines)
        view = f'SOURCE {name}:{start}-{end}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(start,end+1))
        assert sha(view.encode()) == rec['display_sha256'] and view == (P/rec['display']).read_text()
        seen.append((filename.name,name,start,end))
    limits = load('READ-LIMITS.json')['personal_displayed_ranges']
    assert len(seen) == len(limits) == 14
    assert set(seen) == {(r['receipt'],r['source'],*r['range']) for r in limits}

    raw = a.ledger_snapshot.read_bytes(); ledger = json.loads(raw); nav = load('LEDGER-NAV-VIEW.json')
    assert sha(raw) == nav['snapshot_sha256'] and len(raw) == nav['snapshot_bytes']
    rows = ledger['newly_assessed_ledger']['entries']; assert len(rows) == nav['row_count']
    for term,result in nav['results'].items():
        hits = [{'index':i,'id':row.get('id',row.get('finding_id',None))} for i,row in enumerate(rows) if term.casefold() in json.dumps(row,ensure_ascii=False).casefold()]
        assert len(hits) == result['total_matches'] and hits[:30] == result['shown'] and (len(hits)>30) == result['truncated']
    selected = load('LEDGER-SELECT-VIEW.json')
    assert selected['sha256'] == sha(raw)
    for row in selected['selections']:
        assert row['pointer'] in load('LEDGER-SELECT-REGISTRATION.json')['pointers'] and pointer(ledger,row['pointer']) == row['value']
    assert sha((P/'RELATED-REPORT-0.txt').read_bytes()) == '6ba6b0cacd3377affca53aab8e3a8a5c29d26ff619d6f18dde675f3b8392a0a2'
    assert sha((P/'RELATED-REPORT-1.txt').read_bytes()) == '41788f9ba9aedc71630d9bb8bc4b7f5169025a60bef025637d4ac2a8310497ec'
    print('Evidence verified: exact queue entry and views, 13 historical records and pointers, 11 Git-bound source paths, 14 current views, related reports and bounded ledger metadata.')
    print('No product code, build, test, fixture, renderer or runtime probe executed.')

if __name__ == '__main__':
    main()
