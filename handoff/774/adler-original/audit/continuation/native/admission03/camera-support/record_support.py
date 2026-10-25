import json,hashlib
from pathlib import Path
def record(label,groups,next_unread,findings=()):
 o=Path(__file__).resolve().parent;q=json.loads((o/'queue-plan-resolved-002.json').read_text(encoding='utf8'))['queue'];p=o/'coverage.jsonl';old=[json.loads(l)for l in p.read_text(encoding='utf8').splitlines()]if p.exists()else[];rows=[]
 for name,g in groups.items():
  for i in g['camera_indices']:
   r=next(r for r in q if r['camera_queue_index']==i);assert i not in [x['camera_queue_index']for x in old+rows];assert hashlib.sha256(Path(r['resolved_path']).read_bytes()).hexdigest()==r['sha256']
   rows.append(dict(id=f'NATIVE-CAMERA-SUPPORT03-SOURCE-{i:03}',camera_queue_index=i,source_id=r['source_id'],sha256=r['sha256'],source_scope_record=r['source_scope_record'],mandatory_current_role_overlays=r['mandatory_current_role_overlays'],read_ranges=g.get('ranges',{}).get(i,r['assigned_material_ranges']),current_admission_proof=f'contracts-{label:03}.json#'+name,camera_final_audit_authority_retained=True,root_repair_authority_retained=True,source_edits=0,later_lens_credit=False))
 f=o/f'contracts-{label:03}.json';assert not f.exists();f.write_text(json.dumps(dict(id=f'NATIVE-CAMERA-SUPPORT03-CONTRACTS-{label:03}',authority='ROOT-ADM-SUPPORT-AUTHORITY-054',lens='audit-admission-contracts',groups=groups,not_part_of_native443_or_root_batches=True),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 with p.open('a',encoding='utf8')as f:
  for r in rows:f.write(json.dumps(r,ensure_ascii=False)+'\n')
 if findings:
  with(o/'findings.jsonl').open('a',encoding='utf8')as f:
   for r in findings:f.write(json.dumps(r,ensure_ascii=False)+'\n')
 (o/'progress.json').write_text(json.dumps(dict(id=f'NATIVE-CAMERA-SUPPORT03-PROGRESS-{label:03}',assigned=150,accounted=len(old+rows),remaining=150-len(old+rows),accounted_camera_indices=sorted(x['camera_queue_index']for x in old+rows),next_unread=next_unread,source_edits=0,project_execution=0,camera_final_audit_authority_retained=True,root_repair_authority_retained=True,later_lens_credit=False),ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(len(old+rows),'of150')
