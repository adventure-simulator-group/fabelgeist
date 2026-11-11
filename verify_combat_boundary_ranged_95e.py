#!/usr/bin/env python3
"""Verify combat boundary recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-COMBAT-BOUNDARY-RANGED-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2025-2026.json'
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
lengths = {2025:46,2026:1148}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed','bounded-current-trace-only') and item['prior_current_observations'] == row.get('current_observations', [])
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
assert all((root/p).is_file() for p in a['prior_current_evidence'])
boundary,combat=(current[i] for i in lengths)
assert b'boundary.fixed_support().collider()?' in boundary
assert b'if let Some(collider) = collider.into_solid()' in boundary
assert b'pub(crate) struct TacticalConsequenceAccumulator' in combat and b'HashMap<CharacterId, AccumulatedPartyConsequence>' in combat
assert b'pub struct MeleeAttackResolved {' in combat and b'pub attacker: Entity' in combat and b'pub anatomical_subregion: AnatomicalSubregion' in combat
assert b'quantity.get() - 1' in combat
assert b'fn ranged_rejections_happen_before_ammo_scan()' in combat
assert b'.find("q_ammo.iter().find").expect("ammo scan")' in combat
ranged=sources['crates/adventuresim-tactical-server/src/combat/ranged.rs']
assert b'q_ammo.iter().find' not in ranged and b'ammunition.iter().find' in ranged
resolver=ranged[ranged.index(b'pub(super) fn resolve_ranged_attack('):]
assert resolver.index(b'validate_ranged_intent(facts)') < resolver.index(b'RangedAmmunition::find(') < resolver.index(b'authority.authorize_shot(') < resolver.index(b'ammunition.consume(')
assert b'let Some(ammunition) = ammunition else' in resolver
quantity=sources['crates/adventuresim-core/src/inventory_measurement.rs']
assert b'pub struct ItemQuantity(NonZeroU32)' in quantity and b'NonZeroU32::new(value)' in quantity
assert b'#[serde(transparent)]' in quantity
# Preserve, rather than retroactively index, earlier current observations.
def walk(x):
 if isinstance(x,dict):
  yield x
  for v in x.values():yield from walk(v)
 elif isinstance(x,list):
  for v in x:yield from walk(v)
for item in h['historical_source_roles']:
 for prior in item['preexisting_current_support_not_indexed_in_prior_role_row']:
  record=load(prior['assessment']); matches=[x for x in walk(record) if x.get('path')==item['current_comparison']['path'] and x.get('revision')==rev and x.get('ranges')==prior['ranges']]
  assert matches,(item['original_index'],prior)
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
 assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:
  assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in [(678,[2022,2023,2024,2025]),(679,[2026])]:
 p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
 assert [row['original_index'] for row in z['rows']]==indices and z['current_source_reconciled'] is False
 assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
 for row in z['rows']:
  assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
  if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
for n,indices in [(313,[2018,2019,2020,2021,2022,2023,2024,2025]),(314,[2026,2027,2028]),(318,[2043,2044,2045])]:
 p=f'adler/READ-L11-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
 assert [row['original_index'] for row in z['sources']]==indices and z['current_reconciled'] is False
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
 for row in z['sources']+z.get('connected_reinspection',[]):
  rp='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert rp in records and row['registration_sha256']==lookup[rp]['sha256']
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (116,276,1704,1705,1706,1710):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for ident in ('REVIEW-RANGED-AUTHORIZATION-SOURCE-ASSERTION-001','SEGOVB-L12-TACTICAL-EVENT-DISPLAY-NAME-IDENTITY'):
 assert name in next(x for x in ledger if x['id']==ident)['additional_current_evidence']
assert a['conditional_findings']==[]
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified two historical boundary/combat roles, prior current links, original receipts and static ranged source mismatch; no product execution.')
