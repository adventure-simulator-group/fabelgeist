#!/usr/bin/env python3
"""Inventory literal 059 references in publication delegated tarballs, without extracting."""
from pathlib import Path
import argparse,datetime,gzip,hashlib,io,json,tarfile,sys
ap=argparse.ArgumentParser();ap.add_argument('--root',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
r=args.root;patterns={'baseline_marker':b'baseline-059','queue_path':b'baseline-059/queue.json','ledger_path':b'baseline-059/ledger.json','historical_queue_path':b'CLOSURE-QUEUE-100.json','historical_ledger_path':b'continuation-segovb/ledger.json','original_queue_sha256':b'2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08','original_ledger_sha256':b'3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac'}
sha=lambda b:hashlib.sha256(b).hexdigest();cap=64*1024*1024;limit=8;matches=[];unscanned=[];errors=[];archives=[];cache={};member_count=0;gzip_count=0;inner_tar_count=0

def scan(b):
 key=sha(b)
 if key in cache:return cache[key]
 found={}
 for no,line in enumerate(b.splitlines(),1):
  for k,v in patterns.items():
   n=line.count(v)
   if n:
    x=found.setdefault(k,{'occurrences':0,'lines':[]});x['occurrences']+=n;x['lines'].append(no)
 if len(cache)<50000:cache[key]=found
 return found

def body(b,archive,chain,depth):
 global gzip_count,inner_tar_count
 found=scan(b)
 if found:matches.append({'archive':archive,'member_chain':chain,'representation':'stored/member bytes','bytes':len(b),'sha256':sha(b),'matches':found})
 if not b.startswith(b'\x1f\x8b'):return
 gzip_count+=1
 if depth>=limit:
  unscanned.append({'archive':archive,'member_chain':chain,'reason':'nested compression depth limit','stored_bytes':len(b),'stored_sha256':sha(b)});return
 try:
  with gzip.GzipFile(fileobj=io.BytesIO(b)) as g:unpacked=g.read(cap+1)
 except (OSError,EOFError) as e:
  errors.append({'archive':archive,'member_chain':chain,'reason':'gzip decode '+type(e).__name__});return
 if len(unpacked)>cap:
  unscanned.append({'archive':archive,'member_chain':chain,'reason':'decompressed member exceeds 64 MiB scan cap','stored_bytes':len(b),'stored_sha256':sha(b)});return
 # A tar payload must be parsed as a tar, not searched as opaque bytes only.
 if len(unpacked)>=512 and (unpacked[257:262] == b'ustar' or chain.endswith(('.tar.gz','.tgz'))):
  inner_tar_count+=1
  try:
   with tarfile.open(fileobj=io.BytesIO(unpacked),mode='r:') as t:
    for m in t:
     if not m.isfile():continue
     nested=m.name
     if m.size>cap:
      unscanned.append({'archive':archive,'member_chain':chain+'!'+nested,'reason':'tar member exceeds 64 MiB scan cap','bytes':m.size});continue
     body(t.extractfile(m).read(),archive,chain+'!'+nested,depth+1)
  except (tarfile.TarError,OSError,EOFError) as e: errors.append({'archive':archive,'member_chain':chain,'reason':'nested tar '+type(e).__name__})
 else:
  found=scan(unpacked)
  if found:matches.append({'archive':archive,'member_chain':chain,'representation':'gzip-uncompressed bytes','bytes':len(unpacked),'sha256':sha(unpacked),'matches':found})

start=datetime.datetime.now(datetime.timezone.utc).isoformat(); paths=sorted(p for p in r.rglob('*') if p.is_file() and not p.is_symlink() and '.git' not in p.relative_to(r).parts and p.relative_to(r).parts[0]!='delegated' and p.name.endswith(('.gz','.tgz','.crate')))
for i,p in enumerate(paths,1):
 rel=str(p.relative_to(r));before=p.stat()
 try:
  b=p.read_bytes(); h=sha(b)
  if len(b)>cap:unscanned.append({'archive':rel,'reason':'stored file exceeds 64 MiB cap','bytes':len(b),'sha256':h})
  else:body(b,rel,'<file>',0)
  after=p.stat();archives.append({'path':rel,'sha256':h,'bytes':len(b),'changed_while_scanned':(before.st_size,before.st_mtime_ns)!=(after.st_size,after.st_mtime_ns)})
 except Exception as e:errors.append({'archive':rel,'reason':type(e).__name__+': '+str(e)[:160]})
 if i%2500==0:print('scanned',i,'of',len(paths),'non-delegated compressed files;',len(matches),'matching representations;',len(unscanned),'bounded omissions',flush=True)
out={'schema':'726-baseline059-nondelegated-compressed-reference-inventory-v1','started_utc':start,'completed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'All enumerated non-delegated *.gz, *.tgz and *.crate files outside .git at scan start. Stored and decoded bytes scanned recursively, bounded, without extraction.','limits':{'max_member_or_decompressed_bytes':cap,'max_nested_compression_depth':limit,'symlinks_followed':False},'file_count_enumerated':len(paths),'file_records':archives,'gzip_members_seen':gzip_count,'nested_tar_streams_seen':inner_tar_count,'matching_representations':matches,'bounded_unscanned':unscanned,'errors':errors,'patterns':list(patterns),'qualification':'Literal matches are references, not proof that an archived script is active. Listed omissions, errors, unsupported formats, magic-only gzip without enumerated suffix and future/concurrently modified paths remain open.'}
args.output.write_text(json.dumps(out,indent=2)+'\n');print('done files',len(archives),'matches',len(matches),'omissions',len(unscanned),'errors',len(errors),flush=True)
