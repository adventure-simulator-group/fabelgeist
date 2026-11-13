#!/usr/bin/env python3
"""Verify the installed evidence checkpoint without executing product code."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,os,subprocess,sys

if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;stage=p.parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--predecessor',type=Path);a=ap.parse_args();root=a.publication.resolve();assert stage==(root/'delegated/post8ca-followups').resolve() and not (root/'delegated/post8ca-followups').is_symlink();h=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
parent='8caeb2003d10d08320566206817aa0c694231778';source='95e63628f9d70b8dd6e6fede64334696c6995009';tree='1eec97de340d9b7debbb30ca0c0fb2240a73b93f'
def safe(name):
 q=PurePosixPath(name);assert not q.is_absolute() and q.parts and '..' not in q.parts;return q
capture_raw=(stage/'STAGE-CAPTURE.json').read_bytes();capture=json.loads(capture_raw);status=json.loads((stage/'ADOPTION-STATUS.json').read_bytes());assert capture['schema']=='726-post8ca-stage-capture-v1' and capture['parent_checkpoint']==parent and capture['source_revision']==source and capture['source_tree']==tree
assert status['schema']=='726-post8ca-installation-status-v1' and status['parent_checkpoint']==parent and status['stage_capture_sha256']==h(capture_raw) and status['root_paths_matched']==7
paths=list(stage.rglob('*'));assert all(not x.is_symlink() and (x.is_file() or x.is_dir()) for x in paths);assert not any(x.name=='__pycache__' or x.suffix=='.pyc' for x in paths)
files={x.relative_to(stage).as_posix():x for x in paths if x.is_file()};rows={x['path']:x for x in capture['rows']};assert len(rows)==len(capture['rows'])
assert set(files)==set(rows)|{'STAGE-CAPTURE.json','ADOPTION-STATUS.json'} and status['stage_files_matched']==len(rows)
for name,row in rows.items():
 rel=safe(name);f=stage.joinpath(*rel.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(stage);b=f.read_bytes();assert (len(b),h(b),blob(b))==(row['bytes'],row['sha256'],row['git_blob']);assert row['git_mode'] in ('100644','100755') and bool(f.stat().st_mode&0o100)==(row['git_mode']=='100755')
root_capture=(stage/'ROOT-CANDIDATE-CAPTURE-v3.json').read_bytes();assert status['root_candidate_capture_sha256']==h(root_capture);rr=json.loads(root_capture);v2_raw=(stage/'ROOT-CANDIDATE-CAPTURE-v2.json').read_bytes();v2=json.loads(v2_raw);v1_raw=(stage/'ROOT-CANDIDATE-CAPTURE.json').read_bytes();v1=json.loads(v1_raw)
assert rr['schema']=='726-post8ca-root-candidate-capture-v3' and rr['predecessor_capture_sha256']==h(v2_raw) and v2['schema']=='726-post8ca-root-candidate-capture-v2' and v2['predecessor_capture_sha256']==h(v1_raw) and rr['parent_checkpoint']==v2['parent_checkpoint']==v1['parent_checkpoint']==parent and rr['source_revision']==source
expected_root={'BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md','ASSESSMENT-CENTRAL-FOLLOWUPS-23-95E.json','MAIN-RECHECK-POST8CA.json','verify_post8ca_checkpoint.py'}
r1={x['path']:x for x in v1['rows']};r2={x['path']:x for x in v2['rows']};r3={x['path']:x for x in rr['rows']};assert len(r1)==len(r2)==len(r3)==7 and set(r1)==set(r2)==set(r3)==expected_root
assert {n for n in expected_root if r1[n]!=r2[n]}=={'ASSESSMENT-CENTRAL-FOLLOWUPS-23-95E.json'} and {n for n in expected_root if r2[n]!=r3[n]}=={'verify_post8ca_checkpoint.py'}
for row in rr['rows']:
 rel=safe(row['path']);assert len(rel.parts)==1;f=root/rel;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b),blob(b))==(row['bytes'],row['sha256'],row['git_blob']) and not (f.stat().st_mode&0o100) and row['mode']=='100644'
pins=json.loads((p/'PINS.json').read_bytes());expected={'source-transport/MANIFEST.json','pending-source/MANIFEST.json','source-encrypted/MANIFEST.json','source-custody-receipt/MANIFEST.json','source-custody-git/MANIFEST.json','source-mode-peer-review/MANIFEST.json','process-encrypted/MANIFEST.json','process-receipt/MANIFEST.json','later-encrypted/MANIFEST.json','later-receipt/MANIFEST.json','central-peer-review/MANIFEST.json','central-v4-peer-review/MANIFEST.json','central-v5-peer-review/MANIFEST.json','central-v6-peer-review/MANIFEST.json','projection-peer-review/MANIFEST.json','peer-application-v1/MANIFEST.json','peer-application-v2/MANIFEST.json','reconciliation/VERIFY.py','reconciliation/REVIEWER-QUALIFICATION.json','application/MANIFEST.json','application-v2/MANIFEST.json','APPLICATION-CORRECTION-v2.json','ROOT-CANDIDATE-CAPTURE.json','ROOT-CANDIDATE-CAPTURE-v2.json','ROOT-CANDIDATE-CAPTURE-v3.json','INDEX.md'};assert pins['schema']=='726-post8ca-stage-pins-v1' and set(pins['sha256'])==expected
for name,want in pins['sha256'].items():
 f=stage.joinpath(*safe(name).parts);assert f.is_file() and not f.is_symlink() and h(f.read_bytes())==want
assert subprocess.check_output(['git','-C',str(root),'rev-parse',parent+'^{tree}']).decode().strip()=='246d1442c001f653da8f2ea3094620473aae1c32' and subprocess.check_output(['git','-C',str(root),'rev-parse',source+'^{tree}']).decode().strip()==tree
env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1'
def run(path,*args):subprocess.run([sys.executable,'-B',str(stage/path),*[str(x) for x in args]],cwd=root,env=env,check=True)
run('source-transport/VERIFY.py');run('pending-source/VERIFY.py');run('source-encrypted/VERIFY.py')
run('source-custody-git/VERIFY.py','--receipt',stage/'source-custody-receipt','--ciphertexts',stage/'source-encrypted','--final-scan',stage/'SOURCE-CUSTODY-FINAL-SCAN.json')
run('source-mode-peer-review/VERIFY.py');run('process-encrypted/VERIFY.py');run('process-receipt/VERIFY-RECEIPT.py','--ciphertexts',stage/'process-encrypted')
run('later-encrypted/VERIFY.py');run('later-receipt/VERIFY-RECEIPT.py','--ciphertexts',stage/'later-encrypted')
for name in ('central-peer-review','central-v4-peer-review','central-v5-peer-review','central-v6-peer-review','projection-peer-review','peer-application-v1','peer-application-v2'):run(name+'/VERIFY.py')
run('reconciliation/VERIFY.py','--publication',root,'--projection',stage/'reconciliation/PROPOSED-ASSESSMENT.json')
cmd=['--publication',root]
if a.predecessor:cmd+=['--predecessor',a.predecessor]
run('application-v2/VERIFY.py',*cmd)
print('Nested 8ca predecessor guard replayed.' if a.predecessor else 'Nested 8ca predecessor guard not supplied; that replay was skipped.')
print('Ciphertext checks are no-key identity checks; authentication and prior-inventory comparisons are recorded separately, not repeated here.')
git=lambda name:subprocess.check_output(['git','-C',str(root),'show',parent+':'+name]);oldq=json.loads(git('RECONCILIATION-059.json'))['entries'];newq=json.loads((root/'RECONCILIATION-059.json').read_bytes())['entries'];oldb=json.loads(git('BASELINE-95E-20261110.json'));newb=json.loads((root/'BASELINE-95E-20261110.json').read_bytes());oldl=oldb['newly_assessed_ledger']['entries'];newl=newb['newly_assessed_ledger']['entries'];report=json.loads((root/'ASSESSMENT-CENTRAL-FOLLOWUPS-23-95E.json').read_bytes());ids={x['index'] for x in report['assessments']};assert ids=={24,185,689,3100} and len(oldq)==len(newq)==3713 and len(oldl)==1128 and len(newl)==1135 and newl[:1128]==oldl
for i,(before,after) in enumerate(zip(oldq,newq)):
 if i not in ids:assert before==after
 else:assert before['current_reconciliation']=='unassessed' and after['current_reconciliation'].startswith('partial') and {k:v for k,v in after.items() if k not in ('current_reconciliation','further_current_evidence')}=={k:v for k,v in before.items() if k!='current_reconciliation'}
assert sum(x['current_reconciliation'].startswith('partial') for x in newq)==1404 and sum(x['current_reconciliation']=='unassessed' for x in newq)==2309
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_bytes());assert progress['lenses_complete']==progress['queue_records_semantically_reconciled']==0 and progress['repair']=='not started' and progress['product_validation']=='not performed'
obs=json.loads((root/'MAIN-RECHECK-POST8CA.json').read_bytes());assert obs['observed_revision']==source and obs['observed_tree']==tree
print('Verified installed evidence stage, selected history, transports, custody receipts and four-row application. Historical objects, same-machine key, later controls, audit closure and product behavior remain separately qualified.')
