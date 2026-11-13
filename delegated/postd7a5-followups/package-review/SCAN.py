#!/usr/bin/env python3
"""Bounded pattern scan of this review's plaintext files only."""
import json,re,sys
from pathlib import Path
root=Path(__file__).resolve().parent
names=['REPORT.md','SNAPSHOT.json','VERIFY.py','SCAN.py']
patterns=[re.compile(rb'/home/[A-Za-z0-9_.-]+/'),re.compile(rb'/tmp/726-[A-Za-z0-9_.-]+/'),re.compile(rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'),re.compile(rb'BEGIN '+rb'PRIVATE KEY')]
hits=[]
for name in names:
 b=(root/name).read_bytes()
 for i,pat in enumerate(patterns):
  count=len(pat.findall(b))
  if count:hits.append({'file':name,'pattern_index':i,'count':count})
print(json.dumps({'scope':names,'hits':hits,'qualification':'Bounded patterns only; not a complete privacy or ciphertext scan.'},sort_keys=True))
sys.exit(bool(hits))
