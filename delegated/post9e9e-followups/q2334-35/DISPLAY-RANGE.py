#!/usr/bin/env python3
"""Save one prospectively registered exact source display, exclusively."""
import json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent
source,lo,hi,label=sys.argv[1],int(sys.argv[2]),int(sys.argv[3]),sys.argv[4]
regs=[json.loads(f.read_text()) for f in p.glob("SOURCE*REGISTRATION.json")]
assert any(any(x["path"]==source and x["range"][0]<=lo<=hi<=x["range"][1] for x in r.get("sources",[])) for r in regs)
captures=[]
for f in p.glob("SOURCE*CAPTURE*.json"):
 d=json.loads(f.read_text());captures.extend(d if isinstance(d,list) else [d])
match=[x for x in captures if x["path"]==source];assert len(match)==1
lines=(p/match[0]["capture"]).read_text().splitlines();assert 1<=lo<=hi<=len(lines)
out="".join(f"{i:4d} {lines[i-1]}\n" for i in range(lo,hi+1))
with (p/f"DISPLAY-{label}-{lo}-{hi}.txt").open("x") as f:f.write(out)
print(out,end="")
