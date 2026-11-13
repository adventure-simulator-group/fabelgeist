#!/usr/bin/env python3
"""Replay the evidence-only application from its sealed public archive."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,importlib.util,io,json,os,subprocess,sys,tarfile

if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--candidate',type=Path);ap.add_argument('--predecessor',type=Path);a=ap.parse_args();root=a.publication.resolve();candidate=a.candidate.resolve() if a.candidate else None;h=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((p/'MANIFEST.json').read_bytes());archive=(p/'APPLICATION-EVIDENCE.tar.gz').read_bytes();assert m['schema']=='726-post8ca-application-evidence-v1' and (len(archive),h(archive))==(m['archive_bytes'],m['archive_sha256'])
controls=m['controls'];assert set(controls)=={'TRANSFORM.py','ADOPTION-CANDIDATE.json','VERIFY.py','README.md'}
for name,row in controls.items():
 f=p/name;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert (len(data),h(data))==(row['bytes'],row['sha256'])
rows={r['path']:r for r in m['members']};assert len(rows)==len(m['members'])==12;members={}
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as tar:
 for item in tar:
  rel=PurePosixPath(item.name);assert not rel.is_absolute() and '..' not in rel.parts and item.isfile() and item.name not in members and item.name in rows and item.mode==0o644;data=tar.extractfile(item).read();row=rows[item.name];assert (len(data),h(data))==(row['bytes'],row['sha256']);members[item.name]=data
assert set(members)==set(rows) and members['TRANSFORM.py']==(p/'TRANSFORM.py').read_bytes() and members['CANDIDATE.json']==(p/'ADOPTION-CANDIDATE.json').read_bytes()
record=json.loads(members['CANDIDATE.json']);assert record['status']=='candidate only; not installed or published' and record['transform_sha256']==h(members['TRANSFORM.py'])
receipt={r['path']:r for r in record['rows']};assert len(receipt)==len(record['rows'])==10 and set(receipt)==set(members)-{'TRANSFORM.py','CANDIDATE.json'}
for name,row in receipt.items():
 data=members[name];assert (len(data),h(data),hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest())==(row['bytes'],row['sha256'],row['git_blob'])
spec=importlib.util.spec_from_file_location('post8ca_evidence_transform',p/'TRANSFORM.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);assert m['parent_checkpoint']==record['parent_checkpoint']==mod.PARENT
stage=p.parent;projection_raw=(stage/'reconciliation/PROPOSED-ASSESSMENT.json').read_bytes();source_raw=(stage/'reconciliation/SOURCE-MAP.json').read_bytes();history_raw=(stage/'reconciliation/HISTORY-SELECTION.json').read_bytes();assert (h(projection_raw),h(source_raw),h(history_raw))==(record['projection_sha256'],record['source_map_sha256'],record['history_sha256'])
assert h((stage/'projection-peer-review/MANIFEST.json').read_bytes())==record['projection_review_manifest_sha256'];note=json.loads((stage/'reconciliation/REVIEWER-QUALIFICATION.json').read_bytes());assert note['projection_sha256']==h(projection_raw) and note['projection_review_manifest_sha256']==record['projection_review_manifest_sha256']
before={n:members['before/'+n] for n in mod.NAMES};after={n:members['after/'+n] for n in mod.NAMES};report=json.loads(members['root/'+mod.REPORT]);observation=json.loads(members['root/'+mod.MAIN]);assert mod.transform(before,json.loads(projection_raw),report,json.loads(source_raw),json.loads(history_raw),observation)==after
for name in mod.NAMES:
 assert subprocess.check_output(['git','-C',str(root),'show',mod.PARENT+':'+name])==before[name]
 actual=(candidate/'after'/name) if candidate else root/name;assert actual.is_file() and not actual.is_symlink() and actual.read_bytes()==after[name]
for name in (mod.REPORT,mod.MAIN):
 actual=(candidate/'root'/name) if candidate else root/name;assert actual.is_file() and not actual.is_symlink() and actual.read_bytes()==members['root/'+name]
if a.predecessor:
 prior=a.predecessor.resolve();git=lambda *v:subprocess.check_output(['git','-C',str(prior),*v]);assert git('rev-parse','HEAD').decode().strip()==mod.PARENT and not git('status','--porcelain=v1','-z')
 env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1';subprocess.run([sys.executable,'-B',str(prior/'verify_post76fe_checkpoint.py')],cwd=prior,env=env,check=True)
 print('Replayed the clean 8ca predecessor evidence guard using the caller-supplied Git object environment.')
print('Verified the evidence-only before/after application and selected bindings. Source semantics, independent custody, missing checkpoint3821, audit closure and product behavior remain open.')
