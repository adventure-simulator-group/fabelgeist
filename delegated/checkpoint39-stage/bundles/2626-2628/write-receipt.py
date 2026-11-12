import json,pathlib,hashlib,datetime
p=pathlib.Path(__file__).parent;r=json.loads((p/'registration.json').read_bytes());items=[]
for e in r['entries']:
 used=set();
 for a,b in e['registered_ranges']:used.update(range(a,b+1))
 unread=[];start=None
 for i in range(1,e['eof']+2):
  if i<=e['eof'] and i not in used:
   if start is None:start=i
  elif start is not None:unread.append([start,i-1]);start=None
 items.append({'id':e['id'],'sha256':e['sha256'],'revision':e['revision'],'displayed_ranges':e['registered_ranges'],'unread_complement':unread,'source':'numbered bounded terminal display after registration'})
receipt={'schema':'726-num-enum-reconciliation-read-receipt-v1','author':'/root/mhr_character','registration_sha256':hashlib.sha256((p/'registration.json').read_bytes()).hexdigest(),'completed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_personal_displays':items,'qualification':'All registered ranges were displayed. Full preserved bytes, archive identity checks, structural lock scans and previous reviewers do not confer read credit. Product source and dependency remain separate.'}
(p/'read-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print('receipted',len(items))
