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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-ARMOR-BODY-COORD-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2179-2183.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));before={};current={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2179,2184))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical_sha256']==item['sha256'] and regrow['historical_ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]]
  assert c['byte_identical']==(old==body)==(i!=2183) and c['current_personal_ranges']==[[1,c['lines']]] and regrow['current_sha256']==c['sha256'] and regrow['current_ranges']==c['current_personal_ranges']
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==c['lines'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);before[i]=old;current[i]=body
assert b'FieldNormal' not in before[2183] and b'FieldNormal' in current[2183] and len(before[2183].splitlines())==304 and len(current[2183].splitlines())==369
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
body=current[2179];bracer=current[2180];breast=current[2181];coord=current[2183];caller=sources['crates/adventuresim-character-creator/src/device_torso.rs'];input_source=sources['crates/adventuresim-character-creator/src/bracer.rs'];cloth=sources['crates/fabelgeist-armor/src/gpu/body_cloth.rs'];clothwgsl=sources['crates/fabelgeist-armor/src/gpu/body_cloth.wgsl'];clothfield=sources['crates/fabelgeist-armor/src/gpu/body_cloth_extrusion.wgsl'];part=sources['crates/fabelgeist-armor/src/gpu/part.rs'];kernel=sources['crates/fabelgeist-armor/src/gpu/part/shell_kernels.rs'];skirt=sources['crates/fabelgeist-armor/src/gpu/skirt_extrusion.wgsl'];plan=sources['crates/adventuresim-character-creator/src/body_cloth_plan.rs']
assert b'surface.normals.len() != vertices' in body and b'surface.texcoords.len() != vertices' in body and b'gpu.upload(BufferUpload::from_elements(surface.positions))' in body and b'.is_finite()' not in body.split(b'impl GpuBody {',1)[1].split(b'/// Skin and surface coordinates',1)[0]
assert b'pub(crate) fn validate(&self)' in input_source and b'.is_finite()' not in input_source
assert b'.zip(morphs)' in bracer and b'.zip(morphs)' in breast and b'if results.status(slots[0]) != 0' in breast and b'for morph in input.morphs' in caller and b'.map(|morph| morph.name.clone())' in caller and b'breastplate.read_async(gpu, input.domain, &names)' in caller
assert b'kernel.extrusion != KernelExtrusion::Field' in coord and b'heights[attributes] = bitcast<f32>(field.kind)' in coord and b'pub(crate) const CARRIER_ATTRIBUTE_WORDS: u64 = 5' in part and b'choice == FITTED_SHELL_EXTRUSION' in kernel and b'choice != GEOMETRIC_SHELL_EXTRUSION' in kernel and b'CoordExtrusion::FieldNormal' in cloth and b'return ShellExtrusion(GEOMETRIC_SHELL_EXTRUSION' in skirt
assert b'(first..first + layer.point_count)' in cloth and b'[point as f32, layer.offset_metres' in cloth and b'u32(coord.x) * POINT_WORDS' in clothwgsl and b'u32(coord.x) * POINT_WORDS' in clothfield and b'point_count: cut.points.len() as u32' in plan
assert 2**24==16777216 and 2**24+1==16777217 and (2**24+1)-(2**24)==(2**24+2)-(2**24+1) and (2**24//2)%2==0
native=a['native_semantics_evidence']
for x in native:assert sha(contents(x['path']))==x['sha256_uncompressed']
reference=contents(native[0]['path']);primitive=contents(native[1]['path']);assert b'expr.as.numeric.int-as-float' in reference and b'roundTiesToEven' in reference and b'binary32' in primitive
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(726,range(2179,2181)),(727,range(2181,2184))]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);assert x['source_sha256']==roles[i-2179]['sha256'] and x['fresh_personal_body_reads']==[[1,roles[i-2179]['lines']]] and 'Subagent /root/lens12_frontier' in x['attribution']
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(343,range(2179,2181)),(344,range(2181,2184))]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2179]['lines']]] and row['sha256']==roles[i-2179]['sha256']
data=json.loads(records['adler/READ-L11-344.json']);row=next(x for x in data['role_dispositions'] if 2181 in x['source_indices']);assert 'Extendexisting343' in row['disposition']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert q[137]['contract_id']==old[137]['contract_id'] and q[137]['current_reconciliation'].startswith('partial ') and name in q[137]['current_reconciliation']
sharedrow=next(x for x in ledger if x['id']==old[137]['contract_id']);assert sharedrow['record']=='ASSESSMENT-ARMOR-CONSTRUCTION-95E.json' and name in sharedrow['additional_current_records'] and sharedrow['repair']=='not started'
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified original armor body attributions, shared morph finding, changed coordinate sources and casting reference; no product execution.')
