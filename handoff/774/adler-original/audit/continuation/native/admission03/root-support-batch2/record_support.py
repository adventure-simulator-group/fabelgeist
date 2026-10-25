import json,hashlib,datetime
from pathlib import Path
def record(label,groups,next_unread,findings=()):
 o=Path(__file__).resolve().parent;q=json.loads((o/'queue-plan.json').read_text(encoding='utf8'))['queue'];c=o/'coverage.jsonl';old=[json.loads(l) for l in c.read_text(encoding='utf8').splitlines()] if c.exists() else[];rows=[]
 for name,g in groups.items():
  for i in g['root_indices']:
   r=next(r for r in q if r['root_queue_index']==i);assert hashlib.sha256(Path(r['resolved_path']).read_bytes()).hexdigest()==r['sha256'];assert i not in [x['root_queue_index'] for x in old+rows]
   rows.append(dict(id=f'NATIVE-ROOTSUPPORT03-BATCH2-SOURCE-{i:03}',root_queue_index=i,source_id=r['source_id'],sha256=r['sha256'],source_scope_record=r['source_scope_record'],mandatory_current_role_overlays=r['mandatory_current_role_overlays'],read_ranges=g.get('ranges',{}).get(i,[[1,r['actual_EOF']]]),current_admission_proof=f'contracts-{label:03}.json#'+name,source_read_only=True,repair_ownership='Rootunchanged',source_later_lens_credit=False))
 p=o/f'contracts-{label:03}.json';assert not p.exists();p.write_text(json.dumps(dict(id=f'NATIVE-ROOTSUPPORT03-BATCH2-CONTRACTS-{label:03}',authority='ROOT-ADM-SUPPORT-AUTHORITY-049',lens='audit-admission-contracts',groups=groups,not_part_of_native443=True,root_ownership_retained=True),ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 with c.open('a',encoding='utf8') as f:
  for r in rows:f.write(json.dumps(r,ensure_ascii=False)+'\n')
 if findings:
  with(o/'findings.jsonl').open('a',encoding='utf8') as f:
   for r in findings:f.write(json.dumps(r,ensure_ascii=False)+'\n')
 (o/'progress.json').write_text(json.dumps(dict(id=f'NATIVE-ROOTSUPPORT-BATCH2-PROGRESS-{label+1:03}',assigned=48,accounted=len(old+rows),remaining=48-len(old+rows),accounted_root_indices=sorted(r['root_queue_index'] for r in old+rows),next_unread=next_unread,source_edits=0,project_execution=0,root_ownership_retained=True,later_lens_credit=False),ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(len(old+rows),'of48')
