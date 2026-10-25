import json,pathlib,hashlib
O=pathlib.Path(__file__).parent
R=O.parent.parent

def record_scoped_packet(number,source_ids,groups,findings=None):
 inv=json.loads((O/'inventory.json').read_text()); scope=json.loads((R/'audit/continuation/cumulative-source-scope-parallel-072.json').read_text()); catalog={r['source_id']:r for r in json.loads((R/'recovered-sources/catalog.json').read_text())['rows']};roles=[]
 for sid in source_ids:
  row=next(r for r in inv['rows'] if r['source_id']==sid);assert row['continuation_status']=='incomplete'
  material=scope['files'][int(row['scope_pointer'].split('/')[-1])];assert material['source_id']==sid
  ranges=material['material_ranges'];b=pathlib.Path(catalog[sid]['object']).read_bytes();h=hashlib.sha256(b).hexdigest();assert h==row['source_sha256']
  roles.append({'source_id':sid,'sha256':h,'actual_EOF':len(b.decode().splitlines()),'actual_compared_ranges':ranges,'method':'Every listed qualified material body range personally read and analyzed for lens04. Original exact registration/exclusions retained; no whole foreign file/package credit.','scope_pointer':row['scope_pointer']})
  row.update(continuation_status='personally_compared_scoped04_with_findings',continuation_evidence=[f'SEGOVB-INITIAL04-CONTRACT-{number}'],actual_compared_ranges=ranges)
 n=sum(x['continuation_status']=='personally_compared_scoped04_with_findings' for x in inv['rows']);remaining=[x['source_id'] for x in inv['rows'] if x['continuation_status']=='incomplete'];first=remaining[0] if remaining else 'original roles complete; overlays remain'
 packet={'id':f'SEGOVB-INITIAL04-CONTRACT-{number}','lens':'audit-invariant-operations','source_roles':roles,'groups':groups,'findings':findings or [],'status':'Qualified material ranges personally analyzed; whole04 remains incomplete.','first_unfinished':first,'source_edits':False,'project_execution':'none'}
 (O/f'contract-groups-{number}.json').write_text(json.dumps(packet,indent=2)+'\n');inv.update(continuation_completed=n,total_with_supplied_prior_evidence=612+n,first_unfinished=first);(O/'inventory.json').write_text(json.dumps(inv,indent=2)+'\n');c=json.loads((O/'checkpoint.json').read_text());c.update(personally_added_complete_roles=n,total_scope2853_with_prior_evidence=612+n,last_packet=packet['id'],first_unfinished=first,remaining=f'{2853-612-n} original roles plus246overlays and current shared live/delta obligations; no clearance.');(O/'checkpoint.json').write_text(json.dumps(c,indent=2)+'\n');print(n,612+n,first)
