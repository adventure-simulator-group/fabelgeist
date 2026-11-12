#!/usr/bin/env python3
"""Read-only evidence integrity checks; no product or historical verifiers."""
from pathlib import Path
import gzip,hashlib,json,tarfile
b=Path(__file__).resolve().parent;read=lambda p:(b/p).read_bytes();sha=lambda d:hashlib.sha256(d).hexdigest();oid=lambda t,d:hashlib.sha1(t.encode()+b' '+str(len(d)).encode()+b'\0'+d).hexdigest();js=lambda p:json.loads(read(p));jgz=lambda p:json.loads(gzip.decompress(read(p)))
records=[]
for n in ['personal-read-receipt.json','supplement-read-receipt.json','boot-read-receipt.json','root-read-receipt.json','warmup-read-receipt.json']:
 x=js(n);records.append((x['registration'],x['registration_sha256'],x['recorded_utc'],x['reads']))
rr={};entries=[]
for regfile,regsha,at,reads in records:
 reg=js(regfile);assert sha(read(regfile))==regsha and at>=reg['registered_utc'];by={x['id']:x for x in reads};assert len(by)==len(reg['entries'])
 for e in reg['entries']:
  d=read(e['archive']);x=by[e['id']];assert (len(d),sha(d),len(d.splitlines()),oid('blob',d))==(e['bytes'],e['sha256'],e['eof'],e['git_blob']);assert x['sha256']==e['sha256'] and x['eof']==e['eof'] and x['actual_display_ranges']==e['prospective_ranges'];yes={i for a,z in x['actual_display_ranges'] for i in range(a,z+1)};no={i for a,z in x['unread_complement'] for i in range(a,z+1)};assert not yes&no and yes|no==set(range(1,e['eof']+1));rr[e['id']]=x;entries.append(e)
report=js('REPORT.json');assert len(report['bounded_lens_perspectives'])==13
for a in report['observations']:
 for ref in a['references']:
  covered={i for x,y in rr[ref['source']]['actual_display_ranges'] for i in range(x,y+1)};assert all(i in covered for x,y in ref['ranges'] for i in range(x,y+1))
gp=js('git-proofs.json');raw={}
for key,x in gp['objects'].items():
 d=gzip.decompress(read(x['preserved_bytes']));assert (len(d),sha(d),oid(x['type'],d))==(x['bytes'],x['sha256'],key);raw[key]=d
for rev,x in gp['commits'].items():assert raw[rev].splitlines()[0]==b'tree '+x['root_tree'].encode()
def ents(d):
 z={};i=0
 while i<len(d):
  j=d.index(b'\0',i);m,n=d[i:j].split(b' ',1);z[n.decode()]=(m.decode(),d[j+1:j+21].hex());i=j+21
 return z
for x in gp['paths']:
 d=read(x['preserved_bytes']);d=gzip.decompress(d) if x['preserved_encoding']=='gzip' else d;assert (len(d),sha(d),oid('blob',d))==(x['bytes'],x['sha256'],x['git_blob']);cur=gp['commits'][x['revision']]['root_tree'];names=[]
 for y in x['path_chain']:
  assert cur==y['tree'];mode,child=ents(raw[cur])[y['name']];assert (mode,child)==(y['mode'],y['object']);cur=child;names.append(y['name'])
 assert '/'.join(names)==x['path'] and cur==x['git_blob']
for e in entries:assert any(x['revision']==e['revision'] and x['path']==e['path'] and x['git_blob']==e['git_blob'] for x in gp['paths'])
registry=jgz('history/original-registry.json.gz');queue=jgz('history/baseline-059-queue.json.gz');ledger=jgz('history/baseline-059-ledger.json.gz');transport=jgz('history/original-transport-manifest.json.gz');idx={x['path']:x for x in transport['entries']}
for path,file in [('evidence/audit05/original-scope-072.json','history/original-registry.json.gz'),('workflow-20261030/CLOSURE-QUEUE-100.json','history/baseline-059-queue.json.gz'),('evidence/continuation-segovb/ledger.json','history/baseline-059-ledger.json.gz')]:
 d=gzip.decompress(read(file));assert (len(d),sha(d),oid('blob',d))==(idx[path]['bytes'],idx[path]['sha256'],idx[path]['git_blob'])
s=js('history/selections-reconstructed.json')
for i,v in s['registry_roles'].items():assert v==registry['files'][int(i)]
for i,v in s['queue_entries'].items():assert v==queue['entries'][int(i)]
for path,v in s['ledger_pointers'].items():
 x=ledger
 for y in path.split('/')[1:]:x=x[int(y)] if isinstance(x,list) else x[y]
 assert v==x
for n,f in [('registry_sha256','history/original-registry.json.gz'),('queue_sha256','history/baseline-059-queue.json.gz'),('ledger_sha256','history/baseline-059-ledger.json.gz')]:assert s[n]==sha(gzip.decompress(read(f)))
c=js('history/published/BASELINE-95E-20261110.json');w=js('history/published/working-baseline-observed.json');assert s['current_baseline_sha256']==sha(read('history/published/BASELINE-95E-20261110.json')) and s['working_baseline_observed_sha256']==sha(read('history/published/working-baseline-observed.json')) and s['current_baseline_sha256']!=s['working_baseline_observed_sha256'];assert c['source_revision']==w['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
for field,doc in [('current_selected_entries',c),('working_selected_entries',w)]:
 for id,x in s[field].items():assert doc['newly_assessed_ledger']['entries'][x['index']]==x['entry'] and x['entry']['id']==id
assert s['current_selected_entries']==s['working_selected_entries']
assert read('sources/garment-old')==read('sources/garment-now') and read('sources/shape-old')==read('sources/shape-now')
for i,id in [(2623,'garment-old'),(2624,'shape-old')]:assert registry['files'][i]['sha256']==rr[id]['sha256']
a=js('history/records.json')['records']
for x in a:
 if x['kind']=='059-original':
  d=read(x['archive']);z=idx[x['path']];assert (len(d),sha(d),oid('blob',d))==(x['bytes'],x['sha256'],x['git_blob'])==(z['bytes'],z['sha256'],z['git_blob'])
 elif x['kind']=='774-f82':
  d=gzip.decompress(read(x['archive']));assert (len(d),sha(d),oid('blob',d))==(x['bytes_uncompressed'],x['sha256_uncompressed'],x['git_blob']);assert any(y['revision']==x['transport_revision'] and y['path']==x['transport_path'] and y['git_blob']==x['git_blob'] for y in gp['paths'])
 elif x['kind']=='prior-assessment':
  d=read(x['archive']);assert (len(d),sha(d))==(x['bytes'],x['sha256'])
 else:raise AssertionError(x['kind'])
m=js('history/prior-role/MANIFEST.json');mi={x['path']:x for x in m['files']}
for x in a:
 if x['kind']=='prior-assessment' and not x['archive'].endswith('MANIFEST.json'):
  y=mi[x['archive'].split('/')[-1]];assert (x['bytes'],x['sha256'])==(y['bytes'],y['sha256'])
root=jgz('history/history-774/admission03-scope-reconciliation-044.json.gz');pins=root['input_pins'];rp=next(x for x in pins if x.get('supplier')=='rig' and x.get('rows')==13);qp=next(x for x in pins if x.get('supplier')=='rig' and x.get('kind')=='qualification');registration=gzip.decompress(read('history/history-774/source-range-registration.jsonl.gz'));qualification=gzip.decompress(read('history/history-774/source-qualifications.jsonl.gz'));assert sha(registration)=='24fb01a82c91de97fd9895740404dc79a73877fcd79a846c832c45cbf2495bdb';assert sha(b''.join(registration.splitlines(keepends=True)[:13]))=='442704cfbcdb6f971a06981c68f1442451ccbd9eb7e039dfda32e5d639a772d9'!=rp['sha256']=='c75d93513422266bff64a096b243766fbd002d346f1d8c3c5b58f54af25ba06c';assert sha(b''.join(qualification.splitlines(keepends=True)[:13]))==qp['sha256']=='70717aa1c0908935341afa5297937f1bf79513146453c1328076848d00e94e53';assert jgz('history/history-774/completion-112.json.gz')['pins'][4]['sha256']=='698ec787519f08585b75f3ce7f2fd08f0e8793f4f4837f91d88151c2c7783db1'!=sha(registration)
reglines=[json.loads(x) for x in registration.splitlines()];addlines=[json.loads(x) for x in gzip.decompress(read('history/history-774/additional-contracts.jsonl.gz')).splitlines()]
for n,role,id in [('072',2623,'garment-old'),('073',2624,'shape-old')]:
 x=next(x for x in reglines if x['id']=='RIG-ADMISSION03-REGISTER-'+n);y=next(x for x in addlines if x['id']=='RIG-ADMISSION03-ADDITIONAL-'+n);assert x['sha256']==y['sha256']==registry['files'][role]['sha256']==rr[id]['sha256'];actual={j for i,k in rr[id]['actual_display_ranges'] for j in range(i,k+1)};old={j for i,k in x['material_ranges'] for j in range(i,k+1)};assert actual==old and x['material_ranges']==y['actual_source_ranges']
wrapped=next(x for x in addlines if x['id']=='RIG-ADMISSION03-ADDITIONAL-074');assert wrapped['actual_source_ranges']==[[1,54]] and 'TassetSide55+' in wrapped['admission_comparison']
contracts=[json.loads(x) for x in gzip.decompress(read('history/history-774/contract-groups.jsonl.gz')).splitlines()];assert next(x for x in contracts if x['id']=='RIG-ADMISSION03-CONTRACT-610')['actual_source_ranges']==[[1,154]]
p16=js('history/p16-recovery.json');assert sha(read('history/p16-tracked.patch'))==p16['recovered_patch']['sha256']=='fc9002ab1135a438f68eede9217f67d22ef737d2c6938509e6e6660f2fc5b7f7'
pub=js('history/prior-role/published-comparison.json');ck=js('history/published/DELEGATED-EVIDENCE-CHECKPOINT-17.json');entry=next(x for x in ck['bundles'] if x['bundle_id']=='armor-garment-shape');archive=read('history/prior-role/published-evidence.tar.gz');assert sha(archive)==pub['archive_sha256']==entry['archive_sha256'] and entry['scope_original_roles']==[2623,2624]
with tarfile.open(b/'history/prior-role/published-evidence.tar.gz','r:gz') as t:
 for x in pub['comparisons']:
  old=read('history/prior-role/'+x['member']);new=t.extractfile(x['member']).read();assert sha(old)==x['original_sha256'] and sha(new)==x['published_sha256'] and (old==new)==x['byte_identical']
print(json.dumps({'status':'passed','registered_read_entries':len(entries),'git_objects':len(gp['objects']),'git_path_proofs':len(gp['paths']),'historical_or_prior_records':len(a),'registry_roles':[2623,2624],'historical_queue_indices':[1817,1818],'published_member_comparisons':len(pub['comparisons']),'product_execution':'none; evidence-only integrity checks','writes':'none'},indent=2))
