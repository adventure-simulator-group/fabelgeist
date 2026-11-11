#!/usr/bin/env python3
"""Verify provenance, source and package evidence without product execution."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';older='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';name='ASSESSMENT-WEAPON-IDENTITY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2095-2104.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
oldproof=load(proof['older_comparison']['commit_and_shared_trees']);nodes.update({x['git_object']:x for x in oldproof['objects'] if x['type']=='tree'});nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2095:36,2096:59,2097:106,2098:34,2099:261,2100:574,2101:199,2102:114,2103:50,2104:145};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
comparison=proof['older_comparison'];commitinfo=next(x for x in oldproof['objects'] if x['git_object']==older and x['type']=='commit');commit=contents(commitinfo['preserved_bytes']);assert oid('commit',commit)==comparison['revision']==older and commit.splitlines()[0]==b'tree '+comparison['root_tree'].encode()
for item in comparison['source_and_transport_paths']:
 body=contents(item['current_preserved_bytes']);assert body==sources[item['current_path']] and sha(body)==item['source_sha256'] and 'workspace revision' in item['qualification'];chain(item['current_path'],item['source_tree_proof'],body,comparison['root_tree'],nodes);chain(item['transport_path'],item['transport_tree_proof'],body,comparison['root_tree'],nodes)
base='crates/adventuresim-weapon-model/src/'
recipe=sources[base+'recipe/mod.rs'];validator=sources[base+'recipe/validation.rs'];placement=sources[base+'model/placement.rs'];catalog=sources[base+'catalog.rs'];combat=sources['crates/adventuresim-core/src/combat/contact_geometry.rs'];admission=sources['crates/adventuresim-core/src/equipment/parametric_weapon.rs'];instance=sources['crates/adventuresim-stdb-module/src/weapon_instance.rs'];evaluation=sources['crates/adventuresim-stdb-module/src/weapon_instance/evaluation.rs'];holder=current[2099];mesh=current[2104];icon=current[2100];raster=current[2101];studio=current[2102];memo=current[2097]
assert b'.or_else(|| self.label.clone())' in recipe and b'.unwrap_or_else(|| format!("component-{index}"))' in recipe and b'component.resolved_id(index)' in validator and b'let mut recipe = recipe.clone()' in placement and b'component.id = Some(id.clone())' in placement
assert holder.count(b'component.id.as_deref().expect("gameplay component ID")')==2 and b'crate::validation::holder_identity(design)' in holder and b'crate::model::Construction::new(&design.fitted_weapon.recipe)' in holder and b'"club" | "flanged_mace" | "hand_axe" | "war_hammer" => Some(WeaponHolderKind::HaftLoop)' in catalog
actual=json.loads(sources['crates/adventuresim-weapon-model/catalog/gameplay.json']);weapon=next(x for x in actual if x['catalog_id']=='war_hammer');heads=[x for x in weapon['recipe']['components'] if x.get('role')=='Head'];assert [(x['id'],x['label']) for x in heads]==[('poll','poll'),('beak','beak')]
assert b'.filter(|c| c.role == Some(ComponentRole::Head))' in mesh and b'.filter_map(|c| c.id.as_deref())' in mesh and b'if head_ids.contains(&id)' in mesh and b'striking_head_length_m: if head_min.is_finite()' in mesh and b'(0.0..=total_length_m).contains(&striking_head_length_m)' in admission and b'&& striking_head_length_metres > 0.0' in combat
assert b'.filter_map(|component| component.id.as_deref().zip(component.role))' in icon and b'synthetic_head_focus(generated, &mut focus, &mut principal_head)' in icon and b'Png(String)' in icon and b'.map_err(|error| super::IconError::Png(error.to_string()))' in studio
assert b'!(16..=512).contains(&spec.size)' in raster and b'!(1..=8).contains(&spec.supersampling)' in raster and b'coverage.push((covered * 255 / (samples * samples)) as u8)' in raster and b'const RETAINED_EVALUATIONS: usize = 64' in memo and b'let physical = construct()?' in memo
assert b'evaluate_instance(&instance, item_id)?' in instance and b'derived.striking_head_length_m' in instance and b'EvaluatedWeapon::new(design.clone())' in evaluation and b'WeaponInstance::from_evaluation' in evaluation
native=load(a['native_source_evidence']);archive=root/'native-evidence/tiny-skia-0.11.4'/native['archive'];raw=archive.read_bytes();assert len(raw)==native['bytes'] and sha(raw)==native['sha256']=='83d13394d44dae3207b52a326c0c85a8bf87f1541f23b0d143811088497b09ab' and native['sha256'].encode() in sources['Cargo.lock']
with tarfile.open(archive,'r:gz') as t:
 for x in native['inspected_members']:
  body=t.extractfile(x['path']).read();assert len(body)==x['bytes'] and sha(body)==x['sha256']
  if x['path'].endswith('/src/pixmap.rs'):pixmap=body
  if x['path'].endswith('/Cargo.toml'):nativecargo=body
assert sha(pixmap)=='a7845a931a40b5b24eaa24b6053121e61bfe8b3fffa4c0db89f58c368fb9bc56' and pixmap.count(b'pub fn encode_png(&self) -> Result<Vec<u8>, png::EncodingError>')==2 and b'writer.write_image_data(&tmp_pixmap.data)?' in pixmap and b'"png-format"' in nativecargo
assert a['repair']=='not started' and a['validation']=='not performed' and (root/a['prior_current_evidence'][0]).is_file()
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((705,list(range(2095,2100))),(706,list(range(2100,2105)))):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:
  i=row['original_index']
  if i==2100:assert row['fresh_personal_body_reads']==[[1,407]] and row['connected_gap_personal_reads']['ranges']==[[408,574]] and row['connected_gap_personal_reads']['new_population_credit']==0
  else:assert row['fresh_personal_body_reads']==[[1,lengths[i]]]
  if i==2104:assert row['original_requested_ranges']==[[1,150]] and row['excluded_nonexistent_ranges']==[[146,150]] and row['nonexistent_line_coverage']==0 and row['exact_EOF_qualification'] in records
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((327,range(2095,2100)),(328,range(2100,2105))):
 data=json.loads(records[f'adler/READ-L11-{n}.json'])
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,521 if i==2100 else lengths[i]]] and row['sha256']==sha(current[i])
 if n==328:
  nativeview=next(x for x in data['connected_reinspection'] if x['original_index']==26);assert nativeview['fresh_reads']==[[155,182],[385,430]] and nativeview['source_sha256']==sha(pixmap) and nativeview['additional_original_frontier_credit']==0
root1=json.loads(records['segovb/READ-L12-CONNECTED-TEAM-001.json']);assert 'zero personal rereads' in root1['source_attribution'] and 'SEGOVB-L12-HOLDER-ORIGINAL-OPTIONAL-ID-EXPECT' in root1['new_findings'];delegate=json.loads(records['parallel-l12-20261029/READ-REVISIT-PAR-L12-716-HOLDER-IDENTITY.json']);assert 'actual personal prospective' in delegate['attribution'] and 'not zero-reread reuse' in delegate['attribution']
root21=json.loads(records['segovb/READ-L12-CONNECTED-TEAM-021.json']);assert root21['root_personal_rereads']==[] and root21['accepted_unique_count_change']==0
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (124,287,1785,1790,1791,1800,2431):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:
 i=finding['queue'];assert q[i]['contract_id']==old[i]['contract_id']==finding['id'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified ten historical roles, scoped delegate and f82 transport attribution, pinned native package and current source links; no product execution.')
