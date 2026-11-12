#!/usr/bin/env python3
"""Deterministically adapt 15 captured evidence-only verifiers in a fresh directory."""
from pathlib import Path
import argparse, ast, hashlib, json
sha=lambda b:hashlib.sha256(b).hexdigest()
ap=argparse.ArgumentParser();ap.add_argument('--originals',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--plan',type=Path,required=True);args=ap.parse_args()
plan=json.loads(args.plan.read_bytes());assert len(plan['fifteen_affected_checks'])==15
args.output.mkdir(parents=True,exist_ok=True)
header=("import sys as _cw_sys\n_cw_sys.dont_write_bytecode = True\nfrom pathlib import Path as _CW_Path\nfrom qualified_crosswalk import configure as _cw_configure\n_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')\n")
end="\n_CW.finish()\n"
known_old="known={sha((R/p).read_bytes()):len((R/p).read_bytes()) for p in paths}"
known_new="known=_CW.known(paths)"
rep15=[(" if raw is None:raw=p.read_bytes()\n assert sha(raw)==ref['sha256'] and len(raw)==ref['bytes']", " if raw is None:raw=_CW.read(ref['path'])\n assert _CW.matches(ref['path'],raw,ref['sha256'],ref['bytes'])",1,'physics-public-link'), (" raw=(R/a[key+'_path']).read_bytes();assert sha(raw)==a[key+'_sha256']", " raw=_CW.read(a[key+'_path']);assert _CW.matches(a[key+'_path'],raw,a[key+'_sha256'])",1,'sdk-public-link'), (" if raw is None:raw=(R/path).read_bytes()\n assert sha(raw)==sources[key]['sha256'] and len(raw)==sources[key]['bytes']", " if raw is None:raw=_CW.read(path)\n assert _CW.matches(path,raw,sources[key]['sha256'],sources[key]['bytes'])",1,'p16-public-link')]
rep16=[("raw=p.read_bytes();assert sha(raw)==x['sha256'] and len(raw)==x['bytes']", "raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])",2,'direct-published'),(known_old,known_new,2,'known-digest-map')]
rep20=[("raw=p.read_bytes();assert sha(raw)==x['sha256'] and len(raw)==x['bytes']", "raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])",1,'direct-published')]
rep21=[("b=p.read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes']", "b=_CW.read(x['path']);assert _CW.matches(x['path'],b,x['sha256'],x['bytes'])",1,'direct-published')]
rep22=[("raw=p.read_bytes();assert (sha(raw),len(raw))==(x['sha256'],x['bytes'])", "raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])",1,'direct-published')]
rows=[]
for item in plan['fifteen_affected_checks']:
 n=item['checkpoint'];src=args.originals/item['script'];raw=src.read_bytes();assert sha(raw)==item['script_sha256'] and len(raw)==item['script_bytes'];s=raw.decode();lines=s.splitlines(keepends=True);assert lines[0].startswith('#!') and lines[1].startswith('"""') and lines[1].count('"""')==2
 s=''.join(lines[:2])+header+''.join(lines[2:]);token='R=Path(__file__).resolve().parent';assert s.count(token)==1;s=s.replace(token,'R=_CW.root',1)
 if n==15: edits=rep15
 elif n==16: edits=rep16
 elif n in (17,18,19):edits=[(known_old,known_new,1,'known-digest-map')]
 elif n==20:edits=rep20
 elif n==21:edits=rep21
 else:edits=rep22
 for old,new,count,label in edits:
  assert s.count(old)==count,(n,label,s.count(old),count);s=s.replace(old,new)
 s+=end;ast.parse(s)
 # Invert all targeted substitutions, then remove the prelude and trailer.
 reverse=s[:-len(end)]
 for old,new,count,label in reversed(edits):
  assert reverse.count(new)==count,(n,label);reverse=reverse.replace(new,old)
 assert reverse.count('R=_CW.root')==1;reverse=reverse.replace('R=_CW.root',token,1);assert reverse.count(header)==1;reverse=reverse.replace(header,'',1);assert reverse.encode()==raw
 outname=item['script'].replace('.py','_059_public.py');out=(args.output/outname);assert not out.exists();out.write_text(s)
 rows.append({'checkpoint':n,'original':item['script'],'original_sha256':sha(raw),'original_bytes':len(raw),'successor':outname,'successor_sha256':sha(s.encode()),'successor_bytes':len(s.encode()),'exact_edit_counts':{label:count for _,_,count,label in edits},'preservation_proof':'All edits inverted in memory to the exact original bytes; no other original assertion or statement was changed.'})
print(json.dumps({'schema':'726-059-successor-source-map-v1','rows':rows},indent=2))
