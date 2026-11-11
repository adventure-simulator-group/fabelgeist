#!/usr/bin/env python3
"""Verify publication evidence only; do not execute product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
 d={};i=0
 while i<len(body):
  end=body.index(b'\0',i);mode,n=body[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),body[end+1:end+21].hex());i=end+21
 assert i==len(body);return d
def chain(path,proof,body,start,nodes):
 assert [x['name'] for x in proof['chain']]==path.split('/');current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GARMENT-FIT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2265-2266.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2265,2267))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2265,2266)]==[23,715]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
fit=bodies[2266];spec=sources['crates/fabelgeist-garment-code/src/pattern/spec.rs'];core=sources['crates/fabelgeist-garment-code/src/pattern/core.rs'];cloth=sources['crates/fabelgeist-cloth/src/garment/mod.rs'];particles=sources['crates/fabelgeist-xpbd/src/particles.rs'];bvh=sources['crates/fabelgeist-bvh/src/cpu/mod.rs'];placement=sources['crates/adventuresim-character-creator/src/garment/placement.rs'];drape=sources['crates/adventuresim-character-creator/src/garment/drape.rs']
assert b'pub panels: Vec<(String, PanelSpec)>' in spec and b'.find(|(n, _)| n == name)' in spec and b'panel_index.insert(name.clone(), panels.len())' in fit and b'pub panel: String' in spec and b'pub panel: usize' in cloth
assert b'index.get(stitch.sides[0].panel.as_str())' in fit and b'panel_index.get(&stitch.sides[0].panel)' in fit and b'if stitch.sides[0].edge >= a_starts.len()' in fit and b'skipped.push(name.clone())' in fit and b'if !build.skipped.is_empty() || build.mesh.triangles.is_empty()' in drape
assert b'pub endpoints: [usize; 2]' in spec and b'Quadratic { params: Vec<V2> }' in spec and b'Cubic { params: Vec<V2> }' in spec and b'let start = vertices[edge.endpoints[0]];' in core and b'params[0][0]' in core and b'params[1][0]' in core and b'edge_as_curve(&panel.vertices, edge, false)' in fit
assert b'pub units_in_meter: i64' in spec and b'units_in_meter: 100' in spec and b'self.properties.units_in_meter' in spec and b'pub const CM_TO_M: f32 = 0.01;' in fit and b'outline.into_iter().map(|p| p * CM_TO_M)' in fit
assert b'let reach = 0.5f32;' in fit and b'body.closest_point(*position, reach)' in fit and b'Cell::new(max_distance * max_distance)' in bvh and b'if d <= limit.get()' in bvh
assert b'self.set_collision_mesh(vertices, triangles, settings, fabric)?;' in fit and b'self.collisions.set_mesh(Some(mesh))' in fit and b'let Some(body) = &self.body else' in fit and b'fit.set_collision_mesh(' in placement and b'&collision.positions' in placement
assert b'.write_positions(&self.context, &positions)?' in fit and b'let result = self.write(context, positions, &masses)' in particles and b'self.previous.write(context, packed.upload())?' in particles and b'ParticleVelocities::at_rest(&packed)' in particles
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==41
for batch,i,scope in ((360,2265,[[1,23]]),(361,2266,[[1,715]])):
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);x=next(r for r in data['sources'] if r['original_index']==i);assert data['accepted_unique_count_change']==0 and x['fresh_reads']==x['scope']==scope and x['sha256']==roles[i-2265]['sha256']
path='segovb/READ-L12-746.json';delegate=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-746.json']);assert delegate['root_personal_rereads']==[] and delegate['delegated_attribution_preserved'] and delegate['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256']
for i,scope in ((2265,[[1,23]]),(2266,[[1,715]])):
 x=next(r for r in delegate['rows'] if r['original_index']==i);assert x['fresh_personal_body_reads']==x['assessed_ranges']==scope and x['source_sha256']==roles[i-2265]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
for reg in delegate['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
connected=json.loads(records['segovb/READ-L12-CONNECTED-TEAM-003.json']);assert connected['root_personal_rereads']==[] and connected['original_overlay_merged_shared_extension_credit']==0 and connected['accepted_unique_count_change']==0
for proofrow in connected['qualified_future_original_proofs']:
 assert proofrow['ordered_credit_now']==0 and proofrow['receipt'] in records and proofrow['registration'] in records and proofrow['receipt_sha256']==lookup[proofrow['receipt']]['sha256'] and proofrow['registration_sha256']==lookup[proofrow['registration']]['sha256']
assert {x['original_index'] for x in connected['qualified_future_original_proofs']}=={2629,2631} and work['roles'][2629]['status']=='unassessed' and work['roles'][2631]['status']=='unassessed'
assert roles[0]['original_supplier_scopes'][0]['read_ranges']==[[1,23]] and [x['read_ranges'] for x in roles[1]['original_supplier_scopes']]==[[[575,625]],[[1,715]]]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert q[316]['contract_id']==old[316]['contract_id'] and q[316]['current_reconciliation'].startswith('partial ') and name in q[316]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 assert old[finding['queue']]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified garment-fit provenance, future-read attribution and conditional contracts; no product execution.')
