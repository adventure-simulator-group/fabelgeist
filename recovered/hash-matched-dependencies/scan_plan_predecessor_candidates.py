import json,hashlib,re,sys
from pathlib import Path
if len(sys.argv)!=3: raise SystemExit("usage: scan_plan_predecessor_candidates.py SNAPSHOT NEW_DESTINATION")
root=Path(__file__).resolve().parent.parent.parent
dest=Path(sys.argv[2]).absolute()
if dest.exists(): raise FileExistsError("Destination already exists")
snapshot=Path(sys.argv[1]); state=hashlib.sha256()
with snapshot.open("rb") as inp:
 for block in iter(lambda:inp.read(1024*1024),b""): state.update(block)
if state.hexdigest()!="d9fe8bef817fea966bf869df3dbde5dd006f350f5aeef58f09f541a296c1c68f": raise ValueError("Snapshot hash mismatch")
candidate=json.loads((root/"recovered/l13-selected-state-transition-trace/PLAN-CANDIDATE-722-SEARCH.json").read_bytes())
plan=(json.dumps(candidate["template"],indent=2)+"\n").encode()
marker=b"X"*64
if plan.count(marker)!=1:raise ValueError("Plan marker mismatch")
prefix,suffix=plan.split(marker)
plan_target=candidate["plan_sha256"]
dec=json.JSONDecoder();seen_text=set();seen_values=set();found={};blocks=0;roots=0;nodes=0;hashes=set()
def dig(data,ordinal,source):
 h=hashlib.sha256(data).hexdigest()
 if h in hashes:return
 hashes.add(h)
 if hashlib.sha256(prefix+h.encode()+suffix).hexdigest()==plan_target:found[h]=(ordinal,source,data)
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
print("blocks",blocks,"json roots",roots,"unique nodes",nodes,"candidate_hashes",len(hashes),"matches",len(found),flush=True)
if found:
 dest.mkdir(parents=False,exist_ok=False)
 for h,(ordinal,source,data) in found.items():
  (dest/h).write_bytes(data);print(h,ordinal,source,len(data),flush=True)
