#!/usr/bin/env python3
"""Read-only recursive privacy candidate scan using a restricted digest source.

Prints counts and sanitized relative locations only, never candidate values,
individual digests, matched bytes, or restricted filesystem paths.
"""
import argparse,collections,gzip,hashlib,io,json,os,pwd,re,tarfile,zlib
from pathlib import Path,PurePosixPath
ap=argparse.ArgumentParser();ap.add_argument('target');ap.add_argument('--private-capture',required=True);ap.add_argument('--max-bytes',type=int,default=128*1024*1024);ap.add_argument('--max-depth',type=int,default=18);args=ap.parse_args()
root=Path(args.target).resolve();private=Path(args.private_capture).resolve();MAX=args.max_bytes;DEPTH=args.max_depth;account=pwd.getpwuid(os.getuid()).pw_name.encode()
assert root.is_dir() and private.is_dir() and MAX>0 and DEPTH>=0 and private.stat().st_mode&0o077==0
capture=json.loads((private/'CAPTURE.json').read_bytes());oldfile=private/'original-gpu-archive.tar.gz';assert oldfile.stat().st_mode&0o077==0;old=oldfile.read_bytes();assert capture['original_gpu_archive_sha256']=='acacafaaa2607b0c5e4acbf829cd1b12f96edfd8a58d890142bb6eada980b106' and hashlib.sha256(old).hexdigest()==capture['original_gpu_archive_sha256']
with tarfile.open(fileobj=io.BytesIO(old),mode='r:gz') as tar:
 mapping=json.load(tar.extractfile('RECONSTRUCTION-MAP.json'));scans=[json.load(tar.extractfile(n)) for n in ('ORIGINAL-PRIVACY-SCAN.json','PUBLIC-PRIVACY-SCAN.json')]
values=[];counts=collections.Counter()
def collect_map(x):
 if isinstance(x,dict):
  for k,v in x.items():
   if k in ('original_value_sha256','public_value_sha256'):
    add=v if isinstance(v,list) else [v];counts[k+'_fields']+=1;counts[k+'_values']+=len(add);values.extend(add)
   else:collect_map(v)
 elif isinstance(x,list):
  for v in x:collect_map(v)
def collect_scans(x):
 if isinstance(x,dict):
  for k,v in x.items():
   if k=='matched_prefix_sha256':counts['prefix_fields']+=1;values.append(v)
   else:collect_scans(v)
 elif isinstance(x,list):
  for v in x:collect_scans(v)
collect_map(mapping)
for x in scans:collect_scans(x)
assert counts['original_value_sha256_fields']==45 and counts['original_value_sha256_values']==47 and counts['public_value_sha256_fields']==2 and counts['public_value_sha256_values']==2 and counts['prefix_fields']==60
assert all(isinstance(v,str) and re.fullmatch('[a-fA-F0-9]{64}',v) for v in values)
corpus={v.lower() for v in values};assert len(corpus)==40
hexrx=re.compile(b'|'.join(re.escape(v.encode()) for v in sorted(corpus)),re.I)
rawrx=re.compile(b'|'.join(re.escape(bytes.fromhex(v)) for v in sorted(corpus)),re.DOTALL)
patterns={
 'unix-home':re.compile(rb'(?<![A-Za-z0-9_])/(?:home|Users)/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.~+@-]+)*'),
 'unix-temp':re.compile(rb'(?<![A-Za-z0-9_])/(?:private/)?tmp/[A-Za-z0-9_.~+@-]+(?:/[A-Za-z0-9_.~+@-]+)*'),
 'root-private':re.compile(rb'(?<![A-Za-z0-9_])/(?:root/(?:\.codex|\.local|\.cache|tmp)|var/folders|mnt|workspace|workspaces)/[A-Za-z0-9_.~+@-]+'),
 'escaped-home':re.compile(rb'(?<![A-Za-z0-9_])\\/(?:home|Users)\\/[A-Za-z0-9_.-]+'),
 'escaped-temp':re.compile(rb'(?<![A-Za-z0-9_])\\/(?:private\\/)?tmp\\/[A-Za-z0-9_.-]+'),
 'windows-home':re.compile(rb'(?i)(?:[a-z]:\\{1,2}users\\{1,2}|[a-z]:/users/)[A-Za-z0-9_.@-]+'),
 'email':re.compile(rb'(?<![A-Za-z0-9_.+%-])[A-Za-z0-9_.+%-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![A-Za-z0-9_.-])'),
 'private-key':re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----'),
 'credential':re.compile(rb'\b(?:ghp_[A-Za-z0-9]{32,}|github_pat_[A-Za-z0-9_]{60,}|AKIA[A-Z0-9]{16})\b'),
}
stats=collections.Counter();digest_hits=collections.Counter();candidates=collections.Counter();exceptions=[];cached={}
def safe_name(name,fallback):
 b=name.encode('utf-8','replace');p=PurePosixPath(name)
 if p.is_absolute() or '..' in p.parts or account.lower() in b.lower() or any(rx.search(b) or rx.search(b'/'+b) for rx in patterns.values()) or hexrx.search(b) or rawrx.search(b):return fallback
 return name
# Cache by whole-view hash; it is kept only in memory and never reported.
def search_bytes(data,chain,privacy):
 key=(hashlib.sha256(data).digest(),privacy)
 if key not in cached:
  result=collections.Counter();result['forbidden-digest-hex']=len(hexrx.findall(data));result['forbidden-digest-raw']=len(rawrx.findall(data))
  if privacy:
   for k,rx in patterns.items():result[k]=len(rx.findall(data))
  cached[key]=result
 else:stats['cached_views']+=1
 for k,n in cached[key].items():
  if n:
   (digest_hits if k.startswith('forbidden-digest') else candidates)[(chain,k)]+=n
def gzip_metadata(data,chain):
 pos=0
 while pos<len(data):
  if data[pos:pos+3]!=b'\x1f\x8b\x08' or len(data)<pos+10:raise ValueError('invalid gzip member')
  flag=data[pos+3];assert not flag&0xe0;stats['gzip_members']+=1;cursor=pos+10
  if flag&4:
   n=int.from_bytes(data[cursor:cursor+2],'little');cursor+=2;extra=data[cursor:cursor+n];assert len(extra)==n;cursor+=n;search_bytes(extra,chain+'!gzip-extra',True)
  for mask,label in ((8,'name'),(16,'comment')):
   if flag&mask:
    end=data.index(b'\0',cursor);field=data[cursor:end];cursor=end+1;search_bytes(field,chain+'!gzip-'+label,True)
  if flag&2:cursor+=2
  z=zlib.decompressobj(31);out=z.decompress(data[pos:],MAX+1);assert len(out)<=MAX and z.eof;used=len(data[pos:])-len(z.unused_data);assert used>cursor-pos;pos+=used

def tree_names(data,chain):
 i=0
 while i<len(data):
  sp=data.index(b' ',i);nul=data.index(b'\0',sp);name=data[sp+1:nul];assert len(data[nul+1:nul+21])==20;i=nul+21;stats['tree_names']+=1;search_bytes(name,chain+'!tree-name',True)
def visit(data,chain,depth,tree=False):
 stats['views']+=1
 if len(data)>MAX:exceptions.append({'location':chain,'reason':'view-size-limit'});return
 if depth>DEPTH:exceptions.append({'location':chain,'reason':'depth-limit'});return
 if data.startswith(b'\x1f\x8b'):
  stats['gzip_views']+=1;search_bytes(data,chain+'!gzip-stored',False)
  try:
   gzip_metadata(data,chain)
   with gzip.GzipFile(fileobj=io.BytesIO(data)) as f:plain=f.read(MAX+1)
   if len(plain)>MAX:exceptions.append({'location':chain,'reason':'gzip-expanded-size-limit'});return
   visit(plain,chain+'!gzip',depth+1,tree=tree)
  except Exception as e:exceptions.append({'location':chain,'reason':'gzip-error','type':type(e).__name__})
  return
 if len(data)>=512 and data[257:262]==b'ustar':
  stats['tar_views']+=1
  try:
   with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as t:
    for i,m in enumerate(t):
     stats['tar_members']+=1;name=safe_name(m.name,'member-'+str(i));where=chain+'!'+name
     for label,value in [('name',m.name),('linkname',m.linkname),('uname',m.uname),('gname',m.gname),*[(str(k),str(v)) for k,v in m.pax_headers.items()]]:search_bytes(value.encode('utf-8','replace'),chain+'!tar-metadata-'+str(i)+'-'+label,True)
     if m.isfile():
      if m.size>MAX:exceptions.append({'location':where,'reason':'tar-member-size-limit'});continue
      f=t.extractfile(m);visit(f.read(MAX+1),where,depth+1,tree=m.name.endswith('.tree.gz'))
  except Exception as e:exceptions.append({'location':chain,'reason':'tar-error','type':type(e).__name__})
  return
 if data.startswith((b'PK\x03\x04',b'BZh',b'\xfd7zXZ\x00')):exceptions.append({'location':chain,'reason':'unsupported-archive-signature'})
 text=False
 if b'\0' not in data:
  try:data.decode('utf-8');text=True
  except UnicodeDecodeError:pass
 stats['text_views' if text else 'binary_views']+=1;search_bytes(data,chain,text)
 if tree:
  try:tree_names(data,chain)
  except Exception as e:exceptions.append({'location':chain,'reason':'tree-error','type':type(e).__name__})
def public_files(base):
 for directory,dirs,filenames in os.walk(base,followlinks=False):
  dirs.sort();filenames.sort()
  if '.git' in dirs:dirs.remove('.git');stats['git_metadata_dirs_omitted']+=1
  if '.git' in filenames:filenames.remove('.git');stats['git_metadata_files_omitted']+=1
  for name in list(dirs):
   if (Path(directory)/name).is_symlink():dirs.remove(name);exceptions.append({'location':'symlink-directory-omitted','reason':'symlink'})
  for name in filenames:yield Path(directory)/name
for index,f in enumerate(public_files(root)):
 if f.is_symlink():exceptions.append({'location':'symlink-file-omitted','reason':'symlink'});continue
 if f.is_file():
  rel=f.relative_to(root).as_posix();safe=safe_name(rel,'top-file-'+str(index));stats['top_files']+=1
  if f.stat().st_size>MAX:exceptions.append({'location':safe,'reason':'file-size-limit'});continue
  try:visit(f.read_bytes(),safe,0,tree=f.name.endswith('.tree.gz'))
  except Exception as e:exceptions.append({'location':safe,'reason':'read-error','type':type(e).__name__})
result={'schema':'726-private-input-recursive-digest-scan-v1','corpus_counts':{'original_map_fields':45,'original_map_values':47,'public_map_fields':2,'public_map_values':2,'scan_prefix_fields':60,'distinct_sensitive_digests':len(corpus)},'stats':dict(stats),'forbidden_digest_total':sum(digest_hits.values()),'forbidden_digest_locations':[{'location':c,'representation':k,'count':n} for (c,k),n in sorted(digest_hits.items())],'other_candidate_counts':dict(collections.Counter({k:sum(n for (c,kk),n in candidates.items() if kk==k) for c,k in candidates})),'other_candidate_locations':[{'location':c,'kind':k,'count':n} for (c,k),n in sorted(candidates.items())],'exceptions':exceptions,'qualification':'Digest corpus is derived from restricted original GPU evidence, not hardcoded or printed. Text path/credential/email matches are candidates and require contextual classification; binary Git commit metadata remains separately disclosed. Per-view bytes and recursion depth are bounded; unsupported formats, excluded symlinks, unknown roots, encodings or credentials are not cleared. Git administrative metadata and history are omitted; the scan covers the specified working tree only. No product code is executed.'}
print(json.dumps(result,indent=2))
if digest_hits or exceptions:raise SystemExit(1)
