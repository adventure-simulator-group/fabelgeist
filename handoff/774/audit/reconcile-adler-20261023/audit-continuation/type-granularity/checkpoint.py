import json,pathlib,hashlib
O=pathlib.Path(__file__).resolve().parent;R=O.parent.parent
C=json.loads((R/'recovered-sources/catalog.json').read_text())['rows'];S=json.loads((R/'audit/continuation/cumulative-source-scope-parallel-072.json').read_text())['files']
def save(number,groups,next_index,findings=None):
 fn=f'contracts-{number:03d}.json';data=dict(id=f'SEGO-INITIAL11-CONTRACT-{number:03d}',lens='audit-type-granularity',groups=groups,new_findings=[f['id'] for f in (findings or [])],limits='Exact pinned registered ranges personally read for11; no other-lens source credit, source edits, tests/checks, native execution, clearance or implementation plan.')
 (O/fn).write_text(json.dumps(data,indent=2)+'\n')
 with (O/'coverage.jsonl').open('a') as f:
  for g in groups:
   for p in g['sources']:
    x=C[p['scope_index']];f.write(json.dumps(dict(source_id=x['source_id'],scope_index=x['scope_index'],sha256=x.get('expected_sha256') or x.get('verified_object_sha256'),ranges=p['ranges'],actual_eof=x['line_count'],contract=fn,status='accounted_with_findings' if g.get('finding_refs') else 'accounted_no_new_finding',qualification=p.get('qualification'),finding_refs=g.get('finding_refs',[])))+'\n')
 if findings:
  with (O/'findings.jsonl').open('a') as f:
   for x in findings:f.write(json.dumps(x)+'\n')
 coverage=[json.loads(l) for l in (O/'coverage.jsonl').read_text().splitlines() if l];idxs={q['scope_index'] for q in coverage};s=json.loads((O/'STATE.json').read_text());s.update(latest_contract=fn,frozen_roles_accounted=len(idxs),first_unfinished=dict(scope_index=next_index,source_id=C[next_index]['source_id']),status='incomplete');(O/'STATE.json').write_text(json.dumps(s,indent=2)+'\n');(O/'PROGRESS.md').write_text(f'# Type-granularity discovery\n\n{len(idxs)}/2853 frozen exact roles accounted at registered ranges. Latest\n{fn}; first unfinished {next_index} {C[next_index]["source_id"]}.\nMandatory 246 overlays, merged 38 and shared {s.get("shared_snapshot")} remain separately pending.\nNo repaired or passing verdict; all source-specific qualifications retained.\nNo source edits, tests, project checks, execution or implementation plan.\n')
 print('SAVED',fn,'roles',len(idxs),'next',next_index)
