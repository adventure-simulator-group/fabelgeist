#!/usr/bin/env python3
"""Classify four recorded scanner-source candidates without displaying them."""
import argparse,ast,hashlib,json,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=argparse.ArgumentParser();p.add_argument('--capture',type=Path);p.add_argument('--receipt',type=Path);a=p.parse_args();root=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((root/'MANIFEST.json').read_bytes());assert m['schema']=='726-scan-candidate-review-v1';assert {x.name for x in root.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 b=(root/n).read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
if bool(a.capture)!=bool(a.receipt):raise ValueError('Provide both --capture and --receipt')
if not a.capture:print('Verified public review only; captured tree and candidate receipt were not checked.');sys.exit(0)
raw=(a.capture/'CAPTURE.json').read_bytes();assert h(raw)=='32dfb31245ae82c1a71a362e0a9edaf81d4cfe742de3a3c062bbfbe439248ad1';j=json.loads(raw);receipt=a.receipt.read_bytes();assert h(receipt)=='94fc85795ca33d9d0c7778446e6f9bee378629af0906c949027c9f6bb72f710b';q=json.loads(receipt);assert q['forbidden_digest_total']==0 and q['other_candidate_counts']=={'unix-temp':4} and len(q['other_candidate_locations'])==4 and not q['exceptions'];rows=j['rows'];assert len(rows)==j['count']==139;seen=set()
for row in rows:
 name=row['path'];assert name not in seen and not Path(name).is_absolute() and '..' not in Path(name).parts;seen.add(name);f=a.capture/'tree'/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();oid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();assert (len(b),h(b),oid)==(row['bytes'],row['sha256'],row['git_blob'])
assert {x.relative_to(a.capture/'tree').as_posix() for x in (a.capture/'tree').rglob('*') if x.is_file()}==seen
needle=b'/' + b'tmp/';values=[]
for row in q['other_candidate_locations']:
 assert row['kind']=='unix-temp' and row['count']==1 and row['location'] in seen;source=(a.capture/'tree'/row['location']).read_bytes();assert source.count(needle)==1;calls=[]
 for node in ast.walk(ast.parse(source.decode())):
  if isinstance(node,ast.Call) and isinstance(node.func,ast.Attribute) and isinstance(node.func.value,ast.Name) and node.func.value.id=='re' and node.func.attr=='compile' and node.args and isinstance(node.args[0],ast.Constant):
   v=node.args[0].value
   if isinstance(v,str):v=v.encode()
   if isinstance(v,bytes) and needle in v:calls.append(v)
 assert len(calls)==1;values+=calls
assert len(values)==4 and all(x==values[0] for x in values)
print('Verified 139 captured file identities and four equal scanner regex literals; candidate values were not displayed.')
