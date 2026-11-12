import datetime,hashlib,json,pathlib,re
base=pathlib.Path(__file__).resolve().parent
prior=json.loads(pathlib.Path('<PRIVATE-PATH-0006>').read_text())
inputs=[('<PRIVATE-PATH-0003>','original_queue_sha256'),('<PRIVATE-PATH-0004>','reconstructed_queue_sha256')]
records={2366:[],2367:[]};facts={}
for number,(path,key) in enumerate(inputs):
 b=pathlib.Path(path).read_bytes();h=hashlib.sha256(b).hexdigest();assert h==prior[key]
 t=b.decode();pos=re.search(r'"entries"\s*:\s*\[',t).end();dec=json.JSONDecoder()
 for index in range(1,2368):
  while t[pos] in ' \n\r\t,':pos+=1
  start=pos;_,pos=dec.raw_decode(t,pos)
  if index in records:
   data=t[start:pos].encode();records[index].append(data)
   with (base/f'q{index}-entry-{number}.json').open('xb') as f:f.write(data)
   facts[f'{index}-{number}']={'entry_sha256':hashlib.sha256(data).hexdigest(),'byte_start':len(t[:start].encode()),'byte_end_exclusive':len(t[:pos].encode()),'bytes':len(data),'lines':len(data.decode().splitlines()),'whole_sha256':h}
for index,entries in records.items():assert entries[0]==entries[1]
report={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_and_reconstructed_entry_byte_equal':True,'entries':facts}
with (base/'QUEUE-EXTRACTION.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
print(json.dumps(report,indent=2))
