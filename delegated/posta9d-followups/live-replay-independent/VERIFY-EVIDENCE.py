#!/usr/bin/env python3
"""Read-only verification of this review packet's files, views and Git-tree membership.

No historical verifier or product code is run. A named commit's association with
its tree is deliberately external: commit objects and author data are not copied.
"""
import hashlib,json,pathlib,sys
P=pathlib.Path(__file__).resolve().parent

def fail(s): raise SystemExit(s)
def sha(b):return hashlib.sha256(b).hexdigest()
def git_hash(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def read(name):
 p=pathlib.PurePosixPath(name)
 if p.is_absolute() or '..' in p.parts:fail('unsafe packet path')
 f=P.joinpath(*p.parts)
 if f.is_symlink() or not f.is_file():fail('missing or linked packet file: '+name)
 return f.read_bytes()
def parse_tree(raw):
 out={};pos=0
 while pos<len(raw):
  try:
   end=raw.index(b'\0',pos); mode,name=raw[pos:end].split(b' ',1);oid=raw[end+1:end+21]
  except ValueError:fail('malformed Git tree')
  if len(oid)!=20 or name in out:fail('invalid tree entry')
  out[name]=(mode.lstrip(b'0'),oid.hex());pos=end+21
 return out
m=json.loads(read('MANIFEST.json'));rows=m['files'];expected={}
for row in rows:
 n=row['path']
 if n in expected or n=='MANIFEST.json':fail('duplicate or self-referential manifest path')
 b=read(n)
 if (len(b),sha(b))!=(row['bytes'],row['sha256']):fail('file mismatch: '+n)
 expected[n]=row
actual=set()
for f in P.rglob('*'):
 if f.is_symlink():fail('linked path in packet')
 if f.is_file():actual.add(f.relative_to(P).as_posix())
if actual != set(expected)|{'MANIFEST.json'}:fail('manifest member set differs')
proof=json.loads(read('TREE-PROOFS.json'))['proofs']
for pr in proof:
 b=read(pr['stored_as'])
 if (len(b),sha(b))!=(pr['bytes'],pr['sha256']):fail('source bytes differ')
 parts=pr['path'].split('/');chain=pr['chain']
 if len(parts)!=len(chain) or not chain or chain[0]['tree']!=pr['revision_tree']:fail('bad proof shape')
 for i,c in enumerate(chain):
  if c['name']!=parts[i]:fail('path mismatch')
  raw=read('git-trees/'+c['tree'])
  if git_hash('tree',raw)!=c['tree']:fail('tree object mismatch')
  ent=parse_tree(raw).get(c['name'].encode())
  if ent!=(c['mode'].encode().lstrip(b'0'),c['object']):fail('tree membership mismatch')
  if i+1<len(chain):
   if c['kind']!='tree' or chain[i+1]['tree']!=c['object']:fail('tree chain mismatch')
  elif c['kind']!='blob' or git_hash('blob',b)!=c['object']:fail('blob identity mismatch')
views=0
for name in ['receipts-03a.json','receipts-03b.json','receipts-04.json','receipts-06.json','receipts-08.json']:
 for r in json.loads(read(name)):
  lines=read(r['source']).decode().splitlines();lo,hi=r['range'];v=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode()
  if v!=read(r['view']) or r.get('sha256',r.get('view_sha256'))!=sha(v):fail('read-view mismatch')
  views+=1
r=json.loads(read('receipt-01.json'));lines=read(r['source']).decode().splitlines();v=''.join(f'{i:6}\t{x}\n' for i,x in enumerate(lines,1)).encode()
if len(lines)!=112 or v!=read(r['view']) or sha(v)!=r['view_sha256']:fail('proposal-view mismatch')
print(f'Verified {len(rows)} exact packet files, {len(proof)} source tree proofs, and {views+1} registered display views.')
print('Commit-to-root binding, complete dependency closure, privacy, custody and actual product behavior require separate evidence.')
