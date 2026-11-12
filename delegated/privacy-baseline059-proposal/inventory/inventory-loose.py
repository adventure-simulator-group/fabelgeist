#!/usr/bin/env python3
"""Inventory lexical evidence references in an existing, read-only publication tree."""
from pathlib import Path
import argparse,datetime,hashlib,json,re
ap=argparse.ArgumentParser();ap.add_argument('--root',type=Path,required=True);ap.add_argument('--hits',type=Path,required=True);ap.add_argument('--files',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
root=args.root.resolve();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();start=now()
patterns={'baseline_marker':b'baseline-059','queue_path':b'baseline-059/queue.json','ledger_path':b'baseline-059/ledger.json','historical_queue_path':b'CLOSURE-QUEUE-100.json','historical_ledger_path':b'continuation-segovb/ledger.json','original_queue_sha256':b'2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08','original_ledger_sha256':b'3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac'}
ptr=re.compile(rb'baseline-059/(queue|ledger)\.json#(/[^\s"\x27,;)]*)')
paths=sorted(set(x.decode()[2:] if x.startswith(b'./') else x.decode() for x in args.hits.read_bytes().split(b'\0') if x));allfiles=[x for x in args.files.read_bytes().split(b'\0') if x];rows=[];changed=[]
for name in paths:
 p=root/name
 if p.is_symlink() or not p.is_file(): changed.append({'path':name,'condition':'missing or symlink by read time'});continue
 before=p.stat();b=p.read_bytes();after=p.stat()
 if (before.st_size,before.st_mtime_ns)!=(after.st_size,after.st_mtime_ns):changed.append({'path':name,'condition':'changed while read'})
 matches={}; pointers={}
 for no,line in enumerate(b.splitlines(),1):
  for k,v in patterns.items():
   n=line.count(v)
   if n:
    e=matches.setdefault(k,{'occurrences':0,'lines':[]});e['occurrences']+=n;e['lines'].append(no)
  for m in ptr.finditer(line):
   key=(m.group(1).decode(),m.group(2).decode('utf-8','replace'));pointers.setdefault(key,[]).append(no)
 if not matches:changed.append({'path':name,'condition':'no matching bytes at reread'});continue
 refs=[{'document':k[0],'json_pointer':k[1],'lines':v} for k,v in sorted(pointers.items())]
 categories=[]
 if name.endswith('.py'):categories.append('python evidence verifier or helper; literal reference does not itself prove execution or byte comparison')
 if name.endswith('DIRECT-LINKS.json'):categories.append('direct-link manifest candidate; consult exact JSON-pointer inventory')
 if name.endswith('.json') or name.endswith('.jsonl'):categories.append('evidence record, index, manifest or assessment; hashes may identify original historical members rather than current top-level bytes')
 if name.endswith('.md'):categories.append('documentation/reference')
 if ('queue_path' in matches or 'ledger_path' in matches) and ('original_queue_sha256' in matches or 'original_ledger_sha256' in matches):categories.append('path and original digest co-occur; relation requires per-record examination')
 rows.append({'path':name,'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'mtime_ns':after.st_mtime_ns,'matches':matches,'json_pointer_references':refs,'categories':categories})
result={'schema':'726-baseline059-loose-reference-inventory-v1','started_utc':start,'completed_utc':now(),'root_description':'Publication working tree, all rg-enumerated loose regular files, hidden and ignored included, excluding .git and the two source documents. Paths are publication-relative.','enumerated_regular_files':len(allfiles),'matching_files_listed_by_rg':len(paths),'matching_files_successfully_reread':len(rows),'changed_or_unread':changed,'patterns':list(patterns),'files':rows,'qualification':'This is an exhaustive fixed-string search of stored loose-file bytes at scan time for the listed patterns, not a dynamic call graph or a scan through compressed payloads; identify specific behavior with source review. Concurrent changes after enumeration, alternate spellings, symlinks, and non-listed tokens are not excluded.'}
args.output.write_text(json.dumps(result,indent=2)+'\n');print('inventoried',len(rows),'matching files from',len(allfiles),'enumerated; changed',len(changed),'end',result['completed_utc'])
