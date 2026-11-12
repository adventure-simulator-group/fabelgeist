#!/usr/bin/env python3
"""Verify this source-evidence packet; never import or execute product code."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

P = Path(__file__).resolve().parent

def load(name):
    return json.loads((P / name).read_text())

def sha(b):
    return hashlib.sha256(b).hexdigest()

def canon(x):
    return json.dumps(x, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()

def timestamp(s):
    return dt.datetime.fromisoformat(s)

def select(x, pointer):
    if pointer == '/':
        return x
    for part in pointer.strip('/').split('/'):
        part = part.replace('~1', '/').replace('~0', '~')
        x = x[int(part)] if isinstance(x, list) else x[part]
    return x

def intervals(ns):
    result = []
    for n in sorted(ns):
        if not result or result[-1][1] + 1 != n:
            result.append([n, n])
        else:
            result[-1][1] = n
    return result

def git(repo, rev, path):
    return subprocess.check_output(['git', '--no-replace-objects', '-C', str(repo), 'show', f'{rev}:{path}'])

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--git-repo', type=Path, help='repository containing pinned source and specified historical evidence commits')
    ap.add_argument('--parent-stage', type=Path, help='original prepared capture-readiness-q2303 packet')
    ap.add_argument('--publication', type=Path, help='publication root with recovered transport inventory')
    a = ap.parse_args()
    manifest = load('MANIFEST.json')
    actual = {str(p.relative_to(P)) for p in P.rglob('*') if p.is_file() and p.name != 'MANIFEST.json'}
    assert actual == set(manifest['files']), 'packet file set differs'
    for name, meta in manifest['files'].items():
        p = Path(name)
        assert not p.is_absolute() and '..' not in p.parts and not (P / p).is_symlink()
        b = (P / p).read_bytes()
        assert sha(b) == meta['sha256'] and len(b) == meta['bytes'], name
    reg = load('SOURCE-REGISTRATION.json')
    extension = load('SOURCE-EXTENSION-REGISTRATION.json')
    receipt = load('READ-RECEIPT.json')
    source = {e['capture'].split('/')[-1]: e for e in reg['entries']}
    seen = {name: set() for name in source}
    for name, e in source.items():
        b = (P / e['capture']).read_bytes()
        assert sha(b) == e['sha256'] and len(b) == e['bytes'] and len(b.splitlines()) == e['eof_lines']
        assert hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest() == e['git_blob']
    for c in receipt['chunks']:
        out = ''
        for r in c['ranges']:
            name, start, end = r['file'], r['start'], r['end']
            e = source[name]
            allowed = list(e['registered_ranges'])
            if name == 'capture.rs':
                allowed.append(extension['additional_range'])
            assert any(lo <= start <= end <= hi for lo, hi in allowed)
            registered = extension['registered_utc'] if c['display'] == 'DISPLAY-SOURCE-H.txt' else reg['registered_utc']
            assert timestamp(registered) <= timestamp(c['saved_utc'])
            lines = (P / e['capture']).read_text().splitlines()
            out += f'### {name}:{start}-{end}\n' + ''.join(f'{i}: {line}\n' for i, line in enumerate(lines, 1) if start <= i <= end)
            seen[name].update(range(start, end + 1))
        assert (P / c['display']).read_text() == out and sha(out.encode()) == c['sha256']
    for r in receipt['source_coverage']:
        n = r['file']
        assert intervals(seen[n]) == r['actual']
        assert intervals(set(range(1, r['eof'] + 1)) - seen[n]) == r['unread']
    assert sum(map(len, seen.values())) == receipt['total_unique_source_lines_personally_displayed']
    reportreg = load('REPORT-REGISTRATION.json')
    report = load('REPORT-PARENT.json')
    assert sha((P / 'REPORT-PARENT.json').read_bytes()) == reportreg['sha256']
    for d in ('REPORT-DISPLAY-A.json', 'REPORT-DISPLAY-B.json', 'REPORT-DISPLAY-C.json'):
        for k, v in load(d).items():
            assert report[k] == v
    for c in load('OTHER-READ-RECEIPT.json')['chronology']:
        r = load(c['registration'])
        assert timestamp(r['registered_utc']) <= timestamp(c['saved_utc'])
    print('PASS packet file set and hashes; source bodies/blobs; registered bounded displays, complements, and recorded chronology; prepared report selections.')
    if a.git_repo:
        for e in reg['entries']:
            assert git(a.git_repo, e['revision'], e['git_path']) == (P / e['capture']).read_bytes()
            tree = subprocess.check_output(['git', '--no-replace-objects', '-C', str(a.git_repo), 'ls-tree', e['revision'], '--', e['git_path']], text=True).strip()
            assert tree.split('\t')[0].split()[2] == e['git_blob']
        historical = load('HISTORY-REGISTRATION.json')
        qb = git(a.git_repo, historical['queue']['revision'], historical['queue']['path'])
        assert sha(qb) == historical['queue']['sha256'] and len(qb) == historical['queue']['bytes']
        queue = json.loads(qb)
        for q in historical['queue']['selected']:
            assert sha(canon(select(queue, q['selector']))) == q['canonical_sha256']
        q2303 = queue['entries'][2303]
        assert all(q2303[k] == v for k, v in load('HISTORY-Q2303-DISPLAY.json').items())
        assert queue['entries'][1214] == load('HISTORY-Q1214-DISPLAY.json')
        ledger = historical['ledger']
        lb = git(a.git_repo, ledger['revision'], ledger['path'])
        assert sha(lb) == ledger['sha256']
        row = select(json.loads(lb), ledger['selector'])
        assert sha(canon(row)) == ledger['selected_canonical_sha256'] and row == load('LEDGER-DISPLAY.json')
        prior = historical['prior_assessment']
        pb = git(a.git_repo, prior['revision'], prior['path'])
        assert sha(pb) == prior['sha256']
        pj = json.loads(pb)
        assert all(pj[k] == v for k, v in load('PRIOR-DISPLAY.json').items())
        for e in load('E275-IDENTITY-CHECK.json')['files']:
            b = git(a.git_repo, e['historical_revision'], e['path'])
            assert sha(b) == e['historical_sha256'] == e['current_sha256'] and e['equal']
            tree = subprocess.check_output(['git', '--no-replace-objects', '-C', str(a.git_repo), 'ls-tree', e['historical_revision'], '--', e['path']], text=True).strip()
            assert tree.split('\t')[0].split()[2] == e['historical_blob'] == e['current_blob']
        print('PASS fixed Git 95e membership, two e275 file identities, original 059 queue selections, committed ledger selection and prior assessment.')
    else:
        print('NOT REPEATED fixed Git membership and original 059 queue / committed ledger / prior assessment comparisons: supply --git-repo.')
    if a.parent_stage:
        stage = a.parent_stage
        assert sha((stage / 'REPORT.json').read_bytes()) == reportreg['sha256']
        for f in load('ARTIFACT-REGISTRATION.json')['files']:
            b = (stage / f['path']).read_bytes()
            assert sha(b) == f['sha256'] and len(b) == f['bytes']
        for e in load('HISTORY-REGISTRATION.json')['historical_captures']:
            b = (stage / e['capture']).read_bytes()
            assert sha(b) == e['sha256'] and len(b) == e['bytes']
        names = [('parallel-l11-20261029/READ-168.json', 0), ('parallel-l11-20261029/READ-169.json', 1), ('segovb/READ-L11-704.json', 0), ('segovb/READ-L11-705.json', 1)]
        keys = ['registration', 'revision', 'path', 'sha256', 'actual_EOF', 'fresh_personal_body_ranges', 'assessed_ranges', 'whole_file_assessed', 'attribution']
        reconstructed = []
        for name, index in names:
            d = json.loads((stage / 'history-exact' / name).read_text())
            row = d['rows'][index]
            reconstructed.append({'record': name, 'kind': d.get('kind'), 'source_revision': d.get('revision', d.get('source_revision')), 'row': {k: row[k] for k in keys if k in row}, 'adopted_registration_mapping': d.get('adopted_registration_mapping'), 'native_runtime_validation': d.get('native_runtime_validation', d.get('runtime_validation')), 'authority': d.get('authority')})
        assert reconstructed == load('HISTORY-READERS-DISPLAY.json')
        adoption = load('HISTORY-ADOPTION-DISPLAY.json')
        proposal = json.loads((stage / 'history-exact/parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-003.json').read_text())['contract_updates'][3]
        assert all(proposal[k] == v for k, v in adoption['proposal_selected'].items())
        for name in ['VERIFY-L11-704.json', 'VERIFY-L11-705.json']:
            old = json.loads((stage / 'history-exact/segovb' / name).read_text())
            assert all(old[k] == v for k, v in adoption[name].items())
        print('PASS parent prepared report, registered metadata, nine captured historical files, and selected historical attribution/adoption reconstruction.')
    else:
        print('NOT REPEATED parent-stage metadata and exact historical capture comparisons: supply --parent-stage.')
    if a.publication:
        tr = load('TRANSPORT-REGISTRATION.json')
        b = (a.publication / tr['path']).read_bytes()
        assert sha(b) == tr['sha256'] and len(b) == tr['bytes']
        values = json.loads(gzip.decompress(b))
        nodes = []
        def walk(x):
            if isinstance(x, dict):
                if 'sha256' in x:
                    nodes.append(x)
                for v in x.values(): walk(v)
            elif isinstance(x, list):
                for v in x: walk(v)
        walk(values)
        for e in load('TRANSPORT-MATCHES.json')['results']:
            matches = [x for x in nodes if x.get('sha256') == e['sha256']]
            entry = next(x for x in load('HISTORY-REGISTRATION.json')['historical_captures'] if x['capture'] == e['historical_capture'])
            matches = [x for x in matches if x.get('bytes') == entry['bytes']]
            assert len(matches) == e['matching_sha_and_bytes'] and sum(str(x.get('path', '')).endswith(e['historical_capture'].removeprefix('history-exact/')) for x in matches) == e['matching_relative_path_suffix']
        print('PASS nine published transport-inventory metadata matches; no independent historical re-extraction claimed.')
    else:
        print('NOT REPEATED published transport-inventory comparison: supply --publication.')
    print('LIMITS: recorded chronology is self-reported/custody metadata; selected text and checksums cannot independently prove personal comprehension, semantic truth, scheduler behavior, actual rendering, or runtime outcomes.')

if __name__ == '__main__':
    main()
