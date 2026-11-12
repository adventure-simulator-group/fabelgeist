#!/usr/bin/env python3
"""Evidence-only replay for q2366/q2367. Does not execute product code."""
import argparse,datetime,hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent
def sha(b):return hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def load(name):return json.loads((P/name).read_text())
def tree_entries(b):
 out={};i=0
 while i<len(b):
  a=b.index(b' ',i);z=b.index(b'\0',a);out[b[a+1:z].decode()]=(b[i:a].decode(),b[z+1:z+21].hex());i=z+21
 return out
def main():
 ap=argparse.ArgumentParser(description=__doc__)
 for n in ('current-commit','historical-commit','original-queue','reconstructed-queue','current-inventory','ledger-snapshot'):ap.add_argument('--'+n,type=Path,required=True)
 a=ap.parse_args();manifest=load('MANIFEST.json');actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and str(x.relative_to(P))!='MANIFEST.json'};assert actual==set(manifest['files'])
 for n,x in manifest['files'].items():
  b=(P/n).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'],n
 ext=load('QUEUE-EXTRACTION-CORRECTED.json');qreg=load('REGISTRATION-005.json');raw=[a.original_queue.read_bytes(),a.reconstructed_queue.read_bytes()];parsed=[];chosen={}
 for i,b in enumerate(raw):
  assert sha(b)==qreg['inputs'][i]['sha256'];o=json.loads(b);parsed.append(o)
  for n in (2366,2367):
   x=ext['entries'][f'{n}-{i}'];part=b[x['byte_start']:x['byte_end_exclusive']];assert len(part)==x['bytes'] and sha(part)==x['entry_sha256'];assert part==(P/f'q{n}-corrected-entry-{i}.json').read_bytes();assert json.loads(part)==o['entries'][n]
   if i==0:chosen[n]=json.loads(part)
   else:assert part==(P/f'q{n}-corrected-entry-0.json').read_bytes()
 # The erroneous one-based extraction and accidental q2365 display remain replayable.
 wrong=load('QUEUE-EXTRACTION.json')
 for i,b in enumerate(raw):
  for label,index in ((2366,2365),(2367,2366)):
   x=wrong['entries'][f'{label}-{i}'];part=b[x['byte_start']:x['byte_end_exclusive']];assert sha(part)==x['entry_sha256'];assert part==(P/f'q{label}-entry-{i}.json').read_bytes();assert json.loads(part)==parsed[i]['entries'][index]
 refs={};expected={}
 def add(path,sh):
  path=path.removeprefix('<PRIVATE-PATH-0022>')
  if path in refs:assert refs[path]==sh
  refs[path]=sh
 for e in chosen.values():
  for row in e['exact_source_identities_and_assessed_ranges']:
   p=row['source_id'].split(':')[2]
   if p in expected:assert expected[p]==row['source_sha256']
   expected[p]=row['source_sha256']
   for k in ('body_receipt','registration_ref_and_sha256'):add(row[k]['path'],row[k]['sha256'])
   add(row['root_adoption']['path'],row['root_adoption']['sha256']);add(row['root_adoption']['verification'],row['root_adoption']['verification_sha256'])
  add(e['proposal']['path'],e['proposal']['sha256']);bundle=e['related_prior_native_policy_context']['bundle'];add(bundle['path'],bundle['sha256'])
 history=load('HISTORY-COPY-RECEIPT.json');assert len(history)==len(refs)==10
 for x in history:
  b=(P/'data/history'/x['path']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256']==refs[x['path']] and x['matched_recorded_sha256']
 for p,sel in load('HISTORY-SELECTED-ROWS.json').items():
  o=json.loads((P/'data/history'/p).read_text())
  for i,v in sel['rows'].items():assert o['rows'][int(i)]==v
 for n,sel in load('HISTORY-ADOPTION-SELECTED.json').items():
  o=json.loads((P/f'data/history/segovb/READ-L11-{n}.json').read_text());v=json.loads((P/f'data/history/segovb/VERIFY-L11-{n}.json').read_text())
  for i,row in sel['rows'].items():assert o['rows'][int(i)]==row
  for k in ('source_revision','actual_current_target','other_lens_coverage_transferred','runtime_validation','implementation_gate_complete'):assert sel[k]==o[k]
  for k,value in sel['verification'].items():assert value==v[k]
 commits={}
 for revision,path in [('95e63628f9d70b8dd6e6fede64334696c6995009',a.current_commit),('e2755a732c84e67296b5e3441a4259a84f461b2f',a.historical_commit)]:
  b=path.read_bytes();assert oid('commit',b)==revision;commits[revision]=b.split(b'\n',1)[0].split()[1].decode()
 proof=load('PROOF-ANCHORS-RESULT.json');assert not proof['missing'] and not proof['fallback_failures'];seen=set()
 for x in proof['proofs']:
  key=(x['revision'],x['path']);assert key not in seen;seen.add(key);cur=commits[x['revision']];parts=x['path'].split('/');assert len(parts)==len(x['chain'])
  for part,item in zip(parts,x['chain']):
   b=(P/'data2/proof/trees'/(cur+'.tree')).read_bytes();assert oid('tree',b)==cur;mode,nxt=tree_entries(b)[part];assert item=={'tree':cur,'component':part,'mode':mode,'oid':nxt};cur=nxt
  b=(P/'data2/source-candidates'/x['path']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['source_sha256']==x['recorded_e275_sha256']==expected[x['path']] and oid('blob',b)==cur==x['blob']
 assert len(seen)==10 and len(expected)==5
 inventory=a.current_inventory.read_bytes();assert sha(inventory)==proof['inventory_sha256'];lines=inventory.decode().splitlines()
 for x in proof['inventory_rows']:
  assert x['row'] in lines and x['row'].endswith('\t'+x['path']);b=(P/'data2/source-candidates'/x['path']).read_bytes();assert oid('blob',b)==x['row'].split()[2]
 reg=load('REGISTRATION-006.json');receipts=load('SOURCE-DISPLAY-RECEIPTS.json');assert len(receipts)==5
 for x in receipts:
  path=x['source'];lo,hi=x['range'];assert [lo,hi] in reg['displays'][path];assert datetime.datetime.fromisoformat(x['at_utc'])>=datetime.datetime.fromisoformat(reg['at_utc']);b=(P/'data2/source-candidates'/path).read_bytes();assert sha(b)==x['source_sha256'];lines=b.decode().splitlines();assert len(lines)==x['eof'] and 1<=lo<=hi<=len(lines);display=(f'SOURCE {path}:{lo}-{hi}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(lo,hi+1))).encode();assert display==(P/x['display']).read_bytes() and sha(display)==x['display_sha256']
 ledger=a.ledger_snapshot.read_bytes();sel=load('LEDGER-RELATIONSHIPS.json');assert sha(ledger)==sel['ledger_sha256'];rows=json.loads(ledger)['newly_assessed_ledger']['entries'];assert len(rows)==sel['row_count']
 for x in sel['selected']:assert rows[x['index']]['id']==x['id']
 print('Verified two corrected queue entries, retained one-index locator error, ten historical records, ten revision/path proofs, inventory, five registered displays and bounded ledger relationships.')
 print('No product code, parser, build, test, fixture, macro or runtime probe executed.')
if __name__=='__main__':main()
