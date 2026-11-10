import json,hashlib,re,sys
from pathlib import Path
if len(sys.argv)!=3: raise SystemExit("usage: scan_state_hashes.py SNAPSHOT NEW_DESTINATION")
root=Path(__file__).resolve().parent.parent.parent
dest=Path(sys.argv[2]).absolute()
if dest.exists(): raise FileExistsError("Destination already exists")
targets={}
def add(h,label):
 if isinstance(h,str) and len(h)==64:targets.setdefault(h,set()).add(label)
a=json.loads((root/"recovered/l13-late-post-apply-references/MANIFEST.json").read_bytes())
for r in a["roots"]:
 for k in ("ledger","selector","queue","selected_lenses"):add(r[k]["sha256"],f"post-{r['root']}-{k}")
a=json.loads((root/"recovered/l13-late-revisit-references/MANIFEST.json").read_bytes())
for r in a["rows"]:
 for k in ("ledger","selector","queue","selected_lenses","verification"):
  if r.get(k):add(r[k]["sha256"],f"revisit-{r['queue_number']}-{k}")
a=json.loads((root/"recovered/l13-late-verification-references/MANIFEST.json").read_bytes())
for r in a["rows"]:add(r["verification"]["sha256"],f"verification-{r['root']}")
a=json.loads((root/"recovered/l13-plan-reference-inventory/MANIFEST.json").read_bytes())
for r in a["rows"]:
 if not r["plan_original_recovered"]:add(r["plan"]["sha256"],f"plan-{r['root']}")
a=json.loads((root/"RECOVERED-FRONTIER-3821.json").read_bytes());add(a["reported_checkpoint_sha256"],"final-checkpoint-3821");add(a["reported_queue_sha256"],"final-queue-4156")
print("target hashes",len(targets),flush=True)
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
