#!/usr/bin/env python3
"""Evidence-only v1/v2 application and navigation verification."""
import argparse,hashlib,json,os,stat,subprocess,sys,tarfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=argparse.ArgumentParser();g=p.add_mutually_exclusive_group();g.add_argument('--stage',type=Path);g.add_argument('--published',type=Path);p.add_argument('--strict-mode',action='store_true');a=p.parse_args();here=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((here/'MANIFEST.json').read_bytes());assert m['schema']=='726-app-v2-review-v1';assert {x.name for x in here.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 b=(here/n).read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
root=a.stage or a.published
if not root:print('Verified this review packet only; application and navigation were not checked.');sys.exit(0)
def path(n):
 if a.stage:return root/n
 if n.startswith('after-v2/'):return root/n.removeprefix('after-v2/')
 if n.startswith('after/'):return root/n.removeprefix('after/')
 if n.startswith('public/'):return root/'delegated/postd7a5-followups'/n.removeprefix('public/')
 return root/n
def read(n):
 if a.published and n=='after/CONTINUATION.md':
  with tarfile.open(path('public/application/APPLICATION-EVIDENCE.tar.gz'),'r:gz') as t:return t.extractfile('after/CONTINUATION.md').read()
 return path(n).read_bytes()
s=json.loads((here/'SNAPSHOT.json').read_bytes());assert s['schema']=='726-app-v2-review-snapshot-v1'
for row in s['files']:
 n=row['path'];b=read(n);assert (len(b),h(b))==(row['bytes'],row['sha256'])
 if not (a.published and n=='after/CONTINUATION.md'):
  f=path(n);assert f.is_file() and not f.is_symlink();mode=stat.S_IMODE(f.stat().st_mode);expected=int(row['mode'],8);assert mode==expected if a.strict_mode else mode&0o111==expected&0o111
pub=path('public/INDEX.md').parent;old=pub/'application';new=pub/'application-v2';prior=pub/'application-review';assert h((prior/'MANIFEST.json').read_bytes())=='f8364cbaee0f4d5febd2b0e67d3c0dc87f1048a606145c7c28799ed140abc58c'
manifest=json.loads((prior/'MANIFEST.json').read_bytes())
for name,row in manifest['files'].items():
 b=(prior/name).read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
snap=json.loads((prior/'SNAPSHOT.json').read_bytes());by={x['path']:x for x in snap['files']}
for name in ['public/application/APPLICATION-EVIDENCE.tar.gz','public/application/RECORD.json','public/application/TRANSFORM.py','public/application/VERIFY.py','public/PENDING-DELEGATES.json']:
 b=read(name);assert (len(b),h(b))==(by[name]['bytes'],by[name]['sha256'])
r1=(old/'RECORD.json').read_bytes();r2=json.loads((new/'RECORD.json').read_bytes());assert r2['predecessor_record_sha256']==h(r1)
with tarfile.open(old/'APPLICATION-EVIDENCE.tar.gz','r:gz') as x,tarfile.open(new/'APPLICATION-EVIDENCE.tar.gz','r:gz') as y:
 n={z.name for z in x};assert n=={z.name for z in y} and len(n)==11;changed=[z for z in sorted(n) if x.extractfile(z).read()!=y.extractfile(z).read()];assert changed==['after/CONTINUATION.md'];before=x.extractfile(changed[0]).read();after=y.extractfile(changed[0]).read()
insert=b'`delegated/postd7a5-followups/INDEX.md` locates the independent\nreviews, the successor wording correction and later pending packets.\n\n';assert after.count(insert)==1 and after.replace(insert,b'',1)==before
index=(pub/'INDEX.md').read_text();assert all(z in index for z in ['20-obligation snapshot','POSTFREEZE-PENDING.json','adds eight later','SUCCESSOR-CHANGE-v2.json','--published','first-stage inputs'])
frozen=json.loads((pub/'PENDING-DELEGATES.json').read_bytes());later=json.loads((pub/'POSTFREEZE-PENDING.json').read_bytes());oldids=[x['queue_index'] for x in frozen['packets']];newids=[i for x in later['packets'] for i in x['queue_indices']];assert len(oldids)==len(set(oldids))==20 and len(newids)==len(set(newids))==8 and not set(oldids)&set(newids)
for row in later['packets']:
 assert row['status']=='pending central reconciliation'
 for key in ['public_transport','receipt']:assert (pub/row[key]).is_file()
env=dict(os.environ);env['PYTHONDONTWRITEBYTECODE']='1';env['PYTHONNOUSERSITE']='1';env.pop('PYTHONOPTIMIZE',None)
for group in ['application','application-v2','postfreeze-packets']:
 z=subprocess.run([sys.executable,'-B',str(pub/group/'VERIFY.py')],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env);assert z.returncode==0,z.stderr.decode(errors='replace');print(z.stdout.decode().strip())
print('Verified v1 preservation, sole v2 landing change, frozen 20 and later eight pending identities. Git and private provenance remain separate.')
