#!/usr/bin/env python3
"""Check an additive CP7 evidence reconstruction; never run project code."""
import argparse,gzip,hashlib,io,json,re,tarfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--original',type=Path);args=p.parse_args()
R=Path(__file__).resolve().parent
# When published, the index and archive live together under delegated/; CP7
# checkpoint index is one directory above. This also works in a staged tree.
root=R.parent if (R.parent/'DELEGATED-EVIDENCE-CHECKPOINT-7.json').exists() else R.parent.parent
idx=json.loads((R/'INDEX.json').read_text()); sha=lambda b:hashlib.sha256(b).hexdigest()
cp=json.loads((root/'DELEGATED-EVIDENCE-CHECKPOINT-7.json').read_text());b=next(x for x in cp['bundles'] if x['bundle_id']=='strategic-map');m=next(x for x in b['members'] if x['path']=='prior-web-map-route-packet.tar.gz');assert m['original_sha256']==idx['source_original_sha256'] and m['original_bytes']==idx['source_original_bytes'] and m['published_sha256']==idx['previous_public_sha256']
with tarfile.open(root/b['archive'],'r:gz') as outer:
 damaged=outer.extractfile(m['path']).read();assert sha(damaged)==idx['previous_public_sha256'];n=0
 try:
  with tarfile.open(fileobj=io.BytesIO(damaged),mode='r:gz') as t:
   for _ in t:n+=1
 except (tarfile.TarError,EOFError):pass
 assert n==5,(n,'unexpected old archive parse extent')
raw=(R/idx['additive_public_archive']).read_bytes();assert (sha(raw),len(raw))==(idx['public_sha256'],idx['public_bytes']);local=re.compile(rb'/(?:home|Users)/[^"\'\s]+|[A-Za-z]:\\+Users\\+[^"\'\s]+|/tmp/726-[^"\'\s]+',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}',re.I)
allowed={(b'garden',b'/home/building'),(b'source',b'/home/window/viewport/overlay')};found=[]
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 ms=t.getmembers();assert len(ms)==idx['entry_count']==len(idx['entries'])==186
 for info,e in zip(ms,idx['entries']):
  assert info.name==e['path'] and (info.isdir() if e['kind']=='directory' else info.isfile())
  assert not info.name.startswith('/') and '..' not in Path(info.name).parts
  assert not local.search(str((info.name,info.uname,info.gname,info.pax_headers)).encode()) and not secret.search(str((info.name,info.uname,info.gname,info.pax_headers)).encode())
  if info.isdir():continue
  body=t.extractfile(info).read();assert (sha(body),len(body))==(e['public_sha256'],e['public_bytes'])
  plain=gzip.decompress(body) if e['encoding']=='gzip' else body;assert sha(plain)==e['public_uncompressed_sha256'] and not secret.search(plain)
  for match in local.finditer(plain):
   i=match.start();pre=next((pre for pre,val in allowed if match.group()==val and plain[max(0,i-len(pre)):i]==pre),None);assert pre is not None,(info.name,'unexpected private path');found.append((info.name,pre.decode(),match.group().decode()))
assert len(found)==len(idx['prose_restorations'])==2 and sum(e.get('redacted_path_occurrences',0) for e in idx['entries'])==idx['path_redactions']==1085
assert sorted(found)==sorted((e['path'],e['preceding_prose'],e['restored_slash_text']) for e in idx['prose_restorations'])
if args.original:
 original=args.original.read_bytes();assert (sha(original),len(original))==(idx['source_original_sha256'],idx['source_original_bytes'])
 with tarfile.open(fileobj=io.BytesIO(original),mode='r:gz') as o,tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  oms=o.getmembers();pms=t.getmembers();assert len(oms)==len(pms)==186
  for oi,pi,e in zip(oms,pms,idx['entries']):
   assert (oi.name,oi.type,oi.mode,oi.uid,oi.gid,oi.uname,oi.gname,oi.mtime)==(pi.name,pi.type,pi.mode,pi.uid,pi.gid,pi.uname,pi.gname,pi.mtime)
   if oi.isdir():continue
   body=o.extractfile(oi).read();plain=gzip.decompress(body) if e['encoding']=='gzip' else body;assert (sha(body),len(body),sha(plain))==(e['original_sha256'],e['original_bytes'],e['original_uncompressed_sha256'])
   pieces=[];last=0;mapping={};count=0
   for match in local.finditer(plain):
    i=match.start();v=match.group();pieces.append(plain[last:i]);keep=any(v==val and plain[max(0,i-len(pre)):i]==pre for pre,val in allowed)
    if keep:pieces.append(v)
    else:
     if v not in mapping:mapping[v]=len(mapping)
     pieces.append(('<LOCAL_PATH_%d>'%mapping[v]).encode());count+=1
    last=match.end()
   pieces.append(plain[last:]);reconstructed=b''.join(pieces);expected=gzip.compress(reconstructed,mtime=0) if e['encoding']=='gzip' and reconstructed!=plain else body if reconstructed==plain else reconstructed
   assert count==e['redacted_path_occurrences'] and t.extractfile(pi).read()==expected
print('Verified 186-member CP7 tar-aware reconstruction and previous five-member limitation; private original '+('also checked' if args.original else 'not supplied')+'; no product execution or read credit inferred.')
