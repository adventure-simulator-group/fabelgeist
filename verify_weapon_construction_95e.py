#!/usr/bin/env python3
"""Verify preserved source and attribution evidence; do not execute product code."""
import gzip,hashlib,json,tarfile
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';earlier='e2755a732c84e67296b5e3441a4259a84f461b2f';name='ASSESSMENT-WEAPON-CONSTRUCTION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2084-2094.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
eproof=load(proof['earlier_git_comparison']['commit_and_shared_trees']);nodes.update(eproof['additional_tree_objects']);nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2084:111,2085:170,2086:230,2087:318,2088:389,2089:236,2090:359,2091:146,2092:47,2093:22,2094:47};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
comparison=proof['earlier_git_comparison'];commit=contents(eproof['commit']['preserved_bytes']);assert oid('commit',commit)==comparison['revision']==earlier and commit.splitlines()[0]==b'tree '+comparison['root_tree'].encode();body=contents(comparison['current_preserved_bytes']);assert body==current[2089] and oid('blob',body)==tree[comparison['path']];chain(comparison['path'],comparison['proof'],body,comparison['root_tree'],nodes)
base='crates/adventuresim-weapon-model/src/'
recipe=sources[base+'recipe/validation.rs'];dim=sources[base+'recipe/validation/dimensions/furniture.rs'];dispatch=sources[base+'recipe/validation/dimensions.rs'];melee=sources[base+'recipe/validation/melee.rs'];graph=sources[base+'recipe/validation/graphs.rs'];ranged=sources[base+'recipe/validation/ranged.rs'];shields=sources[base+'recipe/validation/shields.rs'];placement=sources[base+'model/placement.rs'];anchors=sources[base+'model/placement/anchors.rs'];mounts=sources[base+'model/mounts.rs'];shape=sources[base+'model/shapes.rs'];model=sources[base+'model/mod.rs'];out=sources[base+'model/output.rs'];quantity=sources[base+'recipe/quantities.rs'];validation=sources[base+'validation.rs'];solids=current[2088];kernel=current[2086];curves=current[2085];surface=current[2089]
assert b'positive(p.length.get())?' in dim and b'positive(p.radius.get())?' in dim and b'require(n.get().abs() <= 128.0' in dim and b'Shape::Grip(p) => furniture::grip(p)' in dispatch and b'p.radius.get() * scale <= MAX_ROUND_GRIP_RADIUS + 1e-9' in melee
assert b'parents == 1' in recipe and b'(!self.components.is_empty() || self.shaft.is_some())' in recipe and b'component.validate()?' in recipe and b'pub fn new(value: f64)' in quantity
assert b'graph.frames.insert(WEAPON_ROOT_FRAME.into(), [0.0; 3])' in placement and b'pub(crate) const WEAPON_ROOT_FRAME: &str = "weapon.root"' in sources[base+'recipe/assembly.rs'] and b'let Some(parent) = parents.iter().find(|p| p.id == owner).or(implicit.as_ref()) else' in mounts and b'component.shape.attachment_anchor' not in anchors
assert b'.attachment_anchor(attachment.at.unwrap_or_default(), range)?' in anchors and b'Self::Grip(p) => [0.0, p.length.get()]' in sources[base+'model/extents.rs'] and b'Shape::Grip(p) => self.grip(p)' in shape and b'p.radius.get() * p.bottom_scale.map_or(1.0, Ratio::get)' in shape
assert b'if profile.len() < 2 || requested < 3 || radial_scale <= 0.0' in solids and b'if profile[ring][1] > 0.0' in solids and b'if profile[row][1] == 0.0' in solids and b'pub(crate) fn construction_budget(triangles: f64)' in kernel
assert b'let physical = PhysicalProperties::from_sources(&sources, resolved.grip)' in model and b'center_of_mass: mul(moment, 1.0 / mass)' in out and b'let center_of_mass = mul(first, 1.0 / mass_kg)' in out and b'let mut min = [f64::INFINITY; 3]' in out and b'let mut max = [f64::NEG_INFINITY; 3]' in out and b'crate::model::Construction::new(&design.recipe)' in validation
assert b'if depth < 8 && (chord > quality.max_chord || deviation > quality.max_deviation)' in curves and b'max_chord: f64::INFINITY' in curves and b'MAX_SURFACE_REFINEMENT_ROUNDS: usize = 12' in surface and b'match m.iter().flatten().count()' in surface and b'self.points.truncate(original_points)' in surface and b'height must be quadratic divided by a positive affine denominator' in surface
assert b'Metres,' in quantity and b'Signed geometric distance in metres' in quantity and b'maximum_deviation: f64' in surface and b'pub(crate) fn error(self, distance: f64)' in kernel
assert b'serde_json::to_vec(self)' in recipe and b'MAX_RECIPE_BYTES: usize = 65536' in recipe and b'bytes.len() > crate::MAX_ENCODED_RECIPE_BYTES' in current[2084] and b'validate(design).map_err(CodecError::InvalidDesign)' in current[2084]
assert a['repair']=='not started' and a['validation']=='not performed' and a['new_current_candidates']==[]
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((701,[2083,2084]),(702,[2085,2086]),(703,[2087,2088,2089]),(704,[2090,2091,2092,2093,2094])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:
  i=row['original_index']
  if i==2089:
   assert row['fresh_personal_body_reads']==[[1,125]] and row['original_requested_ranges']==[[1,125]] and row['connected_gap_personal_reads']['ranges']==[[126,236]] and row['connected_gap_personal_reads']['new_population_credit']==0
  else:assert row['fresh_personal_body_reads']==[[1,lengths.get(i,344)]]
  if 'exact_locator_qualification' in row:assert row['exact_locator_qualification'] in records
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((326,range(2084,2090)),(327,range(2090,2095))):
 data=json.loads(records[f'adler/READ-L11-{n}.json'])
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
assert 'adler/REGISTRATION-ADLER-937.json' in records and 'adler/REGISTRATION-ADLER-948.json' in records
r=json.loads(records['root-l2-extension-20261030/READ-ROOT-L2-EXT-021-033.json']);assert r['rows'][5]['fresh_personal_ranges']==[[1,236]] and r['rows'][9]['fresh_personal_ranges']==[[1,98]] and r['rows'][10]['fresh_personal_ranges']==[[1,75]] and r['rows'][12]['fresh_personal_ranges']==[[76,108]]
r=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-EXT-020-027.json']);assert r['rows'][5]['fresh_personal_ranges']==[[1,236]] and r['rows'][5]['qualified_inherited_same_lens_ranges']==[]
r=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-CURRENT-004.json']);row=r['rows'][4];assert row['revision']==earlier and row['fresh_personal_ranges']==[] and row['qualified_reuse_ranges']==[[1,236]] and row['prior_fresh_personal_ranges']==[[1,236]] and row['prior_own_lens_receipt']['path'] in records
r=json.loads(records['adler/READ-L11-335.json']);assert r['accepted_unique_count_change']==0 and next(x for x in r['sources'] if x['original_index']==2151)['fresh_reads']==[]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (126,1782,1783,1784,1785,2372,3129):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 i=finding['queue'];assert q[i]['contract_id']==old[i]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified eleven historical roles, original reviewer and zero-fresh-reuse attribution, and bounded source links; no product execution.')
