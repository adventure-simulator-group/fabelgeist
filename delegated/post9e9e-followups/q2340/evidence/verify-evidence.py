#!/usr/bin/env python3
"""Verify q2340 captured evidence only, not product behavior."""
import argparse,datetime,hashlib,json,re
from pathlib import Path
P=Path(__file__).resolve().parent;D=P/'data'
def sha(b):return hashlib.sha256(b).hexdigest()
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(n):return json.loads((D/n).read_text())
def entries(b):
 out={};i=0
 while i<len(b):
  s=b.index(b' ',i);z=b.index(b'\0',s);out[b[s+1:z].decode()]=(b[i:s].decode(),b[z+1:z+21].hex());i=z+21
 return out
def main():
 ap=argparse.ArgumentParser(description=__doc__)
 for n in ('current-commit','frozen-commit','original-queue','reconstructed-queue','ledger-snapshot'):ap.add_argument('--'+n,type=Path,required=True)
 ap.add_argument('--skip-manifest',action='store_true');a=ap.parse_args()
 if not a.skip_manifest:
  m=json.loads((P/'MANIFEST.json').read_text());found={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x.name!='MANIFEST.json'};assert found==set(m['files'])
  for n,v in m['files'].items():
   b=(P/n).read_bytes();assert len(b)==v['bytes'] and sha(b)==v['sha256'],n
 ext=load('QUEUE-EXTRACTION.json');bs=[]
 for i,path in enumerate([a.original_queue,a.reconstructed_queue]):
  raw=path.read_bytes();e=ext['entries'][i];assert sha(raw)==e['whole_sha256'];piece=raw[e['byte_start']:e['byte_end_exclusive']];assert piece==(D/f'QUEUE-ENTRY-{i}.json').read_bytes() and sha(piece)==e['entry_sha256'];assert json.loads(raw)['entries'][2340]==json.loads(piece);bs.append(piece)
 assert bs[0]==bs[1] and ext['original_and_reconstructed_entry_byte_equal'];lines=bs[0].decode().splitlines();assert len(lines)==98
 for i,(lo,hi) in enumerate([(1,50),(51,98)],1):assert ''.join(f'{j:4} {lines[j-1]}\n' for j in range(lo,hi+1))==(D/f'HISTORY-DISPLAY-{i}.txt').read_text()
 inv={x['path'].removeprefix('<LOCAL_PATH_0082>'):x['sha256'] for x in load('HISTORY-REFERENCE-INVENTORY.json')}
 hist=load('HISTORY-COPY-RECEIPT.json');assert len(inv)==len(hist)==4
 for e in hist:
  b=(D/'history'/e['path']).read_bytes();assert len(b)==e['bytes'] and sha(b)==e['observed_sha256']==e['expected_sha256']==inv[e['path']] and e['status']=='matched'
 nested=load('NESTED-RECEIPT.json');files=[D/'sources/6533e444fa37552927070cc8d8190907fd3b413e/crates/adventuresim-stdb-module/src/investigation/model.rs',D/'history/parallel-l13-20261029/VERIFY-PL13-261.json'];assert len(nested)==2
 for row,f in zip(nested,files):
  b=f.read_bytes();assert len(b)==row['bytes'] and sha(b)==row['observed_sha256']==row['expected_sha256'] and row['status']=='matched'
 roots={}
 for rev,f in [('95e',a.current_commit),('6533',a.frozen_commit)]:
  b=f.read_bytes();assert oid('commit',b)=={'95e':'95e63628f9d70b8dd6e6fede64334696c6995009','6533':'6533e444fa37552927070cc8d8190907fd3b413e'}[rev];roots[rev]=b.split(b'\n',1)[0].split()[1].decode()
 proofs=load('PROOF-RESULT.json')['paths'];assert len(proofs)==16
 for x in proofs:
  root=roots[x['revision']];cur=root
  for comp,item in zip(x['path'].split('/'),x['chain']):
   raw=(D/'proof/git-trees'/(cur+'.tree')).read_bytes();assert oid('tree',raw)==cur;mode,nextid=entries(raw)[comp];assert item=={'tree':cur,'component':comp,'mode':mode,'oid':nextid};cur=nextid
  candidates=[D/'sources'/('6533e444fa37552927070cc8d8190907fd3b413e' if x['revision']=='6533' else '95e63628f9d70b8dd6e6fede64334696c6995009')/x['path'],D/'sources/95e'/x['path']];b=next(z.read_bytes() for z in candidates if z.exists());assert sha(b)==x['source_sha256'] and len(b)==x['source_bytes'] and oid('blob',b)==cur==x['computed_blob']==x['expected_blob'] and x['match']
 groups=[('SOURCE','SOURCE-REGISTRATION.json','95e63628f9d70b8dd6e6fede64334696c6995009'),('CALLER','CALLER-SOURCE-REGISTRATION.json','95e63628f9d70b8dd6e6fede64334696c6995009'),('CALLER-EXT','CALLER-EXT-REGISTRATION.json','95e63628f9d70b8dd6e6fede64334696c6995009'),('TIME','TIME-DISPLAY-REGISTRATION.json','95e'),('MOVEMENT','MOVEMENT-DISPLAY-REGISTRATION.json','95e')];seen=[]
 for prefix,regname,source_root in groups:
  reg=load(regname)
  for f in sorted(D.glob(prefix+'-RECEIPT-*.json')):
   rec=json.loads(f.read_text());name=rec['source'];lo,hi=rec['range'];assert any(lo>=x and hi<=y for x,y in reg['windows'][name]);assert datetime.datetime.fromisoformat(rec['at_utc'])>=datetime.datetime.fromisoformat(reg['at_utc']);raw=(D/'sources'/source_root/name).read_bytes();assert sha(raw)==rec['source_sha256'];lines=raw.decode().splitlines();assert 1<=lo<=hi<=len(lines);display=f'SOURCE {name}:{lo}-{hi}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(lo,hi+1));assert display==(D/rec['display']).read_text() and sha(display.encode())==rec['display_sha256'];seen.append(f.name)
 assert len(seen)==43,(len(seen),seen)
 raw=a.ledger_snapshot.read_bytes();nav=load('LEDGER-NAV-VIEW.json');assert len(raw)==nav['bytes'] and sha(raw)==nav['sha256'];rows=json.loads(raw)['newly_assessed_ledger']['entries'];assert len(rows)==nav['rows']
 for term,res in nav['terms'].items():
  hits=[{'index':i,'id':row.get('id')} for i,row in enumerate(rows) if term.casefold() in json.dumps(row,ensure_ascii=False).casefold()];assert len(hits)==res['count'] and hits[:30]==res['shown'] and (len(hits)>30)==res['truncated']
 print('Evidence verified: exact q2340 entries, five historical records, frozen source, 16 Git-bound revision/path pairs, 43 registered source displays and bounded ledger snapshot.')
 print('No product code, product parser, build, test, fixture, macro or runtime probe executed.')
if __name__=='__main__':main()
