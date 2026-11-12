#!/usr/bin/env python3
"""Restore the exact q2812 anchors from the authenticated prior replay seal."""
import argparse,hashlib,io,json,os,pathlib,subprocess,sys,tarfile
P=pathlib.Path
H=lambda b:hashlib.sha256(b).hexdigest()
def need(v,m):
 if not v:raise SystemExit('STOP: '+m)
a=argparse.ArgumentParser(description=__doc__);a.add_argument('--seal',type=P,required=True);a.add_argument('--key',type=P,required=True);a.add_argument('--output',type=P,required=True);a=a.parse_args()
need(not sys.flags.optimize,'disable Python optimization')
s=a.seal.resolve();o=a.output
need(H((s/'MANIFEST.json').read_bytes())=='3cbc29b46b53028beb70992b0e4d39047ff3d2349626b1430722d22024b343e7','unexpected seal manifest')
need(H((s/'VERIFY.py').read_bytes())=='fb3152c3620314e7885caa0720afcaa79e5bec9ae84dba0c3445a964efcd944f','unexpected seal verifier')
need(o.parent.is_dir() and not o.exists() and not o.is_symlink(),'output must be absent with existing private parent')
env={k:v for k,v in os.environ.items() if not k.startswith('PYTHON') and not k.startswith('GIT_')};env.update(PYTHONNOUSERSITE='1',PYTHONDONTWRITEBYTECODE='1')
r=subprocess.run([sys.executable,'-B',str(s/'VERIFY.py'),'--key',str(a.key)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
need(r.returncode==0,'prior seal authentication or verification failed')
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
c=(s/'PRIVATE-EVIDENCE.aesgcm').read_bytes();magic=b'726POSTDAC7v1\0';need(c.startswith(magic),'cipher header')
raw=AESGCM(a.key.read_bytes()).decrypt(c[len(magic):len(magic)+12],c[len(magic)+12:],b'726-postdac7-prepared-stream-replay-v1')
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 members=t.getmembers();names=[m.name for m in members]
 need(len(names)==len(set(names)),'duplicate archive member')
 need(all(m.isfile() and not m.name.startswith('/') and '\\' not in m.name and '..' not in pathlib.PurePosixPath(m.name).parts for m in members),'unsafe archive member')
 fs={m.name:t.extractfile(m).read() for m in members}
manifest=json.loads(fs['PRIVATE-MANIFEST.json']);rows={x['path']:x for x in manifest['files']}
need(len(rows)==len(manifest['files']) and set(fs)-{'PRIVATE-MANIFEST.json'}==set(rows),'private manifest set')
for name,x in rows.items():need(len(fs[name])==x['bytes'] and H(fs[name])==x['sha256'],'private manifest content')
prefix='q2812-anchor/';anchor={n[len(prefix):]:fs[n] for n in fs if n.startswith(prefix)}
need(H(anchor['ANCHOR-MANIFEST.json'])=='e3262991171c1b8279952062621c01b95803d573fbb4a20cd05cb8b2cc794a2d','anchor manifest identity')
am=json.loads(anchor['ANCHOR-MANIFEST.json']);wanted={x['file'] for x in am['anchors']}|set(am['support'])|{'ANCHOR-MANIFEST.json'}
need(set(anchor)==wanted and all('/' not in n and '\\' not in n and n not in ('.','..') for n in wanted),'anchor file set')
for n in wanted:need(int(rows[prefix+n]['original_mode'],8) in range(0o1000),'anchor file mode')
os.mkdir(o,0o700)
for n in sorted(wanted):
 mode=int(rows[prefix+n]['original_mode'],8);fd=os.open(o/n,os.O_WRONLY|os.O_CREAT|os.O_EXCL|getattr(os,'O_NOFOLLOW',0),0o600)
 with os.fdopen(fd,'wb') as f:f.write(anchor[n]);os.fchmod(f.fileno(),mode)
need(H((o/'verify-anchor-collection.py').read_bytes())=='68dc9bb7156c588afa6086965048478e54fc790dbfc9e8b6465e4aefccfaa76a','anchor verifier identity')
r=subprocess.run([sys.executable,'-B',str(o/'verify-anchor-collection.py')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
need(r.returncode==0,'restored anchor verification failed; retain output for inspection')
print('Authenticated prior seal; restored and verified the exact private q2812 anchor files and modes.')
print('This verifies evidence recovery only; the key and custody remain separately controlled.')
