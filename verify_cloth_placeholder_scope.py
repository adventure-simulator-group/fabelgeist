#!/usr/bin/env python3
"""Check file-local cloth redaction markers, optionally with private originals."""
import argparse,hashlib,json,re,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;q=json.loads((r/'QUALIFICATION-CLOTH-PLACEHOLDER-SCOPE.json').read_bytes());a=argparse.ArgumentParser();a.add_argument('--private-root',type=Path);args=a.parse_args();H=lambda b:hashlib.sha256(b).hexdigest()
with tarfile.open(r/q['public_packet'],'r:gz') as t:
 assert H(t.extractfile('MANIFEST.json').read())==q['public_manifest_sha256'];index=json.load(t.extractfile('INDEX.json'));rows={x['path']:x for x in index['files']};values=[]
 for name in q['records']:
  row=rows[name];public=t.extractfile('reconstructed/'+name).read();assert (H(public),len(public))==(row['public_stored_sha256'],row['public_stored_bytes']) and row['edits']==[{'kind':'local path token','marker':q['marker'],'count':1}] and public.count(q['marker'].encode())==1
  if args.private_root:
   original=(args.private_root/'cloth/original'/name).read_bytes();assert (H(original),len(original))==(row['original_stored_sha256'],row['original_stored_bytes']);matches=list(re.finditer(rb'/' + b'home' + rb'/[^\s"\x27<>]+',original));assert len(matches)==1;value=matches[0].group();assert original.replace(value,q['marker'].encode(),1)==public;values.append(value)
 if args.private_root:assert len(values)==2 and values[0]!=values[1];print('Verified two distinct private path values and each file-local substitution without displaying values.')
 else:print('Verified public file-local marker identities; private values were not supplied.')
