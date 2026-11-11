#!/usr/bin/env python3
"""Verify source, provenance and symbolic admission evidence, never product behavior."""
import difflib
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda data: hashlib.sha256(data).hexdigest()

def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()

def entries(data):
    out, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        out[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return out

def chain(path, record, body, root_tree, nodes):
    assert [s['name'] for s in record['chain']] == path.split('/')
    current = root_tree
    for step in record['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)
    assert record['lines'] == len(body.splitlines())

revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
e275 = 'e2755a732c84e67296b5e3441a4259a84f461b2f'
a = load('ASSESSMENT-TERRAIN-LANDFORM-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1626.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('TERRAIN-LANDFORM-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
raw = contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json'); tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); tree[path] = header.split()[2]
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
x, = h['historical_source_roles']; role = w['roles'][1626]
assert x['original_index'] == 1626 and x['prior_role_status'] == 'unassessed'
assert x['new_personal_read_ranges'] == [[1,260],[261,520],[521,780],[781,1040],[1041,1300],[1301,1509]]
assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1626.json'
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    old = archive.extractfile(x['sha256']).read()
    assert sha(old) == role['historical_sha256'] == x['sha256'] and len(old.splitlines()) == 1509
    c = x['current_comparison']; cur = contents(c['preserved_bytes'])
    assert sha(cur) == c['sha256'] and len(cur) == c['bytes'] and len(cur.splitlines()) == 1506
    assert oid('blob', cur) == c['git_blob'] == tree[c['path']]
    assert c['current_personal_ranges'] == [[1,320]] and c['byte_identical'] is False
    assert old.split(b'#[cfg(test)]\nmod tests {',1)[0] == cur.split(b'#[cfg(test)]\nmod tests {',1)[0]
    expected = ''.join(difflib.unified_diff(old.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='frozen/'+c['path'],tofile='current/'+c['path'])).encode()
    diff = contents(c['exact_diff']['preserved_bytes'])
    assert diff == expected and sha(diff) == c['exact_diff']['sha256'] and len(diff) == c['exact_diff']['bytes']
    chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
    assert [e['original_index'] for e in a['earlier_historical_source_identities']] == list(range(1627,1636))
    for e in a['earlier_historical_source_identities']:
        prior = archive.extractfile(e['sha256']).read(); role = w['roles'][e['original_index']]
        assert sha(prior) == role['historical_sha256'] and e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit'] == 0
        chain(e['source_id'].split(':',1)[1], proof['paths'][e['source_id'].split(':',1)[1]], prior, proof['root_tree'], nodes)
sources = {}
for s in a['current_supporting_sources']:
    body = contents(s['preserved_bytes'])
    assert s['revision'] == revision and sha(body) == s['sha256'] and len(body) == s['bytes'] and len(body.splitlines()) == s['lines']
    assert oid('blob', body) == s['git_blob'] == tree[s['path']]
    assert all(1 <= lo <= hi <= s['lines'] for lo,hi in s['ranges']); sources[s['path']] = body
native = load(a['native_source_evidence']['manifest']); arc = (root / 'native-evidence/bevy_math-0.19.1/bevy_math-0.19.1.crate').read_bytes()
assert sha(arc) == native['sha256'] == '98ead494dcfaa9b4594eed7f09a27d868b7cc7fc9ebd655c5643322778f4a8a2'
assert [50,195] in native['personally_inspected_ranges'][a['native_source_evidence']['member']]
with tarfile.open(root / 'native-evidence/bevy_math-0.19.1/bevy_math-0.19.1.crate','r:gz') as archive:
    d=archive.extractfile(a['native_source_evidence']['member']).read()
    assert sha(d) == next(x['sha256'] for x in native['inspected_members'] if x['path']==a['native_source_evidence']['member'])
    assert b'length.is_finite() && length > 0.0' in d and b'pub fn new(value: Vec2)' in d
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz')); lookup={x['path']:x for x in manifest['entries']}; records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']]
    assert (r['sha256'],r['bytes'],r['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    body=contents(r['preserved_bytes']); assert sha(body)==r['published_sha256'] and len(body)==r['published_bytes']
    assert r['redacted_local_path_occurrences'] or (sha(body)==r['sha256'] and oid('blob',body)==r['git_blob'])
    records[r['original_path']] = body
for n, indices in [(233,list(range(1626,1632))),(234,list(range(1632,1636)))]:
    p=f'adler/READ-L11-{n}.json';z=json.loads(records[p]);verify=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [row['original_index'] for row in z['sources']] == indices
    assert z['accepted_unique_count_change']==0 and z['current_reconciled'] is False and z['validated'] is False
    assert verify['receipt_sha256']==lookup[p]['sha256'] and verify['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources']+z['connected_reinspection']:
        if row.get('registration_sha256'):
            reg='adler/'+row['registration'].replace('\\','/').split('/adler/')[-1]
            assert reg in records and row['registration_sha256']==lookup[reg]['sha256']
assert json.loads(records['adler/READ-L11-233.json'])['sources'][0]['fresh_reads']==[[1,190],[191,380],[381,570],[571,760],[761,950],[951,1140],[1141,1330],[1331,1509]]
for n, indices in [(514,[1626]),(517,[1633,1634,1635])]:
    p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);verify=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
    assert verify['receipt_sha256']==lookup[p]['sha256'] and verify['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for r in z['rows']:
        assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
    for r in z.get('partial_sequence',[]):
        assert r['path'] in records and r['sha256']==lookup[r['path']]['sha256']
assert json.loads(records['segovb/READ-L12-514.json'])['rows'][0]['fresh_personal_body_reads']==[[1,1509]]
eproof=load('E275-LANDFORM-GIT-PROOFS.json'); data=contents(eproof['commit']['preserved_bytes'])
assert oid('commit',data)==e275 and sha(data)==eproof['commit']['sha256'] and len(data)==eproof['commit']['bytes']
assert data.splitlines()[0] == b'tree '+eproof['root_tree'].encode()
for n in [70,71,72,79,80,81,82,83]:
    p=f'root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-{n:03}.json'; z=json.loads(records[p]); v=f'root-l1-current-e275-20261029/VERIFY-ROOT-L1-CURRENT-{n:03}.json'; check=json.loads(records[v]); state=f'segovb/STATE-L1-CURRENT-{n:03}.json'
    assert z['accepted_unique_count_change']==0 and z['other_lens_credit']==0 and z['implementation_gate_complete'] is False
    assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[state]['sha256']
    if 'registration' in z: assert z['registration'] in records and z['registration_sha256']==lookup[z['registration']]['sha256']
    for r in z.get('registrations',[])+z.get('partial_recovery_chain',[]):
        assert r['path'] in records and r['sha256']==lookup[r['path']]['sha256']
    if 'assessment' in z: assert z['assessment']['path'] in records and z['assessment']['sha256']==lookup[z['assessment']['path']]['sha256']
    for r in z['rows']:
        op=r['source_object'].replace('\\','/').split('726-audit-adler-20261027/',1)[1]; body=records[op]
        assert sha(body)==r['sha256'] and len(body.splitlines())==r['actual_EOF']
        if r['source_id'].startswith('current-main:'):
            assert r['revision']==e275
            chain(r['git_path'],eproof['paths'][r['git_path']],body,eproof['root_tree'],eproof['additional_tree_objects'])
    for r in z.get('qualified_own_reuse',[]):
        ref=r['receipt'];assert ref['path'] in records and ref['sha256']==lookup[ref['path']]['sha256']
        reused=json.loads(records[ref['path']])['rows'][int(r['row_pointer'].split('/')[-1])]
        assert reused==r['source_row_as_recorded']
prior=json.loads(records['root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-082.json'])['rows'][4]
assert prior['actual_EOF']==336 and prior['fresh_personal_body_ranges']==[[1,217]]
assert len(a['selected_e275_to_95e_exact_source_identity'])==6
for r in a['selected_e275_to_95e_exact_source_identity']:
    assert r['byte_identical'] and r['e275_git_blob']==eproof['paths'][r['path']]['git_blob']==r['current_git_blob']==tree[r['path']]
client=sources['crates/adventuresim-tactical-client/src/presentation/terrain/volumetric.rs']
recipe=sources['crates/adventuresim-tactical-core/src/volumetric_terrain/recipe.rs']
fault=sources['crates/adventuresim-tactical-core/src/scene_fault.rs']
validation=sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
generation=sources['crates/adventuresim-tactical-core/src/scene_input/generation.rs']
net=sources['crates/adventuresim-tactical-netcode/src/replication.rs']
surface=sources['crates/adventuresim-tactical-core/src/volumetric_terrain/surface_recipe.rs']
assert b'terrain_landform_patch(terrain, *recipe).ok()' in client
assert b'if let Some(patch) = fault_patch' in client and b'&& let Some(recipe) = landform' in client and b'coverage.mesh(terrain, landform)' in client
assert b'.remove::<PendingTerrainPresentation>()' in cur and b'startup.mark_terrain_prepared_once()' in cur
assert b'crate::scene_fault::validate(self.landform, &self.playable)?' in validation
assert b'crate::scene_fault::generate(self.landform, &terrain)?' in generation and b'.transpose()' in fault and b'SceneValidationError::Terrain(reason)' in fault
assert b'.replicate::<SceneTerrain>()' in net and b'.replicate_once::<TerrainLandformRecipe>()' in net and b'replicate::<SceneTerrainPatch>()' not in net and b'replicate_once::<SceneTerrainPatch>()' not in net
assert b'SedimentaryRock::Sandstone => Self::Sandstone' in surface and b'0.70, 0.90, 0.42, 0.032' in surface
assert b'if origin.x.abs() > half.x + extent.x' in recipe and b'TerrainRecipeError::OutsidePlayable' in recipe
# Symbolic integer/rational admission bounds; no constructors or scene run.
assert 100 <= 600 <= 2000 and 400 <= 1200 <= 5000 and 300 <= 1000 <= 2000
assert 250 >= 100 and 250 < 1200 and 250*2 < 1000
assert 100 > 0.5+12 and 0.18 <= 0.42 <= 4 and 0.004 <= 0.032 <= 0.055 and 0.55 <= 0.70 <= 0.90 <= 1
q=load('RECONCILIATION-059.json')['entries']; assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (86,1348))
baseline=load('BASELINE-95E-20261110.json'); ledger=baseline['newly_assessed_ledger']['entries']; item,=a['conditional_findings']
assert item['id']=='ADLER-L11-LANDFORM-PREPARATION-CAUSE-DISCARD' and item['accepted_historical_unique_count_change']==0
assert sum(r['id']==item['id'] and r['record']=='ASSESSMENT-TERRAIN-LANDFORM-95E.json' for r in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified historical terrain role, earlier receipts, pinned Git paths and symbolic landform bounds; no product execution.')
