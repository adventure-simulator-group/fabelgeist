#!/usr/bin/env python3
"""Read-only evidence verification. No extraction or product execution."""
import argparse,collections,gzip,hashlib,io,json,subprocess,tarfile
from pathlib import Path,PurePosixPath
p=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();j=lambda n:json.loads((p/n).read_bytes())
a=argparse.ArgumentParser();a.add_argument('--repo',help='repository containing the two recorded commits');args=a.parse_args()
m=j('MANIFEST.json');actual={x.relative_to(p).as_posix() for x in p.rglob('*') if x.is_file() and x.relative_to(p).as_posix()!='MANIFEST.json'};assert actual=={x['path'] for x in m['files']} and len(m['files'])==m['file_count']
for x in m['files']:
 f=p/x['path'];assert not f.is_symlink();b=f.read_bytes();assert (len(b),sha(b))==(x['bytes'],x['sha256'])
r=j('REPORT.json');inventory=j('CHANGED-BLOBS.json');objects=j('GIT-COMMIT-OBJECTS.json');locations=json.loads(gzip.decompress((p/'CANDIDATE-LOCATIONS.json.gz').read_bytes()));rows=inventory['entries']
old=r['object_scope']['parent'];new=r['object_scope']['commit'];assert inventory['old_revision']==old and inventory['new_revision']==new and len(rows)==616 and len({x['path'] for x in rows})==616
assert collections.Counter(x['status'] for x in rows)=={'A':600,'M':16} and sum(x['stored_bytes'] for x in rows)==266552743
assert locations['count']==len(locations['entries'])==37660 and locations['group_count']==len({x['group'] for x in locations['entries']})==50
assert collections.Counter(x['kind'] for x in locations['entries'])=={'unix_path':37500,'email':160}
assert not r['scan']['exceptions'] and len(objects['occurrences'])==77 and sum(x['occurrences'] for x in objects['distinct_objects'].values())==77
assert len({x['tracked_archive'] for x in objects['occurrences']})==27
assert all('value_sha256' not in x and 'context' not in x for x in locations['entries'])
if args.repo:
 git=lambda *cmd,input=None:subprocess.check_output(['git','--no-replace-objects','-C',args.repo,*cmd],input=input)
 for oid,key in [(old,'parent_raw_commit_sha256'),(new,'head_raw_commit_sha256')]:
  data=git('cat-file','commit',oid);assert hashlib.sha1(b'commit '+str(len(data)).encode()+b'\0'+data).hexdigest()==oid and sha(data)==r['object_scope'][key]
 parents=git('rev-list','--parents','-n','1',new).decode().split();assert parents[0]==new and old in parents[1:]
 parts=git('diff-tree','--raw','-z','--no-commit-id','-r','--no-renames',old,new).split(b'\0');source=[]
 for i in range(0,len(parts)-1,2):
  h=parts[i].decode().split();assert len(h)==5 and h[4] in ('A','M');source.append((parts[i+1].decode('utf8','surrogateescape'),h[4],h[3],h[1]))
 assert source==[(x['path'],x['status'],x['git_blob'],x['mode']) for x in rows]
 cat=subprocess.Popen(['git','--no-replace-objects','-C',args.repo,'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
 try:
  for x in rows:
   cat.stdin.write((x['git_blob']+'\n').encode());cat.stdin.flush();h=cat.stdout.readline().decode().split();n=int(h[2]);assert h[:2]==[x['git_blob'],'blob'] and n==x['stored_bytes'];h1=hashlib.sha1(b'blob '+str(n).encode()+b'\0');h2=hashlib.sha256();remain=n
   while remain:
    b=cat.stdout.read(min(remain,1048576));assert b;remain-=len(b);h1.update(b);h2.update(b)
   assert cat.stdout.read(1)==b'\n' and h1.hexdigest()==x['git_blob'] and h2.hexdigest()==x['stored_sha256']
 finally:cat.stdin.close();cat.stdout.close();cat.wait()
 last=None;memo={};verified={}
 for x in sorted(objects['occurrences'],key=lambda v:(v['tracked_archive'],v['member_chain'])):
  top=x['tracked_archive']
  if top!=last:memo={top:git('show',new+':'+top)};last=top
  prefix=top;last_name=None
  for step in x['member_chain'].split('!'):
   nextprefix=prefix+'!'+step
   if nextprefix not in memo:
    source=memo[prefix]
    if step.startswith('gzip-member['):
     with gzip.GzipFile(fileobj=io.BytesIO(source)) as f:data=f.read(128*1024*1024+1)
     assert len(data)<=128*1024*1024
    elif step.startswith('tar['):
     n=int(step[4:step.index(']')]);t=tarfile.open(fileobj=io.BytesIO(source),mode='r:');entry=t.getmembers()[n];assert entry.isfile() and entry.size<=128*1024*1024;data=t.extractfile(entry).read()
    else:raise AssertionError('unsupported occurrence locator')
    memo[nextprefix]=data
   if step.startswith('tar['):last_name=step[step.index(']')+2:]
   prefix=nextprefix
  b=memo[prefix];oid=x['commit_oid'];assert last_name.endswith(oid+'.commit') or last_name.endswith(oid+'.commit.gz');assert hashlib.sha1(b'commit '+str(len(b)).encode()+b'\0'+b).hexdigest()==oid and sha(b)==x['uncompressed_sha256'];verified.setdefault(oid,sha(b));assert verified[oid]==sha(b)
 assert set(verified)==set(objects['distinct_objects'])
 previous=git('show',old+':delegated/hair-grow-input-profile-evidence.tar.gz');assert sha(previous)==objects['prior_parent_checkpoint19_archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(previous),mode='r:gz') as t:
  for oid in objects['prior_object_equality_verified']:
   ms=[m for m in t.getmembers() if m.name.endswith('/'+oid+'.commit.gz') or m.name=='git-objects/'+oid+'.commit.gz'];assert len(ms)==1;data=gzip.decompress(t.extractfile(ms[0]).read());assert sha(data)==verified[oid]
 u=r['reconstruction_and_upstream'];orig=git('show',new+':'+u['original_registry_archive']);assert git('show',old+':'+u['original_registry_archive'])==orig and sha(orig)==u['original_package_sha256'];reconstructed=git('show',new+':'+u['candidate_archive']);assert sha(reconstructed)==u['candidate_archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(orig),mode='r:gz') as a,tarfile.open(fileobj=io.BytesIO(reconstructed),mode='r:gz') as b:
  for path,digest in u['compared_member_sha256'].items():
   name='serde-saphyr-1.2.0/'+path;x=a.extractfile(name).read();y=b.extractfile(name).read();assert x==y and sha(x)==digest
 print('PASS: packet, 616 changed Git blobs, 77 Git commit occurrences, 3 parent archive commit comparisons and 4 upstream source members. Context classification and remote publication remain qualified.')
else:print('PASS: packet manifest, 616 recorded changed blobs, 37,660 candidate locations and 77 author-metadata occurrence records. Git identities require optional --repo verification; classification and remote publication remain qualified.')
