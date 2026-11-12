#!/usr/bin/env python3
"""Verify this evidence packet; never import or execute product code."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess

P = Path(__file__).resolve().parent

def read(name):
    return json.loads((P / name).read_text())

def sha(data):
    return hashlib.sha256(data).hexdigest()

def blob(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()

def stamp(text):
    return dt.datetime.fromisoformat(text)

def spans(values):
    result = []
    for n in sorted(values):
        if not result or result[-1][1] + 1 != n:
            result.append([n, n])
        else:
            result[-1][1] = n
    return result

def safe_path(path):
    q = Path(path)
    return not q.is_absolute() and '..' not in q.parts

def git(repo, *args):
    env = dict(os.environ, GIT_NO_LAZY_FETCH='1')
    return subprocess.check_output(['git', '--no-replace-objects', '-C', str(repo), *args], env=env)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--git-repo', type=Path)
    parser.add_argument('--publication', type=Path)
    parser.add_argument('--restricted-predecessor', type=Path)
    args = parser.parse_args()
    manifest = read('MANIFEST.json')
    actual = {str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f != P / 'MANIFEST.json'}
    assert actual == set(manifest['files']), 'packet file set'
    for name, item in manifest['files'].items():
        assert safe_path(name) and not (P/name).is_symlink()
        data = (P/name).read_bytes()
        assert len(data) == item['bytes'] and sha(data) == item['sha256'], name

    records = read('SOURCE-CAPTURES.json') + read('SOURCE-EXTENSION-CAPTURES.json') + [read('SOURCE-OBJECT-CAPTURE.json')]
    captures = {x['path']: x for x in records}
    assert len(captures) == 8
    for x in captures.values():
        data = (P/x['capture']).read_bytes()
        assert len(data) == x['bytes'] and sha(data) == x['sha256'] and blob(data) == x['blob']
        assert len(data.splitlines()) == x['eof']
    receipt = read('SOURCE-READ-RECEIPT.json')
    seen = {x: set() for x in captures}
    for row in receipt['chunks']:
        data = captures[row['source_path']]
        reg = read(row['registration'])
        assert stamp(reg['registered_utc']) <= stamp(row['saved_utc'])
        assert any(x['path'] == row['source_path'] and x['range'][0] <= row['start'] <= row['end'] <= x['range'][1] for x in reg['sources'])
        lines = (P/data['capture']).read_text().splitlines()
        output = ''.join(f'{i:4d} {lines[i-1]}\n' for i in range(row['start'], row['end']+1)).encode()
        assert output == (P/row['display']).read_bytes() and sha(output) == row['sha256']
        seen[row['source_path']].update(range(row['start'], row['end']+1))
    for row in receipt['coverage']:
        assert row['eof'] == captures[row['source_path']]['eof']
        assert row['personally_displayed'] == spans(seen[row['source_path']])
        assert row['unread'] == spans(set(range(1, row['eof']+1)) - seen[row['source_path']])
    assert sum(map(len, seen.values())) == receipt['unique_lines_personally_displayed']
    for row in read('OTHER-READ-RECEIPT.json')['chronology']:
        assert stamp(read(row['registration'])['registered_utc']) <= stamp(row['saved_utc'])

    h = read('HISTORY-REGISTRATION.json')
    old = (P/'Q2318-ORIGINAL-EXACT-SLICE.json').read_bytes()
    public = (P/'Q2318-PUBLIC-EXACT-SLICE.json').read_bytes()
    assert old == public and h['original_and_public_bytes_equal']
    for stream, data in zip(h['streams'], [old, public]):
        assert sha(data) == stream['entry_sha256'] and len(data) == stream['entry_bytes']
    q = json.loads(old)
    assert read('Q2318-DISPLAY-A.json') == {k:v for k,v in q.items() if k != 'exact_source_identities_and_assessed_ranges'}
    assert read('Q2318-DISPLAY-SOURCES.json') == q['exact_source_identities_and_assessed_ranges']
    q17data = (P/'Q2317-ORIGINAL-EXACT-SLICE.json').read_bytes()
    q17public = (P/'Q2317-PUBLIC-EXACT-SLICE.json').read_bytes()
    assert q17data == q17public
    q17 = json.loads(q17data)
    for row in read('Q2317-IDENTITIES.json'):
        assert sha(q17data) == row['entry_sha256'] and len(q17data) == row['entry_bytes']
    assert all(q17[k] == v for k,v in read('Q2317-SELECT.json').items())
    refs = {}
    def collect(value):
        if isinstance(value, dict):
            if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str):
                refs[value['path']] = value['sha256']
            for v in value.values(): collect(v)
        elif isinstance(value, list):
            for v in value: collect(v)
    collect(q)
    reg = read('RECOVERY-REGISTRATION.json')
    results = {x['reference']:x for x in read('RECOVERY-RESULT.json')}
    assert len(refs) == len(results) == len(reg['references']) == 8
    for x in reg['references']:
        row = results[x['path']]
        data = (P/row['packet_path']).read_bytes()
        assert refs[x['path']] == x['expected_sha256'] == row['actual_sha256'] == sha(data)
        assert row['matched'] and len(data) == row['bytes']
    for n in [168, 247]:
        base = P/'historical-originals/parallel-l13-20261029'
        delegated = json.loads((base/f'READ-PL13-{n}.json').read_text())
        registration = json.loads((base/f'REGISTRATION-PL13-{n}.json').read_text())
        root = json.loads((P/f'historical-originals/segovb/READ-L13-PARALLEL-{n-2}.json').read_text())
        selection = {'delegated_status':delegated['status'], 'registered_before_read_attestation':delegated['registered_before_read_attestation'], 'delegated_row_0':delegated['rows'][0], 'source_registration_0':registration['sources'][0], 'root_kind':root['kind'], 'root_at':root['at'], 'root_verified_source_registrations':root['verified_source_registrations'], 'root_personal_rereads':root['root_personal_rereads'], 'root_current_source_reconciled':root['current_source_reconciled'], 'root_runtime_validation':root['runtime_validation'], 'root_implementation_gate_complete':root['implementation_gate_complete']}
        assert selection == read(f'HISTORICAL-SELECT-{n}.json')
    for base, selection in [('ASSESSMENT-L04-COOKING-WATER-PRECISION-95E.json','RELATED-WATER-SELECT.json'),('ASSESSMENT-L1-MICROBIAL-AGGREGATES.json','RELATED-MICROBIAL-SELECT.json')]:
        data = read('related/'+base)
        assert all(data[k] == v for k,v in read(selection).items())
    print('PASS exact packet file set, source blobs, registered display reconstruction and complements, q2317/q2318 entry slices, eight recovered record hashes and attributed selections.')

    if args.git_repo:
        rev = receipt['source_revision']
        for x in captures.values():
            assert git(args.git_repo, 'show', rev+':'+x['path']) == (P/x['capture']).read_bytes()
        for x in read('SOURCE-IDENTITIES.json'):
            data = git(args.git_repo, 'show', x['revision']+':'+x['path'])
            assert sha(data) == x['sha256'] and blob(data) == x['blob'] and len(data) == x['bytes']
        for stream in h['streams']:
            data = git(args.git_repo, 'show', stream['revision']+':'+stream['path'])
            assert sha(data) == stream['stream_sha256'] and len(data) == stream['stream_bytes'] and blob(data) == stream['git_blob']
            assert data[stream['entry_start_byte']:stream['entry_end_byte_exclusive']] == old
            assert json.loads(data)['entries'][2318] == q
        for stream in read('Q2317-IDENTITIES.json'):
            data = git(args.git_repo, 'show', stream['revision']+':baseline-059/queue.json')
            assert sha(data) == stream['stream_sha256'] and len(data) == stream['stream_bytes']
            assert data[stream['entry_start']:stream['entry_end_exclusive']] == q17data
            assert json.loads(data)['entries'][2317] == q17
        related_rev = read('RELATED-REGISTRATION.json')['revision']
        for x in read('RELATED-IDENTITIES.json'):
            data = git(args.git_repo, 'show', related_rev+':'+x['path'])
            assert sha(data) == x['sha256'] and blob(data) == x['blob'] and len(data) == x['bytes']
            if 'packet_path' in x: assert data == (P/x['packet_path']).read_bytes()
        def locator(revision, scopes, pattern):
            env = dict(os.environ, GIT_NO_LAZY_FETCH='1')
            p = subprocess.run(['git','--no-replace-objects','-C',str(args.git_repo),'grep','-n','-i','-F',pattern,revision,'--',*scopes], capture_output=True, text=True, env=env)
            found = []
            for line in p.stdout.splitlines():
                parts = line.split(':',3)
                if len(parts) >= 4:
                    try: found.append({'path':parts[1], 'line':int(parts[2])})
                    except ValueError: pass
            return p.returncode, found
        lr = read('LOCATOR-REGISTRATION.json')
        for pattern, saved in read('LOCATOR-RESULT.json').items():
            status, found = locator(lr['revision'], lr['scopes'], pattern)
            assert status == saved['status'] and len(found) == saved['matches_total'] and found[:100] == saved['displayed'] and max(0,len(found)-100) == saved['omitted_count']
        lr = read('LOCATOR-EXTENSION-REGISTRATION.json')
        for key, saved in read('LOCATOR-EXTENSION-RESULT.json').items():
            group, pattern = key.split(':',1)
            status, found = locator(lr[group+'_revision'], lr[group+'_scopes'], pattern)
            assert status == saved['status'] and len(found) == saved['matches_total'] and found[:100] == saved['displayed'] and max(0,len(found)-100) == saved['omitted_count']
        lr = read('LOCATOR-DETAIL-REGISTRATION.json')
        for pattern, saved in read('LOCATOR-DETAIL-RESULT.json').items():
            status, found = locator(lr['revision'],lr['scopes'],pattern)
            assert status == saved['status'] and len(found) == saved['total'] and found[:100] == saved['locators'] and max(0,len(found)-100) == saved['omitted']
        print('PASS fixed 95e source membership, 6533 source identities, both full original/public queue streams and slices, related committed assessments and bounded locator results.')
    else:
        print('NOT REPEATED fixed Git membership, 6533/95e comparison, full queue stream identities and related assessment membership: supply --git-repo.')

    if args.publication:
        identity = read('TRANSPORT-IDENTITY.json')
        data = (args.publication/identity['path']).read_bytes()
        assert sha(data) == identity['sha256'] and len(data) == identity['bytes']
        nodes = []
        def walk(v):
            if isinstance(v, dict):
                if 'sha256' in v: nodes.append(v)
                for child in v.values(): walk(child)
            elif isinstance(v, list):
                for child in v: walk(child)
        walk(json.loads(gzip.decompress(data)))
        for row in read('TRANSPORT-MATCHES.json'):
            x = results[row['reference']]
            rel = row['reference'].removeprefix('target/726-audit-adler-20261027/')
            matches = [n for n in nodes if n.get('sha256') == x['actual_sha256'] and n.get('bytes') == x['bytes']]
            assert len(matches) == row['digest_size_matches']
            assert sum(str(n.get('path','')).endswith(rel) for n in matches) == row['path_suffix_matches']
        print('PASS published transport metadata comparisons for eight records; off-machine custody not verified.')
    else:
        print('NOT REPEATED transport inventory metadata comparison: supply --publication.')

    if args.restricted_predecessor:
        boundary = read('PRIVACY-BOUNDARY.json')
        pred = args.restricted_predecessor
        data = (pred/'MANIFEST.json').read_bytes()
        assert sha(data) == boundary['restricted_predecessor_manifest_sha256']
        m = json.loads(data)
        actual = {str(x.relative_to(pred)) for x in pred.rglob('*') if x.is_file() and x != pred/'MANIFEST.json'}
        assert actual == {x['path'] for x in m['files']}
        omitted = {x['path']:x for x in boundary['omitted']}
        for x in m['files']:
            assert safe_path(x['path'])
            b = (pred/x['path']).read_bytes()
            assert sha(b) == x['sha256'] and len(b) == x['bytes']
            if x['path'] in omitted: assert x == omitted[x['path']] and not (P/x['path']).exists()
            else: assert (P/x['path']).read_bytes() == b
        print('PASS restricted predecessor seal, two omitted helper crosswalks and unchanged inherited bytes.')
    else:
        print('NOT REPEATED restricted predecessor and omitted helper crosswalk: supply --restricted-predecessor.')
    print('LIMITS: recorded chronology is an attestation; hashes and scans do not establish semantic truth, supported reachability, runtime behavior or independent custody.')

if __name__ == '__main__':
    main()
