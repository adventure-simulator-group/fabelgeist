#!/usr/bin/env python3
"""Verify terrain-diagnostic and recovered receipt provenance without executing product code."""
import gzip, hashlib, json, tarfile, tomllib
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
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end+1:end+21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree'] and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body) and sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-TERRAIN-DIAGNOSTICS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1977-1981.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:
    nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1977:349, 1978:36, 1979:21, 1980:102, 1981:142}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n and sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body

assert all((root / p).is_file() for p in a['prior_current_evidence'])
compare = current[1977]; capture = current[1978]; export = current[1979]; garden = current[1981]
assert b'.expect("all four processing stages are supplied")' in compare
for role in (b'TerrainStageRole::UngradedNearVista', b'TerrainStageRole::RawPlayable', b'TerrainStageRole::PreparedFinePlayable', b'TerrainStageRole::BoundedSupport'):
    assert role in compare
assert b'purpose: TerrainPreparationPurpose::CompleteSceneContext' in compare and b'input.digest()? != stages.input_digest' in compare
assert b'PropertyAssetIsolation' in capture and b'self.input_digest != input.digest()?' in capture and b'self.purpose' not in capture
assert b'Exact complete-context fine terrain' in export and b'let source = self.geographic_surface(input, ungraded_vista)?' in export
assert b'TacticalCombatConfig::default().movement.motor' in garden and b'composed_grade_within_controller_bound' in garden
assert b'controller traversal and collision clearance are separate acceptance checks' in garden
assert b'Never use `unwrap` or `expect` outside tests' in sources['crates/AGENTS.md']
assert b'defaults::canonical().clone()' in sources['crates/adventuresim-tactical-core/src/combat_config.rs']
assert b'movement.motor.maximum_walkable_slope_degrees >= 90.0' in sources['crates/adventuresim-tactical-core/src/combat_config.rs']
assert b'content/tactical/combat.yaml' in sources['crates/adventuresim-tactical-core/src/combat_config/defaults.rs']
assert b'maximum_walkable_slope_degrees: 40.0' in sources['content/tactical/combat.yaml']
server = sources['crates/adventuresim-tactical-server/src/main.rs']
assert b'combat_config: Option<PathBuf>' in server and b'args.combat_config.as_deref()' in server and b'app.insert_resource(combat_config)' in server
startup = sources['crates/adventuresim-tactical-server/src/startup.rs']
assert b'load_combat_config(&combat_path)' in startup and b'.validate()' in startup
physics = sources['crates/adventuresim-tactical-core/src/physics.rs']
assert b'combat_config: Res<crate::combat_config::TacticalCombatConfig>' in physics and b'controller_config::apply_motor_config(&mut controller, motor)' in physics
assert b'controller.min_walk_cos = motor.maximum_walkable_slope_degrees.to_radians().cos()' in sources['crates/adventuresim-tactical-core/src/physics/controller_config.rs']
validator = sources['crates/adventuresim-tactical-server-dispatcher/examples/validate-city-support.rs']
assert b'capture.geographic_surface(&input, &stages.ungraded_vista)?' in validator and b'preparation purpose retained in the capture' in validator
assert b'stages.input_digest != input.digest()?' in sources['crates/adventuresim-tactical-server-dispatcher/examples/inspect_city_grounding/source.rs']
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
    else:
        assert sha(body) == item['sha256'] and oid('blob', body) == item['git_blob']
    records[item['original_path']] = body

n = 663; path = f'segovb/READ-L12-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
assert [r['original_index'] for r in z['rows']] == list(lengths) and z['current_source_reconciled'] is False
assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
for row in z['rows']:
    assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
n = 305; path = f'adler/READ-L11-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
assert [r['original_index'] for r in z['sources']] == [1977,1978,1979,1980,1981,1982] and z['current_reconciled'] is False
assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
for row in z['sources'] + z.get('connected_reinspection',[]):
    p = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
    assert p in records and row['registration_sha256'] == lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
    assert p in records and row['sha256'] == lookup[p]['sha256']
original = load('baseline-059/queue.json')['entries']; q = load('RECONCILIATION-059.json')['entries']
assert original[261]['contract_id'] == 'SEGOVB-L12-TERRAIN-STAGE-LOOKUP-PANIC'
for i in (261,1631,1632,1633,1634,1637,1650):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ids = [x['id'] for x in a['conditional_findings']]
assert ids == [original[261]['contract_id'],'REVIEW-GARDEN-DIAGNOSTIC-DEFAULT-WALK-SLOPE-001']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in ids:
    assert sum(x['id'] == ident for x in ledger) == 1 and next(x for x in ledger if x['id'] == ident)['record'] == name
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified five historical terrain diagnostics, original attributions and bounded configuration source; no diagnostic or product execution.')
