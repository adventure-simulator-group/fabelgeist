import json,pathlib,hashlib,datetime
p=pathlib.Path(__file__).resolve().parent;r=json.loads((p/'HISTORY-SELECTOR-REGISTRATION.json').read_text());groups={'A':['history/00.json','history/01.json','history/02.json','history/03.json'],'B':['history/04.json','history/05.json','history/06.json','history/07.json'],'C':['history/08.json','history/09.json']}
for label,names in groups.items():
 out={}
 for name in names:
  x=json.loads((p/name).read_text());spec=r['selectors'][name]
  if spec=='whole':out[name]=x;continue
  y={k:x[k] for k in r['shared_metadata'] if k in x}
  for ptr in spec:
   q=x
   for comp in ptr.strip('/').split('/'):q=q[int(comp)] if isinstance(q,list) else q[comp]
   y[ptr]=q
  out[name]=y
 data=(json.dumps(out,indent=2,ensure_ascii=True)+'\n').encode();f=p/f'HISTORY-SELECTED-{label}.json'
 with f.open('xb') as dst:dst.write(data)
 print(label,len(data),hashlib.sha256(data).hexdigest())
