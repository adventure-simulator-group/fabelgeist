#!/usr/bin/env python3
"""Read-only independent checkpoint 42 evidence identity and selection review."""
import argparse,gzip,hashlib,io,json,subprocess,tarfile
from pathlib import Path,PurePosixPath
ap=argparse.ArgumentParser();ap.add_argument('stage');ap.add_argument('--git-repo');ap.add_argument('--extraction');ap.add_argument('--private-custody');ap.add_argument('--private-originals');args=ap.parse_args();p=Path(args.stage)
h=lambda b:hashlib.sha256(b).hexdigest()
manifest=(p/'STAGE-MANIFEST.json').read_bytes();assert h(manifest)=='c6a849084f746adbcc392e46937928b624dbec52f8626aba3f6464527361e8b0';s=json.loads(manifest)
files={f.relative_to(p).as_posix() for f in p.rglob('*') if f.is_file()}-{'STAGE-MANIFEST.json'};assert len(s['files'])==len(files)==10 and files=={x['path'] for x in s['files']}
for x in s['files']:
 f=p/x['path'];assert not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(x['bytes'],x['sha256'])
c=json.loads((p/'CATALOG.json').read_bytes());assert c['working_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and c['product_validation']=='not performed'
assert [(x['id'],x['original_roles'],x['author']) for x in c['packets']]==[('hair-surface-grid',[2643,2644],'/root/hair_ops'),('hair-collider',[2645],'/root/mhr_character'),('shell-mesh',[2646,2647],'/root/rig_identity')]
archives={};counts={}
for x in c['packets']:
 data=(p/x['archive']).read_bytes();assert (len(data),h(data))==(x['archive_bytes'],x['archive_sha256'])
 with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as t:
  ms=t.getmembers();assert len(ms)==len({m.name for m in ms});contents={}
  for m in ms:
   path=PurePosixPath(m.name);assert m.isfile() and not path.is_absolute() and '..' not in path.parts and str(path)==m.name;assert (m.uid,m.gid,m.uname,m.gname,m.mtime,m.mode)==(0,0,'','',0,0o644);b=t.extractfile(m).read();assert len(b)==m.size;contents[m.name]=b
 raw=contents[x['manifest']];assert h(raw)==x['manifest_sha256'];items=json.loads(raw)['files'];rows={i['path']:(i['bytes'],i['sha256']) for i in items} if isinstance(items,list) else {k:(v['bytes'],v['sha256']) for k,v in items.items()};assert len(rows)==len(items) and set(contents)==set(rows)|{x['manifest']}
 for n,(size,sha) in rows.items():assert (len(contents[n]),h(contents[n]))==(size,sha)
 archives[x['id']]=contents;counts[x['id']]=len(contents)
assert counts=={'hair-surface-grid':112,'hair-collider':119,'shell-mesh':137}
for x in c['same_machine_receipts']:
 b=(p/x['path']).read_bytes();assert (len(b),h(b))==(x['bytes'],x['sha256'])
custody=(p/c['temporary_custody']).read_bytes();assert h(custody)==c['temporary_custody_sha256'];custody=json.loads(custody);assert 'unknown' in custody['outcome'] and 'read only' in custody['custody'];assert custody['private_same_machine_capture_manifest_sha256']=='dfd4d543b50aa2bee613be76e570fc060d663e30e6f3bc23b456a8c898cddbda'
json_member=lambda packet,n:json.loads(archives[packet][n])
assert json_member('hair-surface-grid','REPORT.json')['author']=='/root/hair_ops'
assert json_member('hair-surface-grid','bundle/REPORT.json')['scope']['roles']==[2643,2644]
attribution=json_member('hair-surface-grid','bundle/REPORT.json')['historical_attribution'][2];assert attribution['original_author']=='Segovb root Lens11' and 'READ-L11-432' in attribution['record']
receipt=json_member('hair-surface-grid','bundle/historical/prior-059--segovb--READ-L11-432.json');assert receipt['sources'][6]['original_index']==2643 and receipt['sources'][6]['fresh_personal_body_reads']==[[1,146],[215,244]] and receipt['sources'][7]['original_index']==2644 and receipt['sources'][7]['fresh_personal_body_reads']==[[1,156]]
assert not any(any(w in k.lower() for w in ('author','reader','reviewer','actor','agent')) for item in (receipt,receipt['sources'][6],receipt['sources'][7]) for k in item)
assert json_member('hair-collider','PUBLIC-REPORT.json')['author']=='/root/mhr_character'
assert json_member('hair-collider','evidence/REPORT.json')['scope']['original_role']==2645
assert json_member('shell-mesh','bundle/REPORT.json')['author']=='/root/rig_identity'
assert json_member('shell-mesh','bundle/REPORT.json')['scope']['original_roles']==[2646,2647]
# Independently check public stored and decoded hashes against each crosswalk.
for id,idx in [('hair-surface-grid','EVIDENCE-IDENTITIES.json'),('shell-mesh','INDEX.json')]:
 d=archives[id];rows=json.loads(d[idx])['members'];nchg=spans=0
 for x in rows:
  b=d[x['public_path']];plain=gzip.decompress(b) if x['encoding']=='gzip' else b
  assert (len(b),h(b),len(plain),h(plain))==(x['public_stored_bytes'],x['public_stored_sha256'],x['public_decoded_bytes'],x['public_decoded_sha256'])
  if x['original_stored_sha256']!=x['public_stored_sha256']:nchg+=1;spans+=len(x['redactions'])
  else:assert x['original_decoded_sha256']==x['public_decoded_sha256']
 assert (nchg,spans)==((7,57) if id=='hair-surface-grid' else (2,16))
coll=archives['hair-collider'];mp=json.loads(coll['TRANSPORT-MAP.json']);assert (len(mp['entries']),mp['content_transformed_files'],mp['path_substitution_occurrences'])==(111,2,3)
for e in mp['entries']:
 b=coll[e['public_path']];assert (len(b),h(b))==(e['public_bytes'],e['public_sha256'])
 if not e['transformation'].startswith('local scratch'):assert (len(b),h(b))==(e['original_bytes'],e['original_sha256'])
old_sha='2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08';q_d7='3695264e6d4531a1137f0c3232e0d5163d941c49849cb4bddef31cd8403fd173';comm='36115b9bc3ebb3dec1ae87ef5a3f2431195665ab611a4b1e995e15cb9548a77f';work='4a8956e6adb9103355075bb9267fc86e73b837ecc0c1bd7e2d4e5a718b6fbe0c'
surf=archives['hair-surface-grid'];shell=archives['shell-mesh'];surface_old=gzip.decompress(surf['bundle/historical/baseline-059-queue-committed-2e1f.json.gz']);shell_old=gzip.decompress(shell['bundle/history/059-original-queue.json.gz']);later=gzip.decompress(surf['bundle/historical/baseline-059-queue-committed-d7.json.gz'])
assert (h(surface_old),h(shell_old),h(later))==('f152abba04afaaf396752904009f16f0be19a71cf521b7361c8f0d4175c948fc','6e8eb5826be050eec393a4dccc9495e23136643dae473e0dac28977294d8cc9f',q_d7)
assert len({old_sha,h(surface_old),h(shell_old),h(later)})==4
registry=gzip.decompress(shell['bundle/history/059-original-registry.json.gz']);assert h(registry)=='d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f' and registry==gzip.decompress(coll['evidence/history/registry-original.json.gz']);assert h(shell['bundle/history/059-original-registry.json.gz'])=='246584f551eb77f70f02fef9ac1cb984c8532bee636c75e00caecc766e4cf228'
# The filenames inside frozen historical evidence describe originals; the outer
# transport crosswalks override any assumed original/public byte identity.
x=json.loads(surf['HISTORICAL-ASSERTION-CROSSWALK.json']);srec=next(y for y in x['members'] if y['public_path'].endswith('queue-committed-2e1f.json.gz'));assert srec['original_decoded_sha256']==old_sha and srec['public_decoded_sha256']==h(surface_old)
x=json.loads(shell['PUBLICATION-QUALIFICATION.json']);r=next(y for y in x['changed_members'] if y['path']=='history/059-original-queue.json.gz');assert (r['original_decoded_sha256'],r['public_decoded_sha256'])==(old_sha,h(shell_old))
old_surf=json.loads(surface_old);old_shell=json.loads(shell_old);new=json.loads(later);comparison=json.loads(surf['bundle/selected-queue-comparison.json']);assert len(comparison['rows'])==7
for row in comparison['rows']:
 i=row['index'];assert row['equal_json_value'] and old_surf['entries'][i]==old_shell['entries'][i]==new['entries'][i]
cb=gzip.decompress(surf['bundle/reference/committed-baseline.json.gz']);wb=gzip.decompress(surf['bundle/reference/working-baseline.json.gz']);assert (h(cb),h(wb))==(comm,work) and cb==gzip.decompress(shell['bundle/history/baseline-committed.json.gz']) and wb==gzip.decompress(shell['bundle/history/baseline-observed-working.json.gz'])
cdoc=json.loads(cb);wdoc=json.loads(wb)
def pointer(x,p):
 for part in p.split('/')[1:]:
  part=part.replace('~1','/').replace('~0','~');x=x[int(part)] if isinstance(x,list) else x[part]
 return x
sreport=json.loads(surf['bundle/REPORT.json']);selected=sreport['ledger_and_queue_snapshot_identity']['selected_current_entries'];assert len(selected)==4
for x in selected:assert x['selected_entry_equal'] and pointer(cdoc,x['committed_pointer'])==pointer(wdoc,x['working_pointer'])
srows=json.loads(shell['bundle/history/current-ledger-selection-reconstructed.json'])['rows'];assert len(srows)==8
for x in srows:assert x['row_values_equal'] and pointer(cdoc,x['committed_pointer'])==x['committed_value']==x['observed_working_value']==pointer(wdoc,x['observed_working_pointer'])
l=json.loads(coll['evidence/history/ledger-selection.json']);assert (l['committed']['sha256'],l['working_observed']['sha256'])==(comm,work) and l['selected_entries_equal'] and l['committed']['entries']==l['working_observed']['entries']
q=json.loads(coll['evidence/history/queue-selection.json']);assert q['selected_56f_d7_equal'] and q['selected_d7_working_equal'] and q['committed_56f_original']['sha256']==old_sha and q['committed_d7_reconstructed']['sha256']==q['working_observed']['sha256']==q_d7
assert q['indices_zero_based']==[1843,1844,1845] and q['committed_56f_original']['entries']==q['committed_d7_reconstructed']['entries']==q['working_observed']['entries']
for i in q['indices_zero_based']:assert q['committed_56f_original']['entries'][str(i)]==old_surf['entries'][i]==old_shell['entries'][i]==new['entries'][i]
for ident,row in l['committed']['entries'].items():assert next(x for x in cdoc['newly_assessed_ledger']['entries'] if x['id']==ident)==row==next(x for x in wdoc['newly_assessed_ledger']['entries'] if x['id']==ident)
external='not requested'
if args.git_repo:
 def git(*a):return subprocess.check_output(['git','-C',args.git_repo,'--no-replace-objects',*a])
 old=git('show','98cc9fcb9373f5deaa39703f6ff4353ad0b4ab07:baseline-059/queue.json');assert h(old)==old_sha and old==git('show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json')
 assert later==git('show','d7b14a15ef132422eca86e7d898b47423af92779:baseline-059/queue.json') and cb==git('show','d7b14a15ef132422eca86e7d898b47423af92779:BASELINE-95E-20261110.json')
 original=json.loads(old)
 for row in comparison['rows']:assert original['entries'][row['index']]==old_surf['entries'][row['index']]
 for row in json.loads(shell['bundle/history/059-queue-selected-reconstructed.json'])['entries']:assert pointer(original,row['json_pointer'])==row['value']==pointer(old_shell,row['json_pointer'])
 # Structural diff reports counts/types only; no original path values are emitted.
 def changes(a,b):
  pending=[(a,b)];different=0
  while pending:
   x,y=pending.pop();assert type(x) is type(y)
   if isinstance(x,dict):assert x.keys()==y.keys();pending.extend((x[k],y[k]) for k in x)
   elif isinstance(x,list):assert len(x)==len(y);pending.extend(zip(x,y))
   elif x!=y:assert isinstance(x,str);different+=1
  return different
 differences={'surface_reconstruction':changes(original,old_surf),'shell_reconstruction':changes(original,old_shell),'d7_queue':changes(original,new)}
 assert differences=={'surface_reconstruction':12,'shell_reconstruction':12,'d7_queue':12}
 external={'checked':'local read-only Git objects, not a live remote or independent custody','changed_string_leaf_counts':differences}
extraction='not requested'
if args.extraction:
 data=(Path(args.extraction)/'workflow-20261030/CLOSURE-QUEUE-100.json').read_bytes();assert h(data)==old_sha
 original_registry=(Path(args.extraction)/'evidence/audit05/original-scope-072.json').read_bytes();assert registry==original_registry
 if args.git_repo:assert data==old
 extraction='059 original queue and registry bytes matched; no claim of 3821 recovery'
def safe_rel(value):
 x=PurePosixPath(value);assert not x.is_absolute() and '..' not in x.parts;return x
private={}
if args.private_custody:
 root=Path(args.private_custody);assert root.stat().st_mode&0o077==0
 raw=(root/'MANIFEST.json').read_bytes();assert h(raw)=='dfd4d543b50aa2bee613be76e570fc060d663e30e6f3bc23b456a8c898cddbda';doc=json.loads(raw);assert len(doc['items'])==26
 for e in doc['items']:
  f=root/safe_rel(e['retained_name']);assert f.is_file() and not f.is_symlink() and f.stat().st_mode&0o077==0;b=f.read_bytes();assert (len(b),h(b))==(e['observed_bytes'],e['observed_sha256'])
 assert sum(e['actor']=='/root' and e['kind']=='helper' for e in doc['items'])==2 and sum(e['actor']=='/root' and e['kind']=='log' for e in doc['items'])==23 and sum(e['actor']=='/root/hair_ops' and e['kind']=='helper' for e in doc['items'])==1
 private['captured_current_bytes']=26
if args.private_originals:
 root=Path(args.private_originals);assert root.stat().st_mode&0o077==0
 raw=(root/'MANIFEST.json').read_bytes();assert h(raw)=='da8dc1d5c5d06da19ceb72fd92b1053a627bd534162333b4c84a855d86af3d36';doc=json.loads(raw);assert len(doc['files'])==124
 for e in doc['files']:
  f=root/safe_rel(e['group'])/safe_rel(e['path']);assert f.is_file() and not f.is_symlink() and f.stat().st_mode&0o077==0;b=f.read_bytes();assert (len(b),h(b))==(e['bytes'],e['sha256'])
 private['retention_manifest_files']=124
print(json.dumps({'status':'PASS','stage_sha256':h(manifest),'archives':counts,'members_total':sum(counts.values()),'changed_members':{'surface':[7,57],'collider':[2,3],'shell':[2,16]},'selected_queue_rows':{'surface':7,'shell':6,'collider':3},'selected_baseline_rows':{'surface':4,'shell':8,'collider':3},'external':external,'extraction':extraction,'private_local':private,'qualification':'Private captures concern post-write current bytes, not unknown earlier contents. No product behavior, runtime or complete privacy verified. Historical rows and current selections remain distinct; checkpoint3821 remains unrecovered.'},indent=2))
