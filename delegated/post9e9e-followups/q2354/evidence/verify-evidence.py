#!/usr/bin/env python3
"""Verify q2354 audit evidence identities only, never product behavior."""
import argparse,datetime,hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent;D=P/'data'
def sha(b):return hashlib.sha256(b).hexdigest()
def oid(k,b):return hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(n):return json.loads((D/n).read_text())
def entries(b):
 out={};i=0
 while i<len(b):
  a=b.index(b' ',i);z=b.index(b'\0',a);out[b[a+1:z].decode()]=(b[i:a].decode(),b[z+1:z+21].hex());i=z+21
 return out
def main():
 ap=argparse.ArgumentParser(description=__doc__)
 for n in ('current-commit','frozen-commit','original-queue','reconstructed-queue','ledger-snapshot'):ap.add_argument('--'+n,type=Path,required=True)
 ap.add_argument('--skip-manifest',action='store_true');a=ap.parse_args()
 if not a.skip_manifest:
  m=json.loads((P/'MANIFEST.json').read_text());actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x.name!='MANIFEST.json'};assert actual==set(m['files'])
  for name,info in m['files'].items():
   b=(P/name).read_bytes();assert len(b)==info['bytes'] and sha(b)==info['sha256'],name
 ext=load('QUEUE-EXTRACTION.json');selected=[]
 for i,f in enumerate([a.original_queue,a.reconstructed_queue]):
  b=f.read_bytes();r=ext['entries'][i];assert sha(b)==r['whole_sha256'];piece=b[r['byte_start']:r['byte_end_exclusive']];assert piece==(D/f'QUEUE-ENTRY-{i}.json').read_bytes() and sha(piece)==r['entry_sha256'];assert json.loads(b)['entries'][2354]==json.loads(piece);selected.append(piece)
 assert selected[0]==selected[1] and ext['original_and_reconstructed_entry_byte_equal'];entry=json.loads(selected[0]);assert len(selected[0].decode().splitlines())==949
 top=load('HISTORY-METADATA-VIEW.json')
 for k,v in top.items():
  if k!='granular_contract_view_keys':assert entry[k]==v
 assert top['granular_contract_view_keys']==[list(x) for x in entry['granular_contract_views']]
 granular=load('GRANULAR-METADATA-VIEW.json')
 for view,selection in zip(entry['granular_contract_views'],granular):assert {k:view.get(k) for k in selection}==selection
 inv={x['path'].removeprefix('<LOCAL_PATH_0200>'):x['sha256'] for x in load('HISTORY-REFERENCE-INVENTORY.json')['references']};records=load('HISTORY-COPY-RECEIPT.json');assert len(inv)==len(records)==21
 for x in records:
  b=(D/'history'/x['path']).read_bytes();assert x['status']=='matched' and len(b)==x['bytes'] and sha(b)==x['expected_sha256']==x['observed_sha256']==inv[x['path']]
 nested=load('NESTED-COPY-VIEW.json');assert len(nested)==6
 for x in nested:
  b=(D/'history'/x['path']).read_bytes();assert x['matched'] and len(b)==x['bytes'] and sha(b)==x['expected_sha256']==x['observed_sha256']
 frozen=load('FROZEN-CAPTURE-VIEW.json');assert len(frozen)==11
 for x in frozen:
  b=(D/'sources/6533'/x['path']).read_bytes();assert x['matched'] and len(b)==x['bytes'] and sha(b)==x['expected_sha256']==x['observed_sha256']
 roots={}
 for rev,f,expected in [('95e',a.current_commit,'95e63628f9d70b8dd6e6fede64334696c6995009'),('6533',a.frozen_commit,'6533e444fa37552927070cc8d8190907fd3b413e')]:
  b=f.read_bytes();assert oid('commit',b)==expected;roots[rev]=b.split(b'\n',1)[0].split()[1].decode()
 trees=[D/'proof/git-trees',D/'proof/more-trees',D/'proof/native-trees',D/'proof/final-trees',D/'proof/additional'];proofs=load('PROOF-RESULT.json')['paths']
 proofs += [dict(x,rev='95e') for x in load('MORE-PROOF-RESULT.json')['paths']]
 proofs += [dict(x,rev='95e') for x in load('NATIVE-EXTRA-VIEW.json')]
 proofs += [dict(x,rev='95e') for x in load('FINAL-SOURCE-VIEW.json')]
 proofs += [dict(load('GLOBAL-PROOF-VIEW.json'),rev='95e')];assert len(proofs)==32
 seen=set()
 for row in proofs:
  rev=row['rev'];path=row['path'];key=(rev,path);assert key not in seen;seen.add(key);cur=roots[rev];parts=path.split('/');assert len(parts)==len(row['chain'])
  for comp,item in zip(parts,row['chain']):
   f=next(t/(cur+'.tree') for t in trees if (t/(cur+'.tree')).exists());raw=f.read_bytes();assert oid('tree',raw)==cur;mode,nxt=entries(raw)[comp];assert item=={'tree':cur,'component':comp,'mode':mode,'oid':nxt};cur=nxt
  b=(D/'sources'/rev/path).read_bytes();expected=row.get('source_sha256',row.get('sha256'));assert sha(b)==expected and len(b)==row['bytes'] and oid('blob',b)==cur==row['blob'] and row['match']
 assert len([x for x in seen if x[0]=='95e'])==21 and len([x for x in seen if x[0]=='6533'])==11
 for x in load('PER-FILE-IDENTITY-VIEW.json'):
  old=(D/'sources/6533'/x['path']).read_bytes();new=(D/'sources/95e'/x['path']).read_bytes();assert sha(old)==x['frozen_sha256'] and sha(new)==x['current_sha256'] and (old==new)==x['byte_equal']
 groups=[('RECEIPT-*.json','DISPLAY-REGISTRATION.json'),('MORE-RECEIPT-*.json','MORE-DISPLAY-REGISTRATION.json'),('CALLER-RECEIPT-*.json','CALLER-DISPLAY-REGISTRATION.json'),('NATIVE-RECEIPT-*.json','NATIVE-DISPLAY-REGISTRATION.json'),('FINAL-RECEIPT-*.json','FINAL-DISPLAY-REGISTRATION.json'),('ROMANCE-RECEIPT-*.json','ROMANCE-CALL-REGISTRATION.json'),('CLOCK-RECEIPT-*.json','CLOCK-DISPLAY-REGISTRATION.json'),('GLOBAL-RECEIPT-*.json','GLOBAL-DISPLAY-REGISTRATION.json')];displays=[]
 for pattern,regname in groups:
  reg=load(regname)
  for f in sorted(D.glob(pattern)):
   x=json.loads(f.read_text());name=x['source'];lo,hi=x['range'];assert any(lo>=a0 and hi<=b0 for a0,b0 in reg['windows'][name]);assert datetime.datetime.fromisoformat(x['at_utc'])>=datetime.datetime.fromisoformat(reg['at_utc']);b=(D/'sources/95e'/name).read_bytes();assert sha(b)==x['source_sha256'];lines=b.decode().splitlines();assert 1<=lo<=hi<=len(lines);text=f'SOURCE {name}:{lo}-{hi}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(lo,hi+1));assert text==(D/x['display']).read_text() and sha(text.encode())==x['display_sha256'];displays.append(f.name)
 assert len(displays)==46,(len(displays),displays)
 b=a.ledger_snapshot.read_bytes();nav=load('LEDGER-NAV-VIEW.json');assert len(b)==nav['bytes'] and sha(b)==nav['sha256'];rows=json.loads(b)['newly_assessed_ledger']['entries'];assert len(rows)==nav['rows']
 for term,res in nav['terms'].items():
  hits=[{'index':i,'id':x.get('id')} for i,x in enumerate(rows) if term.casefold() in json.dumps(x,ensure_ascii=False).casefold()];assert len(hits)==res['count'] and hits[:30]==res['shown'] and (len(hits)>30)==res['truncated']
 for x in load('LEDGER-SELECT-VIEW.json'):assert rows[int(x['pointer'].split('/')[-1])]==x['value']
 print('Evidence verified: selected q2354 entries, 27 historical records, 11 frozen objects, 32 revision/path proofs, 46 registered displays and bounded ledger snapshot.')
 print('No product code, parser, build, test, fixture, macro or runtime probe executed.')
if __name__=='__main__':main()
