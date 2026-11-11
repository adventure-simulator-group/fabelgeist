#!/usr/bin/env python3
"""Verify regional surface source and distinct 059/774 provenance only."""
import difflib
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda value: hashlib.sha256(value).hexdigest()

def oid(kind, value):
    return hashlib.sha1(kind.encode() + b' ' + str(len(value)).encode() + b'\0' + value).hexdigest()

def entries(value):
    out, pos = {}, 0
    while pos < len(value):
        end = value.index(b'\0', pos)
        mode, name = value[pos:end].split(b' ', 1)
        out[name.decode()] = (mode.decode(), value[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(value)
    return out

def check_chain(path, record, body, root_tree, nodes):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = root_tree
    for step in record['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
a = load('ASSESSMENT-REGIONAL-MAP-SURFACE-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1619-1622.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('REGIONAL-SURFACE-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json'); tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); mode, kind, hash_ = header.split(); tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']; raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']] == [1619,1620,1621,1622]
assert [x['lines'] for x in h['historical_source_roles']] == [133,74,231,231]
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']; role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1619-1622.json'
        assert x['prior_role_status'] == 'unassessed' and x['new_personal_read_ranges'] == [[1,x['lines']]]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']; body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] == (original == body) == (i != 1621) and c['current_personal_ranges'] == [[1,c['lines']]]
        check_chain(c['path'], proof['paths'][c['path']], original, proof['root_tree'], nodes)
        if i == 1621:
            diff = ''.join(difflib.unified_diff(original.decode().splitlines(keepends=True), body.decode().splitlines(keepends=True), fromfile='frozen/'+c['path'], tofile='current/'+c['path'])).encode()
            d = c['exact_diff']; assert diff == contents(d['preserved_bytes']) and sha(diff) == d['sha256'] and len(diff) == d['bytes']
            assert b'-        crate::strategic_scene::StrategicCamera,' in diff and b'+        crate::presentation::canvas_view::StrategicCamera,' in diff
        current[i] = body
sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges'])
    sources[x['path']] = body
base = 'crates/adventuresim-tactical-client/src/'
core = 'crates/adventuresim-tactical-core/src/'
geo, markers, module, surface = (current[i] for i in range(1619,1623))
terrain = sources[core+'regional_terrain.rs']; env = sources[core+'regional_environment.rs']
assert b'REGIONAL_TERRAIN_SIDE: usize = 65' in terrain and b'if vertices.len() != REGIONAL_TERRAIN_VERTICES' in terrain
assert b'pub fn new(terrain: RegionalTerrain, connections: Vec<RegionalConnection>)' in env
assert b'pub const MIN: i16 = -500' in sources['crates/adventuresim-world-schema/src/lib.rs']
assert b'pub const MAX: i16 = 9_000' in sources['crates/adventuresim-world-schema/src/lib.rs']
assert b'pub struct EnvironmentalSample' in sources[core+'scene_input/terrain_samples.rs']
assert b'Self::InstallEnvironment { environment }' in module and b'state.environment = Some(environment)' in module
assert all(x in surface for x in [b'surface.source == *terrain.source()',b'surface.request == terrain.request()',b'surface.weather == environment.weather',b'surface.grass == grass'])
city = sources[base+'presentation/regional_map/city.rs']
assert b'let terrain = self.terrain()?' in city and b'super::geographic_surface::position_at_offset(terrain, offset)' in city
assert b'state.ground_position(marker.origin.to_e7().into())?' in markers
assert b'[south_west, south_east, north_west]' in geo and b'[south_east, north_east, north_west]' in geo
mesh = sources[base+'presentation/vista/regional.rs']
assert b'[south_west, south_east, north_west]' in mesh and b'[south_east, north_east, north_west]' in mesh
path = sources[base+'presentation/regional_map/path_geometry.rs']
assert b'for axis in [DVec2::X, DVec2::Y, DVec2::ONE]' in path and b'geographic_surface::segment(terrain, endpoints)' in path
connections = sources[base+'presentation/regional_map/connections.rs']; routes = sources[base+'presentation/regional_map/routes.rs']
assert b'presented.window == environment.terrain().request()' in connections and b'presented.window == terrain.request()' in routes
http = sources['crates/strategic-web/src/routes/map_environment.rs']
assert b'source.as_str() != pack.digest()' in http and b'capture(&pack, request)?' in http and b'sampling revision' in http
assert b'if (resident?.key === key)' in sources['crates/strategic-web/static/regional-environment-request.js']
bridge = sources[base+'browser_runtime.rs']
assert b'let command = serde_json::from_str(json)' in bridge and b'BrowserCommand::RegionalMap { command }' in bridge

manifest = json.loads(contents('recovered/original-transport-manifest.json.gz')); lookup = {x['path']: x for x in manifest['entries']}
records = {}
for x in a['recovered_historical_records']:
    original = lookup[x['original_path']]
    assert (x['sha256'], x['bytes'], x['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    body = contents(x['preserved_bytes']); assert sha(body) == x['published_sha256'] and len(body) == x['published_bytes']
    assert x['redacted_local_path_occurrences'] or (sha(body) == x['sha256'] and oid('blob', body) == x['git_blob'])
    records[x['original_path']] = body
for n, indices in [(510,[1618,1619,1620]),(511,[1621,1622])]:
    path=f'segovb/READ-L12-{n}.json'; receipt=json.loads(records[path]); check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
seg = json.loads(records['segovb/READ-L12-511.json']); qualification = seg['exact_source_qualifications'][0]
assert qualification['path'] in records and qualification['sha256'] == lookup[qualification['path']]['sha256']
assert qualification['raw_requested_ranges'] == [[155,240]] and qualification['effective_requested_body'] == [[155,231]]
assert qualification['nonexistent_tail'] == [[232,240]] and qualification['nonexistent_tail_coverage'] == 0
assert qualification['fresh_personal_body_reads'] == [[1,231]] and qualification['other_lens_coverage_transferred'] is False
reg = json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'))['files'][1622]
assert reg['owner_contract_proofs'][0]['read_ranges'] == [[155,240]]
assert reg['producer_reached_contract_observations'][0]['read_ranges_as_recorded'] == [[155,240]]
assert reg['producer_reached_contract_observations'][0]['source_context']['read_ranges'] == [[155,240]]
assert reg['sha256'] == w['roles'][1622]['historical_sha256']
receipt=json.loads(records['adler/READ-L11-231.json']);check=json.loads(records['adler/VERIFY-L11-231.json'])
assert [x['original_index'] for x in receipt['sources']] == list(range(1613,1623))
assert receipt['accepted_unique_count_change'] == 0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256'] == lookup['adler/READ-L11-231.json']['sha256'] and check['state_sha256'] == lookup['adler/STATE-L11-231.json']['sha256']
for row in receipt['sources'] + receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        path = 'adler/' + row['registration'].replace('\\','/').split('/adler/')[-1]
        assert path in records and row['registration_sha256'] == lookup[path]['sha256']

older = load('OLDER-MAP-QUALIFICATION-GIT-PROOFS.json'); shared_older = load(older['shared_commit_and_tree_objects'])
onodes = {x['git_object']:x for x in shared_older['objects'] if x['type']=='tree'}; onodes.update(older['additional_tree_objects'])
oc = next(x for x in shared_older['objects'] if x['type']=='commit'); raw=contents(oc['preserved_bytes'])
assert oid('commit', raw) == older['revision'] == 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
assert raw.splitlines()[0] == b'tree ' + older['root_tree'].encode()
package = json.loads(contents('recovered/older-774-locator/02.gz')); package_lookup = {x['path']:x for x in package['entries']}
exports = {}
for x in a['recovered_older_774_records']:
    body=contents(x['preserved_bytes']); path=x['path_in_774']; row=package_lookup[path.split('handoff/774/adler-original/',1)[1]]
    assert x['older_publication_revision'] == older['revision'] and sha(body) == x['sha256'] == row['exported_sha256']
    assert x['original_pre_export_sha256'] == row['original_sha256'] != x['sha256'] and len(body)==x['bytes']==row['bytes']
    check_chain(path, older['paths'][path], body, older['root_tree'], onodes); exports[path]=json.loads(body)
rig = next(v for k,v in exports.items() if k.endswith('source-registration-qualification-117.json'))
r = next(x for x in rig['entries'] if x['registration']=='RIG-ADMISSION03-REG-140')
assert r['actual_eof']==231 and r['actual_material_ranges']==[[16,17]] and r['sha256']==reg['sha256']
cam = next(v for k,v in exports.items() if k.endswith('eof-role-qualification-012.json'))
c = next(x for x in cam['per_entry'] if x['id']=='EOF-ROLE-10')
assert c['original_ranges_preserved']==[[155,240]] and c['effective_original_ranges']==[[155,231]] and c['nonexistent_lines_credit']==0
assert c['source_proof']['fresh_read_ranges']==[[155,231]] and c['source_proof']['sha256']==reg['sha256']
q=load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1344,1345))
baseline=load('BASELINE-95E-20261110.json'); ledger=baseline['newly_assessed_ledger']['entries']
assert [x['id'] for x in a['conditional_findings']]==['REVIEW-REGIONAL-MAP-ENVIRONMENT-IDENTITY-001']
for item in a['conditional_findings']:
    assert item['accepted_historical_unique_count_change']==0 and item['repair']=='not started' and item['validation']=='not performed'
    assert sum(x['id']==item['id'] and x['record']=='ASSESSMENT-REGIONAL-MAP-SURFACE-95E.json' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified four historical bodies, strict EOF, older normalized export mappings and bounded environment identity; no product execution.')
