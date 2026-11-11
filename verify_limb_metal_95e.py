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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-LIMB-METAL-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2189-2193.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2189,2194))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['sha256']==item['sha256'] and regrow['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);assert sha(old)==row['historical_sha256'] and old==body and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,n]]
  assert c['byte_identical'] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [x['read_ranges_as_recorded'] for x in roles[0]['original_producer_scopes']]==[[[111,478]]]
assert [x['read_ranges_as_recorded'] for x in roles[1]['original_producer_scopes']]==[[[1,172]]]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
limb=bodies[2189];joint=bodies[2190];bake=bodies[2191];finish=bodies[2192];metal=bodies[2193]
design=sources['crates/fabelgeist-armor/src/limb_armor.rs'];cup=sources['crates/fabelgeist-armor/src/joint_cup_design.rs'];extension=sources['crates/fabelgeist-armor/src/joint_extension.rs'];radial=sources['crates/fabelgeist-armor/src/radial_fluting.rs'];besagew=sources['crates/fabelgeist-armor/src/besagew.rs'];material=sources['crates/fabelgeist-armor/src/material.rs'];textures=sources['crates/fabelgeist-armor/src/gpu/metal/textures.rs'];wgsl=sources['crates/fabelgeist-armor/src/gpu/metal/wgsl.rs'];buffer=sources['crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs']
assert b'design.validate()?' in limb and b'ratio(d.calf_height, 450, 800)' in design and b'(1.0 - calf)' in limb and b'if lames == 1' in joint and b'f32::from(lames - 1)' in joint
assert b'!(1..=4).contains(&self.lame_count)' in extension and b'RaisedCop' in cup and b'extension.validate()?' in cup
assert b'!(4..=48).contains(&self.count.0)' in radial and b'const SAMPLES_PER_SPOKE: usize = 12' in radial and b'fluting.validate()?' in besagew and b'const MINIMUM_COLUMNS: usize = 96' in limb and b'f.columns().max(MINIMUM_COLUMNS)' in limb
assert b'const SIZES: std::ops::RangeInclusive<u32> = 32..=1024' in bake and b'metal.validate()?' in bake and b'scratch_density > 3000' in material and b'FINISH_WORDS: u32 = 3' in finish and b'DRAWS_PER_SCRATCH: u32 = 5' in textures
assert 1024*1024==1048576 and 1048576*3*4==12582912 and 3000*5==15000
assert b'fn device_error' in metal and b'format!' in metal and b'res.map_err(|e| anyhow!("GPU Mapping error: {:?}", e))?' in buffer and b'.map_err(|_| anyhow!("GPU Mapping error"))?' in buffer
assert b'stamp.insert("seed", metal.seed)' in bake and b'parameters.insert("seed", metal.seed)' in bake and b'bake.insert("seed", metal.seed)' in bake and b'bake.insert("scratch_draws", metal.scratch_density * DRAWS_PER_SCRATCH)' in bake
assert b'scratch_density: 500' in material and b'TEXTURE_SIZE: u32 = 1024' in material and b'var state = lcg_skip(params.seed, id.y * DRAWS_PER_SCRATCH)' in textures and b'state = lcg_next(state)' in textures
assert b'const LAYER_STRIDE: u32 = 1048576u' in finish and b'layer * LAYER_STRIDE + j * cells_x + i + 1u' in finish and b'noise(0u, WAVE_CELLS' in finish and b'noise(1u, RIPPLE_CELLS' in finish and b'params.scratch_draws + index + 1u' in textures
assert b'var total_multiplier = 1u' in wgsl and b'var total_increment = 0u' in wgsl and b'if (remaining == 0u)' in wgsl and b'return seed * total_multiplier + total_increment' in wgsl and b'if ((remaining & 1u) != 0u)' in wgsl and b'total_multiplier = total_multiplier * multiplier' in wgsl and b'total_increment = total_increment * multiplier + increment' in wgsl and b'return state * LCG_MULTIPLIER + LCG_INCREMENT' in wgsl
scratch_draws=500*5;texel_index=1046076;assert scratch_draws+texel_index+1==1048576+1 and 0<=texel_index<1024*1024
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(730,[2189,2190]),(731,[2191,2192,2193])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2189]['lines'];assert x['source_sha256']==roles[i-2189]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==[[1,n]] and x['assessed_ranges']==[[1,n]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
data=json.loads(records['adler/READ-L11-346.json']);assert data['accepted_unique_count_change']==0
for i in range(2189,2194):
 row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2189]['lines']]] and row['sha256']==roles[i-2189]['sha256']
assert any(x['original_index']==2194 for x in data['sources'])
x=next(x for x in json.loads(records['segovb/READ-L12-731.json'])['rows'] if x['original_index']==2196);assert x['fresh_personal_body_reads']==[] and x['assessed_ranges']==[[1,331]]
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (136,610,1814,1815,2265,2266,2280):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'checks and propagates the outcome' in q[2266]['current_reconciliation'] and 'outcome is discarded' in q[2266]['current_reconciliation']
entry=next(x for x in ledger if x['id']=='ADLER-L11-ARMOR-ENGRAVING-ERROR-OWNERSHIP');assert name in entry['additional_current_records'] and entry['record']=='ASSESSMENT-ARMOR-CONSTRUCTION-95E.json'
assert a['historical_negative_dispositions'][0]['queue']==2280 and old[2280]['disposition']=='rejected-suspicion-source-qualified'
entry=next(x for x in ledger if x['id']=='REVIEW-METAL-SEED-DRAW-OVERLAP-95E');assert 'baseline-059/queue.json#/entries/2280' in entry['historical_related_records'] and entry['record']==name and entry['repair']=='not started' and entry['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified scoped limb attribution, selected admission, distinct readback errors and symbolic draw overlap; no product execution.')
