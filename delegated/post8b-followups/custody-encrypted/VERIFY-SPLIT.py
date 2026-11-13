#!/usr/bin/env python3
"""Stream-check the split ciphertext transport; optionally restore a fresh packet.

This verifies byte identities and within-group nonce uniqueness without a key.
It does not authenticate ciphertexts or establish independent custody.
"""
import argparse,hashlib,json,os,pathlib,stat,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
MAX_PART=64*1024*1024
ORIGINAL_MANIFEST_SHA256='f2025256d1b1024b9cc27db445745e68bdc42e21995f76ac9af11cd7c7a3b04c'
TARGETS={'post8b-15.aesgcm','post8b-25.aesgcm'}
TRANSPORT_CONTROLS={'ORIGINAL-MANIFEST.json','ORIGINAL-README.md','ORIGINAL-VERIFY.py','ORIGINAL-INVENTORY.json','SPLIT-README.md','VERIFY-SPLIT.py','RESTORE-SPLIT.py'}

def require(condition,why):
 if not condition:raise ValueError(why)
def safe_name(s):
 return isinstance(s,str) and s not in ('','.','..') and all(c.isascii() and (c.isalnum() or c in '._-') for c in s)
def ancestor_check(path):
 path=pathlib.Path(os.path.abspath(path));cur=pathlib.Path(path.anchor)
 for part in path.parts[1:]:
  cur=cur/part;st=os.lstat(cur);require(stat.S_ISDIR(st.st_mode) and not stat.S_ISLNK(st.st_mode),'directory path contains a symlink or non-directory')
 return path

def open_regular(dirfd,name):
 require(safe_name(name),'unsafe member name');fd=os.open(name,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dirfd);st=os.fstat(fd)
 if not stat.S_ISREG(st.st_mode) or st.st_nlink!=1:
  os.close(fd);raise ValueError('member is not a single-link regular file')
 return fd,st

def stable(a,b):
 return (a.st_dev,a.st_ino,a.st_size,a.st_mtime_ns,a.st_ctime_ns,a.st_mode)==(b.st_dev,b.st_ino,b.st_size,b.st_mtime_ns,b.st_ctime_ns,b.st_mode)
def read_small(dirfd,name,limit=4*1024*1024):
 fd,st=open_regular(dirfd,name)
 try:
  require(st.st_size<=limit,'control exceeds size limit');data=b''
  while True:
   x=os.read(fd,1024*1024)
   if not x:break
   data+=x
  require(stable(st,os.fstat(fd)) and len(data)==st.st_size,'control changed while reading');return data,st
 finally:os.close(fd)
def sha(b):return hashlib.sha256(b).hexdigest()

def mode_ok(actual,recorded,strict):
 return actual==recorded if strict else bool(actual & stat.S_IXUSR)==bool(recorded & stat.S_IXUSR)

def read_manifest(rootfd,strict):
 b,st=read_small(rootfd,'SPLIT-MANIFEST.json');require(mode_ok(stat.S_IMODE(st.st_mode),0o644,strict),'split manifest mode mismatch');m=json.loads(b)
 require(m['schema']=='726-post8b-lossless-split-v3' and m['max_piece_bytes']==MAX_PART,'invalid split schema or piece limit')
 require(m['original_manifest_sha256']==ORIGINAL_MANIFEST_SHA256,'source manifest anchor mismatch');require(len(m['objects'])==46,'incorrect object count')
 controls=m['controls'];require(isinstance(controls,list) and len(controls)==len(TRANSPORT_CONTROLS),'incorrect controls');controlmap={}
 for c in controls:
  n=c['name'];require(safe_name(n) and n not in controlmap,'duplicate or unsafe control');controlmap[n]=c
 require(set(controlmap)==TRANSPORT_CONTROLS,'control names mismatch');controlbytes={}
 for n,c in controlmap.items():
  data,st=read_small(rootfd,n);require((len(data),sha(data))==(c['bytes'],c['sha256']) and mode_ok(stat.S_IMODE(st.st_mode),c['transport_mode'],strict),'control identity or mode mismatch');controlbytes[n]=data
 require(sha(controlbytes['ORIGINAL-MANIFEST.json'])==ORIGINAL_MANIFEST_SHA256,'original manifest bytes mismatch')
 orig=json.loads(controlbytes['ORIGINAL-MANIFEST.json']);inv=json.loads(controlbytes['ORIGINAL-INVENTORY.json'])
 require(orig['schema']=='726-post8b-custody-encrypted-v1' and len(orig['objects'])==46 and orig['prior_nonce_count']==772 and orig['new_nonce_count']==46,'original manifest shape mismatch')
 require(sha(controlbytes['ORIGINAL-INVENTORY.json'])==m['original_inventory_sha256'] and inv['root_mode']==m['original_root_mode'],'inventory identity/root mode mismatch')
 inventory={x['name']:x for x in inv['entries']};require(len(inventory)==len(inv['entries'])==49 and all(x['kind']=='file' for x in inv['entries']),'source inventory is not exact regular-file set')
 names={'MANIFEST.json','README.md','VERIFY.py'}|{x['ciphertext'] for x in orig['objects']};require(set(inventory)==names,'source inventory member set mismatch')
 originals=[('ORIGINAL-MANIFEST.json','MANIFEST.json'),('ORIGINAL-README.md','README.md'),('ORIGINAL-VERIFY.py','VERIFY.py')]
 for transport,name in originals:
  c=controlmap[transport];i=inventory[name];require(c['original_name']==name and c['original_mode']==i['mode'] and c['bytes']==i['bytes'],'original control mode or size mismatch')
 chunks=set();expected=set(TRANSPORT_CONTROLS)|{'SPLIT-MANIFEST.json'}
 for i,(o,x) in enumerate(zip(orig['objects'],m['objects']),1):
  label='post8b-'+str(i).zfill(2);name=label+'.aesgcm';require(o['label']==x['label']==label and o['ciphertext']==x['original_name']==name,'original label/order mismatch');require(o['associated_data']=='726-audit-private-evidence:'+label+':v1','associated data mismatch')
  require((x['original_bytes'],x['original_sha256'])==(o['ciphertext_bytes'],o['ciphertext_sha256']),'object binding mismatch');require(x['original_mode']==inventory[name]['mode'] and x['original_bytes']==inventory[name]['bytes'],'original object mode or size mismatch')
  ps=x['pieces'];require(len(ps)==(2 if name in TARGETS else 1),'piece count mismatch')
  for k,part in enumerate(ps,1):
   want=name+'.part-'+str(k).zfill(3) if name in TARGETS else name;n=part['name'];require(n==want and safe_name(n) and n not in chunks and n not in expected,'piece name invalid or duplicate');require(0<part['bytes']<=MAX_PART,'piece exceeds limit or is empty');require(part['transport_mode']==0o644,'piece mode mismatch');chunks.add(n);expected.add(n)
  require(sum(x['bytes'] for x in ps)==o['ciphertext_bytes'],'piece sizes do not sum to original');
  if name in TARGETS:require(ps[0]['bytes']==MAX_PART,'first split piece not 64 MiB')
 require(set(os.listdir(rootfd))==expected,'extra or missing transport member');return m,controlbytes,originals

def stream_objects(rootfd,m,outfd=None,strict=False):
 nonces=set();total=0;piece_count=0
 for x in m['objects']:
  whole=hashlib.sha256();count=0;prefix=b'';out=None
  if outfd is not None:
   out=os.open(x['original_name'],os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,x['original_mode'],dir_fd=outfd);os.fchmod(out,x['original_mode'])
  try:
   for part in x['pieces']:
    fd,st=open_regular(rootfd,part['name']);piece=hashlib.sha256();size=0
    try:
     require(mode_ok(stat.S_IMODE(st.st_mode),part['transport_mode'],strict) and st.st_size==part['bytes'],'piece mode or size mismatch')
     while True:
      b=os.read(fd,1024*1024)
      if not b:break
      if len(prefix)<12:prefix+=(b[:12-len(prefix)])
      piece.update(b);whole.update(b);size+=len(b);count+=len(b)
      if out is not None:
       view=memoryview(b)
       while view:
        written=os.write(out,view);require(written>0,'short output write');view=view[written:]
     require(stable(st,os.fstat(fd)),'piece changed while streaming');require(size==part['bytes'] and piece.hexdigest()==part['sha256'],'piece byte identity mismatch');piece_count+=1
    finally:os.close(fd)
   require(count==x['original_bytes'] and whole.hexdigest()==x['original_sha256'],'reconstructed ciphertext identity mismatch');require(count>=28 and len(prefix)==12 and prefix not in nonces,'invalid or duplicate nonce');nonces.add(prefix);total+=count
   if out is not None:os.fsync(out)
  finally:
   if out is not None:os.close(out)
 require(len(nonces)==46,'nonce count mismatch');return {'ciphertext_objects':len(m['objects']),'pieces':piece_count,'ciphertext_bytes':total,'within_group_unique_nonces':len(nonces)}

def fresh_destination(path,root):
 q=pathlib.Path(os.path.abspath(path));parent=ancestor_check(q.parent)
 canonical_root=pathlib.Path(os.path.realpath(root,strict=True));canonical_parent=pathlib.Path(os.path.realpath(parent,strict=True));canonical_q=canonical_parent/q.name
 require(q!=root and not q.is_relative_to(root) and canonical_q!=canonical_root and not canonical_q.is_relative_to(canonical_root),'destination is the source transport or lies beneath it')
 rootstat=os.stat(root,follow_symlinks=False)
 for ancestor in (parent,*parent.parents):
  st=os.stat(ancestor,follow_symlinks=False)
  require((st.st_dev,st.st_ino)!=(rootstat.st_dev,rootstat.st_ino),'destination parent aliases source transport')
 try:os.lstat(q)
 except FileNotFoundError:return q
 raise ValueError('destination already exists or is a symlink')
def output_control(outfd,name,data,mode):
 fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,mode,dir_fd=outfd)
 try:
  os.fchmod(fd,mode);view=memoryview(data)
  while view:
   n=os.write(fd,view);require(n>0,'short output write');view=view[n:]
  os.fsync(fd)
 finally:os.close(fd)

def main():
 ap=argparse.ArgumentParser(description='No-key streaming identity check; optional exclusive fresh original-packet restoration.');ap.add_argument('--destination');ap.add_argument('--strict-modes',action='store_true');a=ap.parse_args()
 root=ancestor_check(pathlib.Path(os.path.abspath(__file__)).parent);dest=fresh_destination(a.destination,root) if a.destination else None;rootfd=os.open(root,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
 try:
  m,controls,originals=read_manifest(rootfd,a.strict_modes);result=stream_objects(rootfd,m,strict=a.strict_modes)
  if dest is not None:
   rootstat=os.fstat(rootfd);pathstat=os.stat(root,follow_symlinks=False);require((rootstat.st_dev,rootstat.st_ino)==(pathstat.st_dev,pathstat.st_ino),'source path changed during verification');dest=fresh_destination(dest,root)
   os.mkdir(dest,0o700);outfd=os.open(dest,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
   try:
    os.fchmod(outfd,m['original_root_mode']);controlmap={x['name']:x for x in m['controls']}
    for transport,name in originals:output_control(outfd,name,controls[transport],controlmap[transport]['original_mode'])
    second=stream_objects(rootfd,m,outfd,strict=a.strict_modes);require(second==result,'source changed between passes');require(set(os.listdir(outfd))=={x['original_name'] for x in m['objects']}|{name for _,name in originals},'restored packet set mismatch');os.fsync(outfd)
   finally:os.close(outfd)
   result['fresh_packet_restored']=True
  else:result['fresh_packet_restored']=False
  result['original_manifest_sha256']=ORIGINAL_MANIFEST_SHA256;result['transport_mode_check']='strict full recorded local modes' if a.strict_modes else 'Git owner-executable state only';result['prior_nonce_comparison']='not performed';result['qualification']='No-key stream identities and within-group nonce uniqueness only; no ciphertext authentication, decryption, source-custody independence or product validation.';print(json.dumps(result,indent=2))
 finally:os.close(rootfd)
if __name__=='__main__':
 try:main()
 except Exception as exc:
  print('Split transport verification failed:',str(exc),file=sys.stderr);sys.exit(1)
