import json,pathlib,hashlib,datetime
D=pathlib.Path(__file__).parent

def record_partial(number,index,ranges,analysis,traits=None,derives=None,next_line=None):
 inv=json.loads((D/'inventory.json').read_text());r=inv['rows'][index];p=pathlib.Path(r['source_provenance']['object']);b=p.read_bytes();assert hashlib.sha256(b).hexdigest()==r['sha256'];assert r['status']=='PENDING_OWN_LENS10_SOURCE_ANALYSIS'
 packet={'id':f'SEGOVB-INITIAL10-CONTRACT-{number:03}','at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'lens':'10 audit-derivability','status':'PARTIAL_SOURCE_ROLE_INCOMPLETE','scope_index':index,'source_id':r['source_id'],'sha256':r['sha256'],'actual_compared_ranges':ranges,'existing_derives':derives or [],'trait_inventory':traits or [],'analysis':analysis,'first_unfinished':{'scope_index':index,'line':next_line},'execution':'none'}
 (D/f'contract-groups-{number:03}.json').write_text(json.dumps(packet,indent=2)+'\n');c=json.loads((D/'checkpoint.json').read_text());c.update(last_packet=packet['id'],active_partial={'scope_index':index,'source_id':r['source_id'],'actual_compared_ranges':ranges,'next_unread_line':next_line,'evidence':packet['id']});(D/'checkpoint.json').write_text(json.dumps(c,indent=2)+'\n');print(packet['id'],'partial',index,'next',next_line)
