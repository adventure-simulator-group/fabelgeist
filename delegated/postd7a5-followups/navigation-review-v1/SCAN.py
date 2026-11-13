#!/usr/bin/env python3
import json,re,sys
from pathlib import Path
p=Path(__file__).resolve().parent;names=['REPORT.md','SNAPSHOT.json','VERIFY.py','SCAN.py'];patterns=[re.compile(rb'/home/[A-Za-z0-9_.-]+/'),re.compile(rb'/tmp/726-[A-Za-z0-9_.-]+/'),re.compile(rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'),re.compile(rb'BEGIN '+rb'PRIVATE KEY')];hits=[]
for n in names:
 b=(p/n).read_bytes()
 for i,pat in enumerate(patterns):
  count=len(pat.findall(b))
  if count:hits.append({'file':n,'pattern_index':i,'count':count})
print(json.dumps({'scope':names,'hits':hits,'qualification':'Bounded patterns only; no complete privacy or ciphertext claim.'},sort_keys=True));sys.exit(bool(hits))
