#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,stat
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--ciphertexts',type=Path);a.add_argument('--receipt',type=Path);a.add_argument('--control',type=Path);a.add_argument('--control-replay',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();read=lambda q:json.loads(q.read_bytes())
def file(q):
 assert q.is_file() and not q.is_symlink();return q.read_bytes()
def safe(name):
 q=PurePosixPath(name);assert not q.is_absolute() and len(q.parts)==1 and q.name not in ('.','..');return q.name
m=read(p/'MANIFEST.json');assert m['schema']=='726-post76fe-binding-manifest-v1';rows=m['files'];assert set(rows)=={'BINDING.json','VERIFY-BINDING.py','PUBLIC-SCAN.json'};assert {q.name for q in p.iterdir()}==set(rows)|{'MANIFEST.json'}
for n,r in rows.items():
 b=file(p/safe(n));assert (len(b),h(b))==(r['bytes'],r['sha256'])
x=read(p/'BINDING.json');ids=x['identities'];assert x['schema']=='726-post76fe-additive-binding-v1';checks=['binding files']
if args.ciphertexts:
 root=args.ciphertexts;raw=file(root/'MANIFEST.json');assert h(raw)==ids['ciphertext_manifest_sha256'];cm=json.loads(raw);assert cm['schema']=='726-encrypted-private-evidence-v1' and len(cm['objects'])==17;names=set();nonces=set()
 for row in cm['objects']:
  n=safe(row['ciphertext']);assert n==row['label']+'.aesgcm' and n not in names;b=file(root/n);assert (len(b),h(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']);assert len(b[:12])==12 and b[:12] not in nonces;names.add(n);nonces.add(b[:12])
 assert {q.name for q in root.iterdir()}==names|{'MANIFEST.json','README.md','VERIFY.py'};assert h(file(root/'README.md'))==m['ciphertext_readme_sha256'] and h(file(root/'VERIFY.py'))==m['ciphertext_verifier_sha256'];checks.append('ciphertext members and mutually distinct new nonces')
if args.receipt:
 root=args.receipt;raw=file(root/'MANIFEST.json');assert h(raw)==ids['receipt_manifest_sha256'];rm=json.loads(raw);assert rm['ciphertext_manifest_sha256']==ids['ciphertext_manifest_sha256'];assert {q.name for q in root.iterdir()}==set(rm['files'])|{'MANIFEST.json'}
 for n,r in rm['files'].items():
  b=file(root/safe(n));assert (len(b),h(b))==(r['bytes'],r['sha256'])
 checks.append('public receipt members')
if args.control:
 root=args.control;raw=file(root/'CONTROL-MANIFEST.json');assert h(raw)==ids['separately_held_restricted_control_manifest_sha256'];cm=json.loads(raw);assert cm['schema']=='726-post76fe-restricted-control-v1';assert oct(stat.S_IMODE(root.stat().st_mode))==cm['root_mode'];names=set()
 for row in cm['files']:
  n=safe(row['path']);assert n not in names;f=root/n;b=file(f);assert (len(b),h(b),oct(stat.S_IMODE(f.stat().st_mode)))==(row['bytes'],row['sha256'],row['mode']);names.add(n)
 assert {q.name for q in root.iterdir()}==names|{'CONTROL-MANIFEST.json'};checks.append('restricted control members and modes')
if args.control_replay:
 root=args.control_replay;raw=file(root/'MANIFEST-PRIVATE.json');assert h(raw)==ids['separately_held_control_replay_manifest_sha256'];rm=json.loads(raw);assert rm['control_manifest_sha256']==ids['separately_held_restricted_control_manifest_sha256'];names=set()
 for row in rm['checks']:
  n=safe(row['name']);assert row['exit']==0
  for ext in ('stdout','stderr'):
   fn=n+'.'+ext;b=file(root/fn);assert h(b)==row[ext+'_sha256'];names.add(fn)
   if ext=='stderr':assert len(b)==row['stderr_bytes']==0
 assert {q.name for q in root.iterdir()}==names|{'MANIFEST-PRIVATE.json'};checks.append('restricted control replay logs')
print(json.dumps({'checks':checks,'qualification':'Identity replay only. Decryption, the 550-object prior nonce inventory, Git object availability, provenance, independent custody and product behavior are separate.'}))
