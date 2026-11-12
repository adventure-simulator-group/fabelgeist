#!/usr/bin/env python3
"""Read-only public transport and source-evidence verification."""
import argparse,datetime,gzip,hashlib,json,subprocess,tarfile,io
from pathlib import Path
root=Path(__file__).resolve().parent;p=root/'evidence';h=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();load=lambda n:json.loads((p/n).read_bytes());numbers=lambda r:{i for a,z in r for i in range(a,z+1)}
ap=argparse.ArgumentParser();ap.add_argument('--git-repo');ap.add_argument('--extraction');a=ap.parse_args()
m=json.loads((root/'PUBLIC-MANIFEST.json').read_bytes());actual={f.relative_to(root).as_posix() for f in root.rglob('*') if f.is_file()}-{'PUBLIC-MANIFEST.json'};assert len(m['files'])==m['file_count'] and actual=={x['path'] for x in m['files']}
for x in m['files']:
 f=root/x['path'];assert not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(x['bytes'],x['sha256'])
t=json.loads((root/'TRANSPORT-MAP.json').read_bytes());entries=t['entries'];assert len(entries)==t['file_count']==79 and t['content_transformations']==0 and len({x['original_path'] for x in entries})==79
for x in entries:
 assert x['public_path']=='evidence/'+x['original_path']+('.txt' if x['original_path'].endswith('.py') else '');b=(root/x['public_path']).read_bytes();assert (len(b),h(b))==(x['public_bytes'],x['public_sha256'])==(x['original_bytes'],x['original_sha256'])
original=load('MANIFEST.json');assert h((p/'MANIFEST.json').read_bytes())==t['original_manifest_sha256']=='3dd7c0e19743c1344c04731a2c6e5ee0a7d4acbd8f2574e15d01e31886da9f0a';oldrows={x['original_path']:x for x in entries};assert set(oldrows)=={x['path'] for x in original['files']}|{'MANIFEST.json'}
for x in original['files']:
 y=oldrows[x['path']];assert (x['bytes'],x['sha256'])==(y['original_bytes'],y['original_sha256'])
scan=json.loads((root/'PUBLIC-PRIVACY-SCAN.json').read_bytes());assert scan['forbidden_digest_total']==0 and not scan['other_candidate_counts'] and not scan['exceptions']
reg=load('REGISTRATION.json');rec=load('READ-RECEIPT.json');assert rec['registration_sha256']==h((p/'REGISTRATION.json').read_bytes()) and datetime.datetime.fromisoformat(reg['registered_utc'])<datetime.datetime.fromisoformat(rec['recorded_utc']) and len(reg['entries'])==len(rec['reads'])==10
views={};lines=[]
for e,v in zip(reg['entries'],rec['reads']):
 b=(p/e['archive']).read_bytes();assert e['tag']==v['tag'] and e['revision']==v['revision'] and (len(b),h(b),blob(b),len(b.splitlines()))==(e['bytes'],e['sha256'],e['git_blob'],e['eof']) and v['sha256']==e['sha256'];seen=numbers(v['displayed_chunks']);unread=numbers(v['unread_complement']);assert seen==numbers(e['ranges']) and not seen&unread and seen|unread==set(range(1,e['eof']+1));views[e['tag']]=seen;body=b.decode().splitlines();lines.append('=== '+e['tag']+' ===')
 for lo,hi in v['displayed_chunks']:
  for n in range(lo,hi+1):lines.append(f'{n:4d} {body[n-1]}')
display=('\n'.join(lines)+'\n').encode();assert display==(p/'DISPLAY.txt').read_bytes() and h(display)==rec['display_sha256']
for i in (2650,2651):assert (p/f'source/{i}-historical-f82.rs').read_bytes()==(p/f'source/{i}-current-95e.rs').read_bytes()
previous=load('prior/PRIOR-INDEX.json');priorraw=(p/'prior/MANIFEST.json').read_bytes();assert h(priorraw)==previous['original_manifest_sha256'];priorrows={x['path']:x for x in json.loads(priorraw)['files']}
for x in previous['selected_members']:
 b=(p/x['original_archive']).read_bytes();r=priorrows[x['path']];assert (len(b),h(b))==(r['bytes'],r['sha256']) and h(b)==x['original_sha256'];assert x['equal'] and x['original_sha256']==x['published_sha256']
ci=(p/'prior/checkpoint20-index.json').read_bytes();assert h(ci)==previous['checkpoint_index_sha256'];hist=load('history/HISTORY-INDEX.json');transport=(p/'history/059-transport-manifest.json.gz').read_bytes();assert h(transport)==hist['transport_manifest_sha256'];idx={x['path']:x for x in json.loads(gzip.decompress(transport))['entries']};assert len(hist['records_059'])==13
for x in hist['records_059']:
 b=(p/x['archive']).read_bytes();z=idx[x['original_path']];assert (len(b),h(b),blob(b))==(x['bytes'],x['sha256'],x['git_blob'])==(z['bytes'],z['sha256'],z['git_blob'])
 if a.extraction:assert (Path(a.extraction)/x['original_path']).read_bytes()==b
registry=gzip.decompress((p/hist['registry']['archive']).read_bytes());assert (len(registry),h(registry),blob(registry))==(hist['registry']['bytes'],hist['registry']['sha256'],hist['registry']['git_blob']);doc=json.loads(registry);selected=load('history/REGISTRY-SELECTION.json');assert all(doc['files'][i]==selected['roles'][str(i)] for i in (2650,2651)) and selected['pr_working']['head']=='6533e444fa37552927070cc8d8190907fd3b413e' and len(selected['pr_working']['status_porcelain'])==4
if a.extraction:assert (Path(a.extraction)/'evidence/audit05/original-scope-072.json').read_bytes()==registry
raw774={}
for name,x in hist['source_774'].items():
 b=gzip.decompress((p/x['archive']).read_bytes());assert (len(b),h(b),blob(b))==(x['bytes'],x['sha256'],x['git_blob']) and h((p/x['archive']).read_bytes())==x['stored_sha256'];raw774[name]=b
files={'registration':'source-range-registration.jsonl','additional':'additional-contracts.jsonl','findings':'findings.jsonl'}
for x in hist['selected_774']:
 row=raw774[files[x['collection']]].splitlines()[x['row_1based']-1];assert json.loads(row)==x['record'] and x['record']['id']==x['id']
assert len(hist['selected_774'])==5
q=load('history/QUEUE-SELECTION.json');assert [z['index_zero_based'] for z in q['entries']]==[435,479,480] and q['original_queue_sha256']!=q['later_queue_sha256'] and q['selected_equal']
if a.extraction:
 raw=(Path(a.extraction)/'workflow-20261030/CLOSURE-QUEUE-100.json').read_bytes();assert (len(raw),h(raw),blob(raw))==(q['original_queue_bytes'],q['original_queue_sha256'],q['original_git_blob']);docq=json.loads(raw)
 for x in q['entries']:assert x['entry']==docq['entries'][x['index_zero_based']]
l11=load('history/059/segovb/READ-L11-433.json');l12=load('history/059/segovb/READ-L12-914.json');assert l12['root_personal_rereads']==[]
for role,eof in [(2650,54),(2651,19)]:
 x=next(z for z in l11['sources'] if z['original_index']==role);y=next(z for z in l12['rows'] if z['original_index']==role);assert x['fresh_personal_body_reads']==y['fresh_personal_body_reads']==[[1,eof]] and '/root/lens12_frontier' in y['attribution'] and not any(k in x for k in ('reader','reviewer','actor','author'))
proof=load('TREE-PROOFS.json');objects={}
for oid,x in proof['objects'].items():
 b=gzip.decompress((p/x['archive']).read_bytes());assert (len(b),h(b),h((p/x['archive']).read_bytes()))==(x['bytes'],x['sha256'],x['stored_sha256']) and hashlib.sha1(b'tree '+str(len(b)).encode()+b'\0'+b).hexdigest()==oid;objects[oid]=b
def entries(b):
 result={};i=0
 while i<len(b):
  sp=b.index(b' ',i);end=b.index(b'\0',sp);result[b[sp+1:end].decode()]=(b[i:sp].decode(),b[end+1:end+21].hex());i=end+21
 return result
for x in proof['proofs']:
 cur=proof['roots'][x['revision']];assert cur==x['root_tree']
 for name,z in zip(x['path'].split('/'),x['chain']):
  assert z['tree']==cur and z['name']==name;mode,oid=entries(objects[cur])[name];assert mode==z['mode'] and oid==z['object'];cur=oid
 b=(p/x['archive']).read_bytes();b=gzip.decompress(b) if x['encoding']=='gzip' else b;assert (len(b),h(b),blob(b))==(x['bytes'],x['sha256'],x['git_blob']) and cur==x['git_blob']
assert len(proof['proofs'])==16
report=load('REPORT.json');assert len(report['thirteen_lens_perspectives'])==13
for x in report['observations']:
 for z in x['trace']:assert numbers(z['ranges'])<=views[z['tag']]
b=load('history/BASELINE-SELECTION.json');assert b['committed_sha256']!=b['working_sha256'] and all(v['committed']==v['working_observed'] for v in b['rows'].values())
if a.git_repo:
 git=lambda *x:subprocess.check_output(['git','-C',a.git_repo,'--no-replace-objects',*x])
 for rev,root in proof['roots'].items():assert git('rev-parse',rev+'^{tree}').decode().strip()==root
 raw=git('show',previous['checkpoint_revision']+':DELEGATED-EVIDENCE-CHECKPOINT-20.json');assert raw==ci;archive=git('show',previous['checkpoint_revision']+':'+previous['published_archive_path']);assert h(archive)==previous['published_archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
  for x in previous['selected_members']:assert h(t.extractfile(x['path']).read())==x['published_sha256']
 committed=git('show',b['committed_revision']+':BASELINE-95E-20261110.json');assert (len(committed),h(committed),blob(committed))==(b['committed_bytes'],b['committed_sha256'],b['committed_git_blob']);entries=json.loads(committed)['newly_assessed_ledger']['entries']
 for k,v in b['rows'].items():assert next(x for x in entries if x['id']==k)==v['committed']
 qlater=json.loads(git('show',q['later_committed_revision']+':baseline-059/queue.json'))
 for x in q['entries']:assert qlater['entries'][x['index_zero_based']]==x['entry']
print('PASS: manifest, 10 registered personal scopes, 16 tree paths, 13 original 059 records, 5 supplier rows, selected historical/current identities, prior attribution and 13 bounded perspectives'+('; trusted Git checked' if a.git_repo else '')+('; user extraction checked' if a.extraction else '')+'. No product behavior validated.')
