#!/usr/bin/env python3
"""Verify source provenance and qualified interval/profile evidence only."""
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
        assert oid('tree', raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-SUPPORT-INTERVAL-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1831-1843.json'
a, h, worklist = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
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
lengths = {1831:279,1832:37,1833:156,1834:51,1835:64,1836:260,1837:61,1838:127,1839:115,1840:281,1841:120,1842:66,1843:115}
ranges = {1831:[[1,230],[247,258]],1832:[[1,36]],1833:[],1834:[],1835:[[6,32],[35,55]],1836:[[85,130]],1837:[],1838:[[1,75]],1839:[[1,90]],1840:[[1,220]],1841:[[1,120]],1842:[[1,64]],1843:[[1,115]]}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = worklist['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == ('bounded-current-trace-only' if i in (1831,1835) else 'unassessed')
        assert item['prior_current_observations'] == row.get('current_observations',[])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == ranges[i]
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert b'let columns = xs.len() - 1' in current[1831]
assert b'for z in zs.windows(2)' in current[1831] and b'for x in xs.windows(2)' in current[1831]
assert b'pub fn expanded(self, distance: f64)' in current[1833]
assert b'if ratio.is_finite() && ratio > 0.0' in current[1835]
assert b'pub(super) enum ProfileError' in current[1836]
assert b'Self::stepped(points).map_err(|error|' in current[1836]
assert b'let mut observations = self.observations;' in current[1838]
assert current[1838].index(b'compound::street_gate_level') < current[1838].index(b'CompoundSupportPlan::compile')
assert b'if doors.is_empty()' in current[1843] and b'SelectedFloor::bind(*doors[0]' in current[1843]
assert b'let constraints = apron.floor_interval(request.policy)' in current[1843]
assert b'SupportElevation::from_metres(elevation_metres)' in current[1843]
assert b'(available / policy.stairs.minimum_going_metres.metres()).floor()' in current[1840]
assert b'minimum: SupportElevation(' in current[1840] and b'maximum: SupportElevation(' in current[1840]
assert b'if mesh.maximum_grade() > bounds.limits.maximum_grade.ratio()' in current[1841]
length = sources['crates/fabelgeist-geometry/src/length.rs']
assert b'if !value.is_finite()' in length and b'if value == 0.0' in length
policy = sources['crates/adventuresim-tactical-core/src/city_layout/compiled/support/policy.rs']
assert b'positive_metres(0.25)' in policy
source = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/source.rs']
assert b'.expanded(f64::from(tolerance_metres) * self.boundary_extension_factor)' in source
bank = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/foundations/banks.rs']
assert b'.intersecting_boundary_segment(' in bank and b'plan.limits.contact_tolerance_metres.metres()' in bank
profile = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/compound/profiles.rs']
assert b'let points = [' in profile and b'profile: court_sequence(property, points.into_iter().collect())?' in profile
stairs = sources['crates/adventuresim-tactical-core/src/city_layout/grounding/stairs.rs']
assert b'pub(super) fn shift_floor' in stairs and b'pub fn local_cuts' in stairs
native = a['native_source_evidence']['installed_rust_std']; body = contents(native['preserved_bytes'])
assert len(body) == native['bytes'] and sha(body) == native['sha256']
assert b'The precision of this function is non-deterministic.' in body
assert b'pub const fn floor(self) -> f32' in body and b'core::f32::math::floor(self)' in body
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob',body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(601,[1831,1832,1833,1834,1835]),(602,[1836,1837,1838,1839]),(603,[1840,1841,1842,1843])]:
    path = f'segovb/READ-L12-{n}.json'; receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']] == indices and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256']
    assert check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads'] == [[1,lengths[row['original_index']]]]
    for row in receipt.get('qualified_supporting_same_lens_reuse',[]):
        assert row['fresh_personal_body_reads_this_batch'] == []
        assert row['receipt'] in records and row['receipt_sha256'] == lookup[row['receipt']]['sha256']
assert [x['original_index'] for x in json.loads(records['segovb/READ-L12-602.json'])['qualified_supporting_same_lens_reuse']] == [1808,1831]
assert [x['original_index'] for x in json.loads(records['segovb/READ-L12-603.json'])['qualified_supporting_same_lens_reuse']] == [1803,1804,1835,1839]
for n, indices in [(275,list(range(1831,1837))),(276,list(range(1837,1841))),(277,list(range(1841,1844)))]:
    receipt = json.loads(records[f'adler/READ-L11-{n}.json'])
    assert [x['original_index'] for x in receipt['sources']] == indices
    assert receipt['current_reconciled'] is False and receipt['validated'] is False
q = load('RECONCILIATION-059.json')['entries']
for i in [104,260,1518,*range(1521,1531)]:
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'ASSESSMENT-GEOGRAPHY-GRADE-95E.json' in q[1518]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
    assert finding['accepted_historical_unique_count_change'] == 0
    assert sum(x['id'] == finding['id'] and x['record'] == name for x in ledger) == 1
assert len(a['conditional_findings']) == 2
assert 'not an executed PreparedApron' in a['conditional_findings'][0]['trace']
assert 'precision is unspecified' in a['conditional_findings'][0]['trace']
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0
assert p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified thirteen historical roles, attributed receipts and conditional interval/profile traces; no product execution.')
