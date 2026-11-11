#!/usr/bin/env python3
"""Verify historical/base/current source evidence; no product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-GENERATION-IDENTITY-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1535-1540.json');w=load('COVERAGE-L01-ROLES.json');proof=load('GENERATION-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
def check_chain(path,p,expected):
 assert [s['name'] for s in p['chain']]==path.split('/');curtree=proof['root_tree']
 for step in p['chain']:
  info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
  assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
 assert curtree==p['git_blob']==oid('blob',expected) and p['sha256']==sha(expected)
assert [x['original_index'] for x in h['historical_source_roles']]==[1535,1536,1538,1539,1540] and [x['lines'] for x in h['historical_source_roles']]==[60,103,54,320,88]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1535-1540.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert raw==cur and c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur;check_chain(c['path'],proof['paths'][c['path']],raw)
d=load('ASSESSMENT-L01-DELETED-1537.json');role=w['roles'][1537];registry=json.loads(contents(w['registry']))['files'][1537];assert d['original_index']==1537 and registry['state']=='deleted' and registry['sha256'] is None and role['historical_sha256'] is None and not role['historical_object_available'] and role['status']=='deleted-base-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-DELETED-1537.json' and d['snapshot_sha256'] is None
base=(r/d['preserved_base_bytes']).read_bytes();assert sha(base)==d['base_sha256']==registry['base_sha256'] and len(base)==d['base_bytes'] and len(base.splitlines())==d['base_lines']==111 and oid('blob',base)==d['base_git_blob'] and d['new_personal_base_read_ranges']==[[1,111]]
b=contents(d['commit_proof']['preserved_bytes']);assert sha(b)==d['commit_proof']['sha256'] and len(b)==d['commit_proof']['bytes'] and oid('commit',b)==d['base_revision']=='3a50b708381ad00c116445c2c999a49c3943088b';node=b.splitlines()[0].split()[1].decode()
for part,step in zip(registry['path'].split('/'),d['tree_proof']):
 b=contents(step['preserved_bytes']);assert sha(b)==step['sha256'] and len(b)==step['bytes'] and oid('tree',b)==step['tree']==node and part==step['component'];assert entries(b)[part]==(step['entry_mode'],step['entry_oid']);node=step['entry_oid']
assert node==oid('blob',base) and registry['path'] not in tree
absence=proof['frozen_absence'];node=proof['root_tree']
for step in absence['parent_chain']:
 info=nodes[node];b=contents(info['preserved_bytes']);assert sha(b)==info['sha256'] and oid('tree',b)==node==step['tree'] and entries(b)[step['name']]==(step['mode'],step['child']);node=step['child']
info=nodes[node];b=contents(info['preserved_bytes']);assert node==absence['parent_tree'] and oid('tree',b)==node and absence['missing_name'] not in entries(b) and absence['path']==registry['path']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
successor='crates/adventuresim-tactical-client/src/presentation/preparation.rs';check_chain(successor,proof['supporting_paths'][successor],s[successor]);assert sha(s[successor])=='35db8b8b632500bacb59de9fffe065979e5da709c52dbaf0ea4d80ae6847af12' and b'VistaMesh(#[from] VistaMeshError)' in s[successor] and b'VistaMesh(' not in base
city=current[1536];install=current[1538];ownership=current[1539];requests=current[1540];residency=s['crates/adventuresim-tactical-client/src/presentation/generation/residency.rs'];wasm=s['crates/adventuresim-tactical-client/src/presentation/generation/wasm.rs'];worker=s['crates/adventuresim-tactical-client/src/presentation/generation/worker_products.rs'];scene=s['crates/adventuresim-tactical-client/src/presentation/generation/scene.rs'];venue=s['crates/adventuresim-tactical-client/src/presentation/generation/venue.rs'];mapinstall=s['crates/adventuresim-tactical-client/src/presentation/regional_map/city/installation.rs'];js=s['crates/strategic-web/static/strategic-generation.js'];pool=s['crates/strategic-web/static/strategic-generation-pool.js'];request=s['crates/strategic-web/static/regional-city-request.js']
assert b'self.document == *document && self.graphics == graphics' in city and b'config.rendering.vista.maximum_lods' in city and b'!products' in city and b'city.matches(&document, graphics)' in city
assert b'slot.staged = slot.active.clone();' in ownership and b'slot.staged.begin_generation();' in ownership and b'self.facades.clear();' in residency and b'self.placements.clear();' in residency and b'regional_city' not in residency
assert b'.is_some_and(|city| city.document == **document)' in ownership and b'PreparationInputIdentity::RegionalCity(Box::new(document.clone()))' in ownership and b'wasm_finish_regional_city(ticket_json: &str, document_json: &str)' in wasm and b'activate_city(preparation, &document)' in mapinstall
assert b'if city.matches(&input, &graphics)' in worker and b'job.require_owner(ticket)?' in worker and b'if slot.preparing != Some(ticket)' in ownership and b'checked_add(1)' in ownership
assert js.index(b'await run(jobs);')<js.index(b'runtime.wasm_finish_regional_city(preparation, document)') and b'await prepare(preparation, run, discover)' in js and b'signal?.throwIfAborted();\n    const finishStarted' in js and b'await Promise.all(' in pool and b'await receive({' in pool and b'while (preparing)' in request
assert b'if primary != placement' in scene and b'if *placement != (*distant).into()' in scene and b'input.buildings.iter().any(|primary|' in scene and b'Some(recipe) if recipe.program == program' in venue
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(471,[1534,1535,1536]),(472,[1537]),(473,[1538,1539,1540])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
oldread=json.loads(records['segovb/READ-L12-472.json'])['rows'][0];qual=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1537-DELETION.json']);assert oldread['deleted_role_base_read'] and oldread['current_body_credit']==0 and qual['raw_current_sha256'] is None and qual['exact_base_sha256']==d['base_sha256'] and qual['canonical_base_git_revision']==d['base_revision']
adler=json.loads(records['adler/READ-L11-218.json']);check=json.loads(records['adler/VERIFY-L11-218.json']);assert adler['accepted_unique_count_change']==0 and adler['current_reconciled'] is False and adler['validated'] is False and check['receipt_sha256']==m['adler/READ-L11-218.json']['sha256']
future=next(x for x in adler['connected_reinspection'] if x.get('original_index')==1612);assert future['fresh_reads']==[[1,115]] and future['additional_original_frontier_credit']==0 and future['source_sha256']==sha(s[successor])
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1305,1306,1307));ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==1 and sum(x['id']==a['conditional_findings'][0]['id'] and x['record']=='ASSESSMENT-GENERATION-IDENTITY-95E.json' for x in ledger)==1 and a['conditional_findings'][0]['accepted_historical_unique_count_change']==0
p=load('CONTINUATION-PROGRESS-95E.json');assert d['historical_object_available'] is False and d['snapshot_sha256'] is None and 'ASSESSMENT-L01-DELETED-1537.json' in p['post_transition_deleted_base_reads'] and p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five original generation bodies, separate deleted base/absence, earlier attribution and conditional identity source; no product execution.')
