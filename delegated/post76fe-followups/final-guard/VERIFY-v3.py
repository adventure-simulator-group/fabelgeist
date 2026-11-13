#!/usr/bin/env python3
"""Verify the installed audit-evidence checkpoint, not product behavior."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,os,stat,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;stage=p.parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--predecessor',type=Path);args=ap.parse_args();root=args.publication.resolve();assert stage==(root/'delegated/post76fe-followups').resolve() and not (root/'delegated/post76fe-followups').is_symlink();sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
parent='76fe53635e2bf1e3e08bbfaa364314f5e7888bf8';source='95e63628f9d70b8dd6e6fede64334696c6995009';tree='1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
def safe(name):
 q=PurePosixPath(name);assert not q.is_absolute() and '..' not in q.parts and q.parts;return q
capture_bytes=(stage/'STAGE-CAPTURE.json').read_bytes();capture=json.loads(capture_bytes);status=json.loads((stage/'ADOPTION-STATUS.json').read_bytes());assert capture['schema']=='726-post76fe-stage-capture-v1' and capture['parent_checkpoint']==parent and capture['source_revision']==source
assert status['schema']=='726-post76fe-installation-status-v1' and status['parent_checkpoint']==parent and status['stage_capture_sha256']==sha(capture_bytes) and status['root_paths_matched']==7
paths=list(stage.rglob('*'));assert all(not x.is_symlink() and (x.is_file() or x.is_dir()) for x in paths);assert not any(x.name=='__pycache__' or x.suffix=='.pyc' for x in paths)
files={x.relative_to(stage).as_posix():x for x in paths if x.is_file()};rows={r['path']:r for r in capture['rows']};assert len(rows)==len(capture['rows']);closing={f'closing/{x}' for x in ('CAPTURE.json','SCAN.json','README.md','VERIFY.py')};present=closing&set(files);assert not present or present==closing
assert set(files)==set(rows)|{'STAGE-CAPTURE.json','ADOPTION-STATUS.json'}|present
for name,row in rows.items():
 rel=safe(name);f=stage.joinpath(*rel.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(stage);data=f.read_bytes();assert (len(data),sha(data),blob(data))==(row['bytes'],row['sha256'],row['git_blob']);assert bool(f.stat().st_mode&0o111)==(row['git_mode']=='100755') and row['git_mode'] in ('100644','100755')
root_capture=(stage/'ROOT-CANDIDATE-CAPTURE.json').read_bytes();assert status['root_candidate_capture_sha256']==sha(root_capture);rr=json.loads(root_capture);assert rr['parent_checkpoint']==parent and len(rr['rows'])==7 and {x['path'] for x in rr['rows']}=={'BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md','ASSESSMENT-CENTRAL-FOLLOWUPS-22-95E.json','MAIN-RECHECK-20261113-1012.json','verify_post76fe_checkpoint.py'}
for row in rr['rows']:
 rel=safe(row['path']);assert len(rel.parts)==1;f=root/rel;assert f.is_file() and not f.is_symlink();data=f.read_bytes();assert (len(data),sha(data),blob(data))==(row['bytes'],row['sha256'],row['git_blob']) and not (f.stat().st_mode&0o111) and row['mode']=='100644'
pins=json.loads((p/'PINS.json').read_bytes());expected={'source-transport/MANIFEST.json','binding-transport/MANIFEST.json','minimal-recovery/MANIFEST.json','process-encrypted/MANIFEST.json','process-receipt/MANIFEST.json','process-binding/MANIFEST.json','process-binding-replay/MANIFEST.json','source-encrypted/MANIFEST.json','source-restoration/MANIFEST.json','source-privacy-scan-v1/MANIFEST.json','source-privacy-scan/MANIFEST.json','source-metadata-binding/MANIFEST.json','source-metadata-binding-scan/MANIFEST.json','central-encrypted/MANIFEST.json','central-receipt/MANIFEST.json','metadata-encrypted/MANIFEST.json','metadata-receipt/MANIFEST.json','application-encrypted/MANIFEST.json','application-custody-receipt/MANIFEST.json','peer-v3/MANIFEST.json','peer-v4/MANIFEST.json','peer-projection/MANIFEST.json','peer-application/MANIFEST.json','reconciliation/VERIFY.py','application/VERIFY.py','ROOT-CANDIDATE-CAPTURE.json','INDEX.md'};assert set(pins['sha256'])==expected
for name,want in pins['sha256'].items():
 f=stage.joinpath(*safe(name).parts);assert f.is_file() and not f.is_symlink() and sha(f.read_bytes())==want
for previous,current,correction in [('INDEX-v1.md','INDEX-v2.md','INDEX-CORRECTION.json'),('INDEX-v2.md','INDEX-v3.md','INDEX-CORRECTION-v3.json'),('INDEX-v3.md','INDEX.md','INDEX-CORRECTION-v4.json')]:
 c=json.loads((stage/correction).read_bytes());assert c['predecessor_sha256']==sha((stage/previous).read_bytes()) and c['successor_sha256']==sha((stage/current).read_bytes())
assert subprocess.check_output(['git','-C',str(root),'rev-parse',parent+'^{tree}']).decode().strip()=='91ba34105092b5c0f3059613fc3f91ad43c784ac'
assert subprocess.check_output(['git','-C',str(root),'rev-parse',source+'^{tree}']).decode().strip()==tree
env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1'
def run(relative,*extra):subprocess.run([sys.executable,'-B',str(stage/relative),*[str(x) for x in extra]],cwd=root,env=env,check=True)
run('source-transport/VERIFY.py');run('binding-transport/VERIFY.py');run('process-encrypted/VERIFY.py');run('process-receipt/VERIFY-CUSTODY.py','--ciphertexts',stage/'process-encrypted');run('process-binding/VERIFY-BINDING.py','--ciphertexts',stage/'process-encrypted','--receipt',stage/'process-receipt')
run('source-encrypted/VERIFY.py');run('source-restoration/VERIFY.py','--ciphertext-dir',stage/'source-encrypted')
for name in ('source-privacy-scan-v1','source-privacy-scan'):run(name+'/VERIFY.py','--ciphertext-dir',stage/'source-encrypted','--restoration-receipt-dir',stage/'source-restoration')
run('source-metadata-binding/VERIFY.py','--ciphertext',stage/'source-encrypted','--restoration',stage/'source-restoration','--scan-predecessor',stage/'source-privacy-scan-v1','--scan-successor',stage/'source-privacy-scan');run('source-metadata-binding-scan/VERIFY.py','--binding-dir',stage/'source-metadata-binding')
for name in ('central','metadata','application'):
 run(name+'-encrypted/VERIFY.py');run(('application-custody-receipt' if name=='application' else name+'-receipt')+'/VERIFY-CUSTODY.py','--ciphertexts',stage/(name+'-encrypted'))
run('peer-v3/VERIFY.py');run('peer-v4/VERIFY.py');run('peer-projection/VERIFY.py','--projection',stage/'reconciliation/PROPOSED-ASSESSMENT.json');run('peer-application/VERIFY.py','--candidate',stage/'application/ADOPTION-CANDIDATE.json','--projection',stage/'reconciliation/PROPOSED-ASSESSMENT.json');run('reconciliation/VERIFY.py','--publication',root)
cmd=['--publication',root]
if args.predecessor:cmd+=['--predecessor',args.predecessor]
run('application/VERIFY.py',*cmd)
git=lambda spec:subprocess.check_output(['git','-C',str(root),'show',parent+':'+spec]);oldq=json.loads(git('RECONCILIATION-059.json'))['entries'];newq=json.loads((root/'RECONCILIATION-059.json').read_bytes())['entries'];oldb=json.loads(git('BASELINE-95E-20261110.json'));newb=json.loads((root/'BASELINE-95E-20261110.json').read_bytes());oldl=oldb['newly_assessed_ledger']['entries'];newl=newb['newly_assessed_ledger']['entries'];report=json.loads((root/'ASSESSMENT-CENTRAL-FOLLOWUPS-22-95E.json').read_bytes());ids={x['index'] for x in report['assessments']};assert len(ids)==21 and len(oldq)==len(newq)==3713 and len(oldl)==1115 and len(newl)==1128 and newl[:1115]==oldl
for i,(a,b) in enumerate(zip(oldq,newq)):
 if i not in ids:assert a==b
 else:assert a['current_reconciliation']=='unassessed' and b['current_reconciliation'].startswith('partial') and {k:v for k,v in b.items() if k not in ('current_reconciliation','further_current_evidence')}=={k:v for k,v in a.items() if k!='current_reconciliation'}
assert sum(x['current_reconciliation'].startswith('partial') for x in newq)==1400 and sum(x['current_reconciliation']=='unassessed' for x in newq)==2313
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_bytes());assert progress['lenses_complete']==progress['queue_records_semantically_reconciled']==0 and progress['repair']=='not started' and progress['product_validation']=='not performed'
obs=json.loads((root/'MAIN-RECHECK-20261113-1012.json').read_bytes());assert obs['observed_revision']==source and obs['observed_tree']==tree
print('Verified installed stage, selected history, source transports, custody receipts and 21-row evidence application. Predecessor, historical objects, same-machine key and later controls remain qualified; no semantic closure or product validation is established.')
