#!/usr/bin/env python3
"""Verify garden source provenance and recorded evidence, without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, body):
    return hashlib.sha1(kind.encode() + b' ' + str(len(body)).encode() + b'\0' + body).hexdigest()
def entries(body):
    result, pos = {}, 0
    while pos < len(body):
        end = body.index(b'\0',pos); mode, name = body[pos:end].split(b' ',1)
        result[name.decode()] = (mode.decode(),body[end+1:end+21].hex()); pos = end+21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree',raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'],part['child']); current = part['child']
    assert current == proof['git_blob'] == oid('blob',body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'; old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-GARDEN-ADMISSION-95E.json'; hist = 'ASSESSMENT-L01-HISTORICAL-1798-1802.json'
a,h,w = load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence']); shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']: nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit) == proof['revision'] == old and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json'); tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header,path = line.split('\t',1); tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1798:250,1799:152,1800:97,1801:270,1802:61}; assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = w['roles'][i]; before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(before.splitlines()) == n and sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == []
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob',body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes); current[i] = body
    item = a['historical_asset_527']; oldasset = archive.extractfile(item['sha256']).read()
    assert sha(oldasset) == w['roles'][527]['historical_sha256'] == item['sha256'] and len(oldasset.splitlines()) == item['lines'] == 77
    check_chain(item['path'],proof['paths'][item['path']],oldasset,proof['root_tree'],nodes)
    assert item['earlier_lens1_record'] == w['roles'][527]['historical_assessment']
    diff = contents(item['current_exact_diff']['preserved_bytes'])
    assert sha(diff) == item['current_exact_diff']['sha256'] and len(diff) == item['current_exact_diff']['bytes']
    assert b'diff --git ' in diff and b'@@ ' in diff
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body) == item['git_blob'] == tree[item['path']]; sources[item['path']] = body
newasset = sources['content/tactical/garden-hazel-envelope.json']; assert oldasset != newasset
oldjson,newjson = json.loads(oldasset),json.loads(newasset)
assert (oldjson['geometry_revision'],len(oldjson['hull_metres']),oldjson['vertex_count']) == (1,17,97397)
assert (newjson['geometry_revision'],len(newjson['hull_metres']),newjson['vertex_count']) == (4,18,97029)
assert oldjson['seed'] == newjson['seed'] and newjson['hull_metres'][0][0] < -1.5
assert b'if value.is_finite() && value > 0.0' in current[1800] and b'*point * self.scale.value()' in current[1800]
geometry = current[1801]
assert geometry.index(b'self.validate_bounds()?;') < geometry.index(b'self.validate_access(streets)?;') < geometry.index(b'self.validate_plants()')
assert b'|| self.access.is_empty()' in geometry and b'plant.scale.value() > MAX_PLANT_SCALE' in geometry
assert b'const MAX_PLANT_SCALE: f32 = 2.0;' in geometry
scene = sources['crates/adventuresim-tactical-core/src/scene_input/gardens.rs']
assert b'pub fn from_support(' in scene and b'if plant_support.len() != garden.plants.len()' in scene
projection = sources['crates/adventuresim-tactical-core/src/scene_input/gardens/support.rs']
assert b'This binds roots only' in projection and b'Self::from_support(garden, plant_support)' in projection
assert b'self.validate()?;' in sources['crates/adventuresim-tactical-core/src/scene_input/generation/supported.rs']
assert b'urban::validate(self)?;' in sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
assert b'gardens::validate(input)?;' in sources['crates/adventuresim-tactical-core/src/scene_input/urban.rs']
assert b'.with_scale(Vec3::splat(plant.scale.value()))' in sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/gardens.rs']
assert b'GardenSpecimen::CommonHazel.envelope().seed' in sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/specimens.rs']
assert b'let mut compiled = self.compile_properties(seed)?;' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled.rs']
assert b'compiled.finalize_packing(&context)?;' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled.rs']
transport = json.loads(contents('recovered/original-transport-manifest.json.gz')); lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']: assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n,indices in [(586,[1798,1799,1800]),(587,[1801,1802])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
reuse = json.loads(records['segovb/READ-L12-587.json'])['qualified_supporting_same_lens_reuse'][0]
assert reuse['original_index'] == 1800 and reuse['fresh_personal_body_reads_this_batch'] == []
assert reuse['receipt_sha256'] == lookup[reuse['receipt']]['sha256'] and reuse['actual_EOF'] == 97
qual = json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1800-EOF-001.json'])
assert qual['actual_EOF'] == 97 and qual['body_coverage'] == 0 and len(qual['exact_metadata_range_qualifications']) == 5
assert all(x['effective_existing_requested_body'] == [[1,97]] and x['nonexistent_tail_coverage'] == 0 for x in qual['exact_metadata_range_qualifications'])
q = load('RECONCILIATION-059.json')['entries']
for i in (1478,1479,1480,1481): assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
finding = a['conditional_findings'][0]
assert finding['id'] == 'Q-L12-24d53e6298002ffe684f' and finding['accepted_historical_unique_count_change'] == 0
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
print('Verified five historical bodies, separate envelope revisions and scoped receipts; no product execution.')
