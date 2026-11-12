#!/usr/bin/env python3
"""Verify registered source and historical evidence, without product execution."""
import argparse, gzip, hashlib, json, pathlib, subprocess
from pathlib import PurePosixPath

R = pathlib.Path(__file__).resolve().parent
H = lambda b: hashlib.sha256(b).hexdigest()
def load(name):
    return json.loads((R / name).read_bytes())
def at(obj, pointer):
    for token in pointer.split('/')[1:]:
        token = token.replace('~1', '/').replace('~0', '~')
        obj = obj[int(token)] if isinstance(obj, list) else obj[token]
    return obj
def oid(kind, b):
    return hashlib.sha1(kind.encode() + b' ' + str(len(b)).encode() + b'\0' + b).hexdigest()
def tree_entry(raw, expected):
    i = 0
    hits = []
    while i < len(raw):
        stop = raw.index(b'\0', i)
        mode, name = raw[i:stop].split(b' ', 1)
        value = raw[stop+1:stop+21]
        assert len(value) == 20
        if name.decode() == expected:
            hits.append((mode.decode(), value.hex()))
        i = stop + 21
    assert i == len(raw) and len(hits) == 1
    return hits[0]
def proof(e, root, objects):
    cur = root
    for part, hop in zip(e['path'].split('/'), e['chain'], strict=True):
        assert hop['tree'] == cur and hop['name'] == part
        meta = objects[cur]
        raw = gzip.decompress((R / meta['path']).read_bytes())
        assert (len(raw), H(raw), oid('tree', raw)) == (meta['bytes'], meta['sha256'], cur)
        mode, cur = tree_entry(raw, part)
        assert (mode, cur) == (hop['mode'], hop['object'])
    source = (R / e['capture']).read_bytes()
    assert cur == e['git_blob'] == oid('blob', source)
    assert (len(source), H(source), len(source.splitlines())) == (e['bytes'], e['sha256'], e['eof'])

a = argparse.ArgumentParser()
a.add_argument('--publication', type=pathlib.Path)
a.add_argument('--ledger-snapshot', type=pathlib.Path)
a.add_argument('--private-extraction', type=pathlib.Path)
args = a.parse_args()
m = load('MANIFEST.json')
seen = set()
for row in m['files']:
    p = PurePosixPath(row['path'])
    assert not p.is_absolute() and '..' not in p.parts and p.as_posix() not in seen
    seen.add(p.as_posix())
    f = R / p
    assert f.is_file() and not f.is_symlink()
    b = f.read_bytes()
    assert (len(b), H(b)) == (row['bytes'], row['sha256'])
assert {p.relative_to(R).as_posix() for p in R.rglob('*') if p.is_file()} == seen | {'MANIFEST.json'}
main = load('REGISTRATION.json')
e = main['current_source']
proof(e, e['root_tree'], main['tree_objects'])
b = (R / e['capture']).read_bytes()
lines = b.decode().splitlines()
display = ''.join(f'{i:4d} {line}\n' for i, line in enumerate(lines, 1)).encode()
rec = load('SOURCE-READ-RECEIPT.json')
assert rec['registration_sha256'] == H((R/'REGISTRATION.json').read_bytes())
assert rec['display_sha256'] == H(display) and display == (R/'SOURCE-DISPLAY.txt').read_bytes()
assert rec['displayed_ranges'] == e['planned_ranges'] and not rec['unread_complement']
support = load('SUPPORT-REGISTRATION.json')
assert support['navigation_registration_sha256'] == H((R/'SUPPORT-NAVIGATION-REGISTRATION.json').read_bytes())
for e in support['entries']:
    proof(e, support['root_tree'], support['tree_objects'])
entries = {e['tag']:e for e in support['entries']}
seen_ranges = {tag:[] for tag in entries}
for group in 'ABCD':
    rec = load(f'SUPPORT-RECEIPT-{group}.json')
    assert rec['registration_sha256'] == H((R/'SUPPORT-REGISTRATION.json').read_bytes())
    parts = []
    for row in rec['rows']:
        e = entries[row['tag']]
        a, z = row['displayed_range']
        assert [a,z] in e['planned_ranges']
        seen_ranges[row['tag']].append([a,z])
        lines = (R/e['capture']).read_text().splitlines()
        parts.append(f"--- {row['tag']} {a}-{z} ---\n")
        parts.extend(f'{i:4d} {lines[i-1]}\n' for i in range(a,z+1))
    d = ''.join(parts).encode()
    assert H(d) == rec['display_sha256'] and d == (R/f'SUPPORT-DISPLAY-{group}.txt').read_bytes()
assert all(seen_ranges[tag] == e['planned_ranges'] for tag,e in entries.items())
for prefix in ('SUPPORT-EXT','CALENDAR-EXT'):
    reg = (R/f'{prefix}-REGISTRATION.json').read_bytes()
    x = json.loads(reg)
    assert x['parent_registration_sha256'] == H((R/'SUPPORT-REGISTRATION.json').read_bytes())
    source = (R/x['capture']).read_bytes()
    assert H(source) == x['sha256']
    lines = source.decode().splitlines()
    d = ''.join(f'{i:4d} {lines[i-1]}\n' for first,last in x['planned_ranges'] for i in range(first,last+1)).encode()
    rec = load(f'{prefix}-RECEIPT.json')
    assert rec['registration_sha256'] == H(reg) and rec['display_sha256'] == H(d)
    assert rec['displayed_ranges'] == x['planned_ranges'] and d == (R/f'{prefix}-DISPLAY.txt').read_bytes()
for name in ('HISTORY-REGISTRATION.json','HISTORY-VERIFY-REGISTRATION.json'):
    for row in load(name)['records']:
        if row['status'].startswith(('matched','captured')):
            b = (R/row['capture']).read_bytes()
            assert (len(b), H(b)) == (row['bytes'], row['sha256'])
history = load('HISTORY-SELECTION-REGISTRATION.json')
assert history['parent_registration_sha256'] == H((R/'HISTORY-REGISTRATION.json').read_bytes())
for record in load('HISTORY-SELECTIONS-RECONSTRUCTED.json')['records']:
    b = (R/'history-exact'/record['path']).read_bytes()
    assert H(b) == record['original_sha256']
    j = json.loads(b)
    for selection in record['selections']:
        assert at(j, selection['pointer']) == selection['value']
rec = load('HISTORY-READ-RECEIPT.json')
assert rec['selection_registration_sha256'] == H((R/'HISTORY-SELECTION-REGISTRATION.json').read_bytes())
assert rec['display_sha256'] == H((R/'HISTORY-SELECTIONS-RECONSTRUCTED.json').read_bytes())
preg = (R/'PROPOSAL-SELECTION-REGISTRATION.json').read_bytes()
x = json.loads(preg)
source = (R/x['capture']).read_bytes()
assert H(source) == x['source_sha256']
j = json.loads(source)
for item in load('PROPOSAL-SELECTIONS-RECONSTRUCTED.json')['selections']:
    assert item['pointer'] in x['pointers'] and at(j,item['pointer']) == item['value']
rec = load('PROPOSAL-SELECTION-RECEIPT.json')
assert rec['registration_sha256'] == H(preg) and rec['display_sha256'] == H((R/'PROPOSAL-SELECTIONS-RECONSTRUCTED.json').read_bytes())
c = load('HISTORY-SOURCE-CORRECTION.json')
assert c['prior_record'] == 'HISTORY-REGISTRATION.json' and c['commit_type'] == 'commit'
historic = (R/c['capture']).read_bytes()
assert (len(historic), H(historic), oid('blob',historic)) == (c['bytes'], c['source_sha256'], c['source_blob'])
assert historic == (R/main['current_source']['capture']).read_bytes()
h = {'path':c['source_path'],'chain':c['chain'],'capture':c['capture'],'git_blob':c['source_blob'],'bytes':c['bytes'],'sha256':c['source_sha256'],'eof':c['eof']}
proof(h,c['root_tree'],c['tree_objects'])
rec = load('QUEUE-READ-RECEIPT.json')
assert rec['registration_sha256'] == H((R/'REGISTRATION.json').read_bytes())
assert rec['selection_sha256'] == H((R/'QUEUE-SELECTION-RECONSTRUCTED.json').read_bytes())
rec = load('RELATED-SELECTION-RECEIPT.json')
assert rec['registration_sha256'] == H((R/'RELATED-SELECTION-REGISTRATION.json').read_bytes())
assert rec['display_sha256'] == H((R/'RELATED-SELECTIONS-RECONSTRUCTED.json').read_bytes())
for reg, results in [('NAVIGATION-REGISTRATION.json','NAVIGATION-RESULTS.json'),('SUPPORT-NAVIGATION-REGISTRATION.json','SUPPORT-NAVIGATION-RESULTS.json'),('LEDGER-NAVIGATION-REGISTRATION.json','LEDGER-NAVIGATION-RESULTS.json'),('LEDGER-NAVIGATION-EXT-REGISTRATION.json','LEDGER-NAVIGATION-EXT-RESULTS.json'),('LEDGER-NAVIGATION-EXT2-REGISTRATION.json','LEDGER-NAVIGATION-EXT2-RESULTS.json'),('PROPOSAL-NAVIGATION-REGISTRATION.json','PROPOSAL-NAVIGATION-RESULTS.json')]:
    assert load(results)['registration_sha256'] == H((R/reg).read_bytes())
if args.publication:
    root = args.publication.resolve()
    git = lambda *x: subprocess.check_output(['git','-C',str(root),*x])
    assert git('rev-parse',main['current_source']['revision']+'^{tree}').decode().strip() == main['current_source']['root_tree']
    assert git('rev-parse',c['verified_commit']+'^{tree}').decode().strip() == c['root_tree']
    for e in [main['current_source'],*support['entries']]:
        assert git('show',main['current_source']['revision']+':'+e['path']) == (R/e['capture']).read_bytes()
    assert git('show',c['verified_commit']+':'+c['source_path']) == historic
    q = (root/main['historical_selection']['path']).read_bytes()
    assert H(q) == main['historical_selection']['sha256']
    assert at(json.loads(q),main['historical_selection']['pointer']) == load('QUEUE-SELECTION-RECONSTRUCTED.json')['value']
    for item in load('RELATED-SELECTIONS-RECONSTRUCTED.json')['selections']:
        if item['source'] == main['historical_selection']['path']:
            assert at(json.loads(q),item['pointer']) == item['value']
    print('Optional local Git commit binding and public 059 selected row comparison passed.')
else:
    print('Commit-to-root and full public 059 queue comparisons require --publication.')
if args.ledger_snapshot:
    reg = load('RELATED-SELECTION-REGISTRATION.json')
    expected = next(x['sha256'] for x in reg['files'] if x['path']=='BASELINE-95E-20261110.json')
    data = args.ledger_snapshot.read_bytes()
    assert H(data) == expected
    j = json.loads(data)
    for x in load('RELATED-SELECTIONS-RECONSTRUCTED.json')['selections']:
        if x['source']=='BASELINE-95E-20261110.json':
            assert at(j,x['pointer']) == x['value']
    print('Optional exact earlier ledger snapshot and selected value comparison passed.')
else:
    print('Earlier live-ledger selected value comparison requires --ledger-snapshot.')
if args.private_extraction:
    root = args.private_extraction.resolve()
    for name in ('HISTORY-REGISTRATION.json','HISTORY-VERIFY-REGISTRATION.json'):
        for row in load(name)['records']:
            if 'capture' in row:
                assert (root/row['path']).read_bytes() == (R/row['capture']).read_bytes()
    assert (root/c['historical_relative_path']).read_bytes() == historic
    print('Optional protected extraction comparison passed, read only.')
else:
    print('Comparison against independently supplied extracted originals requires --private-extraction.')
print('Verified registered displays, object proofs, recovered history and correction; semantic correctness and product behavior remain separately qualified.')
