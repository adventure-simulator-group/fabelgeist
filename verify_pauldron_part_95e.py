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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-PAULDRON-PART-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2196-2199.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));oldbodies={};bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2196,2200))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i)
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=c['lines'];assert sha(old)==row['historical_sha256']==regrow['historical']['sha256'] and len(old.splitlines())==n and regrow['historical']['ranges']==item['new_personal_read_ranges']==[[1,n]]
  assert sha(body)==c['sha256']==regrow['current']['sha256'] and len(body.splitlines())==m and regrow['current']['ranges']==c['current_personal_ranges']==[[1,m]] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
  assert (old==body)==c['byte_identical']==regrow['byte_identical']==(i==2199);chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);oldbodies[i]=old;bodies[i]=body
assert roles[1]['lines']==362 and roles[1]['original_supplier_scopes'][0]['read_ranges']==[[1,160],[335,369]]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
pauldron=bodies[2199];part=bodies[2197];layout=bodies[2198];design=sources['crates/fabelgeist-armor/src/pauldron.rs'];limb=sources['crates/fabelgeist-armor/src/limb_armor.rs'];build=sources['crates/fabelgeist-armor/src/gpu/limb_build.rs'];frame=sources['crates/fabelgeist-armor/src/frame.rs'];wgsl=sources['crates/fabelgeist-armor/src/gpu/wgsl.rs'];recipe=sources['crates/fabelgeist-armor/src/gpu/recipe.rs'];fitted=sources['crates/adventuresim-character-creator/src/device_pauldron.rs'];dispatch=sources['crates/adventuresim-character-creator/src/device_fit.rs'];shell=sources['crates/fabelgeist-armor/src/gpu/shell_plan.rs'];shell_gpu=sources['crates/fabelgeist-armor/src/gpu/part/shell_kernels.rs']
assert b'(65..=125).contains(&self.arm_length.0)' in design and b'(1..=3).contains(&self.upper_lames)' in design and b'(3..=7).contains(&self.lower_lames)' in design and b'pub const THICKNESS_RANGE: std::ops::RangeInclusive<u16> = 1..=3' in design and b'!(1..=6).contains(&self.thickness.0)' in limb and b'gauge.validate()?' in limb
assert b'LimbArmorDesign::Pauldron(design.clone()).validate()?' in pauldron and b'pub fn record_plates(' in pauldron and b'0.004 / d.arm_length.metres()' in pauldron and b'Permille::ONE.0 * u16::from(i + 1) / u16::from(d.lower_lames)' in pauldron and 1000*7<=65535<1000*66
assert b'length(fit.y.xz) < 0.01' in pauldron and b'fail(STATUS_INVALID_SURFACE); return' in pauldron and b'vec3<f32>(v[6], v[7], v[8])' in wgsl and b'words[3 + axis * 3..6 + axis * 3].copy_from_slice(values)' in recipe and b'.all(|v| v.is_finite())' in frame and b'let orthogonal' in frame
assert b'return super::pauldron::DevicePauldronCarrier::record(gpu, batch, d, frame)?' in build and b'.record_plates(gpu, batch, d)' in build and b'gpu.read::<u32>(&carrier.status)?[0] != 0' in build and b'read_u32("carrier_status", &carrier.status)' in fitted and b'atomic("status", part.status())' in fitted and b'atomicOr(&status[0], carrier_status[0])' in fitted and b'wearer.record_fitted_pauldron(' in dispatch
record_plates=pauldron.split(b'    pub fn record_plates(',1)[1].split(b'\nfn chart(',1)[0];assert b'&self.frame_points' in record_plates and b'&self.status' not in record_plates and b'.validate()' not in record_plates
assert b'let final_count = plan.sources.len() as u32' in layout and b'self.final_count += final_count' in layout and b'(0..count)' in shell and b'VertexSource::Outer' in shell and b'VertexSource::Inner' in shell and b'const INNER_BIT: u32 = 1 << 31' in shell and b'let carrier = source & ~INNER_BIT' in shell_gpu and 2*(2*(1<<30))==(1<<32)
assert b'textile_coordinates: Option<Buffer>' not in oldbodies[2197] and b'textile_coordinates: Option<Buffer>' in part and b'pub texcoords: Option<Vec<[f32; 2]>>' in part
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
qual=json.loads(records['parallel-l12-20261029/QUALIFICATION-PAR-L12-ORIGINAL2197-EOF.json']);assert qual['historical_requested_range']==[335,369] and qual['actual_EOF']==362 and qual['effective_range']==[335,362] and qual['nonexistent_tail']==[363,369] and qual['nonexistent_tail_coverage']==0
reg=json.loads(records['adler/REGISTRATION-ADLER-1063.json']);assert reg['prospective'] and reg['ranges']==[[1,362]] and reg['inherited_scope_extent_qualification']['excluded_nonexistent_ranges']==[[363,369]] and reg['inherited_scope_extent_qualification']['superseded_registration_sha256']==lookup['adler/REGISTRATION-ADLER-1055.json']['sha256']
for batch,indices in [(731,[2196]),(732,[2197,2198,2199])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2196]['lines'];assert x['source_sha256']==roles[i-2196]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['assessed_ranges']==[[1,n]] and x['fresh_personal_body_reads']==([] if i==2196 else [[1,n]])
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(347,[2196,2197]),(348,[2198,2199])]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2196]['lines']]] and row['sha256']==roles[i-2196]['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (610,1816,1817,1818):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified original EOF qualification, distinct source versions, bounded shoulder admission and conditional status handoff; no product execution.')
