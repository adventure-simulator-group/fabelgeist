#!/usr/bin/env python3
"""Verify bounded source evidence; never execute product, fixture or shader code."""
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
with tarfile.open(R/'delegated/xpbd-incidence-address-hair-ops-evidence.tar.gz','r:gz') as t:commit=gzip.decompress(t.extractfile(f'git-objects/{rev}.commit.gz').read())
assert oid(b'commit',commit)==rev and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
sets=[('REGISTRATION.json',['DISPLAY-A.txt','DISPLAY-B.txt'],'READ-RECEIPT.json'),('EXT-REGISTRATION.json',['EXT-DISPLAY.txt'],'EXT-READ-RECEIPT.json')]
for regname,displaynames,receiptname in sets:
 rb=(P/regname).read_bytes();reg=json.loads(rb);receipt=json.loads((P/receiptname).read_bytes())
 assert reg['reader']==receipt['reader']=='/root' and H(rb)==receipt['registration_sha256'] and receipt['registration']==regname
 assert [x['path'] for x in receipt['displays']]==displaynames
 for row in receipt['displays']:assert H((P/row['path']).read_bytes())==row['sha256']
 assert len(reg['entries'])==len(receipt['rows']);out=[]
 for e,row in zip(reg['entries'],receipt['rows'],strict=True):
  assert e['revision']==row['revision']==rev and (e['tag'],e['git_path'],e['registered_ranges'])==(row['tag'],row['git_path'],row['displayed_ranges'])
  source=(P/e['capture']).read_bytes();assert (len(source),H(source),oid(b'blob',source))==(e['bytes'],e['sha256'],e['git_blob'])
  pathproof=next(x for x in proof['proofs'] if x['tag']==e['tag']);assert pathproof['path']==e['git_path'] and pathproof['blob']==e['git_blob'];cur=proof['root_tree']
  for name,hop in zip(e['git_path'].split('/'),pathproof['chain'],strict=True):
   assert (hop['tree'],hop['name'])==(cur,name);meta=proof['objects'][cur];raw=gzip.decompress((P/meta['file']).read_bytes());assert (len(raw),H(raw),oid(b'tree',raw))==(meta['decoded_bytes'],meta['decoded_sha256'],cur);mode,cur=tree_entry(raw,name);assert (mode,cur)==(hop['mode'],hop['object'])
  assert cur==e['git_blob'];lines=source.decode().splitlines();assert len(lines)==e['eof'];read=set()
  for first,last in e['registered_ranges']:
   out.append(f"--- {e['tag']} {first}-{last} ---\n");out.extend(f'{n:4d} {lines[n-1]}\n' for n in range(first,last+1));read.update(range(first,last+1))
  unread={n for first,last in row['unread_complement'] for n in range(first,last+1)};assert unread==set(range(1,len(lines)+1))-read
 assert ''.join(out).encode()==b''.join((P/name).read_bytes() for name in displaynames)
cor=json.loads((P/'DRAFT-CORRECTION.json').read_bytes());assert H((P/'ASSESSMENT-DRAFT-INITIAL.json').read_bytes())==cor['initial_draft_sha256'] and H((P/'ASSESSMENT-DRAFT.json').read_bytes())==cor['corrected_draft_sha256']
initial=json.loads((P/'ASSESSMENT-DRAFT-INITIAL.json').read_bytes());final=json.loads((P/'ASSESSMENT-DRAFT.json').read_bytes());a=next(x for x in initial['source_trace'] if x['path']=='crates/fabelgeist-armor/src/wrapped_tassets.rs');b=next(x for x in final['source_trace'] if x['path']=='crates/fabelgeist-armor/src/wrapped_tassets.rs');assert a['lines']==[[1,56]] and b['lines']==[[1,21],[24,56]];a['lines']=b['lines'];assert initial==final
print('Verified five commit-anchored source chains, registered windows and transparently corrected draft citation.')
print('No product, macro, formatting, UI, fixture execution or historical closure is established.')
