import datetime,hashlib,json,pathlib,re
base=pathlib.Path(__file__).resolve().parent
reg=json.loads((base/'REGISTRATION-005.json').read_text());wanted={2366:[],2367:[]};facts={}
for number,meta in enumerate(reg['inputs']):
 b=pathlib.Path(meta['path']).read_bytes();h=hashlib.sha256(b).hexdigest();assert h==meta['sha256']
 t=b.decode();pos=re.search(r'"entries"\s*:\s*\[',t).end();dec=json.JSONDecoder();parsed=json.loads(t)
 for index in range(2368):
  while t[pos] in ' \n\r\t,':pos+=1
  start=pos;v,pos=dec.raw_decode(t,pos)
  if index in wanted:
   assert v==parsed['entries'][index];data=t[start:pos].encode();wanted[index].append(data)
   with (base/f'q{index}-corrected-entry-{number}.json').open('xb') as f:f.write(data)
   facts[f'{index}-{number}']={'entry_sha256':hashlib.sha256(data).hexdigest(),'byte_start':len(t[:start].encode()),'byte_end_exclusive':len(t[:pos].encode()),'bytes':len(data),'lines':len(data.decode().splitlines()),'whole_sha256':h}
for entries in wanted.values():assert entries[0]==entries[1]
report={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'index_convention':'zero-based entries array','original_and_reconstructed_entry_byte_equal':True,'entries':facts}
with (base/'QUEUE-EXTRACTION-CORRECTED.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n')
print(json.dumps(report,indent=2))
