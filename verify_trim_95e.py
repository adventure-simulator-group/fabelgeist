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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-TRIM-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2220-2223.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2220,2224))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==(i!=2223) and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert len(bodies[2223].splitlines())==278 and b'textile: None' in bodies[2223]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
trim=sources['crates/fabelgeist-armor/src/trim.rs'];edges=bodies[2220];finish=bodies[2221];split=bodies[2222];fixtures=bodies[2223];embedded=sources['crates/fabelgeist-armor/src/tiling/embedded.rs'];bracer=sources['crates/fabelgeist-armor/src/gpu/bracer.rs'];fit=sources['crates/adventuresim-character-creator/src/device_fit.rs'];targets=sources['crates/adventuresim-character-creator/src/character_morphs.rs'];export=sources['crates/adventuresim-character-creator/src/export/morphs.rs'];dec=sources['crates/adventuresim-character-creator/src/decoration.rs'];inventory=sources['crates/adventuresim-character-creator/src/inventory.rs'];engraving=sources['crates/fabelgeist-armor/src/engraving.rs']
assert b'normal_deltas: deltas(&normals, &morph_normals)' in embedded and b'normal_deltas: deltas(&normals, &target_normals)' in bracer and b'normal_deltas: deltas(&base.normals, &endpoint.normals)' in fit
assert b'let normal = lerp(armor.normals[a], armor.normals[b], t)' in split and b'normal.map(|x| x / length)' in split and b'&mut morph.normal_deltas' in split and b'values.push(lerp(values[a], values[b], t))' in split
assert b'normal_deltas: &target.normal_deltas' in targets and b'append_accessor(target.normal_deltas, false' in export and b'normal_deltas: vec![[0.0; 3]; count]' in fixtures
# At t=1/2, the unnormalized base has squared length 1/2. Its normalized
# x coordinate exceeds 1/2 because (1/2)^2 < 1/2; the interpolated delta is
# -1/2. Thus the reconstructed x component is nonzero in ideal arithmetic.
assert Fraction(1,2)**2<Fraction(1,2)
assert b'!self.width.is_finite()' in finish and b'Invalid trim width' in finish and b'self.metal.validate()' in finish and b'self.trim.as_ref().map_or(Ok(()), Trim::validate)' in dec and b'decoration.validate().map_err(anyhow::Error::msg)' in dec and b'.decoration\n' in inventory
assert b'self.period.is_finite()' in trim and b'self.period > 0.0' in trim and b'(length / band.period).round().max(1.0)' in edges and b'periods * band.period / length' in edges and b'span[0] * scale' in edges
assert b'pub const MIN_TILES: f32 = 0.25' in engraving and b'pub const MAX_TILES: f32 = 64.0' in engraving and b'1.0 / (Metal::TILES_PER_METRE * self.tiles())' in finish
# 4 / 2^-126 = 2^128 exceeds binary32's finite maximum (2^24-1)*2^104.
assert 1<<128>((1<<24)-1)*(1<<104) and Fraction(1,4*64)==Fraction(1,256) and Fraction(1,4*Fraction(1,4))==1
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
path='segovb/READ-L12-737.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-737.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for i in range(2220,2224):
 x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2220]['lines'];assert x['source_sha256']==roles[i-2220]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]]
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
adler=json.loads(records['adler/READ-L11-353.json']);assert adler['accepted_unique_count_change']==0
for i in range(2220,2224):
 row=next(x for x in adler['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2220]['lines']]] and row['sha256']==roles[i-2220]['sha256']
assert any('tinyTrimBandperiodoverflow' in x['disposition'] for x in adler['role_dispositions']) and any('morphnormalcorrespondence' in x['disposition'] for x in adler['role_dispositions'])
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (143,292,1821,1822):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical trim attribution, current source traces and symbolic conditions; no product execution.')
