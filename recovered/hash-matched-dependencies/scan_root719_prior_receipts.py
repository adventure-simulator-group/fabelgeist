import json,hashlib,re,sys
from pathlib import Path
if len(sys.argv)!=3: raise SystemExit("usage: scan_root719_prior_receipts.py SNAPSHOT NEW_DESTINATION")
root=Path(__file__).resolve().parent.parent.parent
dest=Path(sys.argv[2]).absolute()
if dest.exists(): raise FileExistsError("Destination already exists")
targets={
"0fdb8c8faa044fe3513459ee36952db71d9e42ce48bb2f9c83fd0d485a06857b":{"receipt-314"},
"983359fd62fcab7e225516239e3873e78204fc66b314293862089f6f8dc1014f":{"receipt-352"},
"808100bf8d5ae5d4b628e6c73388bbdbc7b9c63c78ee61811c50309248ebf9a9":{"receipt-362"}}
snapshot=Path(sys.argv[1]); state=hashlib.sha256()
with snapshot.open("rb") as inp:
 for block in iter(lambda:inp.read(1024*1024),b""): state.update(block)
if state.hexdigest()!="d9fe8bef817fea966bf869df3dbde5dd006f350f5aeef58f09f541a296c1c68f": raise ValueError("Snapshot hash mismatch")
dec=json.JSONDecoder();seen_text=set();seen_values=set();found={};blocks=0;roots=0;nodes=0
def dig(data,ordinal,source):
 h=hashlib.sha256(data).hexdigest()
 if h in targets and h not in found:found[h]=(ordinal,source,data)
def walk(obj,ordinal):
 global nodes
 stack=[obj]
 while stack:
  v=stack.pop()
  if isinstance(v,str):
   if len(v)<2000000:dig(v.encode(),ordinal,"embedded-string")
   continue
  if not isinstance(v,(dict,list)):continue
  stack.extend(v.values() if isinstance(v,dict) else v)
  canonical=json.dumps(v,ensure_ascii=False,separators=(",",":")).encode();key=hashlib.sha256(canonical).digest()
  if key in seen_values:continue
  seen_values.add(key);nodes+=1
  for asc in (True,False):
   for indent in (2,4,None):
    b=json.dumps(v,ensure_ascii=asc,indent=indent).encode()
    for suffix in (b"",b"\n",b"\r\n"):dig(b+suffix,ordinal,"reserialized-json")
with snapshot.open(encoding="utf8") as f:
 for line in f:
  e=json.loads(line);payload=e.get("payload",{})
  if e.get("type")!="response_item" or payload.get("type") not in ("custom_tool_call_output","function_call_output"):continue
  raw=payload.get("output",[]);texts=[raw] if isinstance(raw,str) else [z.get("text","") for z in raw if isinstance(z,dict)]
  for text in texts:
   if not text:continue
   key=hashlib.sha256(text.encode()).digest()
   if key in seen_text:continue
   seen_text.add(key);blocks+=1;dig(text.encode(),e["ordinal"],"output-block")
   candidates=[text]
   try:
    wrapper=json.loads(text[text.index("{"):]);v=wrapper.get("result",{}).get("value",{}) if isinstance(wrapper,dict) and isinstance(wrapper.get("result"),dict) else wrapper
    if isinstance(v,dict) and isinstance(v.get("output"),str):candidates.append(v["output"])
   except (ValueError,KeyError):pass
   for body in candidates:
    dig(body.encode(),e["ordinal"],"unwrapped-output")
    for m in re.finditer(r"(?m)^[ \t]*([\{\[])",body):
     try:value,end=dec.raw_decode(body,m.start(1))
     except (ValueError,RecursionError):continue
     if isinstance(value,(dict,list)):
      roots+=1;walk(value,e["ordinal"])
print("blocks",blocks,"json roots",roots,"unique nodes",nodes,"matches",len(found),flush=True)
if found:
 dest.mkdir(parents=False,exist_ok=False)
 for h,(ordinal,source,data) in found.items():
  (dest/h).write_bytes(data);print(h,sorted(targets[h]),ordinal,source,len(data),flush=True)
