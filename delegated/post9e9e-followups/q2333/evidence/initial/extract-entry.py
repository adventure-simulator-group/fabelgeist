from pathlib import Path
import hashlib,json,re,datetime
s=Path(__file__).resolve().parent;r=json.loads((s/'HISTORY-REGISTRATION.json').read_text());out=[]; views=[]
for i,src in enumerate(r['sources']):
 b=Path(src['path']).read_bytes(); h=hashlib.sha256(b).hexdigest();assert h==src['expected_sha256'];t=b.decode(); pos=re.search(r'"entries"\s*:\s*\[',t).end();dec=json.JSONDecoder()
 for n in range(2334):
  while t[pos] in ' \n\r\t,':pos+=1
  start=pos; obj,pos=dec.raw_decode(t,pos)
 data=t[start:pos].encode();(s/f'QUEUE-ENTRY-{i}.json').write_bytes(data);views.append(data);out.append({'role':src['role'],'whole_sha256':h,'entry_sha256':hashlib.sha256(data).hexdigest(),'byte_start':len(t[:start].encode()),'byte_end_exclusive':len(t[:pos].encode()),'lines':len(data.decode().splitlines())})
assert views[0]==views[1]; rec={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entries':out,'selected_entry_byte_equal':True};(s/'QUEUE-EXTRACTION.json').write_text(json.dumps(rec,indent=2)+'\n');print(json.dumps(rec,indent=2))
