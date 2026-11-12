#!/usr/bin/env python3
"""Verify bounded qualifications in frozen evidence, not product behavior."""
import gzip
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
cp = load('DELEGATED-EVIDENCE-CHECKPOINT-29.json')
bundles = {b['bundle_id']: b for b in cp['bundles']}
registry = json.loads(gzip.decompress((root / 'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']

def packet(name):
    return tarfile.open(root / bundles[name]['archive'])

def item(t, path):
    return json.load(t.extractfile(path))

def rows(t, path):
    return [json.loads(x) for x in gzip.decompress(t.extractfile(path).read()).splitlines()]

def supplier_pair(name, supplier, role_ids):
    bundle = bundles[name]
    members = {m['path']: m for m in bundle['members']}
    prefix = 'history/774/' + supplier + '-owner-'
    red, old = prefix + 'redacted.jsonl.gz', prefix + 'original-paths.jsonl.gz'
    assert not members[red]['changed'] and members[old]['changed']
    assert members[red]['original_uncompressed_sha256'] != members[old]['original_uncompressed_sha256']
    with packet(name) as t:
        a, z = rows(t, red), rows(t, old)
        for i in role_ids:
            ref = registry[i]['owner_contract_proofs'][0]
            assert ref['supplier'] == supplier
            x, y = a[ref['row']], z[ref['row']]
            keys = {'path'} if supplier == 'rig' else {'path', 'native_source_path'}
            assert all(x.get(k) != y.get(k) for k in keys)
            assert {k: v for k, v in x.items() if k not in keys} == {k: v for k, v in y.items() if k not in keys}
            assert x['source_id'] == registry[i]['source_id']
            assert x['observed_sha256'] == registry[i]['sha256'] and x['read_ranges'] == ref['read_ranges']
            if supplier == 'rig':
                assert x['contract_refs'] == ref['contract_refs']
            else:
                assert ref['read_status'] is None and x['source_read_status'] and x['trace_refs'] == ref['contract_refs']

supplier_pair('wgpu-native-0-27-30', 'rig', [0, 27, 28, 29, 30])
supplier_pair('bevy-observer-3-8', 'rig', [4, 5])
supplier_pair('bevy-observer-3-8', 'native', [3, 6, 7, 8])
supplier_pair('stdb-native-20-24', 'native', [20, 21, 22])
with packet('direction-parry-15-16') as t:
    paths = ['history/774/camera-producer-redacted.jsonl.gz', 'history/774/camera-producer-original-paths.jsonl.gz']
    a, z = (next(x for x in rows(t, p) if x.get('trace_id') == 'CAM-PPS-CAMERA-BOUNDARIES-048') for p in paths)
    assert {k for k in a if a[k] != z[k]} == {'current_sources', 'additional_actual_source_proofs'}
    for role, ix in [(15, 7), (16, 10)]:
        x, y = a['additional_actual_source_proofs'][ix], z['additional_actual_source_proofs'][ix]
        assert x['path'] != y['path']
        assert {k: v for k, v in x.items() if k != 'path'} == {k: v for k, v in y.items() if k != 'path'}
        assert x['sha256'] == registry[role]['sha256']
        assert x['read_ranges'] == registry[role]['expanded_producer_contract_observations'][0]['read_ranges_as_recorded']
with packet('bevy-observer-3-8') as t:
    reg = next(x for x in item(t, 'registration.json')['entries'] if x['role'] == 4)
    rec = next(x for x in item(t, 'read-receipt.json')['entries'] if x['role'] == 4)
    assert reg['original_registered_ranges'] == [[1, 159]] and reg['eof'] == 129
    assert reg['nonexistent_original_tail'] == rec['nonexistent_original_tail'] == [[130, 159]]
    assert rec['actual_display_chunks'] == [[1, 129]]
with packet('web-routes-2370-2371') as t:
    reg = next(x for x in item(t, 'support3-registration.json')['entries'] if x['tag'] == 'site-context')
    rec = next(x for x in item(t, 'support3-read-receipt.json')['entries'] if x['source'] == 'support/site-context.rs')
    assert reg['registered_ranges'] == [[1, 80]] and reg['EOF'] == 40
    assert rec['displayed_chunks'] == [[1, 40]] and rec['registered_nonexistent_ranges'] == [[41, 80]]
with packet('p16-combat-config-40') as t:
    report, manifest, q = (item(t, p) for p in ('REPORT.json', 'MANIFEST.json', 'prior-reader-qualification.json'))
    assert 'unrecovered whole-file registration identity' in manifest['qualification']
    assert q['earlier_full_read']['revision'] == 'e2755a732c84e67296b5e3441a4259a84f461b2f'
    assert q['earlier_full_read']['actual_ranges'] == [[1, 346]]
    assert report['reader'] == '/root/hair_ops' and 'do not transfer' in q['chronology']
with packet('roof-historical-2496-2497') as t:
    rec = item(t, 'read-receipt.json')
    assert 'zero historical displays' in rec['qualification']
    assert {(x['original_index'], x['kind']) for x in rec['reads']} == {(i, k) for i in (2496, 2497) for k in ('historical recovered pr-working object', 'pinned current source')}
    assert all(x['unread_complement'] == [] for x in rec['reads'])
with packet('p16-grounding-config-2524-2525') as t:
    reg = item(t, 'registration.json')
    a = [x for x in reg['entries'] if x['role'] == 2524]
    assert len(a) == 2 and all(x['registered_ranges'] == [[55, 115]] and x['original_material_ranges'] == [[73, 86], [104, 110]] for x in a)
    assert all(x['source_id'].startswith('physical-p16:') for x in a)
q = load('QUALIFICATION-REGISTRATION-HASH-MISMATCHES-29.json')
assert q['roles'] == q['entries'] == q['equal_whole_file_entries'] == q['unproved_in_packet'] == []
assert 'scope_registration' in q['scope_qualification'] and 'do not establish' in q['scope_qualification']
print('Verified selected supplier variant rows, bounded EOF corrections, distinct readers and late reads; no whole-file identity, later checkpoint or extra read credit inferred.')
