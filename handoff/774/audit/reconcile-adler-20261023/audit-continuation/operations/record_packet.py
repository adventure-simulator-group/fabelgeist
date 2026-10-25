import json,pathlib,hashlib
O=pathlib.Path(__file__).parent

def record_packet(number,names,groups,findings=None,fresh=None):
 inv=json.loads((O/'inventory.json').read_text());roles=[]
 for variant in ['physical-p16','pr-working']:
  for name in names:
   f='crates/adventuresim-stdb-module/src/'+name;p=O/'source'/variant/f;b=p.read_bytes();end=len(b.decode().splitlines());roles.append({'source_id':variant+':'+f,'sha256':hashlib.sha256(b).hexdigest(),'actual_EOF':end,'actual_compared_ranges':[[1,end]],'method':'Complete literal P16 body personally read and full actual PR textual comparison consumed; operation contracts reanalyzed for both exact roles. No hash-only coverage.'})
 packet={'id':f'SEGOVB-INITIAL04-CONTRACT-{number}','lens':'audit-invariant-operations','source_roles':roles,'groups':groups,'findings':findings or [],'fresh_dependencies':fresh or [],'status':f'{len(roles)} original roles personally compared; whole04 remains incomplete.','source_edits':False,'project_execution':'none'}
 for row in inv['rows']:
  for role in roles:
   if row['source_id']==role['source_id']:
    assert row['continuation_status']=='incomplete',row['source_id']
    row.update(continuation_status='personally_compared_scoped04_with_findings',continuation_evidence=[packet['id']],actual_compared_ranges=role['actual_compared_ranges'])
 n=sum(x['continuation_status']=='personally_compared_scoped04_with_findings' for x in inv['rows']);remaining=[x['source_id'] for x in inv['rows'] if x['continuation_status']=='incomplete'];packet['first_unfinished']=remaining[0] if remaining else 'original roles complete; overlays remain';(O/f'contract-groups-{number}.json').write_text(json.dumps(packet,indent=2)+'\n');inv.update(continuation_completed=n,total_with_supplied_prior_evidence=612+n,first_unfinished=packet['first_unfinished']);(O/'inventory.json').write_text(json.dumps(inv,indent=2)+'\n');c=json.loads((O/'checkpoint.json').read_text());c.update(personally_added_complete_roles=n,total_scope2853_with_prior_evidence=612+n,last_packet=packet['id'],first_unfinished=packet['first_unfinished'],remaining=f'{2853-612-n} original source roles plus overlay/qualification/delta obligations; no clearance.');(O/'checkpoint.json').write_text(json.dumps(c,indent=2)+'\n');print(n,612+n,packet['first_unfinished'])
