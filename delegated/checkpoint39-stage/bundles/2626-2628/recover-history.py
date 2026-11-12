import gzip,hashlib,json,pathlib,shutil,subprocess,tarfile
p=pathlib.Path(__file__).parent;pub=pathlib.Path('<PRIVATE_PATH_0001>');old=pathlib.Path('<PRIVATE_PATH_0038>');prior=pathlib.Path('<PRIVATE_PATH_0039>');sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();(p/'reference').mkdir(exist_ok=True);refs=[]
for src,dst in [('recovered/original-registry-source-archive/registry.json.gz','registry.json.gz'),('recovered/original-transport-manifest.json.gz','original-transport-manifest.json.gz'),('RECOVERY-PHYSICAL-P16-PATCH-20261111.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json'),('DELEGATED-CHECKPOINT-18-QUALIFICATION.json','DELEGATED-CHECKPOINT-18-QUALIFICATION.json'),('DELEGATED-EVIDENCE-CHECKPOINT-18.json','DELEGATED-EVIDENCE-CHECKPOINT-18.json'),('delegated/num-enum-derive-report.json','published-prior-report.json'),('ASSESSMENT-FBX-95E.json','ASSESSMENT-FBX-95E.json'),('BASELINE-95E-20261110.json','BASELINE-95E-20261110.json')]:
 b=(pub/src).read_bytes();(p/'reference'/dst).write_bytes(b);refs.append({'publication_path':src,'file':'reference/'+dst,'sha256':sha(b),'bytes':len(b)})
for name,expected in [('queue','2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08'),('ledger','3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac')]:
 b=(pub/f'baseline-059/{name}.json').read_bytes();assert sha(b)==expected;(p/f'reference/{name}.json.gz').write_bytes(gzip.compress(b,mtime=0));refs.append({'publication_path':f'baseline-059/{name}.json','file':f'reference/{name}.json.gz','encoding':'gzip','sha256':sha(b),'bytes':len(b),'qualification':'Reconstructed earlier 059, not checkpoint3821.'});globals()[name]=json.loads(b)
qs=(152,153,2942);journals={}
for i in qs:
 for path in queue['entries'][i].get('journal_revisits',[]):
  z=ledger
  for part in path.strip('/').split('/'):z=z[int(part)] if isinstance(z,list) else z[part]
  journals[path]=z
(p/'059-selection-reconstructed.json').write_text(json.dumps({'queue_source_sha256':sha((pub/'baseline-059/queue.json').read_bytes()),'entries':{str(i):queue['entries'][i] for i in qs},'ledger_source_sha256':sha((pub/'baseline-059/ledger.json').read_bytes()),'journal_entries':journals,'qualification':'Exact reconstructed selected records; preserve recorded nulls and granular prior authors. Not checkpoint3821.'},indent=2)+'\n');(p/'reference-index.json').write_text(json.dumps({'records':refs},indent=2)+'\n')
raw=gzip.decompress((p/'reference/registry.json.gz').read_bytes());assert sha(raw)=='d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f';reg=json.loads(raw);(p/'registry-selection-reconstructed.json').write_text(json.dumps({'registry_sha256':sha(raw),'roles':{str(i):reg['files'][i] for i in (2626,2627,2628)},'snapshot_rows':[s for s in reg['snapshots'] if 'native' in s['id'] or s['id']=='pr-working'],'qualification':'Registry rows reconstructed exactly; f82 transports native bytes, not project revision.'},indent=2)+'\n')
m=(p/'reference/original-transport-manifest.json.gz').read_bytes();assert sha(m)=='7e45ce9d858f93e0c5e5d50f360f79ed8941e28049f305053f1fd2bdda364834';idx={e['path']:e for e in json.loads(gzip.decompress(m))['entries']};wanted={x['original_path'] for x in json.loads((prior/'historical-attribution.json').read_bytes())['records_059']}
def walk(o):
 if isinstance(o,dict):
  for v in o.values():walk(v)
 elif isinstance(o,list):
  for v in o:walk(v)
 elif isinstance(o,str):
  for marker in ('segovb/','adler/','parallel-l12-20261029/'):
   if marker in o:
    rel=o[o.find(marker):]
    if rel in idx:wanted.add(rel)
for i in qs:walk(queue['entries'][i])
wanted.add('adler/READ-L11-358.json')
records=[];missing=[]
for rel in sorted(wanted):
 if rel not in idx or not (old/rel).is_file():missing.append({'path':rel,'in_transport_manifest':rel in idx,'in_extraction':(old/rel).is_file()});continue
 b=(old/rel).read_bytes();e=idx[rel];assert (len(b),sha(b),blob(b))==(e['bytes'],e['sha256'],e['git_blob']);dst=p/'history-059'/rel;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(b);records.append({'kind':'original-059','transport_path':rel,'file':str(dst.relative_to(p)),'sha256':sha(b),'bytes':len(b),'git_blob':blob(b)})
manifest=(prior/'MANIFEST.json').read_bytes();assert sha(manifest)=='479935e112b1bf71cd2b7908cad96cec99f72965f1fb2312828eaf615c9ceba9';(p/'prior').mkdir(exist_ok=True);(p/'prior/MANIFEST.json').write_bytes(manifest);pri={e['path']:e for e in json.loads(manifest)['files']};selected=[]
for rel in ['REPORT.json','registration.json','support-registration.json','helper-registration.json','reader-registration.json','personal-read-receipt.json','historical-attribution.json','git-proofs.json','published-prior-packet-links.json','prior-lock-evidence-links.json']:
 b=(prior/rel).read_bytes();assert (len(b),sha(b))==(pri[rel]['bytes'],pri[rel]['sha256']);dst=p/'prior'/rel;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(b);selected.append({'path':rel,'bytes':len(b),'sha256':sha(b)})
(p/'prior-index.json').write_text(json.dumps({'author':'/root/rig_identity','original_manifest_sha256':sha(manifest),'selected_files':selected,'published_archive':'delegated/num-enum-derive-evidence.tar.gz','published_archive_sha256':'d950333afec410db16143f054d4a3a17bdf9e0673eafa9ed52f5b17ead4a9951','qualification':'Original selected members verified against frozen prior manifest, not read-credit transfer; publication may redact.'},indent=2)+'\n')
f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';paths=['continuation/root/admission03-scope-reconciliation-046.json','continuation/root/admission03-scope-reconciliation-050.json','continuation/rig/admission03/source-range-registration.jsonl','continuation/rig/admission03/source-qualifications.jsonl','continuation/rig/admission03/additional-contracts.jsonl','continuation/rig/admission03/contract-groups.jsonl']
for rel in paths:
 path='handoff/774/adler-original/audit/'+rel;b=subprocess.check_output(['git','-C',str(pub),'show',f82+':'+path]);dst=p/'history-774'/(rel+'.gz');dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(gzip.compress(b,mtime=0));records.append({'kind':'historical-774','transport_path':path,'file':str(dst.relative_to(p)),'encoding':'gzip','sha256':sha(b),'bytes':len(b),'git_blob':blob(b)})
rr=gzip.decompress((p/'history-774/continuation/rig/admission03/source-range-registration.jsonl.gz').read_bytes());rq=gzip.decompress((p/'history-774/continuation/rig/admission03/source-qualifications.jsonl.gz').read_bytes());a=json.loads(gzip.decompress((p/'history-774/continuation/root/admission03-scope-reconciliation-046.json.gz').read_bytes()));b=json.loads(gzip.decompress((p/'history-774/continuation/root/admission03-scope-reconciliation-050.json.gz').read_bytes()));qa={'root046_at':a['at'],'root050_at':b['at'],'root046_registration_pin':a['input_pins'][3],'root050_registration_pin':b['input_pins'][2],'later_registration_first35_sha256':sha(b''.join(rr.splitlines(keepends=True)[:35])),'later_registration_first139_sha256':sha(b''.join(rr.splitlines(keepends=True)[:139])),'later_registration_full_sha256':sha(rr),'root046_qualification_pin':a['input_pins'][4],'root050_qualification_pin':b['input_pins'][3],'later_qualification_first31_sha256':sha(b''.join(rq.splitlines(keepends=True)[:31])),'later_qualification_first45_sha256':sha(b''.join(rq.splitlines(keepends=True)[:45])),'rows80_to_83':[json.loads(x) for x in rr.splitlines()[79:83]],'qualification':'Root046 captures 35 before rows80–83; root050 captures first139 and selected 138. Registration prefix hashes differ between captured and later bodies; cause unproved. Qualification prefix hashes match. Exact registry embeds rows independently.'}
(p/'historical-index.json').write_text(json.dumps({'records':records,'unavailable_requested_records':missing,'rig_registration_pin_qualification':qa,'qualification':'059 checked against transport manifest; 774 history separately Git-tree-proved. All prior revisions/authors remain separate; no 3821 promotion.'},indent=2)+'\n')
archive=pub/'delegated/num-enum-derive-evidence.tar.gz';assert sha(archive.read_bytes())=='d950333afec410db16143f054d4a3a17bdf9e0673eafa9ed52f5b17ead4a9951';e=next(z for z in json.loads((p/'reference/DELEGATED-EVIDENCE-CHECKPOINT-18.json').read_bytes())['bundles'] if z['bundle_id']=='num-enum-derive');members=[]
with tarfile.open(archive,'r:gz') as tar:
 for x in e['members']:
  if x['path'] not in ['MANIFEST.json','REPORT.json','registration.json','personal-read-receipt.json','historical-attribution.json']:continue
  v=tar.extractfile(x['path']).read();assert (len(v),sha(v))==(x['published_bytes'],x['published_sha256']);dest=p/'reference'/('public-prior-'+x['path']);dest.write_bytes(v);members.append({'path':x['path'],'original_sha256':x['original_sha256'],'published_sha256':x['published_sha256'],'changed':x['changed'],'file':str(dest.relative_to(p))})
(p/'public-prior-members.json').write_text(json.dumps({'archive_sha256':sha(archive.read_bytes()),'members':members,'qualification':'Selected public members checked against checkpoint metadata. Registration is redacted and differs; original prior registration retained separately. Individual historical 059 public redactions are qualified by checkpoint 18 metadata.'},indent=2)+'\n')
for name in ('rules','skills'):shutil.copytree('<PRIVATE_PATH_0040>'+name,p/name)
for name in ('rules.json','rules-local.json','skills.json','build-manifest.py','verify-manifest.py'):shutil.copyfile('<PRIVATE_PATH_0040>'+name,p/name)
print('059',len(records)-len(paths),'missing',missing,'774',len(paths))
