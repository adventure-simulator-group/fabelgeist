from pathlib import Path
import hashlib,json,re,datetime
s=Path(__file__).resolve().parent;r=json.loads((s/'HISTORY-REGISTRATION.json').read_text());inputs=[('<LOCAL_PATH_0004>','original_queue_sha256'),('<LOCAL_PATH_0005>','reconstructed_queue_sha256')];out=[];entries=[]
for n,(path,key) in enumerate(inputs):
 b=Path(path).read_bytes();assert hashlib.sha256(b).hexdigest()==r[key];t=b.decode();pos=re.search(r'"entries"\s*:\s*\[',t).end();dec=json.JSONDecoder()
 for _ in range(2363):
  while t[pos] in ' \n\r\t,':pos+=1
  start=pos;_,pos=dec.raw_decode(t,pos)
 data=t[start:pos].encode();entries.append(data)
 with (s/f'QUEUE-ENTRY-{n}.json').open('xb') as f:f.write(data)
 out.append({'whole_sha256':hashlib.sha256(b).hexdigest(),'entry_sha256':hashlib.sha256(data).hexdigest(),'byte_start':len(t[:start].encode()),'byte_end_exclusive':len(t[:pos].encode()),'lines':len(data.decode().splitlines())})
assert entries[0]==entries[1];v={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_and_reconstructed_entry_byte_equal':True,'entries':out}
with (s/'QUEUE-EXTRACTION.json').open('x') as f:json.dump(v,f,indent=2);f.write('\n')
print(json.dumps(v,indent=2))
