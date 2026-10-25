import json,hashlib,sys
from pathlib import Path
out=Path(__file__).resolve().parent
root=out.parent.parent
cat=json.loads((root/'recovered-sources/catalog.json').read_text())['rows']
contract=Path(sys.argv[1]); data=json.loads(contract.read_text())
coverage=out/'coverage.jsonl'
rows=[json.loads(x) for x in coverage.read_text().splitlines() if x.strip()]
inherited=[json.loads(x) for x in (out/'inherited-coverage.jsonl').read_text().splitlines() if x.strip()]
seen={x['source_id'] for x in inherited+rows}
for g in data['groups']:
 for i in g['indices']:
  c=cat[i]; body=Path(c['object']).read_bytes()
  assert hashlib.sha256(body).hexdigest()==(c['expected_sha256'] or c['verified_object_sha256'])
  ranges=g.get('ranges',{}).get(str(i),[[1,c['line_count']]]) if c['expected_sha256'] else []
  row={'id':f'SEGO-INITIAL09-COVERAGE-{len(inherited)+len(rows)+1:03}','scope_index':i,'source_id':c['source_id'],'sha256':c['expected_sha256'],'actual_eof':c['line_count'],'ranges':ranges,'counts_as_new_role':c['source_id'] not in seen,'contract':contract.name,'new_finding_refs':g.get('finding_refs',[]),'limits':'Fresh exact source comparison for lens09 only. No execution, repair or other-lens clearance; scoped ranges and all historical qualifications remain binding.'}
  if not c['expected_sha256']:
   row.update(observation_kind='deleted-role absence and base comparison',actual_eof=0,base_eof=c['line_count'],base_sha256=c['verified_object_sha256'],base_read_ranges=[[1,c['line_count']]],deletion_evidence=f'deleted-role-{i}.json',limits='Current source is absent; base body read only as deletion evidence, never current-body or clearance credit.')
  if c['kind']=='binary':
   row.update(actual_eof=None,ranges=[],observation_kind='binary image source inspected' if c['expected_sha256'] else 'deleted-role absence and binary base comparison',binary_bytes=c['bytes'],base_read_ranges=[] if not c['expected_sha256'] else None,limits='Exact binary image inspected visually; no textual line or enum-source credit. Deleted base is never a current body. Consumer contracts remain separate.')
  rows.append(row);seen.add(c['source_id'])
coverage.write_text(''.join(json.dumps(x)+'\n' for x in rows))
state=json.loads((out/'STATE.json').read_text())
state.update(new_roles=len(seen)-len({x['source_id'] for x in inherited}),total_accounted_roles=len(seen),coverage_observations=len(inherited)+len(rows),latest_contract=contract.name)
state['deleted_roles_accounted']=len({x['source_id'] for x in rows if x.get('observation_kind','').startswith('deleted-role')})
state['present_source_body_roles']=len(seen)-state['deleted_roles_accounted']
next_i=int(sys.argv[2]); c=cat[next_i] if next_i >= 0 else {'source_id':'none: frozen inventory accounted; separate dimensions pending','expected_sha256':None}
state['first_unfinished']={'scope_index':next_i,'source_id':c['source_id'],'sha256':c['expected_sha256']} if next_i >= 0 else None
state['unaccounted_frozen_roles']=len({x['source_id'] for x in cat}-seen)
state['coverage_limits']=f'{len(seen)} distinct roles from {len(inherited)+len(rows)} observations. Progress only. Inherited qualifications and 246 mandatory overlays remain separately accountable. Every scoped range exception is listed in STATE.scoped_range_exceptions and exact coverage rows; role accounting is never whole-file or lens clearance. Findings unresolved.'
(out/'STATE.json').write_text(json.dumps(state,indent=2)+'\n')
(out/'PROGRESS.md').write_text(f'''# Initial 09 enum-shape audit continuation

Status: incomplete. No lens clearance, source edits, checks, tests, CI,
publication or implementation plan.

`STATE.json` records {len(seen)} distinct source roles from
{len(inherited)+len(rows)} observations: 132 inherited roles and
{state['new_roles']} newly accounted roles (including separately marked deletions). `qualification-001.json`
corrects the repeated value-error role. Recovery proves exact source provenance,
not audit coverage.

Latest packet: `{contract.name}`. Earlier packets and all source-specific
qualifications remain applicable. Each coverage row records exact ranges.
All 246 mandatory overlays require explicit reconciliation.
New material outside the frozen catalog is separately registered and recorded in
`source-extensions.jsonl`; its credit is never silently folded into2853.

The eleven inherited findings and all new findings listed in STATE remain unresolved.
New findings cover the carried home-supply Boolean role, the diagnostic-only
NotSelected variant retained in an accepted surface, and fabricated return
observations when no return walk was attempted. Later packets record additional
migration suggestions and open trace candidates. Normal producer qualifications
remain recorded in their contracts.

Next frozen inventory cell: {next_i if next_i >= 0 else 'none'}, `{c['source_id']}`.

Exact scoped exceptions, including all later partial source roles, are recorded
in `STATE.json.scoped_range_exceptions` and each coverage row. Role1811 keeps its
qualified1–209 union. Displayed out-of-scope context receives no credit.
Additional follow-up ranges remain separate in `source-extensions.jsonl`.
''')
print(json.dumps({'roles':len(seen),'observations':len(inherited)+len(rows),'next':next_i,'contract':contract.name}))
