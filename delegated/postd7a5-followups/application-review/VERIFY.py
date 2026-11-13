#!/usr/bin/env python3
"""Replay the bounded evidence-only application review."""
import argparse,hashlib,json,os,stat,subprocess,sys,tarfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions must be enabled')
p=argparse.ArgumentParser();g=p.add_mutually_exclusive_group();g.add_argument('--stage',type=Path);g.add_argument('--published',type=Path);p.add_argument('--repo',type=Path);p.add_argument('--strict-mode',action='store_true');a=p.parse_args();here=Path(__file__).resolve().parent
h=lambda b:hashlib.sha256(b).hexdigest();manifest=json.loads((here/'MANIFEST.json').read_bytes());assert manifest['schema']=='726-application-review-v1';assert {x.name for x in here.iterdir()}==set(manifest['files'])|{'MANIFEST.json'}
for n,row in manifest['files'].items():
 data=(here/n).read_bytes();assert (len(data),h(data))==(row['bytes'],row['sha256'])
root=a.stage or a.published
if not root:
 print('Verified review packet only; staged transformation and Git not checked.');sys.exit(0)
def resolve(n):
 if a.stage:return root/n
 if n.startswith('after/'):return root/n.removeprefix('after/')
 if n.startswith('public/'):return root/'delegated/postd7a5-followups'/n.removeprefix('public/')
 return root/n
snapshot=json.loads((here/'SNAPSHOT.json').read_bytes());assert snapshot['schema']=='726-application-review-snapshot-v1'
for row in snapshot['files']:
 f=resolve(row['path']);assert f.is_file() and not f.is_symlink();data=f.read_bytes();mode=stat.S_IMODE(f.stat().st_mode);expected=int(row['mode'],8);assert (len(data),h(data))==(row['bytes'],row['sha256']);assert mode==expected if a.strict_mode else mode&0o111==expected&0o111
stage_pub=resolve('public/PENDING-DELEGATES.json').parent
report=json.loads(resolve('ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json').read_bytes());pending=json.loads(resolve('public/PENDING-DELEGATES.json').read_bytes());main=resolve('MAIN-RECHECK-20261113-0033.json').read_bytes()
assert report['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009';assert report['parent_published_checkpoint']=='d7a5e77ebdbd44a05cde30881493d6fc22ef025d'
assert report['counts']=={'new_partial_queue_assessments':6,'context_only_observations':2,'semantic_queue_closures':0,'new_planning_relationships':6,'accepted_unique_defect_increment':0,'lens_completions':0}
assert len(pending['packets'])==20 and len({x['queue_index'] for x in pending['packets']})==20
assert {x['queue_index'] for x in pending['packets'] if 'context only' in x['central_status']}=={2968,3016}
v1=resolve('public/SUCCESSOR-CHANGE.json').read_bytes();v2=json.loads(resolve('public/SUCCESSOR-CHANGE-v2.json').read_bytes());assert v2['predecessor_sha256']==h(v1) and v2['report_sha256']==h(resolve('ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json').read_bytes())
env=dict(os.environ);env.update({'GIT_OPTIONAL_LOCKS':'0','PYTHONDONTWRITEBYTECODE':'1','PYTHONNOUSERSITE':'1'});env.pop('PYTHONOPTIMIZE',None);env.pop('GIT_DIR',None);env.pop('GIT_WORK_TREE',None)
for name in ['application','dispatch-transport']:
 z=subprocess.run([sys.executable,'-B',str(stage_pub/name/'VERIFY.py')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env);assert z.returncode==0,z.stderr.decode(errors='replace');print(z.stdout.decode().strip())
N=['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md'];d7='d7a5e77ebdbd44a05cde30881493d6fc22ef025d'
with tarfile.open(stage_pub/'application/APPLICATION-EVIDENCE.tar.gz','r:gz') as t:
 before={n:t.extractfile('before/'+n).read() for n in N};after={n:t.extractfile('after/'+n).read() for n in N}
 for n in N:assert after[n]==resolve('after/'+n).read_bytes()
 assert t.extractfile('inputs/report.json').read()==resolve('ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json').read_bytes();assert t.extractfile('inputs/pending.json').read()==resolve('public/PENDING-DELEGATES.json').read_bytes();assert t.extractfile('inputs/main-observation.json').read()==main
with tarfile.open(stage_pub/'central/CENTRAL-EVIDENCE.tar.gz','r:gz') as t:previous=t.extractfile('drafts/CONSOLIDATED-DRAFT.json').read()
assert h(previous)==report['consolidated_draft_sha256'];prior=json.loads(previous)
for key in ['partial_queue_assessments','context_only_observations','ledger_relationships']:assert prior[key]==report[key]
q0=json.loads(before[N[2]]);q1=json.loads(after[N[2]]);b0=json.loads(before[N[0]]);b1=json.loads(after[N[0]]);g1=json.loads(after[N[1]])
assert [i for i,(x,y) in enumerate(zip(q0['entries'],q1['entries'])) if x!=y]==[2945,2957,3011,3040,3059,3166];assert len(q0['entries'])==len(q1['entries'])==3713
for i in [2968,3016]:assert q0['entries'][i]==q1['entries'][i] and q1['entries'][i]['current_reconciliation']=='unassessed'
old=b0['newly_assessed_ledger']['entries'];new=b1['newly_assessed_ledger']['entries'];assert len(old)==1093 and len(new)==1099 and new[:1093]==old
rels=[x['record'] for x in report['ledger_relationships']];assert all({k:v for k,v in x.items() if k!='record'}==y and x['record']=='ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json' for x,y in zip(new[1093:],rels));assert new[1094]['lens'] is None and new[1096]['lens'] is None
assert g1['queue_records_semantically_reconciled']==0 and g1['lenses_complete']==0 and g1['repair']=='not started' and g1['product_validation']=='not performed'
if a.repo:
 for n in N:
  z=subprocess.run(['git','-C',str(a.repo),'show',d7+':'+n],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env);assert z.returncode==0 and z.stdout==before[n]
 for row in pending['packets']:
  path=row['public_transport'].rstrip('/');prefix='delegated/postd7a5-followups/'
  if path.startswith(prefix):assert (stage_pub/path.removeprefix(prefix)).exists()
  else:
   z=subprocess.run(['git','-C',str(a.repo),'cat-file','-t',d7+':'+path],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env);assert z.returncode==0
 print('Verified local d7 Git before-images and all 20 transport pointers.')
else:print('Git not supplied: d7 commit binding and inherited transport pointers not checked.')
print('Verified 19 named files, exact transform, queue/ledger deltas and successor correction; unlisted siblings excluded.')
