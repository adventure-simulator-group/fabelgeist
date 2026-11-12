#!/usr/bin/env python3
"""Inventory exact 059 hash-link bindings; reads only publication evidence."""
from pathlib import Path
import argparse,datetime,gzip,hashlib,json,tarfile
ap=argparse.ArgumentParser();ap.add_argument('--root',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
r=args.root;sha=lambda b:hashlib.sha256(b).hexdigest();targets={'queue':'2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08','ledger':'3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac'};direct=[];archive_refs=[];counts=[]
for f in sorted(r.glob('DELEGATED-CHECKPOINT-*-DIRECT-LINKS.json')):
 raw=f.read_bytes();d=json.loads(raw)
 for i,x in enumerate(d.get('links',[])):
  for kind,h in targets.items():
   if x.get('path')=='baseline-059/'+kind+'.json' or x.get('sha256')==h:
    direct.append({'manifest':f.name,'manifest_sha256':sha(raw),'pointer':'/links/'+str(i),'path':x.get('path'),'sha256':x.get('sha256'),'bytes':x.get('bytes'),'document':kind,'linked_script':'verify_delegated_links_'+f.name.split('-')[2]+'.py' if x.get('path')=='baseline-059/'+kind+'.json' else None})
for cp in range(16,20):
 f=r/f'DELEGATED-EVIDENCE-CHECKPOINT-{cp}.json';index_raw=f.read_bytes();d=json.loads(index_raw)
 for b in d['bundles']:
  archive=r/b['archive']
  with tarfile.open(archive) as t:
   for m in t.getmembers():
    if m.name not in ('historical-manifest.json','evidence-links.json') or not m.isfile():continue
    body=t.extractfile(m).read();x=json.loads(body)
    for array in ('hash_links','links','published'):
     for i,v in enumerate(x.get(array,[])):
      if isinstance(v,dict):
       for kind,h in targets.items():
        if v.get('sha256')==h:
         archive_refs.append({'checkpoint':cp,'checkpoint_index_sha256':sha(index_raw),'bundle_id':b['bundle_id'],'archive':b['archive'],'archive_sha256':b['archive_sha256'],'member':m.name,'member_published_sha256':sha(body),'pointer':'/'+array+'/'+str(i),'document':kind,'expected_sha256':h,'expected_bytes':v.get('bytes'),'recorded_path':v.get('path'),'consumer':'verify_delegated_links_'+str(cp)+'.py'})
def flat(m):
 yield m
 for y in m.get('members',[]):yield from flat(y)
for upto in (18,19):
 hits=[]
 for n in range(14,upto+1):
  for b in json.loads((r/f'DELEGATED-EVIDENCE-CHECKPOINT-{n}.json').read_bytes())['bundles']:
   for m in b['members']:
    for z in flat(m):
     for kind,h in targets.items():
      if z.get('original_sha256')==h:hits.append({'checkpoint':n,'bundle_id':b['bundle_id'],'member':z.get('path'),'document':kind})
 counts.append({'prior_checkpoint_range':[14,upto],'stored_original_sha_match_count':len(hits),'matches':hits,'qualification':'The scripts 18/19 fallback compares member original_sha256, not original_uncompressed_sha256; compressed members with matching decompressed originals do not satisfy that fallback.'})
sdk=json.loads((r/'delegated/reconcile-sdk-noiz-sats-report.json').read_bytes())['authority'];assert sdk['queue_sha256']==targets['queue'] and sdk['ledger_sha256']==targets['ledger']; frozen=json.loads((r/'FROZEN-PROPOSAL-DEPENDENCIES-15.json').read_bytes());saved=[x['prior_path'] for x in frozen['rows']];assert not any(x in saved for x in ('baseline-059/queue.json','baseline-059/ledger.json'))
for cp in range(20,30):
 chosen=[x for x in direct if x['manifest']==f'DELEGATED-CHECKPOINT-{cp}-DIRECT-LINKS.json' and x['path'] in ('baseline-059/queue.json','baseline-059/ledger.json')];assert len(chosen)==2
for cp in range(30,39):assert not [x for x in direct if x['manifest']==f'DELEGATED-CHECKPOINT-{cp}-DIRECT-LINKS.json' and x['path'] in ('baseline-059/queue.json','baseline-059/ledger.json')]
assert all(x['stored_original_sha_match_count']==0 for x in counts)
x={'schema':'726-baseline059-direct-hash-link-inventory-v1','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'direct_link_manifest_records':direct,'archived_hash_link_records_checkpoint16_19':archive_refs,'fallback_index_scan':counts,'other_direct_script_checks':[{'script':'verify_reconciliation_proposal_links_15.py','source_lines':[7,8,9,10,11,16,17,18,19,20,21,22,23,24,25,26,27,28],'report':'delegated/reconcile-sdk-noiz-sats-report.json','report_sha256':sha((r/'delegated/reconcile-sdk-noiz-sats-report.json').read_bytes()),'queue_and_ledger_expected_sha256':targets,'qualification':'The SDK loop reads queue/ledger directly and asserts original digests. Frozen proposal dependencies 15 do not substitute either top-level file. Physics and P16 report links require separate exact treatment, also recorded in loose references.'}], 'predicted_impact':'The checked scripts compare original bytes or original digest against current top-level paths. A redacted replacement cannot satisfy those assertions; this conclusion is from static code and evidence examination, not running any existing verifier. 20–29 each contain explicit path+digest checks; 16–19 archive links include original digests and known-byte checks, including fallback analysis above. Earlier checks may independently fail first.', 'not_a_global_rewrite':'Original-hash declarations inside historical records, member indices and 059 transport links continue to identify original historical bytes. Do not replace those values with reconstruction hashes; add versioned public-representation identities and narrowly adapted verification.', 'scope':'All loose DIRECT-LINKS files were examined; checkpoint 16–19 selected evidence-links/historical-manifest members were examined as named. Other nested archives, dynamic constructed paths and external consumers remain beyond this targeted static analysis.'}
args.output.write_text(json.dumps(x,indent=2)+'\n'); print('direct manifest records',len(direct),'checkpoint16-19 archive references',len(archive_refs))
