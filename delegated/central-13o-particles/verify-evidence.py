#!/usr/bin/env python3
"""Verify registered particle evidence identities; no product execution."""
import argparse,gzip,hashlib,json,pathlib,tarfile
P=pathlib.Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def oid(k,b):return hashlib.sha1(k+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def tree_entry(b,name):
 i=0;matches=[]
 while i<len(b):
  j=b.index(b'\0',i);mode,n=b[i:j].split(b' ',1);obj=b[j+1:j+21];assert len(obj)==20
  if n.decode()==name:matches.append((mode.decode(),obj.hex()))
  i=j+21
 assert len(matches)==1;return matches[0]
a=argparse.ArgumentParser();a.add_argument('--publication',type=pathlib.Path,required=True);args=a.parse_args();R=args.publication
rev='95e63628f9d70b8dd6e6fede64334696c6995009';proof=json.loads((P/'TREE-PROOFS.json').read_bytes());assert proof['revision']==rev
with tarfile.open(R/'delegated/hair-collider-evidence.tar.gz','r:gz') as t:commit=gzip.decompress(t.extractfile(f'git-objects/{rev}.commit.gz').read())
assert oid(b'commit',commit)==rev and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
for regname,displayname,receiptname in [('REGISTRATION.json','DISPLAY.txt','READ-RECEIPT.json'),('EXT-REGISTRATION.json','EXT-DISPLAY.txt','EXT-READ-RECEIPT.json')]:
 rb=(P/regname).read_bytes();db=(P/displayname).read_bytes();reg=json.loads(rb);receipt=json.loads((P/receiptname).read_bytes());assert reg['reader']==receipt['reader']=='/root' and H(rb)==receipt['registration_sha256'] and H(db)==receipt['display_sha256'] and len(reg['entries'])==len(receipt['rows']);out=[]
 for e,row in zip(reg['entries'],receipt['rows'],strict=True):
  assert e['revision']==row['revision']==rev and (e['tag'],e['git_path'],e['registered_ranges'])==(row['tag'],row['git_path'],row['displayed_ranges']);source=(P/e['capture']).read_bytes();assert (len(source),H(source),oid(b'blob',source))==(e['bytes'],e['sha256'],e['git_blob'])
  pathproof=next(x for x in proof['proofs'] if x['tag']==e['tag']);assert pathproof['path']==e['git_path'] and pathproof['blob']==e['git_blob'];cur=proof['root_tree']
  for name,hop in zip(e['git_path'].split('/'),pathproof['chain'],strict=True):
   assert (hop['tree'],hop['name'])==(cur,name);meta=proof['objects'][cur];raw=gzip.decompress((P/meta['file']).read_bytes());assert (len(raw),H(raw),oid(b'tree',raw))==(meta['decoded_bytes'],meta['decoded_sha256'],cur);mode,cur=tree_entry(raw,name);assert (mode,cur)==(hop['mode'],hop['object'])
  assert cur==e['git_blob'];lines=source.decode().splitlines();assert len(lines)==e['eof'];read=set()
  for first,last in e['registered_ranges']:
   out.append(f"--- {e['tag']} {first}-{last} ---\n");out.extend(f'{n:4d} {lines[n-1]}\n' for n in range(first,last+1));read.update(range(first,last+1))
  unread={n for first,last in row['unread_complement'] for n in range(first,last+1)};assert unread==set(range(1,len(lines)+1))-read
 assert ''.join(out).encode()==db
assert H((P/'ASSESSMENT-DRAFT.json').read_bytes())=='e74618959889fe0f2b43d3e58d7128302778fdf2633e9d979d69ca60c239526a'
print('Verified six commit-anchored source chains, two registrations, displays and receipts; no semantic, historical-closure or product claim.')
