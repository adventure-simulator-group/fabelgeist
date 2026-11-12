#!/usr/bin/env python3
"""Evidence-only integrity and restricted-recovery verification."""
import argparse,hashlib,io,json,pathlib,tarfile
sha=lambda b:hashlib.sha256(b).hexdigest()
def need(x,m):
 if not x:raise SystemExit('FAIL: '+m)
def obj(b):return json.loads(b)
a=argparse.ArgumentParser();a.add_argument('--key');a=a.parse_args();root=pathlib.Path(__file__).resolve().parent;m=obj((root/'MANIFEST.json').read_bytes())
for x in m['files']:
 n=x['path'];need('/' not in n and '\\' not in n and n not in ('.','..'),'unsafe public path');b=(root/n).read_bytes();need(len(b)==x['bytes'] and sha(b)==x['sha256'],'public '+n)
print('Public integrity:',len(m['files']),'files')
if not a.key:print('Restricted commands, output and inputs were not checked; separate key required.');raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
key=pathlib.Path(a.key).read_bytes();need(len(key)==32,'key length');c=(root/'PRIVATE-EVIDENCE.aesgcm').read_bytes();magic=b'726POSTDAC7v1\0';need(c.startswith(magic),'cipher header');nonce=c[len(magic):len(magic)+12];raw=AESGCM(key).decrypt(nonce,c[len(magic)+12:],b'726-postdac7-prepared-stream-replay-v1');t=tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz');ms=t.getmembers();need(all(x.isfile() and not x.name.startswith('/') and '..' not in pathlib.PurePosixPath(x.name).parts for x in ms),'private path');need(len({x.name for x in ms})==len(ms),'duplicate path');fs={x.name:t.extractfile(x).read() for x in ms};pm=obj(fs['PRIVATE-MANIFEST.json']);rows={x['path']:x for x in pm['files']};need(set(fs)-{'PRIVATE-MANIFEST.json'}==set(rows),'private set')
for n,x in rows.items():need(len(fs[n])==x['bytes'] and sha(fs[n])==x['sha256'],'private file '+n)
def filemode(path):return int(rows[path]['original_mode'],8)
def check_manifest(prefix,kind):
 manifest=obj(fs[prefix+'/MANIFEST.json']);items=manifest['files'];wanted={x['path']:x for x in items} if isinstance(items,list) else items;actual={k[len(prefix)+1:] for k in fs if k.startswith(prefix+'/') and k!=prefix+'/MANIFEST.json'};need(actual==set(wanted),'manifest set '+prefix)
 for n,x in wanted.items():
  path=prefix+'/'+n;b=fs[path];need(len(b)==x['bytes'] and sha(b)==x['sha256'],'manifest bytes '+path)
  if kind=='prepared':need(filemode(path)==x['mode'],'manifest mode '+path)
 return len(wanted)
for pre,kind in [('prepared/public','prepared'),('prepared/restricted','prepared'),('prepared/v3-spotcheck','prepared'),('stream/public','stream'),('stream/restricted','stream'),('stream/scan','stream')]:print('Captured manifest',pre,check_manifest(pre,kind))
a=obj(fs['q2812-anchor/ANCHOR-MANIFEST.json']);required=set()
for x in a['anchors']:
 name=x['file'];required.add(name);path='q2812-anchor/'+name;b=fs[path];need(len(b)==x['bytes'] and sha(b)==x['sha256'],'anchor bytes '+name);need(format(filemode(path),'04o')==x['source_mode_octal']==x['preserved_mode_octal'],'anchor mode '+name)
for name,x in a['support'].items():
 required.add(name);path='q2812-anchor/'+name;b=fs[path];need(len(b)==x['bytes'] and sha(b)==x['sha256'] and format(filemode(path),'04o')==x['mode_octal'],'anchor support '+name)
need({k[len('q2812-anchor/'):] for k in fs if k.startswith('q2812-anchor/')}-{'ANCHOR-MANIFEST.json'}==required,'anchor set')
q=fs['q2812-anchor/original-059-queue.json'];r=fs['q2812-anchor/reconstructed-059-queue.json'];need(sha(q)!=sha(r),'historical streams distinct');print('Captured q2812 anchors verified:',len(a['anchors']))
labels=['anchor','prepared_prior_full','prepared_v3_full','prepared_public_full','stream_restricted','stream_public_full','stream_scan_receipt']
for n in labels:
 x=obj(fs['replay/'+n+'.result.json']);out=fs['replay/'+n+'.stdout'];err=fs['replay/'+n+'.stderr'];need(x['exit_code']==0 and not x['timeout'] and len(err)==0 and len(out)==x['stdout_bytes'] and sha(out)==x['stdout_sha256'] and sha(err)==x['stderr_sha256'],'recorded replay '+n)
s=obj(fs['replay/stream_public_full.stdout']);need(s['result']=='PASS' and all(s['external_bindings'].values()),'stream full bindings');v=obj(fs['replay/stream_restricted.stdout']);need(v['result']=='PASS' and v['public_manifest_bound'],'stream restricted binding');z=obj(fs['replay/stream_scan_receipt.stdout']);need(z['result']=='PASS' and all(z['external_manifest_bindings'].values()),'stored scan receipt binding');print('Captured seven complete successful command results and six stream external bindings verified.')
print('This verifies recorded evidence and input manifests; it does not repeat the historical scan, rerun product code, prove independent custody or restore checkpoint 3821.')
