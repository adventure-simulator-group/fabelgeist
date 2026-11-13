#!/usr/bin/env python3
"""Verify reconstructed public proposal projections without claiming adoption."""
from pathlib import Path
import argparse,copy,hashlib,json,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--q10-original',type=Path);ap.add_argument('--role-original',type=Path);a=ap.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();cross=json.loads((p/'CROSSWALK.json').read_bytes());assert cross['schema']=='726-posta422-proposal-reconstruction-v1' and len(cross['rows'])==2;assert subprocess.check_output(['git','-C',str(a.publication),'rev-parse','a4223bf23487b5872bc5efe5b272d40f859456f2^{tree}']).decode().strip()=='86ca312a55790c3cfcc315c09d6f9c62f5dc39ff';q=json.loads(subprocess.check_output(['git','-C',str(a.publication),'show','a4223bf23487b5872bc5efe5b272d40f859456f2:RECONCILIATION-059.json']))['entries'];assert len(q)==3713
for row,orig in zip(cross['rows'],[a.q10_original,a.role_original]):
 b=(p/row['public_file']).read_bytes();assert h(b)==row['public_sha256'];y=json.loads(b);assert y['repair']=='not started' and y['product_validation']=='not performed' and y['counts']['semantic_queue_closures']==y['counts']['accepted_unique_defect_increment']==y['counts']['lens_completions']==0
 if row['public_file'].startswith('Q10'):
  for z,i,lens in zip(y['partial_queue_assessments'],[10,13],[13,12]):assert z['queue_index']==i and z['historical_lens']==lens and z['historical_contract_id']==q[i]['contract_id'] and q[i]['current_reconciliation']=='unassessed'
 else:
  for z,i,lens in zip(y['partial_queue_assessments'],[3447,3448,3450,3451],[1,2,5,6]):
   r=z['record'];assert r['queue_index']==i and r['historical_lens']==lens and r['historical_contract_id']==q[i]['contract_id'] and q[i]['current_reconciliation']=='unassessed'
 if orig:
  raw=orig.read_bytes();assert h(raw)==row['restricted_original_sha256'];x=json.loads(raw)
  if row['public_file'].startswith('Q10'):
   for i,r in enumerate(x['partial_queue_assessments']):r['delegate']='reviewer-q10-role0057' if i==0 else 'reviewer-q13';r['coordinator']='central-coordinator'
  else:
   for z in x['partial_queue_assessments']:z['record']['delegate']='reviewer-q10-role0057';z['record']['central_reviewer']='central-coordinator'
  assert x==y
print('Verified public proposal identities and a422 queue state; optional private inputs verify exact parsed projections. All proposals remain unadopted and semantic review is separate.')
