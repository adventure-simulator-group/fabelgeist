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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HAIR-GROWTH-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2285.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2285,2286))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert len(bodies[2285].splitlines())==400 and [x['lines'] for x in roles]==[400]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
follicles=bodies[2285];head=sources['crates/fabelgeist-hair/src/head.rs'];strands=sources['crates/fabelgeist-hair/src/strands.rs'];hairdo=sources['crates/fabelgeist-hair/src/hairdo.rs'];orient=sources['crates/fabelgeist-hair/src/ops/orient.rs'];packed=sources['crates/fabelgeist-hair/src/gpu/guides/packed.rs'];wgsl=sources['crates/fabelgeist-hair/src/gpu/wgsl/groom.rs'];units=sources['crates/fabelgeist-math/src/units.rs'];look=sources['crates/fabelgeist-hair/src/look.rs'];vec=sources['crates/fabelgeist-math/src/vector/vec3.rs']
for literal in (b'Growth::Whorl => vec![Whorl::new',b'Growth::Radial => vec![Whorl::new',b'Growth::DoubleWhorl => vec![',b'Growth::Cowlick => {',b'Growth::Forward | Growth::Backward | Growth::Down => Vec::new()',b'pub struct GrowthField',b'pub growth: Growth',b'pub whorls: Vec<Whorl>',b'_ => down',b'v - n * v.dot(n)'):assert literal in follicles
assert b'field.direction(root, normal)' in hairdo and b'safe_normalize(normals[s], Vec3::up())' in orient and b'self.field.direction(root, n)' in orient and b'pub fn safe_normalize(v: Vec3, fallback: Vec3)' in strands and b'.normalize()' in head
assert b'field.whorls.len() as u32' in packed and b'.position(|g| *g == field.growth)' in packed and b'.unwrap_or(0) as u32' in packed
for i in range(7):assert (b'case '+str(i).encode()+b'u' in wgsl) or (i==3 and b'case 0u, 3u' in wgsl)
assert b'pub struct Metres(f32)' in units and b'pub struct Ratio(f32)' in units and b'pub const fn from_metres(value: f32)' in units and b'pub const fn from_ratio(value: f32)' in units and b'pub variation: Ratio' in look and b'pub clump_size: Metres' in look
assert b'Self::new(0.0, -1.0, 0.0)' in vec and b'Self::new(1.0, 0.0, 0.0)' in vec
# Symbolic rational computation for a non-unit input; no product function runs.
from fractions import Fraction as F
n=(F(0),F(2),F(0));v=(F(35,100),F(-1),F(0));dot=lambda a,b:sum(x*y for x,y in zip(a,b));tangent=lambda x:tuple(xi-ni*dot(x,n) for xi,ni in zip(x,n));first=tangent(v);last=tangent(first);assert first==(F(35,100),F(3),F(0)) and last==(F(35,100),F(-9),F(0)) and dot(last,n)==-18
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==23
adler=json.loads(records['adler/READ-L11-364.json']);x=next(r for r in adler['sources'] if r['original_index']==2285);assert x['fresh_reads']==x['scope']==[[1,400]] and x['sha256']==roles[0]['sha256'] and adler['accepted_unique_count_change']==0 and not adler['current_reconciled']
delegate=json.loads(records['segovb/READ-L12-750.json']);x=next(r for r in delegate['rows'] if r['original_index']==2285);assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,400]] and x['source_sha256']==roles[0]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and delegate['root_personal_rereads']==[] and delegate['accepted_unique_count_change']==0
# A later separately registered read may advance a role without changing the
# earlier connected-support attribution recorded in this checkpoint.
for later_index in (2286,2288):
 row=work['roles'][later_index]
 if row['status']!='unassessed':
  assert row['status']=='full-historical-body-trace-open'
  later=load(row['historical_assessment'])
  item=next(x for x in later['historical_source_roles'] if x['original_index']==later_index)
  assert item['prior_role_status']=='unassessed' and item['local_registration']['path']

q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert q[1835]['contract_id']==old[1835]['contract_id'] and q[1835]['current_reconciliation'].startswith('partial ') and name in q[1835]['current_reconciliation']
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed';p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified hair growth provenance, qualified source relationships and symbolic normal example; no product execution.')
