#!/usr/bin/env python3
"""Verify equipment topology source and recovered evidence only."""
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
        end = body.index(b'\0', pos); mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex()); pos = end + 21
    assert pos == len(body)
    return result
def verify_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for step in proof['chain']:
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == step['tree'] and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[step['name']] == (step['mode'], step['child']); current = step['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
pr = '6533e444fa37552927070cc8d8190907fd3b413e'
pre = 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
name = 'ASSESSMENT-EQUIPMENT-TOPOLOGY-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2048-2053.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence']); shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']: nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][pr]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == pr and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {2048:1126, 2049:248, 2050:11, 2051:252, 2052:29, 2053:95}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed', 'bounded-current-trace-only') and item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n and sha(before) == row['historical_sha256'] and before == body
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        verify_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes); current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert all((root/p).is_file() for p in a['prior_current_evidence'])
def walk(x):
    if isinstance(x, dict):
        yield x
        for v in x.values(): yield from walk(v)
    elif isinstance(x, list):
        for v in x: yield from walk(v)
for item in h['historical_source_roles']:
    for prior in item['preexisting_current_support_not_indexed_in_prior_role_row']:
        record=load(prior['assessment']); path=item['current_comparison']['path']; ranges=prior['ranges_as_recorded']
        assert prior['source_revision']==rev and any(x.get('path')==path and x.get('revision')==rev and x.get('ranges')==ranges for x in walk(record)),(item['original_index'],prior)
server,attachment,environment,fit,lifecycle,transfer=(current[i] for i in lengths)
schema=sources['crates/adventuresim-core/src/item_catalog_schema.rs'];validation=sources['crates/adventuresim-core/src/item_catalog_validation.rs'];deps=sources['crates/adventuresim-core/src/item_catalog_validation_occupancy.rs'];strategic=sources['crates/adventuresim-stdb-module/src/character.rs'];catalog=json.loads(sources['content/items/catalog.yaml']);rules=sources['crates/AGENTS.md']
assert b'A placement may combine physical character anchors with any number of' in schema and b'pub occupancy: Vec<OccupancyRequirement>' in schema and b'pub parents: Vec<ParentRequirement>' in schema
assert b'if occupancy.is_empty() && parents.is_empty()' in validation and b'validate_dependencies(&parents, placement' in validation and b'serde_json::from_value::<crate::item_catalog_schema::ParentRequirement>' in deps
assert b'let root_occupancies = placement' in strategic and b'let mut parent_occupancies = Vec::new()' in strategic and b'for (requirement_index, requirement) in root_occupancies' in strategic and b'in parent_occupancies' in strategic
assert b'placement.parents.is_empty()' in server and b'if placement.occupancy.is_empty() && !placement.parents.is_empty()' in attachment
assert b'right_selected.cmp(&left_selected)' in attachment and b'available.iter().find(|target|' in attachment and b'point.order == requirement.order' in attachment and b'!chosen.iter().any' in attachment
assert b'destination.as_ref().unwrap()' in transfer and transfer.count(b'destination.as_ref().unwrap()')==2 and b'Never use `unwrap` or `expect` outside tests' in rules
assert b'only character anchors have fit zones' in fit and b'self.fit_zone.is_none()' in sources['crates/adventuresim-core/src/item_catalog_fit.rs']
assert b'.find_map(|(child, _, _, topology, _, _, _, _)|' in server and b'if has_children(item, items)' in server and b'ItemOf(actor),' in server and b'EquipmentTopology::default(),' in server
c=a['authored_catalog_census'];placements=[(item['id'],p) for item in catalog['items'] for p in item.get('equipment',{}).get('placements',[])];mixed=[(i,p['id']) for i,p in placements if p.get('occupancy') and p.get('parents')];multi=[{'item':i,'placement':p['id'],'parents':p['parents']} for i,p in placements if len(p.get('parents',[]))>1]
assert c['item_count']==len(catalog['items']) and c['placement_count']==len(placements) and c['mixed_body_and_parent_placements']==mixed==[] and c['multi_parent_placements']==multi and len(multi)==1 and multi[0]['item']=='sword_sheath' and [p['order'] for p in multi[0]['parents']]==[0,1]
padded=next(x for x in catalog['items'] if x['id']=='padded_chausses')['equipment'];point=next(x for x in padded['attachment_points'] if x['id']=='mail_knee_voider');voider=next(x for x in catalog['items'] if x['id']=='mail_knee_voider')['equipment']
assert point['channel']=='flexible_armor' and point.get('order',0)==0 and point['capacity']==1 and 'mail_knee_voider' in point['accepts_tags'] and 'mail_knee_voider' in voider['attachment_tags']
assert {p['occupancy'][0]['location'] for p in padded['placements']}=={'left_leg','right_leg'}
transport=json.loads(contents('recovered/original-transport-manifest.json.gz')); lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    if item['redacted_local_path_occurrences']: assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
    else: assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
    records[item['original_path']]=body
for n,indices in ((689,[2048]),(690,[2049,2050,2051,2052,2053,2054])):
    path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['current_source_reconciled'] is False
    assert data['plan'] in records and data['plan_sha256']==lookup[data['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in data['rows']:
        nlines=60 if row['original_index']==2054 else lengths[row['original_index']]
        assert row['fresh_personal_body_reads']==[[1,nlines]] and row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
data=json.loads(records['adler/READ-L11-319.json']);assert [x['original_index'] for x in data['sources']]==list(range(2046,2054)) and data['current_reconciled'] is False
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (273,1743,1744,1745,1746,1747,1748,1749,1750): assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
    row=next(x for x in ledger if x['id']==finding['id']); assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert name in next(x for x in ledger if x['id']=='REVIEW-PICKUP-ANCHOR-WORLD-PROJECTION-001')['additional_current_evidence']
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historical equipment roles, original guarded finding, authored catalog census and conditional topology reasoning; no product execution.')
