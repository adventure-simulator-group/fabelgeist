#!/usr/bin/env python3
"""Verify a bounded q2364/q2365 source-only packet; never run product code."""
import argparse,gzip,hashlib,json,os,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
REV='95e63628f9d70b8dd6e6fede64334696c6995009'
OLD='e2755a732c84e67296b5e3441a4259a84f461b2f'
def j(n):return json.loads((P/n).read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def check_manifest():
 m=j('MANIFEST.json');items=list(P.iterdir());assert all(x.is_file() and not x.is_symlink() for x in items)
 assert {x.name for x in items if x.name!='MANIFEST.json'}==set(m['files'])
 for n,v in m['files'].items():
  assert pathlib.Path(n).name==n
  b=(P/n).read_bytes();assert len(b)==v['bytes'] and sha(b)==v['sha256']
def refs(v,out):
 if isinstance(v,dict):
  for a,b in [('path','sha256'),('verification','verification_sha256'),('receipt','receipt_sha256'),('registration','registration_sha256')]:
   if isinstance(v.get(a),str) and isinstance(v.get(b),str) and len(v[b])==64:out.add((v[a],v[b]))
  for x in v.values():refs(x,out)
 elif isinstance(v,list):
  for x in v:refs(x,out)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=pathlib.Path);ap.add_argument('--transport-manifest',type=pathlib.Path);ap.add_argument('--live-ledger',type=pathlib.Path);a=ap.parse_args();check_manifest()
 caps=[]
 for f in sorted(P.glob('SOURCE-CAPTURES*.json')):caps.extend(json.loads(f.read_text()))
 assert len(caps)==10 and len({x['path'] for x in caps})==10;bypath={x['path']:x for x in caps}
 for x in caps:
  b=(P/x['capture']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and len(b.splitlines())==x['eof']
 regs={f.name:json.loads(f.read_text()) for f in P.glob('SOURCE*REGISTRATION.json')};receipt=j('SOURCE-READ-RECEIPT.json');assert len(receipt['displays'])==17;spans={x['path']:[] for x in caps}
 for x in receipt['displays']:
  lo,hi=x['range'];r=regs[x['registration']];c=bypath[x['path']];assert r['revision']==x['revision']==REV and x['registered_before_display']
  assert any(z['path']==x['path'] and z['range'][0]<=lo<=hi<=z['range'][1] for z in r['sources'])
  lines=(P/c['capture']).read_text().splitlines();assert 1<=lo<=hi<=len(lines);b=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1)).encode()
  assert b==(P/x['display']).read_bytes() and sha(b)==x['sha256'];spans[x['path']].append((lo,hi))
 for x in receipt['coverage']:
  merged=[]
  for lo,hi in sorted(spans[x['path']]):
   if merged and lo<=merged[-1][1]+1:merged[-1][1]=max(merged[-1][1],hi)
   else:merged.append([lo,hi])
  assert x['personally_displayed_union']==merged and x['personal_displayed_lines']==sum(b-a+1 for a,b in merged) and x['unread_lines']==x['eof']-x['personal_displayed_lines']
 assert sum(x['personal_displayed_lines'] for x in receipt['coverage'])==1269
 assert sum(x['unread_lines'] for x in receipt['coverage'])==18944
 ids=j('HISTORY-IDENTITIES.json');assert len(ids)==4
 for x in ids:
  b=(P/x['capture']).read_bytes();assert sha(b)==x['entry_sha256'] and len(b)==x['entry_bytes']
 direct=set();expected={}
 for n in (2364,2365):
  b=(P/f'Q{n}-ORIGINAL-EXACT.json').read_bytes();assert b==(P/f'Q{n}-RECONSTRUCTED-EXACT.json').read_bytes();d=json.loads(b);refs(d,direct)
  assert d['named_disposition_recorded'] and d['audit_discovery_closed_within_exact_named_scope'] and not d['all_lenses_or_coarse_parent_closed'] and not d['global_exact_alias_inventory_complete'] and not d['repair_runtime_passed'] and not d['implementation_gate_complete'] and d['body_or_population_credit']==0
  assert d['disposition']=='documented-source-owner-disposition-binary-runtime-independent' and d['parent_canonical_id'] is None
  for x in d['exact_source_identities_and_assessed_ranges']:
   path=x['source_id'].split(':')[2];assert x['revision_or_explicit_transport_qualification']==OLD
   assert path not in expected or expected[path]==x['source_sha256'];expected[path]=x['source_sha256']
 assert len(expected)==6
 for path,h in expected.items():assert bypath[path]['e275_sha256']==h and bypath[path]['e275_same_bytes'] and bypath[path]['e275_matches_historical']
 related=j('RELATED-IDENTITIES.json');assert len(related)==8
 for x in related:assert sha((P/x['capture']).read_bytes())==x['sha256']
 for n in (2315,2316,2362,2363):assert (P/f'RELATED-Q{n}-ORIGINAL.json').read_bytes()==(P/f'RELATED-Q{n}-RECONSTRUCTED.json').read_bytes()
 registered={(x['path'],x['expected_sha256']) for x in j('RECOVERY-REGISTRATION.json')['references']};assert direct==registered and len(direct)==10
 recovered=[]
 for regn,resn in [('RECOVERY-REGISTRATION.json','RECOVERY-RESULT.json'),('NESTED-REGISTRATION.json','NESTED-RESULT.json')]:
  reg=j(regn)['references'];res=j(resn);assert len(reg)==len(res)
  for x in res:
   assert any(z['path']==x['reference'] and z.get('expected_sha256',z.get('sha256'))==x['expected_sha256'] for z in reg)
   b=(P/x['capture']).read_bytes();assert x['matched'] and len(b)==x['bytes'] and sha(b)==x['expected_sha256'];recovered.append(x)
 assert len(recovered)==14 and j('TRANSPORT-RESULT.json')['matched']==j('TRANSPORT-RESULT.json')['total']==14
 for i,id in [(1037,'relationship-npz-count-zip64-policy-95e'),(1038,'relationship-npz-member-selection-identity-95e'),(1039,'relationship-npz-error-context-policy-95e'),(1040,'relationship-npz-decoded-integrity-budget-95e')]:assert j(f'LIVE-LEDGER-{i}.json')['id']==id
 assert sha((P/'DIFF-CHILD-MANIFEST.txt').read_bytes())==j('DIFF-IDENTITY.json')['sha256']
 print('PASS manifest, ten source identities, 17 registered displays, historical named scopes, 12 queue slices and 14 recovered records.')
 if a.git_repo:
  env=dict(os.environ,GIT_NO_LAZY_FETCH='1')
  def git(*args):return subprocess.check_output(['git','-C',str(a.git_repo),'--no-replace-objects',*args],env=env)
  for x in caps:
   b=git('show',f'{REV}:{x["path"]}');ob=git('show',f'{OLD}:{x["path"]}');assert b==(P/x['capture']).read_bytes() and sha(ob)==x['e275_sha256'] and (b==ob)==x['e275_same_bytes']
  diff=git('diff','--no-ext-diff','--no-textconv',OLD,REV,'--','crates/fabelgeist-numpy-storage/Cargo.toml');assert diff==(P/'DIFF-CHILD-MANIFEST.txt').read_bytes()
  streams={}
  for x in ids:
   key=(x['revision'],x['path'])
   if key not in streams:streams[key]=git('show',f'{key[0]}:{key[1]}')
   b=streams[key];assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and blob(b)==x['git_blob'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==(P/x['capture']).read_bytes()
  for x in related:
   b=streams[(x['revision'],'baseline-059/queue.json')];assert b[x['start']:x['end']]==(P/x['capture']).read_bytes()
  print('PASS Git object store: pinned sources, e275 identities, manifest diff and both historical queue streams/slices.')
 else:print('NOT REPEATED: Git source/stream membership (supply --git-repo).')
 if a.transport_manifest:
  d=json.load(gzip.open(a.transport_manifest,'rt'));assert d['transport_only'] and not d['source_read_credit']
  for x in recovered:
   suffix=x['reference'].removeprefix('target/726-audit-adler-20261027/');assert any(e['path'].endswith(suffix) and e['sha256']==x['expected_sha256'] and e['bytes']==x['bytes'] for e in d['entries'])
  print('PASS 14 supplied transport-metadata matches; no independent custody or source read credit.')
 else:print('NOT REPEATED: transport correspondence (supply --transport-manifest).')
 if a.live_ledger:
  b=a.live_ledger.read_bytes();assert sha(b)==j('LIVE-LEDGER-REGISTRATION.json')['snapshot_sha256'] and len(b)==j('LIVE-LEDGER-IDENTITIES.json')['snapshot_bytes'];d=json.loads(b)['newly_assessed_ledger']['entries']
  for i in (1037,1038,1039,1040):assert d[i]==j(f'LIVE-LEDGER-{i}.json')
  print('PASS supplied live-ledger snapshot: four selected reserialized objects belong to the identified stream.')
 else:print('NOT REPEATED: selected live-ledger membership (supply --live-ledger).')
 print('LIMITS: hashes and timestamps do not prove comprehension, binary fixture contents, parser outcomes, policy, product behavior, global alias disposition or independent custody.')
if __name__=='__main__':main()
