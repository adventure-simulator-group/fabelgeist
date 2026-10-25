import json,hashlib,datetime
from pathlib import Path
def record(label,groups,next_unread):
 O=Path(__file__).resolve().parent;q=json.loads((O/'queue-plan.json').read_text(encoding='utf-8-sig'))['queue'];old=[json.loads(l) for l in (O/'coverage.jsonl').read_text(encoding='utf8').splitlines()];rows=[]
 for name,g in groups.items():
  g['later_lens_credit']=False
  for i in g['indices']:
   r=q[i-1];raw=Path(r['locator_navigation']['resolved_path']).read_bytes();assert hashlib.sha256(raw).hexdigest()==r['sha256']
   ranges=g.get('read_ranges_by_index',{}).get(i,[[1,len(raw.decode('utf-8-sig').splitlines())]])
   rows.append(dict(id=f'NATIVE-ADMISSION03-SOURCE-{i:03}',queue_index=i,source_id=r['source_id'],sha256=r['sha256'],source_scope_record=r['source_scope_record'],mandatory_current_role_overlays=r['mandatory_current_role_overlays'],read_ranges=ranges,read_mode=g.get('read_mode','fresh complete current03 requested body incl originaltests; source-only no execution'),contract_group=f'contract-groups-{label:03}.json#'+name,admission_discovery='accounted_exact_admission_routes_with_source_specific_qualifications',repair='frozen_initial_all13',validation='not_executed',later_lens_credit=False))
 assert not ({r['queue_index'] for r in old}&{r['queue_index'] for r in rows})
 target=O/f'contract-groups-{label:03}.json';assert not target.exists()
 target.write_text(json.dumps(dict(id=f'NATIVE-ADMISSION03-CONTRACT-GROUPS-{label:03}',lens='audit-admission-contracts',groups=groups,later_lens_credit=False),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 with (O/'coverage.jsonl').open('a',encoding='utf8') as f:
  for r in rows:f.write(json.dumps(r,ensure_ascii=False)+'\n')
 p=json.loads((O/'progress.json').read_text(encoding='utf8'));allrows=old+rows
 p.update(id=f'NATIVE-ADMISSION03-PROGRESS-{label+1:03}',at=datetime.datetime.now(datetime.timezone.utc).isoformat(),accounted=len(allrows),remaining=443-len(allrows),accounted_indices=sorted(r['queue_index'] for r in allrows),next_unread=next_unread)
 (O/'progress.json').write_text(json.dumps(p,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(dict(accounted=len(allrows),remaining=443-len(allrows))))
