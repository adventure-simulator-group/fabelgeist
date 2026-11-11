#!/usr/bin/env python3
"""Verify source and recovery evidence only; do not run product behavior."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-STAGING-TASSETS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2200-2209.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));oldbodies={};bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2200,2210))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i)
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=c['lines'];assert sha(old)==row['historical_sha256']==regrow['historical']['sha256'] and len(old.splitlines())==n and regrow['historical']['ranges']==item['new_personal_read_ranges']==[[1,n]]
  assert sha(body)==c['sha256']==regrow['current']['sha256'] and len(body.splitlines())==m and regrow['current']['ranges']==c['current_personal_ranges']==[[1,m]] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
  assert (old==body)==c['byte_identical']==regrow['byte_identical']==(i!=2200);chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);oldbodies[i]=old;bodies[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
assert roles[5]['original_supplier_scopes'][0]['read_ranges']==[[1,160]] and roles[5]['original_producer_scopes'][0]['read_ranges_as_recorded']==[[1,160]]
staging=bodies[2201];tassets=bodies[2204];limb_design=bodies[2205];metal=bodies[2206];ornament=bodies[2207];skin=bodies[2209];mod=sources['crates/fabelgeist-armor/src/gpu/mod.rs'];part=sources['crates/fabelgeist-armor/src/gpu/part.rs'];limb=sources['crates/fabelgeist-armor/src/gpu/limb.rs'];native=sources['crates/fabelgeist-compute/src/readback.rs'];garment=sources['crates/fabelgeist-armor/src/garment_armor.rs'];shape=sources['crates/fabelgeist-armor/src/garment_plate_design.rs'];creator=sources['crates/adventuresim-character-creator/src/device_wrapped_tasset.rs']
assert b'pub struct Staged(usize)' in staging and b'pub struct StagedResults' in staging and b'self.bytes[staged.0]' in staging and b'values.truncate(count)' in staging and b'self.get::<u32>(staged)[0]' in staging
assert b'pub fn stage<' in mod and b'pub fn finish(' in mod and b'pub struct PartSlots' in part and b'#[derive(Clone, Copy, Debug)]' in part and b'results.status(slots.area)' in part and b'positions.truncate(count)' in part and b'faces: self.layout.final_faces()' in part and b'grids: self.layout.grids()' in part
assert b'for lame in 0..lames' in limb and b'(2..=7).contains(&d.lame_count)' in limb_design and b'besagew: None' in limb_design and b'PartRecipe::new()' in limb and b'textile_chart: bool' in bodies[2200] and b'self.textile_chart = true' in bodies[2200]
assert b'.checked_mul(wgpu::COPY_BUFFER_ALIGNMENT)' in native and b'.checked_after(BufferByteLength::from(padded))' in native and b'self.ranges.iter().map(|range| range.bytes(&data)).collect()' in native
assert b'design.validate()?' in tassets and b'GarmentPlateShape::WrappedTassets(shape)' in tassets and b'kind == GarmentArmorKind::Tassets' in shape and b'1..=if self.kind == GarmentArmorKind::Tassets' in garment and b'shape.section_break >= self.lame_count' in garment
assert b'read_u32("left", &left.status)' in creator and b'read_u32("right", &right.status)' in creator and b'atomic("status", part.status())' in creator
assert b'Motif::Wave { amplitude: 0.8 }' in ornament and b'pub const MAX_LINE: f32 = 0.2' in ornament and b'pub fn blend' not in skin and b'pub(crate) fn blend' in skin and b'if total <= 0.0' in skin
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(732,[2200,2201]),(733,[2202,2203,2204,2205,2206,2207]),(734,[2208,2209])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2200]['lines'];assert x['source_sha256']==roles[i-2200]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['assessed_ranges']==x['fresh_personal_body_reads']==[[1,n]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(348,[2200,2201]),(349,[2202,2203,2204]),(350,[2205,2206,2207,2208,2209])]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2200]['lines']]] and row['sha256']==roles[i-2200]['sha256']
data=json.loads(records['adler/READ-L11-350.json']);d=next(x['disposition'] for x in data['role_dispositions'] if x['source_indices']==[2207]);assert 'Filletsline.2 mayconsumeroom' in d
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (140,2274,2275,290,291,1817,1818):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
entry=next(x for x in ledger if x['id']=='ADLER-L11-STAGED-READBACK-IDENTITY');assert old[140]['contract_id']==entry['id'] and entry['record']==name and entry['repair']=='not started' and entry['validation']=='not performed' and 'baseline-059/queue.json#/entries/2274' in entry['historical_related_records'] and 'baseline-059/queue.json#/entries/2275' in entry['historical_related_records']
orn=next(x for x in ledger if x['id']=='REVIEW-ORNAMENT-FILLET-WAVE-ROOM-95E');assert name in orn['additional_current_records'] and 'Original Adler350' in orn['relationship_qualification'];prior=load('ASSESSMENT-ORNAMENT-TEXTURE-95E.json');assert 'Original Adler350 already noted' in prior['historical_attribution_qualification']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical staging family, scoped source attribution and corrected ornament precedent; no product execution.')
