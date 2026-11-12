#!/usr/bin/env python3
"""Read-only bounded candidate scan of public evidence; never outputs values."""
import collections,gzip,hashlib,io,json,re,tarfile,zlib,sys
from pathlib import Path
src=Path(sys.argv[1]).resolve()
MAX_BYTES=128*1024*1024; MAX_DEPTH=18
h=lambda b:hashlib.sha256(b).hexdigest()
# Candidate detection only: paths in regex literals, documentation examples,
# and test strings must be interpreted in context. Values are never recorded.
patterns={
 'unix_home':rb'(?<![A-Za-z0-9_])/(?:home|Users)/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.~+@-]+)*',
 'tmp':rb'(?<![A-Za-z0-9_])/(?:private/)?tmp/[A-Za-z0-9_.~+@-]+(?:/[A-Za-z0-9_.~+@-]+)*',
 'unix_local':rb'(?<![A-Za-z0-9_])/(?:root/(?:\.codex|\.local|\.cache|tmp)|var/folders|mnt|workspace|workspaces)/[A-Za-z0-9_.~+@-]+(?:/[A-Za-z0-9_.~+@-]+)*',
 'escaped_unix_home':rb'(?<![A-Za-z0-9_])\\/(?:home|Users)\\/[A-Za-z0-9_.-]+',
 'escaped_unix_temp':rb'(?<![A-Za-z0-9_])\\/(?:private\\/)?tmp\\/[A-Za-z0-9_.-]+',
 'windows_home':rb'(?i)(?:[a-z]:\\{1,2}users\\{1,2}|[a-z]:/users/)[A-Za-z0-9_.@-]+',
 'email':rb'(?<![A-Za-z0-9_.+%-])[A-Za-z0-9_.+%-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![A-Za-z0-9_.-])',
 'private_key':rb'-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----',
 'credential':rb'\b(?:ghp_[A-Za-z0-9]{32,}|github_pat_[A-Za-z0-9_]{60,}|AKIA[A-Z0-9]{16})\b',
}
pat={k:re.compile(v) for k,v in patterns.items()}; hits=[]; exceptions=[]; stats=collections.Counter(); seen=set()
def inspect_gzip(data,chain):
 pos=0
 while pos<len(data):
  assert data[pos:pos+3]==b'\x1f\x8b\x08' and len(data)>=pos+10
  flags=data[pos+3];assert not (flags&0xe0);stats['gzip_members']+=1;stats[f'gzip_flag_{flags}']+=1;cursor=pos+10
  if flags&4:
   n=int.from_bytes(data[cursor:cursor+2],'little');cursor+=2
   extra=data[cursor:cursor+n];assert len(extra)==n;cursor+=n;stats['gzip_extra_fields']+=1
   for kind,r in pat.items():
    for m in r.finditer(extra):hits.append({'chain':chain+'!gzip-extra','kind':kind,'candidate_sha256':h(m.group()),'view_sha256':h(extra)})
  for mask,label in [(8,'name'),(16,'comment')]:
   if flags&mask:
    end=data.index(b'\0',cursor);value=data[cursor:end];cursor=end+1;stats['gzip_'+label+'_fields']+=1
    for kind,r in pat.items():
     for m in r.finditer(value):hits.append({'chain':chain+'!gzip-'+label,'kind':kind,'candidate_sha256':h(m.group()),'view_sha256':h(value)})
  if flags&2:cursor+=2
  z=zlib.decompressobj(31);expanded=z.decompress(data[pos:],MAX_BYTES+1);assert len(expanded)<=MAX_BYTES and z.eof
  used=len(data[pos:])-len(z.unused_data);assert used>cursor-pos;pos+=used
def inspect_tree_names(data,chain):
 pos=0
 while pos<len(data):
  sp=data.index(b' ',pos);nul=data.index(b'\0',sp);name=data[sp+1:nul];assert len(data[nul+1:nul+21])==20;pos=nul+21;stats['git_tree_entry_names']+=1
  for kind,r in pat.items():
   for m in r.finditer(name):hits.append({'chain':chain+'!tree-name','kind':kind,'candidate_sha256':h(m.group()),'view_sha256':h(name)})
def visit(data,chain,depth,tree=False):
 stats['views']+=1
 if len(data)>MAX_BYTES: exceptions.append({'chain':chain,'reason':'size limit','bytes':len(data)}); return
 if depth>MAX_DEPTH: exceptions.append({'chain':chain,'reason':'depth limit','depth':depth}); return
 # Raw bytes are scanned only when textual UTF-8 without binary NULs; gzip and
 # tar are inspected in decoded form. Git trees are binary and are not prose.
 if b'\0' not in data:
  try: data.decode('utf-8'); text=True
  except UnicodeDecodeError: text=False
  if text:
   stats['text_views']+=1
   for k,r in pat.items():
    for m in r.finditer(data):
     before=data[max(0,m.start()-90):m.start()]; after=data[m.end():m.end()+90]
     # Only hashed candidate values, never output private values or contexts.
     hits.append({'chain':chain,'kind':k,'line':data[:m.start()].count(b'\n')+1,'candidate_sha256':h(m.group()),'view_sha256':h(data),'context_sha256':h(before+m.group()+after)})
  else: stats['non_utf8_views']+=1
 else: stats['binary_views']+=1
 if tree and chain.endswith('!gzip'):
  try: inspect_tree_names(data,chain)
  except Exception as e: exceptions.append({'chain':chain,'reason':'Git tree parse error','type':type(e).__name__})
 if data.startswith(b'\x1f\x8b'):
  stats['gzip_views']+=1
  try:
   inspect_gzip(data,chain)
   with gzip.GzipFile(fileobj=io.BytesIO(data)) as f: decoded=f.read(MAX_BYTES+1)
   if len(decoded)>MAX_BYTES: exceptions.append({'chain':chain,'reason':'gzip expanded size limit'}); return
   visit(decoded,chain+'!gzip',depth+1,tree=tree)
  except Exception as e: exceptions.append({'chain':chain,'reason':'gzip error','type':type(e).__name__})
  return
 # tar probe is bounded; no extraction and no product execution.
 if len(data)>=512 and (data[257:262] in (b'ustar',)):
  stats['tar_views']+=1
  try:
   with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as t:
    for idx,m in enumerate(t):
     stats['tar_entries']+=1
     for label,val in [('name',m.name),('linkname',m.linkname),('uname',m.uname),('gname',m.gname),*[(str(k),str(v)) for k,v in m.pax_headers.items()]]:
      z=val.encode('utf-8',errors='replace')
      for k,r in pat.items():
       for match in r.finditer(z): hits.append({'chain':chain+f'!tar-entry-{idx}-{label}','kind':k,'candidate_sha256':h(match.group()),'view_sha256':h(z)})
     if m.isfile():
      if m.size>MAX_BYTES: exceptions.append({'chain':chain+f'!tar-entry-{idx}','reason':'tar member size limit','bytes':m.size});continue
      f=t.extractfile(m)
      if f: visit(f.read(MAX_BYTES+1),chain+f'!tar-entry-{idx}',depth+1,tree=m.name.endswith('.tree.gz'))
  except Exception as e: exceptions.append({'chain':chain,'reason':'tar error','type':type(e).__name__})
for p in sorted(src.rglob('*')):
 if p.is_file(): stats['top_files']+=1; visit(p.read_bytes(),p.relative_to(src).as_posix(),0,tree=p.name.endswith('.tree.gz'))
summary={'schema':'726-checkpoint42-bounded-privacy-review-v1','stats':dict(stats),'candidates_by_kind':dict(collections.Counter(x['kind'] for x in hits)),'exceptions':len(exceptions),'qualification':'Candidate values are omitted. Email candidates need separate attribution review; this scan alone does not prove privacy clearance. Selected roots and credentials, UTF-8, gzip and ustar only; 128 MiB per view and depth 18. Git tree names and gzip/tar metadata included; unknown encodings, credential types, roots and archive formats excluded.'}
print(json.dumps(summary,indent=2))
if exceptions or any(x['kind']!='email' for x in hits):raise SystemExit(1)
