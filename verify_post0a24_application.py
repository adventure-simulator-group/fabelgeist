#!/usr/bin/env python3
"""Verify the recorded central audit application, not product behavior."""
import hashlib,io,json,os,subprocess,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
R=Path(__file__).resolve().parent;P=R/'delegated/post0a24-central-application';H=lambda b:hashlib.sha256(b).hexdigest();rec=json.loads((P/'RECORD.json').read_bytes());assert rec['schema']=='726-post0a24-application-evidence-v1' and rec['parent_checkpoint']=='0a2466a7485388575ef272e0a8355b9714204a37';blob=(P/rec['archive']).read_bytes();assert (len(blob),H(blob))==(rec['archive_bytes'],rec['archive_sha256']);files={}
with tarfile.open(fileobj=io.BytesIO(blob),mode='r:gz') as t:
 for m in t.getmembers():
  n=PurePosixPath(m.name);assert m.isfile() and not n.is_absolute() and n.parts[0]=='evidence' and '..' not in n.parts and m.name not in files;b=t.extractfile(m).read();files[m.name]=(b,m.mode)
assert len(files)==len(rec['members'])
for row in rec['members']:
 b,mode=files[row['path']];assert (len(b),H(b),oct(mode))==(row['bytes'],row['sha256'],row['mode'])
def data(name):return files['evidence/'+name][0]
def obj(name):return json.loads(data(name))
env=os.environ.copy();env.update({'GIT_NO_REPLACE_OBJECTS':'1','GIT_NO_LAZY_FETCH':'1'})
for name in ['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']:
 assert subprocess.check_output(['git','show',rec['parent_checkpoint']+':'+name],cwd=R,env=env)==data(name+'.before')
assert obj('RESULT.json')['exit']==0
before=obj('BASELINE-95E-20261110.json.before');after=obj('BASELINE-95E-20261110.json.after');b=before['newly_assessed_ledger']['entries'];a=after['newly_assessed_ledger']['entries'];reports=[json.loads((R/('ASSESSMENT-CENTRAL-FOLLOWUPS-'+x+'-95E.json')).read_bytes()) for x in ('15B','15C')];added=[v for report in reports for v in report['ledger_relationships']];assert len(b)==1078 and len(a)==1084 and a==b+added;before['latest_main_recheck']='MAIN-RECHECK-20261112-2030.json';before['newly_assessed_ledger']['entries']=a;assert before==after
qb=obj('RECONCILIATION-059.json.before');qa=obj('RECONCILIATION-059.json.after');selected=set(range(2355,2370));assert len(qb['entries'])==len(qa['entries'])==3713
for i,(old,new) in enumerate(zip(qb['entries'],qa['entries'])):
 if i in selected:
  assert old['current_reconciliation']=='unassessed' and new['current_reconciliation'].startswith('partial ');copy=dict(old);copy['current_reconciliation']=new['current_reconciliation'];assert copy==new
 else:assert old==new
qb['entries']=qa['entries'];assert qb==qa
pb=obj('CONTINUATION-PROGRESS-95E.json.before');pa=obj('CONTINUATION-PROGRESS-95E.json.after');assert pb['new_current_ledger_candidates']==1078 and pa['new_current_ledger_candidates']==1084 and pa['lenses_complete']==pa['queue_records_semantically_reconciled']==0
snap=obj('FOLLOWUP-POST0A24-EVIDENCE.json.after');assert (R/'FOLLOWUP-POST0A24-EVIDENCE.json').read_bytes()==data('FOLLOWUP-POST0A24-EVIDENCE.json.after') and snap['added_ledger_relationship_ids']==[x['id'] for x in added]
assert data('CONTINUATION.md.before')!=data('CONTINUATION.md.after') and b'checkpoint 0a24' in data('CONTINUATION.md.after')
print('Verified recorded parent and bounded central application snapshots; later working-state changes, semantic truth and product behavior remain separate.')
