import json,pathlib,datetime
R=pathlib.Path('/home/brunosegovia/.codex/worktrees/ae46/adventure-simulator/target/bespoke-audit-774-merged-20261022/reconcile-adler-20261023');G=R/'audit-continuation/aggregate-states';ROWS=json.loads((R/'recovered-sources/catalog.json').read_text())['rows']
def save(number,notes,next_index,findings=(),qualified=()):
 p=json.loads((G/'progress.json').read_text());now=datetime.datetime.now(datetime.timezone.utc).isoformat(); nav={x['scope_index']:x for x in json.loads((G/'inherited-proof-navigation.json').read_text())['rows']};ms=[];added=0
 for i,ns in notes.items():
  row=ROWS[i];inherited=i in nav;prior=i in p['additional_frozen_roles'];m={'scope_index':i,'source_id':row['source_id'],'sha256':row['expected_sha256'],'personally_read_ranges':[[1,row['line_count']]],'aggregate_trace_and_dispositions':ns}
  if inherited:m['coverage_qualification']='Fresh supporting read only; full historical same-lens source proof already counted, no new role credit.';p['fresh_support_for_inherited_roles']=sorted(set(p['fresh_support_for_inherited_roles']+[i]))
  elif not prior:p['additional_frozen_roles'].append(i);added+=1
  else:m['coverage_qualification']='Already counted ahead, supporting only.'
  ms.append(m)
 n=f'contract-groups-{number:03}.json';packet={'id':f'SEGO-INITIAL05-GROUP-{number:03}','at':now,'lens':'audit-aggregate-states','source_members':ms,'findings':list(findings),'qualified_connected_requirements':list(qualified),'discovery':f'{added} newly accounted original roles; other reads support already counted historical roles. Lens incomplete.','repair':'not_started','validation':'not_run'}
 (G/n).write_text(json.dumps(packet,indent=2)+'\n');p['at']=now;p['frozen_roles_accounted']+=added;p['remaining_frozen_roles']=2853-p['frozen_roles_accounted'];p['first_unfinished_scope_index']=next_index;p['packets'].append(n);(G/'progress.json').write_text(json.dumps(p,indent=2)+'\n');row=ROWS[next_index];(G/'ACTIVE-READ.json').write_text(json.dumps({'at':now,'scope_index':next_index,'source_id':row['source_id'],'sha256':row['expected_sha256'],'actual_fresh_ranges':[],'first_unread_line':1,'full_role_counted':False,'previous_completed_role_receipt':n},indent=2)+'\n');print(n,p['frozen_roles_accounted'],'next',next_index)
