#!/usr/bin/env python3
"""Verify vista/view evidence and symbolic arithmetic without product execution."""
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
name = 'ASSESSMENT-VISTA-DIAMETER-AND-REVIEW-VIEWS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1731-1741.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('VISTA-METRIC-HISTORICAL-GIT-PROOFS.json')
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
lengths = {1731:83,1732:94,1733:164,1734:48,1735:304,1736:83,1737:84,1738:312,1739:149,1740:96,1741:98}
diffs = {1731,1739,1740}
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
        assert c['byte_identical'] == (old == body) == (i not in diffs)
        assert c['current_personal_ranges'] == ([[1,n]] if i in diffs else [])
        assert c['current_content_read_via_identical_historical_ranges'] == ([] if i in diffs else [[1,n]])
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        if i in diffs:
            assert old.replace(b'use crate::camera::authored_metres;',b'use crate::camera::config::authored_metres;',1) == body
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
cases = [('gable-review',1733,'GABLE_REVIEW_VIEWS',15),('garden-review',1734,'GARDEN_REVIEW_VIEWS',4),('heating-review',1735,'HEATING_REVIEW_VIEWS',29),('parish-review',1738,'PARISH_REVIEW_VIEWS',37)]
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
assert b'species == adventuresim_plant_generator::PlantSpecies::Fungus(requested)' in current[1740]
assert b'_ => true' in current[1740] and b'FungusSpecies::FlyAgaric' in current[1731]
assert b'views[index].suppress_leaves = true;' in current[1739] and b'views[index].vista_visible = false;' in current[1739]
vista = current[1741]
assert b'diameter.max(f32::from(lod.width.saturating_sub(1)) * lod.spacing_metres)' in vista
assert b'let center_z = f32::from(lod.depth.saturating_sub(1)) * 0.5' in vista
assert b'let Some(extrema) = extrema else' in vista and b'return Ok(None);' in vista
source = sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'width < 2 || depth < 2 || width > max_side || depth > max_side' in source
assert b'grid.heights_metres.len() != expected || grid.environment.len() != expected' in source
assert b'self.validate_source()?;' in source and b'self.validate_grounding()' in source
validation = sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
assert b'validate_grid(&grid, u16::MAX as usize, SampleGridKind::Vista)?' in validation
assert b'lod.spacing_metres <= previous_spacing' in validation and b'vista_samples > MAX_VISTA_SAMPLES' in validation
bounds = sources['crates/adventuresim-tactical-client/src/tactical_scene_viewer/vista_validation.rs']
assert b'Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))' in bounds
assert b'lod.origin_east_metres as f32' in bounds and b'lod.origin_north_metres as f32' in bounds
producer = sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs']
assert b'dimensions: GridDimensions::square(spec.side)' in producer and b'width: spec.side' in producer and b'depth: spec.side' in producer
fixtures = sources['crates/adventuresim-tactical-core/src/bin/generate-scene-fixtures.rs']
assert b'grid(side, side, spacing, terrain, environment)' in fixtures
assert (3-1)*100 == 200 and (9-1)*100 == 800 and 3*9 == 27
viewer = load(a['earlier_current_viewer'])['current_comparison']
manifest = load(a['earlier_current_manifest'])['historical_source_roles'][2]['current_comparison']
assert b'diameter_metres: state.vista_geometry.map(|geometry| geometry.diameter())' in contents(viewer['preserved_bytes'])
assert b'pub(super) diameter_metres: Option<SignedLength>' in contents(manifest['preserved_bytes'])

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
for n, indices in [(558,[1731,1732,1733,1734]), (559,[1735,1736,1737]), (560,[1738,1739,1740,1741])]:
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
    if n == 560:
        row = next(x for x in receipt['rows'] if x['original_index'] == 1741)
        assert 'Diametercomputedwidth-1*spacing39 only' in json.dumps(row['function_contract'])
finding = a['conditional_findings'][0]
assert finding['id'] == 'REVIEW-VISTA-WIDTH-DIAMETER-001'
assert finding['accepted_historical_unique_count_change'] == 0 and finding['repair'] == 'not started' and finding['validation'] == 'not performed'
q = load('RECONCILIATION-059.json')['entries']
for i in (1322,1431,1436,1437,1438):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
baseline = load('BASELINE-95E-20261110.json')
ledger = baseline['newly_assessed_ledger']['entries']
assert sum(item['id'] == finding['id'] and item['record'] == name for item in ledger) == 1
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['new_current_ledger_candidates'] == len(ledger) and progress['lenses_complete'] == 0 and progress['queue_records_semantically_reconciled'] == 0
assert load(baseline['latest_main_recheck'])['observed_revision'] == rev
print('Verified vista/view roles, original receipts and conditional width metric; no product execution.')
