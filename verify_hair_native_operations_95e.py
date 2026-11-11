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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-HAIR-NATIVE-OPERATIONS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2302-2305.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2302,2306))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in (2302,2303,2304,2305)]==[129,256,99,81] and [x['lines'] for x in roles]==[129,256,99,81]
orient,settle,shrink,trim=(bodies[i] for i in (2302,2303,2304,2305))
for literal in (b'pub fn equilibrium(&self, rest: &[Vec3]',b'solve(rest, load, self, collider)',b'let segments = n - 1',b'if magnitude < 1e-6'):assert literal in settle
assert settle.index(b'let segments = n - 1')<settle.index(b'if magnitude < 1e-6') and b'if self.relative && (load - reference).length() < 1e-4' in settle
assert b'if self.memory <= 0.0' in orient and b'for p in strand.iter_mut().skip(1)' in orient
assert b'j == 0 && self.lock_roots' in shrink and b'if self.factor >= 1.0' in shrink
assert b'if cut.len() < 2' in trim and trim.index(b'if cut.len() < 2')<trim.index(b'resample_into(cut, scratch, strand)')
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for x in a['recovered_historical_records']:
 original=lookup[x['original_path']];raw=contents(x['preserved_bytes']);assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==x['published_sha256'] and len(raw)==x['published_bytes']
 if x['redacted_local_path_occurrences']:assert raw.count(b'<LOCAL_PATH_')==x['redacted_local_path_occurrences'] and 'pinned complete059 transport' in x['original_recovery']
 else:assert sha(raw)==x['sha256'] and oid('blob',raw)==x['git_blob']
 records[x['original_path']]=raw
adler=json.loads(records['adler/READ-L11-369.json']);verification=json.loads(records['adler/VERIFY-L11-369.json']);assert adler['accepted_unique_count_change']==0 and verification['head']=='b97bcedc262f91de07635c69cd1a4fc4f9a7d13f' and verification['head']!=pr and verification['head']!=rev
for i,n,batch in ((2302,129,758),(2303,256,758),(2304,99,759),(2305,81,759)):
 x=next(x for x in adler['sources'] if x['original_index']==i);d=json.loads(records[f'segovb/READ-L12-{batch}.json']);y=next(x for x in d['rows'] if x['original_index']==i);assert x['scope']==x['fresh_reads']==y['assessed_ranges']==y['fresh_personal_body_reads']==[[1,n]] and x['sha256']==y['source_sha256']==roles[i-2302]['sha256'] and d['root_personal_rereads']==[] and d['accepted_unique_count_change']==0
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert q[374]['contract_id']==old[374]['contract_id'] and q[374]['current_reconciliation'].startswith('partial ') and name in q[374]['current_reconciliation'] and name in q[381]['further_current_evidence']
for i in (374,381):
 row=next(x for x in ledger if x['id']==old[i]['contract_id']);assert row['repair']=='not started' and row['validation']=='not performed' and (row['record']==name if i==374 else name in row['further_current_evidence'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and a['repair']=='not started' and a['validation']=='not performed'
print('Verified native hair operation source, historical 369/758/759 scopes and empty-rod record; no product execution.')
