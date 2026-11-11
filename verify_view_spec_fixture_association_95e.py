#!/usr/bin/env python3
"""Verify historical roles and static fixture associations without product execution."""
import gzip
import hashlib
import json
import re
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
def entries(data):
    result, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']

rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-VIEW-SPEC-FIXTURE-ASSOCIATION-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1725-1730.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('VIEW-SPEC-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1725:1101,1726:677,1727:69,1728:76,1729:133,1730:323}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] == (old == body) == (i != 1725)
        assert c['current_personal_ranges'] == ([[1,n]] if i == 1725 else [])
        assert c['current_content_read_via_identical_historical_ranges'] == ([] if i == 1725 else [[1,n]])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        if i == 1725:
            oldlines, newlines = old.splitlines(), body.splitlines()
            assert oldlines[0] == b'use crate::camera::authored_metres;'
            assert newlines[0] == b'use crate::camera::config::authored_metres;'
            assert oldlines[1:991] == newlines[1:991] and oldlines[996:] == newlines[996:]
            assert all(line.startswith(b'//') for line in oldlines[991:996]+newlines[991:996])
            assert (root / c['diff']).read_bytes().startswith(b'diff --git ')
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
mapping = load(a['static_fixture_mapping'])
cases = [('shop-sign-review',1726,'SHOP_REVIEW_VIEWS',18),('workplace-review',1726,'WORKPLACE_REVIEW_VIEWS',62),('compound-review',1728,'COMPOUND_REVIEW_VIEWS',7),('facade-review',1730,'FACADE_REVIEW_VIEWS',31)]
assert mapping['source_revision'] == rev and len(mapping['profiles']) == len(cases)
for m, (profile,i,key,count) in zip(mapping['profiles'],cases):
    text = current[i].decode()
    text = text[text.index('const '+key):].split('];',1)[0]
    rows = re.findall(r'CaptureViewSpec::new\(\s*"([^"]+)",\s*"[^"]+",\s*CapturePose::CityExterior\s*\{\s*camera:\s*(\d+)\s*\}',text)
    assert rows[0][0] == 'warmup'
    rows = rows[1:]
    fixturepath = 'assets/tactical-scenes/'+profile+'.review.json'
    fixture = json.loads(sources[fixturepath])['views']
    assert (m['profile'],m['source_role'],m['fixture_path'],m['count']) == (profile,i,fixturepath,count)
    assert len(rows) == len(fixture) == len(m['rows']) == count
    for index, ((slug,camera), f, row) in enumerate(zip(rows,fixture,m['rows'])):
        assert row == {'position':index,'spec_slug':slug,'camera_index':int(camera),'fixture_slug':f['slug']}
        assert row['camera_index'] == index and row['spec_slug'] == row['fixture_slug']
setup = sources['crates/adventuresim-tactical-client/src/tactical_scene_viewer/building_review.rs']
assert b'fixture.views.len()' in setup and b'specs.len() - 1' in setup
assert b'for (view, spec) in fixture.views.iter().zip(&specs[1..])' in setup and b'assert_eq!(view.slug, spec.slug)' in setup
assert b'.map(|view| view.camera(buildings, &fixture.signs))' in setup
camera = load(a['earlier_current_camera'])['historical_source_roles'][2]['current_comparison']
cb = contents(camera['preserved_bytes'])
assert b'&state.city_exterior_cameras' in cb and b'.get(usize::from(camera))' in cb
parent = current[1725]
assert b'pub understory_species: Option<&\'static str>' in parent
assert b'pub const fn understory(mut self, common_name: &\'static str)' in parent
assert all(b'.understory("'+v+b'")' in parent for v in (b'common hazel',b'blackthorn',b'common hawthorn'))
assert parent.count(b'traversal_view("tree-cold-first-') == 16 and parent.count(b'traversal_view("tree-retreat-') == 8 and parent.count(b'traversal_view("tree-warm-second-') == 16
resident = sources['crates/adventuresim-tactical-client/src/presentation/obstacles/tree/presentation.rs']
assert b'ensure_tree_assets_resident(\n            cached,\n            EVERY_TREE_TIER,\n            None,' in resident
manifest = load(a['earlier_current_manifest'])['historical_source_roles'][2]['current_comparison']
mb = contents(manifest['preserved_bytes'])
assert b'cold.canopy_pixel_bps >= 5' in mb and b'u32::from(warm.canopy_pixel_bps) * 4' in mb
prior = load('ASSESSMENT-PLANT-WORK-AND-SPECIES-95E.json')
identity = next(item for item in prior['earlier_historical_source_identities'] if item['original_index']==1725)
assert identity['new_personal_full_read'] is False and identity['additional_original_frontier_credit'] == 0
oldreceipt = next(item for item in prior['recovered_historical_records'] if item['original_path']=='segovb/READ-L12-555.json')
assert [x['original_index'] for x in json.loads(contents(oldreceipt['preserved_bytes']))['rows']] == [1725]

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(556,[1726]), (557,[1727,1728,1729,1730])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for x in receipt.get('partial_sequence', []):
        assert x['path'] in records and x['sha256'] == lookup[x['path']]['sha256']
    for x in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
        assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
assert a['new_unique_findings'] == []
q = load('RECONCILIATION-059.json')['entries']
for i in (82,258,1433,1434,1435):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert q[1432]['current_reconciliation'].startswith('partial ') and 'ASSESSMENT-PLANT-LOD-ROOT-ASSOCIATION-95E.json' in q[1432]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified view-spec roles, original receipts and static fixture associations; no product execution.')
