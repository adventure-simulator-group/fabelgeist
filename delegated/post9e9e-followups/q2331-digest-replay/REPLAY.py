#!/usr/bin/env python3
"""Replay bounded digest checks without exposing removed path values or hashes."""
import argparse,hashlib,json,pathlib
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
ap=argparse.ArgumentParser();ap.add_argument('--packet',type=pathlib.Path,required=True);ap.add_argument('--private-key',type=pathlib.Path);ap.add_argument('--receipts',type=pathlib.Path,action='append',default=[]);a=ap.parse_args();p=a.packet;here=pathlib.Path(__file__).resolve().parent
anchor=json.loads((here/'ANCHOR.json').read_text());m=(p/'MANIFEST.json').read_bytes();assert hashlib.sha256(m).hexdigest()==anchor['q2331_packet_manifest_sha256'];manifest=json.loads(m);files={x.name for x in p.iterdir() if x.is_file() and x.name!='MANIFEST.json'};assert files==set(manifest['files']) and all(not x.is_symlink() for x in p.iterdir())
for n,x in manifest['files'].items():
 b=(p/n).read_bytes();assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256']
if a.private_key is None:
 print('PASS q2331 packet manifest. NOT REPEATED: path-value digest comparison requires separately held key; receipt directories are optional inputs.');raise SystemExit(0)
key=a.private_key.read_bytes();assert len(key)==32;idx=json.loads((p/'INDEX.json').read_text())['files'];meta=json.loads((p/'ENCRYPTED-ORIGINALS.json').read_text())['originals'];vals=[]
def walk(old,new):
 if new=='<redacted-local-path>':assert isinstance(old,str);vals.append(old);return
 if isinstance(old,dict) and isinstance(new,dict):
  assert old.keys()==new.keys()
  for k in old:walk(old[k],new[k])
 elif isinstance(old,list) and isinstance(new,list):
  assert len(old)==len(new)
  for x,y in zip(old,new):walk(x,y)
 else:assert old==new
for x in meta:
 c=(p/x['ciphertext']).read_bytes();assert hashlib.sha256(c).hexdigest()==x['ciphertext_sha256'];plain=AESGCM(key).decrypt(c[:12],c[12:],x['logical_path'].encode());assert hashlib.sha256(plain).hexdigest()==x['original_sha256'];walk(json.loads(plain),json.loads((p/idx[x['logical_path']]['object']).read_bytes()))
assert len(vals)==sum(x['redacted_occurrences'] for x in meta);candidates=set(vals)
for v in vals:
 if v.startswith('/'):
  parts=v.split('/')
  for n in range(3,len(parts)):candidates.add('/'.join(parts[:n]))
def scan(dir):
 files=[x for x in dir.iterdir() if x.is_file() and not x.is_symlink()];hits=[]
 for f in files:
  data=f.read_bytes()
  for v in candidates:
   for alg in ('sha256','sha1','md5'):
    raw=hashlib.new(alg,v.encode()).digest()
    for label,needle in (('raw',raw),('lower-hex',raw.hex().encode()),('upper-hex',raw.hex().upper().encode())):
     if needle in data:hits.append({'file':f.name,'algorithm':alg,'encoding':label,'kind':'exact value' if v in vals else 'directory prefix'})
 return {'files':len(files),'matches':hits}
result={'originals_authenticated':len(meta),'removed_occurrences':len(vals),'distinct_exact_values':len(set(vals)),'exact_values_plus_directory_prefixes':len(candidates),'digest_algorithms':['sha256','sha1','md5'],'formats':['raw','lower-hex','upper-hex'],'packet':scan(p),'receipt_directories':[scan(x) for x in a.receipts],'limits':'Exact removed UTF-8 strings and their absolute directory prefixes only; no unknown values, other encodings/transforms, unsupported inputs, or independent custody claim. Ciphertext is scanned as raw bytes, not decrypted by the scanner.'}
print(json.dumps(result,indent=2));assert not result['packet']['matches'] and all(not x['matches'] for x in result['receipt_directories'])
