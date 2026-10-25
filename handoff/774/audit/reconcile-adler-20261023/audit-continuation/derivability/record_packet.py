import json,pathlib,hashlib,datetime
D=pathlib.Path(__file__).parent

def record_packet(number,cells,groups,findings=None):
 inv=json.loads((D/'inventory.json').read_text());roles=[]
 for cell in cells:
  i=cell['scope_index'];r=inv['rows'][i];assert r['status']=='PENDING_OWN_LENS10_SOURCE_ANALYSIS'
  p=pathlib.Path(r['source_provenance']['object']);b=p.read_bytes();assert hashlib.sha256(b).hexdigest()==r['sha256']
  role={'scope_index':i,'source_id':r['source_id'],'sha256':r['sha256'],'actual_EOF':len(b.decode().splitlines()),'requested_registration':r['registration'],'method':'Every stated exact material range personally consumed and analyzed for lens10; source-backed trait inventory/absence only, no other-lens or search-only coverage.',**cell}
  r.update(status='PERSONALLY_ANALYZED_LENS10_WITH_QUALIFICATIONS',actual_compared_ranges=cell['actual_compared_ranges'],trait_inventory=cell.get('trait_inventory',[]),evidence=[f'SEGOVB-INITIAL10-CONTRACT-{number:03}']);roles.append(role)
 todo=[r for r in inv['rows'] if r['status']=='PENDING_OWN_LENS10_SOURCE_ANALYSIS'];first={'dimension':'original','scope_index':todo[0]['scope_index'],'source_id':todo[0]['source_id']} if todo else None
 p={'id':f'SEGOVB-INITIAL10-CONTRACT-{number:03}','at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'lens':'10 audit-derivability','source_roles':roles,'groups':groups,'findings':findings or [],'first_unfinished':first,'execution':'none','status':'Bounded actual lens10 discovery WITH_QUALIFICATIONS; full lens remains incomplete.'};(D/f'contract-groups-{number:03}.json').write_text(json.dumps(p,indent=2)+'\n');inv.update(completed=len(inv['rows'])-len(todo),first_unfinished=first);(D/'inventory.json').write_text(json.dumps(inv,indent=2)+'\n');c=json.loads((D/'checkpoint.json').read_text());c.update(original_completed=inv['completed'],last_packet=p['id'],first_unfinished=first);(D/'checkpoint.json').write_text(json.dumps(c,indent=2)+'\n');print('Lens10',inv['completed'],'/2853 first',first)
