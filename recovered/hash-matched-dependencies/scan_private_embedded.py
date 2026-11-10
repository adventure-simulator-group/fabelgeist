import json,hashlib,re,sys
from pathlib import Path
if len(sys.argv)!=3: raise SystemExit("usage: scan_private_embedded.py SNAPSHOT NEW_DESTINATION")
root=Path(__file__).resolve().parent.parent.parent
dest=Path(sys.argv[2]).absolute()
if dest.exists(): raise FileExistsError("Destination already exists")
c=json.loads((root/"recovered/hash-matched-dependencies/DEPENDENCY-COVERAGE.json").read_bytes()); later=json.loads((root/"recovered/hash-matched-dependencies/LATER-MATCHES.json").read_bytes()); missing={r["sha256"] for r in c["references"] if r["availability"]=="not_located_by_hash"}-{r["sha256"] for r in later["matches"]}
snapshot=Path(sys.argv[1])
hash_state=hashlib.sha256()
with snapshot.open("rb") as inp:
 for block in iter(lambda: inp.read(1024*1024),b""): hash_state.update(block)
if hash_state.hexdigest()!="d9fe8bef817fea966bf869df3dbde5dd006f350f5aeef58f09f541a296c1c68f": raise ValueError("Snapshot hash mismatch")
decoder=json.JSONDecoder(); outputs=set(); seen=set(); found={}; roots=0; nodes=0; outs=0
with snapshot.open(encoding="utf8") as stream:
 for line in stream:
  rec=json.loads(line); payload=rec.get("payload",{})
  if rec.get("type")!="response_item" or payload.get("type") not in ("custom_tool_call_output","function_call_output"):continue
  output=payload.get("output",[]); texts=[output] if isinstance(output,str) else [v.get("text","") for v in output if isinstance(v,dict)]
  for text in texts:
   if not text:continue
   digest=hashlib.sha256(text.encode()).digest()
   if digest in outputs:continue
   outputs.add(digest);outs+=1
   for m in re.finditer(r"(?m)^[ \t]*([\{\[])",text):
    try:value,end=decoder.raw_decode(text,m.start(1))
    except (ValueError,RecursionError):continue
    if not isinstance(value,(dict,list)):continue
    roots+=1; stack=[value]
    while stack:
     obj=stack.pop()
     if not isinstance(obj,(dict,list)):continue
     stack.extend(v for v in (obj.values() if isinstance(obj,dict) else obj) if isinstance(v,(dict,list)))
     base=json.dumps(obj,ensure_ascii=False,separators=(",",":")).encode(); key=hashlib.sha256(base).digest()
     if key in seen:continue
     seen.add(key);nodes+=1
     for ascii_only in (True,False):
      for indent in (2,4,None):
       raw=json.dumps(obj,ensure_ascii=ascii_only,indent=indent).encode()
       for suffix in (b"",b"\n",b"\r\n"):
        data=raw+suffix; h=hashlib.sha256(data).hexdigest()
        if h in missing and h not in found:found[h]=(rec["ordinal"],data)
print("outputs",outs,"decoded_roots",roots,"unique_values",nodes,"matches",len(found))
if found:
 dest.mkdir(parents=False,exist_ok=False)
 for h,(ordinal,data) in found.items():
  (dest/h).write_bytes(data);print(h,ordinal,len(data))
