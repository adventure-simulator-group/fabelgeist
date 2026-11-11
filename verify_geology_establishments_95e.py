#!/usr/bin/env python3
"""Verify geology and establishment recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-GEOLOGY-AND-ESTABLISHMENTS-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2001-2006.json'
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
lengths = {2001:42,2002:534,2003:164,2004:108,2005:42,2006:38}
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
binding, geology, capture, fixture, settlement, sampling = (current[i] for i in lengths)
for phrase in (b'business operator crosses the requested settlement boundary',b'business operator identity is duplicated',b'placed business is missing its resident operator',b'business operator has no placed establishment'):
    assert phrase in binding
assert b'let Some(settlement) = settlement else' in binding and b'.remove(&site.key)' in binding
assert b'const HALF_LENGTH_CM: u16 = 1_200' in geology and b'const HALF_WIDTH_CM: u16 = 1_000' in geology
assert b'const CORRIDOR_HALF_WIDTH_METRES: f32 = 7.0' in geology and b' / 100.0 + 2.0' in geology
assert b'origin.y.abs() - radius <= CORRIDOR_HALF_WIDTH_METRES' in geology
assert b'let local_half_width = self.half_width_metres * (1.0 - variation + width_noise * variation)' in sources['crates/adventuresim-tactical-core/src/terrain_transition.rs']
assert b'SCARP_RUPTURE_WANDER_METRES: f32 = 1.25' in sources['crates/adventuresim-tactical-core/src/volumetric_terrain.rs']
assert b'SCARP_WIDTH_VARIATION_BPS: u16 = 1_200' in sources['crates/adventuresim-tactical-core/src/volumetric_terrain.rs']
assert b'pub fn inclusive_unit_f32(value: u64) -> f32' in sources['crates/fabelgeist-determinism/src/lib.rs']
assert b'canopy_percent: bytes[4]' in sources['crates/adventuresim-terrain/src/lib.rs']
assert b'let canopy = if canopy <= 100 { canopy } else { 0 }' in sources['crates/adventuresim-terrain/src/builder.rs']
assert b'canopy_bps: u16::from(cell.canopy_percent) * BASIS_POINTS_PER_PERCENT' in sampling
assert b'.any(|value| value > BASIS_POINTS_PER_WHOLE)' in sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'if vertices.len() != REGIONAL_TERRAIN_VERTICES' in sources['crates/adventuresim-tactical-core/src/regional_terrain.rs']
assert b'VistaSampling(String)' in capture and b'absolute_elevation: self.absolute_elevation.get()' in capture
assert b'pub struct ElevationMeters {' in sources['crates/adventuresim-world-schema/src/lib.rs'] and b'meters: i16' in sources['crates/adventuresim-world-schema/src/lib.rs']
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



for n, indices in [(671,[2001,2002]),(672,[2003,2004,2005,2006])]:
    path = f'segovb/READ-L12-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']] == indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256'] == lookup[z['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
        if 'locator_qualification' in row:
            assert row['locator_qualification'] in records and row['locator_qualification_sha256'] == lookup[row['locator_qualification']]['sha256']
    for row in z.get('qualified_supporting_same_lens_reuse',[]) + z.get('qualified_connected_original_reuse',[]):
        assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads_this_batch'] == []
for n, indices in [(309,[2000,2001,2002]),(310,[2003,2004,2005,2006,2007,2008])]:
    path = f'adler/READ-L11-{n}.json'; z = json.loads(records[path]); check = json.loads(records[f'adler/VERIFY-L11-{n}.json'])
    assert [r['original_index'] for r in z['sources']] == indices and z['current_reconciled'] is False
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources'] + z.get('connected_reinspection',[]):
        p = 'adler/' + row['registration'].replace('\\','/').split('/')[-1]
        assert p in records and row['registration_sha256'] == lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p = 'adler/' + row['receipt'].replace('\\','/').split('/')[-1]
        assert p in records and row['sha256'] == lookup[p]['sha256']
qual = json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL2006-LOCATOR.json'])
assert qual['original_index'] == 2006 and qual['body_coverage'] == 0 and qual['historical_locator_proof_body_not_read'] is True
oldroot, oldnative, locator = a['recovered_older_774_records']
for item in (oldroot,oldnative,locator):
    body = contents(item['preserved_bytes'])
    assert sha(body) == item['sha256'] and len(body) == item['bytes'] and oid('blob',body) == item['git_blob']
    assert item['older_publication_revision'] == 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
assert oldroot['package_original_sha256'] == oldroot['package_exported_sha256'] == oldroot['sha256']
assert oldnative['package_original_sha256'] == oldnative['package_exported_sha256'] == oldnative['sha256']
assert locator['package_original_sha256'] != locator['package_exported_sha256'] == locator['sha256']
def record(item,ident):
    return next(json.loads(line) for line in contents(item['preserved_bytes']).splitlines() if json.loads(line).get('id') == ident)
rootrow = record(oldroot,'ROOT-PPS-TERRAIN-DISPATCH-WEB-037')['source_proofs'][1]
nativerow = record(oldnative,'NATIVE-PS-ROOT-TERRAIN-SOURCE-PACKET-015')['related_current_reads'][1]
for old,new in ((qual['raw_producer_observations'][0]['source_context'],rootrow),(qual['raw_producer_observations'][1]['source_context'],nativerow)):
    for key in ('source_id','sha256','read_ranges'):
        assert old[key] == new[key]
    assert old['scope'] != new['scope']
assert rootrow['path'] == qual['raw_producer_observations'][0]['source_context']['path']
assert nativerow['resolved_path'] == qual['raw_producer_observations'][1]['source_context']['resolved_path']
q = load('RECONCILIATION-059.json')['entries']
for i in (114,1660,1662,1663,1664,1665,1666,1667,1668):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert len(a['conditional_findings']) == 1 and a['conditional_findings'][0]['id'] == 'REVIEW-ESTABLISHMENT-BINDING-ERROR-IDENTITY-001'
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == a['conditional_findings'][0]['id'] for x in ledger) == 1
assert next(x for x in ledger if x['id'] == a['conditional_findings'][0]['id'])['record'] == name
for ident in ('ADLER-L11-SCENE-PROVISIONING-ERROR-OWNERSHIP','Q-L12-3ba77276b61acbc148a8'):
    assert name in next(x for x in ledger if x['id'] == ident)['additional_current_evidence']
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified six historical geological and binding roles, original and older qualified attestations, and bounded current source; no product execution.')
