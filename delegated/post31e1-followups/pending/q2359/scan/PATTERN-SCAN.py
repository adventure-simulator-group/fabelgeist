#!/usr/bin/env python3
"""Bounded path/credential candidate scan, reporting category counts only."""
import argparse,collections,gzip,json,os,re
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('target',type=Path);args=ap.parse_args()
patterns={
 'unix-home':re.compile(rb'(?<![A-Za-z0-9_])/(?:home|Users)/[A-Za-z0-9_.-]+'),
 'temp-path':re.compile(rb'(?<![A-Za-z0-9_])/(?:private/)?tmp/[A-Za-z0-9_.-]+'),
 'windows-home':re.compile(rb'(?i)[A-Za-z]:[\\/]+Users[\\/]+[A-Za-z0-9_.-]+'),
 'email':re.compile(rb'(?<![A-Za-z0-9_.+%-])[A-Za-z0-9_.+%-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![A-Za-z0-9_.-])'),
 'private-key':re.compile(rb'-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----'),
 'credential':re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[A-Za-z0-9]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|Bearer\s+[A-Za-z0-9._~+/-]{24,}',re.I)}
counts=collections.Counter();stats=collections.Counter();exceptions=[];limit=128*1024*1024
root=args.target.resolve()
def visit(data,depth):
 stats['views']+=1
 if len(data)>limit or depth>8:exceptions.append('size-or-depth');return
 if data.startswith(b'\x1f\x8b'):
  try:visit(gzip.decompress(data),depth+1)
  except Exception:exceptions.append('gzip-decode')
  return
 if data.startswith((b'PK\x03\x04',b'BZh',b'\xfd7zXZ\x00')):exceptions.append('unsupported-archive')
 if len(data)>262 and data[257:262]==b'ustar':exceptions.append('unhandled-tar')
 norm=re.sub(rb'(?i)\\u002f',b'/',data.replace(b'\\/',b'/'))
 for k,rx in patterns.items():counts[k]+=len(rx.findall(norm))
for directory,dirs,files in os.walk(root,followlinks=False):
 dirs.sort();files.sort()
 for n in list(dirs):
  if (Path(directory)/n).is_symlink():exceptions.append('symlink-directory');dirs.remove(n)
 for n in files:
  p=Path(directory)/n
  if p.is_symlink():exceptions.append('symlink-file');continue
  stats['files']+=1;visit(p.read_bytes(),0)
result={'schema':'726-13r-bounded-pattern-scan-v1','stats':dict(stats),'candidate_counts':{k:v for k,v in counts.items() if v},'exceptions':exceptions,'qualification':'This is a bounded regex and gzip scan, not a complete privacy or history review. It reports no candidate values or digests. External published archive and Git history are outside scope.'}
print(json.dumps(result,indent=2));raise SystemExit(bool(result['candidate_counts'] or exceptions))
