#!/usr/bin/env python3
"""Verify successor draft identity and bounded parsed correction scope."""
from pathlib import Path
import json,hashlib,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest()
raw={n:(p/n).read_bytes() for n in ['DRAFT-v2.json','DRAFT-v3.json','DRAFT-v4.json','CORRECTION-v2.json','CORRECTION-v3.json','CORRECTION-v4.json']};d={n:json.loads(b) for n,b in raw.items()}
a,b,c=(d['DRAFT-v2.json'],d['DRAFT-v3.json'],d['DRAFT-v4.json']);x,y,z=(d['CORRECTION-v2.json'],d['CORRECTION-v3.json'],d['CORRECTION-v4.json'])
assert x['successor_sha256']==h(raw['DRAFT-v2.json']) and y['predecessor_sha256']==h(raw['DRAFT-v2.json']) and y['successor_sha256']==h(raw['DRAFT-v3.json']) and z['predecessor_sha256']==h(raw['DRAFT-v3.json']) and z['successor_sha256']==h(raw['DRAFT-v4.json'])
def diff(a,b,path=''):
 if isinstance(a,dict) and isinstance(b,dict):
  return sum((diff(a.get(k),b.get(k),path+'/'+k) if k in a and k in b else [path+'/'+k] for k in sorted(a.keys()|b.keys())),[])
 if isinstance(a,list) and isinstance(b,list):
  if len(a)!=len(b):return [path]
  return sum((diff(x,y,path+'/'+str(i)) for i,(x,y) in enumerate(zip(a,b))),[])
 return [] if a==b else [path]
assert set(diff(a,b))=={'/predecessor_draft_sha256','/sanitization_qualification','/process_qualification','/review_qualification','/partial_queue_assessments/2/record/summary','/partial_queue_assessments/6/record/summary','/partial_queue_assessments/6/record/history','/partial_queue_assessments/6/record/source_refs/2'}
assert set(diff(b,c))=={'/predecessor_draft_sha256','/chronology_qualification','/process_qualification','/successor_created_utc'}
assert a['ledger_relationships']==b['ledger_relationships']==c['ledger_relationships'] and a['counts']==b['counts']==c['counts']
assert [x['record']['queue_index'] for x in c['partial_queue_assessments']]==[3243,3246,3255,3258,3244,3247,3256,3259]
print('Verified v2-v4 draft identities and exact parsed correction scopes; v1 original, displayed-event attestations and restricted evidence remain separate.')
