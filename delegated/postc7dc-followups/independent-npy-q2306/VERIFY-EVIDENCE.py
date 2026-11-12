#!/usr/bin/env python3
"""Verify registered evidence bytes; never execute product or fixture code."""
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

def digest(b):
    return hashlib.sha256(b).hexdigest()

def timestamp(s):
    return dt.datetime.fromisoformat(s)

def ranges(ns):
    out = []
    for n in sorted(ns):
        if not out or out[-1][1] + 1 != n:
            out.append([n, n])
        else:
            out[-1][1] = n
    return out

def git(repo, *args):
    return subprocess.check_output(['git', '--no-replace-objects', '-C', str(repo), *args])

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--git-repo', type=Path)
    parser.add_argument('--parent-stage', type=Path)
    parser.add_argument('--publication', type=Path)
    args = parser.parse_args()
    manifest = load('MANIFEST.json')
    actual = {str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x.name != 'MANIFEST.json'}
    assert actual == set(manifest['files']), 'packet file set differs'
    for name, meta in manifest['files'].items():
        path = Path(name)
        assert not path.is_absolute() and '..' not in path.parts and not (P / path).is_symlink()
        b = (P / path).read_bytes()
        assert digest(b) == meta['sha256'] and len(b) == meta['bytes'], name
    r = load('SOURCE-REGISTRATION.json')
    source = {e['capture'].split('/')[-1]: e for e in r['entries']}
    seen = {n: set() for n in source}
    for e in r['entries']:
        b = (P / e['capture']).read_bytes()
        assert digest(b) == e['sha256'] and len(b) == e['bytes'] and len(b.splitlines()) == e['eof']
        assert hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest() == e['blob']
    receipt = load('SOURCE-READ-RECEIPT.json')
    for c in receipt['chunks']:
        assert timestamp(r['registered_utc']) <= timestamp(c['saved_utc'])
        display = ''
        for section in c['ranges']:
            name, start, end = section['file'], section['start'], section['end']
            e = source[name]
            assert any(lo <= start <= end <= hi for lo, hi in e['planned_ranges'])
            lines = (P / e['capture']).read_text().splitlines()
            display += f'### {name}:{start}-{end}\n' + ''.join(f'{i}: {line}\n' for i, line in enumerate(lines, 1) if start <= i <= end)
            seen[name].update(range(start, end + 1))
        assert (P / c['display']).read_text() == display and digest(display.encode()) == c['sha256']
    for c in receipt['coverage']:
        name = c['file']
        assert ranges(seen[name]) == c['actual']
        assert ranges(set(range(1, c['eof'] + 1)) - seen[name]) == c['unread']
    assert sum(map(len, seen.values())) == receipt['total_unique_displayed_lines_across_registered_bodies']
    for c in load('OTHER-READ-RECEIPT.json')['display_chronology']:
        assert timestamp(load(c['registration'])['registered_utc']) <= timestamp(c['saved_utc'])
    rr = load('REPORT-REGISTRATION.json')
    assert digest((P / 'REPORT-PARENT.json').read_bytes()) == rr['sha256']
    report = load('REPORT-PARENT.json')
    for name in ['REPORT-DISPLAY-A.json', 'REPORT-DISPLAY-B.json', 'REPORT-DISPLAY-C.json']:
        assert all(report[k] == v for k, v in load(name).items())
    q = (P / 'Q2306-EXACT-SLICE.json').read_bytes()
    h = load('HISTORY-REGISTRATION.json')
    for stream in h['queue_streams']:
        assert digest(q) == stream['entry_sha256'] and len(q) == stream['entry_bytes']
    entry = json.loads(q)
    assert all(entry[k] == v for k, v in load('HISTORY-Q2306-DISPLAY.json').items())
    print('PASS packet file set and hashes; Git blob identities; registered source displays/complements and recorded chronology; prepared report and exact selected-entry subset.')
    if args.git_repo:
        for e in r['entries']:
            b = git(args.git_repo, 'show', f"{e['revision']}:{e['path']}")
            assert b == (P / e['capture']).read_bytes()
            line = git(args.git_repo, 'ls-tree', e['revision'], '--', e['path']).decode().strip()
            assert line.split('\t')[0].split()[2] == e['blob']
            if 'historical_same_blob_revision' in e:
                assert git(args.git_repo, 'show', f"{e['historical_same_blob_revision']}:{e['path']}") == b
                oldline = git(args.git_repo, 'ls-tree', e['historical_same_blob_revision'], '--', e['path']).decode().strip()
                assert oldline.split('\t')[0].split()[2] == e['historical_blob'] == e['blob']
        t = load('TREE-DISPLAY.json')
        for rev, paths in t['trees'].items():
            for path, tree in paths.items():
                expr = f'{rev}^{{tree}}' if path == 'root' else f'{rev}:{path}'
                assert git(args.git_repo, 'rev-parse', expr).decode().strip() == tree
        revisions = load('TREE-REGISTRATION.json')['revisions']
        diff = git(args.git_repo, 'diff-tree', '-r', '--no-commit-id', '--name-status', *revisions, '--', 'crates/fabelgeist-numpy-storage').decode().splitlines()
        assert diff == t['crate_diff_name_status']
        for stream in h['queue_streams']:
            b = git(args.git_repo, 'show', f"{stream['revision']}:{stream['path']}")
            assert digest(b) == stream['sha256'] and len(b) == stream['bytes']
            assert b[stream['entry_byte_start']:stream['entry_byte_end_exclusive']] == q
            assert json.loads(b)['entries'][2306] == entry
            line = git(args.git_repo, 'ls-tree', stream['revision'], '--', stream['path']).decode().strip()
            assert line.split('\t')[0].split()[2] == stream['blob']
        print('PASS fixed Git source/manifests, seven e275/95e Rust blob identities, both tree comparisons and both distinct 059 queue stream/slice comparisons.')
    else:
        print('NOT REPEATED Git commit membership, subtree comparison or both original/reconstructed queue stream comparisons: supply --git-repo.')
    if args.parent_stage:
        stage = args.parent_stage
        assert digest((stage / 'REPORT.json').read_bytes()) == rr['sha256']
        for item in load('METADATA-REGISTRATION.json')['files']:
            b = (stage / item['path']).read_bytes()
            assert digest(b) == item['sha256'] and len(b) == item['bytes']
        for e in r['entries']:
            if 'historical_same_blob_revision' in e:
                assert (stage / 'sources' / e['path']).read_bytes() == (P / e['capture']).read_bytes()
        for e in h['historical_captures']:
            b = (stage / e['retained_parent_path']).read_bytes()
            assert digest(b) == e['sha256'] and len(b) == e['bytes']
        assert (stage / 'HISTORY-Q2306-EXACT.json').read_bytes() == q
        assert (stage / 'HISTORY-Q2306-ORIGINAL-EXACT.json').read_bytes() == q
        root = stage / 'historical-originals'
        specs = [('parallel-l11-20261029/READ-170.json', [0, 1, 2, 3]), ('parallel-l11-20261029/READ-011.json', [6, 7, 8])]
        fields = ['registration', 'revision', 'path', 'sha256', 'actual_EOF', 'fresh_personal_body_ranges', 'assessed_ranges', 'whole_file_assessed', 'attribution']
        readers = []
        for name, indexes in specs:
            d = json.loads((root / name).read_text())
            readers.append({'record': name, 'revision': d['revision'], 'rows': [{k: d['rows'][i][k] for k in fields if k in d['rows'][i]} for i in indexes], 'native_runtime_validation': d.get('native_runtime_validation'), 'implementation_gate_complete': d.get('implementation_gate_complete')})
        assert readers == load('HISTORY-READERS-DISPLAY.json')
        adopted = []
        for e in load('HISTORY-ADOPTION-EXT-REGISTRATION.json')['entries']:
            d = json.loads((stage / e['capture']).read_text())
            adopted.append({'capture': e['capture'], 'top_level_keys': list(d), 'adopted_registration_mapping': d.get('adopted_registration_mapping'), 'kind': d.get('kind'), 'at': d.get('at')})
        assert adopted == load('HISTORY-ADOPTION-METADATA.json')
        dispositions = []
        for name in ['READ-L11-706.json', 'READ-L11-547.json']:
            d = json.loads((root / 'segovb' / name).read_text())
            dispositions.append({'record': name, 'source_revision': d.get('source_revision'), 'actual_current_target': d.get('actual_current_target'), 'new_target_reconciliation_complete': d.get('new_target_reconciliation_complete'), 'accepted_unique_count_change': d.get('accepted_unique_count_change'), 'disposition_0': d['finding_connections_and_scope_dispositions'][0]})
        assert dispositions == load('HISTORY-DISPOSITIONS-DISPLAY.json')
        print('PASS parent report/metadata, seven captured Rust bodies, both exact entry slices, nine historical captures, and attribution/adoption/disposition reconstructions.')
    else:
        print('NOT REPEATED parent prepared artifact and historical-capture attribution comparisons: supply --parent-stage.')
    if args.publication:
        tr = load('TRANSPORT-REGISTRATION.json')
        b = (args.publication / tr['path']).read_bytes()
        assert digest(b) == tr['sha256'] and len(b) == tr['bytes']
        nodes = []
        def walk(x):
            if isinstance(x, dict):
                if 'sha256' in x: nodes.append(x)
                for y in x.values(): walk(y)
            elif isinstance(x, list):
                for y in x: walk(y)
        walk(json.loads(gzip.decompress(b)))
        for row in load('TRANSPORT-MATCHES.json')['rows']:
            e = next(e for e in h['historical_captures'] if e['original_path'] == row['original_path'])
            matches = [n for n in nodes if n.get('sha256') == e['sha256'] and n.get('bytes') == e['bytes']]
            assert len(matches) == row['digest_and_size_matches'] and sum(str(n.get('path', '')).endswith(e['original_path']) for n in matches) == row['matching_path_suffix']
        print('PASS published transport-inventory metadata matches; this is not independent re-extraction of historical originals.')
    else:
        print('NOT REPEATED published transport-inventory metadata comparison: supply --publication.')
    print('LIMITS: recorded chronology is custody metadata; checksums do not prove personal comprehension, semantic correctness, native behavior, whole-tree equivalence, or runtime results.')

if __name__ == '__main__':
    main()
