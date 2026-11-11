#!/usr/bin/env python3
"""Verify provenance and inspected source bytes, without product execution."""
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
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
older = 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
name = 'ASSESSMENT-GATE-AND-SUPPORT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1795-1797.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('GATE-SUPPORT-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1795:127, 1796:291, 1797:67}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = w['roles'][i]
        before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(before.splitlines()) == n and sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob',body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
gate = current[1795]; support = current[1796]; street = current[1797]
assert b'let steps = (door.open_angle_radians.radians().abs() / SWEEP_STEP_RADIANS).ceil() as usize;' in gate
assert b'let step = door.open_angle_radians.radians() / steps as f32;' in gate and b'for index in 0..steps' in gate
boundary = sources['crates/adventuresim-tactical-core/src/city_layout/compound/boundary.rs']
assert b'const GATE_OPEN_ANGLE_RADIANS: f32 = core::f32::consts::FRAC_PI_2;' in boundary
assert b'open_angle_radians: Radians::new(self.hinge.sign() * GATE_OPEN_ANGLE_RADIANS)?' in boundary
side = sources['crates/adventuresim-tactical-core/src/city_layout/compound.rs']
assert b'Self::Left => -1.0' in side and b'Self::Right => 1.0' in side
clearance = sources['crates/adventuresim-tactical-core/src/city_layout/compiled/property/clearance.rs']
assert b'let door = compound.boundary.gate.door(compound.id)?;' in clearance
assert b'super::gate_sweep::clear(door, &obstacles)?' in clearance
assert b'ground_floor_contact_bounds()' in support and b'.ok_or_else(binding)?' in support
assert b'binding: SupportBindingRole::Threshold(outward)' in support
bearing = sources['crates/adventuresim-tactical-core/src/city_layout/compiled/single_support/bearing.rs']
assert b'SupportBindingRole::GroundBearing' in bearing
collision = sources['crates/adventuresim-building-generator/src/collision.rs']
assert b'for point in solid.ground_contact()?.points()' in collision and b'Ok(bounds)' in collision
assert b'SupportConstraint::StairGoing' in street and b'candidate(maximum)' in street
olderproof = load(a['older_recovered_trace']); oldmanifest = load(olderproof['shared_objects'])
assert olderproof['revision'] == oldmanifest['commit'] == older
oldnodes = {z['git_object']:z for z in oldmanifest['objects'] if z['type'] == 'tree'}
oldnodes.update(olderproof['additional_tree_objects'])
oldcommit = next(x for x in oldmanifest['objects'] if x['type'] == 'commit')
raw = contents(oldcommit['preserved_bytes'])
assert oid('commit',raw) == older and raw.splitlines()[0] == b'tree ' + olderproof['root_tree'].encode()
trace = contents(olderproof['preserved_bytes'])
check_chain(olderproof['path'], olderproof['path_proof'], trace, olderproof['root_tree'], oldnodes)
assert sha(trace) == '512940c3d587e1ff6c1a7539f0fec9e485b7b2a4ca066b030089fb00c6b8bb74' and len(trace) == 631913
rows = trace.splitlines(); assert len(rows) == 28 and sha(rows[1]) == olderproof['trace_line_sha256']
t = json.loads(rows[1]); assert t['id'] == 'NATIVE-PS-DOORS-002' and t['validation'] == 'none' and t['source_repair'] == 'frozen'
assert t['extra_receivers'][0]['sha256'] == w['roles'][1795]['historical_sha256'] and t['extra_receivers'][0]['ranges'] == [[1,100]]
manifest = json.loads(contents(olderproof['evidence_manifest']))
nativepath = olderproof['path'].split('adler-original/',1)[1]
record = next(x for x in manifest['audit_files'] if x['path'] == nativepath)
assert record['original_sha256'] == record['exported_sha256'] == sha(trace) and record['bytes'] == len(trace)
locatorbody = contents(olderproof['locator_export']); locator = json.loads(locatorbody)
lrecord = next(x for x in manifest['audit_files'] if x['path'] == 'audit/continuation/rig/source-scope-reconciliation-044.json')
assert sha(locatorbody) == lrecord['exported_sha256'] == '1221f858b25f889ab7918895b1f4a3329fb4cb0614b8664531ffd821e7202a31'
assert lrecord['original_sha256'] == 'fe7e2e81a37b125318a22c961f2d9bead728e93d3721e1e3a4142ff1f62defa1' != sha(locatorbody)
entry = locator['new_variant_identity_and_range_map'][105]
assert entry['recorded_sha256'] == w['roles'][1795]['historical_sha256']
assert entry['classification'] == 'genuinely-new-reached-source-variant' and entry['resolved_candidate']['actual_line_count'] == 127
assert entry['observations'][0]['source_context']['ranges'] == [[1,100]] and entry['observations'][0]['range_metadata']['status'] == 'qualified-endpoints-only'
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(584,[1795]),(585,[1796,1797])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
qualification = json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1795-LOCATOR.json'])
assert qualification['historical_locator_proof_body_not_read'] is True
assert qualification['inherited_requested_ranges'] == [[1,100]] and qualification['actual_EOF'] == 127
adler = json.loads(records['adler/READ-L11-266.json'])
adgate = next(x for x in adler['sources'] if x['original_index'] == 1795)
assert adgate['scope'] == [[1,100]] and adgate['fresh_reads'] == [[1,100]]
assert 'revision' not in adgate
q = load('RECONCILIATION-059.json')['entries']
for i in (100,1474,1475,1476,1477):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
findings = a['conditional_findings']; assert len(findings) == 1
assert findings[0]['id'] == 'ADLER-L11-SUPPORT-BINDING-ERROR-ROLE' and findings[0]['accepted_historical_unique_count_change'] == 0
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == findings[0]['id'] and x['record'] == name for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three source roles, original receipts and exact older native trace; no product execution.')
