#!/usr/bin/env python3
"""Replay the evidence-only candidate and optional predecessor guard."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,importlib.util,io,json,os,subprocess,sys,tarfile
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--candidate',type=Path);ap.add_argument('--predecessor',type=Path);a=ap.parse_args();root=a.publication.resolve();candidate=a.candidate.resolve() if a.candidate else None;sha=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());archive=(p/'APPLICATION-EVIDENCE.tar.gz').read_bytes();assert m['schema']=='726-post76fe-application-evidence-v1' and (len(archive),sha(archive))==(m['archive_bytes'],m['archive_sha256'])
rows={r['path']:r for r in m['members']};assert len(rows)==len(m['members'])==12
members={}
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as tar:
 for item in tar:
  rel=PurePosixPath(item.name);assert not rel.is_absolute() and '..' not in rel.parts and item.isfile() and item.name not in members and item.name in rows and item.mode==0o644;data=tar.extractfile(item).read();row=rows[item.name];assert (len(data),sha(data))==(row['bytes'],row['sha256']);members[item.name]=data
assert set(members)==set(rows) and members['TRANSFORM.py']==(p/'TRANSFORM.py').read_bytes() and members['CANDIDATE.json']==(p/'ADOPTION-CANDIDATE.json').read_bytes();record=json.loads(members['CANDIDATE.json']);assert record['status']=='candidate only; not installed or published' and record['transform_sha256']==sha(members['TRANSFORM.py'])
spec=importlib.util.spec_from_file_location('post76fe_evidence_transform',p/'TRANSFORM.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);assert m['parent_checkpoint']==record['parent_checkpoint']==mod.PARENT
projection=json.loads((p.parent/'reconciliation/PROPOSED-ASSESSMENT.json').read_bytes());source_map=json.loads((p.parent/'reconciliation/SOURCE-MAP.json').read_bytes());assert sha((p.parent/'reconciliation/PROPOSED-ASSESSMENT.json').read_bytes())==record['public_projection_sha256'] and sha((p.parent/'reconciliation/SOURCE-MAP.json').read_bytes())==record['source_map_sha256']
before={name:members['before/'+name] for name in mod.NAMES};after={name:members['after/'+name] for name in mod.NAMES};report=json.loads(members['root/'+mod.REPORT]);observation=json.loads(members['root/'+mod.MAIN]);assert mod.transform(before,projection,report,source_map,observation)==after
for name in mod.NAMES:
 assert subprocess.check_output(['git','-C',str(root),'show',mod.PARENT+':'+name])==before[name]
 actual=(candidate/'after'/name) if candidate else root/name;assert actual.is_file() and not actual.is_symlink() and actual.read_bytes()==after[name]
for name in (mod.REPORT,mod.MAIN):
 actual=(candidate/'root'/name) if candidate else root/name;assert actual.is_file() and not actual.is_symlink() and actual.read_bytes()==members['root/'+name]
if a.predecessor:
 prior=a.predecessor.resolve();git=lambda *args:subprocess.check_output(['git','-C',str(prior),*args]);assert git('rev-parse','HEAD').decode().strip()==mod.PARENT and not git('status','--porcelain=v1','-z');env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1';subprocess.run([sys.executable,'-B',str(prior/'verify_postb111_checkpoint.py')],cwd=prior,env=env,check=True);print('Replayed the prior evidence checkpoint with the caller-supplied Git object environment.')
print('Verified evidence-only before/after application and source-map bindings. Candidate or installed state is explicit; independent custody, historical closure and product behavior are not established.')
