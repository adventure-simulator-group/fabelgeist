#!/usr/bin/env python3
"""Verify q2362/q2363 historical and source evidence, not product behavior."""
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
 for n in ('current-commit','historical-commit','original-queue','reconstructed-queue','ledger-snapshot'):ap.add_argument('--'+n,type=Path,required=True)
 ap.add_argument('--skip-manifest',action='store_true');a=ap.parse_args()
 if not a.skip_manifest:
  manifest=json.loads((P/'MANIFEST.json').read_text());actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and str(x.relative_to(P))!='MANIFEST.json'};assert actual==set(manifest['files'])
  for name,info in manifest['files'].items():
   b=(P/name).read_bytes();assert len(b)==info['bytes'] and sha(b)==info['sha256'],name
 queues=[a.original_queue.read_bytes(),a.reconstructed_queue.read_bytes()];parsed=[json.loads(q) for q in queues]
 for n in [2362,2363]:
  ext=load(f'{n}/QUEUE-EXTRACTION.json');parts=[]
  for i,q in enumerate(queues):
   x=ext['entries'][i];assert sha(q)==x['whole_sha256'];part=q[x['byte_start']:x['byte_end_exclusive']];assert part==(D/str(n)/f'QUEUE-ENTRY-{i}.json').read_bytes() and sha(part)==x['entry_sha256'];assert parsed[i]['entries'][n]==json.loads(part);parts.append(part)
  assert parts[0]==parts[1] and ext['original_and_reconstructed_entry_byte_equal'];entry=json.loads(parts[0]);sel=load(f'{n}/METADATA-VIEW.json');assert all(entry[k]==v for k,v in sel.items());assert entry['exact_source_identities_and_assessed_ranges']==load(f'{n}/SOURCE-METADATA-VIEW.json')
 inv=load('HISTORY-REFERENCE-INVENTORY.json');records=load('HISTORY-COPY-RECEIPT.json');refs={x['path'].removeprefix('<LOCAL_PATH_0078>'):x['sha256'] for x in inv['references']};assert len(records)==len(refs)==10
 for x in records:
  b=(D/'history'/x['path']).read_bytes();assert x['matched'] and sha(b)==x['expected_sha256']==x['observed_sha256']==refs[x['path']] and len(b)==x['bytes']
 for name,selection in load('HISTORY-ROWS-VIEW.json').items():
  obj=json.loads((D/'history'/name).read_text())
  for i,row in selection['rows'].items():assert obj['rows'][int(i)]==row
 prior=load('PRIOR-COPY-VIEW.json');assert len(prior)==7
 for x in prior:
  b=(D/('prior-'+x['label'])/x['name']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256']
 roots={}
 for rev,f,expected in [('95e',a.current_commit,'95e63628f9d70b8dd6e6fede64334696c6995009'),('e275',a.historical_commit,'e2755a732c84e67296b5e3441a4259a84f461b2f')]:
  b=f.read_bytes();assert oid('commit',b)==expected;roots[rev]=b.split(b'\n',1)[0].split()[1].decode()
 proof=load('PROOF-RESULT.json')['paths'];assert len(proof)==12;seen=set()
 for x in proof:
  key=(x['rev'],x['path']);assert key not in seen;seen.add(key);cur=roots[x['rev']];parts=x['path'].split('/');assert len(parts)==len(x['chain'])
  for part,item in zip(parts,x['chain']):
   b=(D/'proof/git-trees'/(cur+'.tree')).read_bytes();assert oid('tree',b)==cur;mode,nxt=entries(b)[part];assert item=={'tree':cur,'component':part,'mode':mode,'oid':nxt};cur=nxt
  root='95e' if x['rev']=='95e' else 'e275-reconstructed';b=(D/'sources'<LOCAL_PATH_0079>['path']).read_bytes();assert x['match'] and x['historical_record_hash_match'] and len(b)==x['bytes'] and sha(b)==x['source_sha256'] and oid('blob',b)==cur==x['blob']==x['expected_blob']
 assert len([x for x in seen if x[0]=='95e'])==len([x for x in seen if x[0]=='e275'])==6
 for x in load('MHR-CAPTURE-PROOF-VIEW.json'):
  cur=roots['95e'];parts=x['path'].split('/');assert len(parts)==len(x['chain'])
  for part,item in zip(parts,x['chain']):
   b=(D/'proof/mhr-trees'/(cur+'.tree')).read_bytes();assert oid('tree',b)==cur;mode,nxt=entries(b)[part];assert item=={'tree':cur,'component':part,'mode':mode,'oid':nxt};cur=nxt
  b=(D/'sources/95e'/x['path']).read_bytes();assert x['match'] and len(b)==x['bytes'] and sha(b)==x['source_sha256'] and oid('blob',b)==cur==x['blob']==x['expected_blob']
 groups=[('RECEIPT-*.json','DISPLAY-REGISTRATION.json'),('MHR-RECEIPT-*.json','MHR-DISPLAY-REGISTRATION.json')];count=0
 for pattern,regname in groups:
  reg=load(regname)
  for f in D.glob(pattern):
   x=json.loads(f.read_text());path=x['source'];lo,hi=x['range'];assert any(lo>=a0 and hi<=b0 for a0,b0 in reg['windows'][path]);assert datetime.datetime.fromisoformat(x['at_utc'])>=datetime.datetime.fromisoformat(reg['at_utc']);raw=(D/'sources/95e'/path).read_bytes();assert sha(raw)==x['source_sha256'];lines=raw.decode().splitlines();assert 1<=lo<=hi<=len(lines);text=f'SOURCE {path}:{lo}-{hi}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(lo,hi+1));assert text==(D/x['display']).read_text() and sha(text.encode())==x['display_sha256'];count+=1
 assert count==10
 raw=a.ledger_snapshot.read_bytes();nav=load('LEDGER-NAV-VIEW.json');assert len(raw)==nav['bytes'] and sha(raw)==nav['sha256'];rows=json.loads(raw)['newly_assessed_ledger']['entries'];assert len(rows)==nav['rows']
 for t,res in nav['terms'].items():
  hits=[{'index':i,'id':r.get('id')} for i,r in enumerate(rows) if t.casefold() in json.dumps(r,ensure_ascii=False).casefold()];assert len(hits)==res['count'] and hits[:30]==res['shown'] and (len(hits)>30)==res['truncated']
 for x in load('LEDGER-SELECT-VIEW.json'):assert rows[int(x['pointer'].split('/')[-1])]==x['value']
 print('Evidence verified: two exact 059 entries, 10 historical records, prior q2315/q2316 context, 14 revision/path proofs, 10 registered current displays and bounded ledger snapshot.')
 print('No product code, parser, build, test, fixture, macro or runtime probe executed.')
if __name__=='__main__':main()
