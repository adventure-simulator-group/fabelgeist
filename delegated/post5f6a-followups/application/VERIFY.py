#!/usr/bin/env python3
"""Replay the evidence-only successor and optionally its predecessor checks."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,io,json,os,subprocess,sys,tarfile
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
from TRANSFORM import NAMES,REPORT,MAIN,QNAME,RNAME,IDS,LENSES,CONTRACTS,RELIDS,PARENT,transform
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--candidate',type=Path);ap.add_argument('--predecessor',type=Path);a=ap.parse_args();root=a.publication.resolve();candidate=a.candidate.resolve() if a.candidate else None;h=lambda b:hashlib.sha256(b).hexdigest();load=lambda f:json.loads(f.read_bytes());m=load(p/'RECORD.json');assert m['schema']=='726-post5f6a-application-archive-v1' and m['parent_checkpoint']==PARENT and m['parent_tree']=='84c63be7bd8e5b00b33bd63f480fe8ebb7b900c7' and h((p/'TRANSFORM.py').read_bytes())==m['transform_sha256'];raw=(p/m['archive']).read_bytes();assert (len(raw),h(raw))==(m['archive_bytes'],m['archive_sha256']);wanted={x['path']:x for x in m['members']};assert len(wanted)==len(m['members'])==17;data={}
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 for item in t:
  name=PurePosixPath(item.name);assert not name.is_absolute() and '..' not in name.parts and item.isfile() and item.name in wanted and item.name not in data;b=t.extractfile(item).read();w=wanted[item.name];assert (len(b),h(b),oct(item.mode))==(w['bytes'],w['sha256'],w['mode']);data[item.name]=b
assert set(data)==set(wanted)
git=lambda *cmd:subprocess.check_output(['git','-C',str(root),*cmd]);assert git('rev-parse',PARENT+'^{tree}').decode().strip()==m['parent_tree'];before={n:data['before/'+n] for n in NAMES};inputs=json.loads(data['inputs/input-identities.json']);assert inputs['parent_checkpoint']==PARENT and inputs['parent_tree']==m['parent_tree'] and len(inputs['before'])==4
for x in inputs['before']:
 n=x['path'];b=git('show',PARENT+':'+n);assert b==before[n] and (len(b),h(b),git('rev-parse',PARENT+':'+n).decode().strip())==(x['bytes'],x['sha256'],x['git_blob'])
proposals={QNAME:data['inputs/q10-q13-projection.json'],RNAME:data['inputs/role0057-projection.json']}
for n,b in proposals.items():assert git('show',PARENT+':'+n)==b
draft=json.loads(data['inputs/draft.json']);report=json.loads(data['inputs/report.json']);obs=json.loads(data['inputs/main-observation.json']);assert inputs['draft_sha256']==h(data['inputs/draft.json'])==report['adopted_from_draft_sha256'] and inputs['main_observation_sha256']==h(data['inputs/main-observation.json']);assert draft['status']=='draft for independent application review; not adopted' and report['schema']=='726-post5f6a-central-assessment-v1' and report['status']=='adopted as six bounded partial assessments and three planning relationships; no semantic closure'
assert set(report)-set(draft)=={'adopted_from_draft_sha256','independent_application_review','adoption_qualification'} and all(report[k]==draft[k] for k in draft if k not in ('schema','status'))
peer=report['independent_application_review'];assert peer['public_manifest_sha256']=='3fecc064492e45fa2a39c91eca43856988465877b405c946611160c9fff85c69' and peer['public_transport']=='delegated/post5f6a-followups/application/review-transport/application-review.tar.gz'
subprocess.run([sys.executable,'-B',str(p/'review-transport/VERIFY.py')],check=True)
with tarfile.open(p/'review-transport/application-review.tar.gz','r:gz') as t:
 review_manifest=t.extractfile('application-review/MANIFEST.json').read();review_v4=t.extractfile('application-review/V4-CHECK.json').read();assert h(review_manifest)==peer['public_manifest_sha256'] and json.loads(review_v4)['result']=='pass';review_files=json.loads(review_manifest)['files'];assert h(review_v4)==review_files['V4-CHECK.json']['sha256']
for i in (2,3,4):assert json.loads(data['inputs/correction-v'+str(i)+'.json'])['evidence_replay_exits']==[0,0]
after=transform(before,report,obs,proposals);assert all(after[n]==data['after/'+n] for n in NAMES)
adopt=load(p/'ADOPTION-CANDIDATE.json');assert adopt['draft_sha256']==h(data['inputs/draft.json']) and adopt['report_sha256']==h(data['inputs/report.json']) and adopt['v4_transform_sha256']==h((p/'TRANSFORM.py').read_bytes());assert set(adopt['after'])==set(NAMES)
for n in NAMES:
 f=(candidate/'after'/n) if candidate else (root/n);b=f.read_bytes();assert b==after[n] and (len(b),h(b))==(adopt['after'][n]['bytes'],adopt['after'][n]['sha256'])
targets={REPORT:(candidate/'root'/REPORT) if candidate else root/REPORT,MAIN:(candidate/'root'/MAIN) if candidate else root/MAIN,'delegated/post5f6a-followups/REMOTE-RECOVERY.json':p.parent/'REMOTE-RECOVERY.json','delegated/post5f6a-followups/application/README.md':p/'README.md','delegated/post5f6a-followups/application/review-transport/application-review.tar.gz':p/'review-transport/application-review.tar.gz'}
assert set(targets)==set(adopt['targets'])
for n,f in targets.items():
 assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(adopt['targets'][n]['bytes'],adopt['targets'][n]['sha256'])
assert targets[REPORT].read_bytes()==data['inputs/report.json'] and targets[MAIN].read_bytes()==data['inputs/main-observation.json']
hist=report['historical_provenance'];assert h((root/'baseline-059/queue.json').read_bytes())==hist['reconstructed_queue_sha256'] and h((root/'baseline-059/ledger.json').read_bytes())==hist['reconstructed_ledger_sha256'];old=load(root/'BASELINE-20261110.json')['historical_checkpoint_059'];assert (old['queue_sha256'],old['ledger_sha256'])==(hist['original_queue_sha256'],hist['original_ledger_sha256'])
if a.predecessor:
 prior=a.predecessor.resolve();assert prior!=root and subprocess.check_output(['git','-C',str(prior),'rev-parse','HEAD']).decode().strip()==PARENT and subprocess.check_output(['git','-C',str(prior),'rev-parse','HEAD^{tree}']).decode().strip()==m['parent_tree'] and not subprocess.check_output(['git','-C',str(prior),'status','--porcelain=v1','-z']);env=dict(os.environ);env.pop('PYTHONOPTIMIZE',None);env['PYTHONDONTWRITEBYTECODE']='1';env['PYTHONNOUSERSITE']='1'
 for script in ['verify_postf2ec_checkpoint.py','delegated/posta422-followups/final-guard/VERIFY.py','delegated/posta422-followups/closing/VERIFY.py']:subprocess.run([sys.executable,'-B',script],cwd=prior,env=env,check=True)
 print('Replayed three evidence-only predecessor checks against the supplied exact clean checkout.')
else:print('Predecessor replay not requested; recorded same-machine receipt and exact Git before-images remain separately available.')
print('Verified exact before/after application, staged targets and proposal/review identities. Structural replay does not independently establish factual prose, independent custody, semantic closure or product behavior.')
