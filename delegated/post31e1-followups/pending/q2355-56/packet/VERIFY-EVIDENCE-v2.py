#!/usr/bin/env python3
"""Verify this bounded evidence packet, never import or execute product code."""
import argparse,gzip,hashlib,json,os,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
REV='95e63628f9d70b8dd6e6fede64334696c6995009'
OLD='e2755a732c84e67296b5e3441a4259a84f461b2f'
def j(name): return json.loads((P/name).read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def check_manifest():
 m=j('MANIFEST.json'); entries=list(P.iterdir())
 assert all(x.is_file() and not x.is_symlink() for x in entries)
 assert {x.name for x in entries if x.name!='MANIFEST.json'}==set(m['files'])
 for n,v in m['files'].items():
  assert pathlib.Path(n).name==n
  b=(P/n).read_bytes(); assert len(b)==v['bytes'] and sha(b)==v['sha256'],n

def refs(v,out):
 if isinstance(v,dict):
  for a,b in [('path','sha256'),('verification','verification_sha256'),('receipt','receipt_sha256'),('registration','registration_sha256')]:
   if isinstance(v.get(a),str) and isinstance(v.get(b),str) and len(v[b])==64: out.add((v[a],v[b]))
  for x in v.values(): refs(x,out)
 elif isinstance(v,list):
  for x in v: refs(x,out)

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=pathlib.Path);ap.add_argument('--transport-manifest',type=pathlib.Path);a=ap.parse_args();check_manifest()
 caps=[]
 for f in sorted(P.glob('SOURCE-CAPTURES*.json')): caps.extend(json.loads(f.read_text()))
 assert len(caps)==42 and len({c['path'] for c in caps})==42
 bypath={c['path']:c for c in caps}
 for c in caps:
  b=(P/c['capture']).read_bytes(); assert sha(b)==c['sha256'] and blob(b)==c['blob'] and len(b)==c['bytes'] and len(b.splitlines())==c['eof']
 regs={f.name:json.loads(f.read_text()) for f in P.glob('SOURCE*REGISTRATION.json')}
 receipt=j('SOURCE-READ-RECEIPT.json'); assert len(receipt['displays'])==45
 spans={c['path']:[] for c in caps}
 for row in receipt['displays']:
  lo,hi=row['range']; r=regs[row['registration']]; c=bypath[row['path']]
  assert r['revision']==row['revision']==REV and row['registered_before_display']
  assert any(s['path']==row['path'] and s['range'][0]<=lo<=hi<=s['range'][1] for s in r['sources'])
  lines=(P/c['capture']).read_text().splitlines();assert 1<=lo<=hi<=len(lines)
  out=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1)).encode()
  assert out==(P/row['display']).read_bytes() and sha(out)==row['sha256'];spans[row['path']].append((lo,hi))
 for row in receipt['coverage']:
  merged=[]
  for lo,hi in sorted(spans[row['path']]):
   if merged and lo<=merged[-1][1]+1: merged[-1][1]=max(merged[-1][1],hi)
   else: merged.append([lo,hi])
  assert row['personally_displayed_union']==merged and row['personal_displayed_lines']==sum(b-a+1 for a,b in merged) and row['unread_lines']==row['eof']-row['personal_displayed_lines']
 assert sum(x['personal_displayed_lines'] for x in receipt['coverage'])==3763
 ids=j('HISTORY-IDENTITIES.json'); assert len(ids)==4
 for x in ids:
  b=(P/x['capture']).read_bytes();assert len(b)==x['entry_bytes'] and sha(b)==x['entry_sha256']
 for n in (2355,2356):
  original=(P/f'Q{n}-ORIGINAL-EXACT.json').read_bytes();recon=(P/f'Q{n}-PUBLIC_RECONSTRUCTION-EXACT.json').read_bytes();assert original==recon
  d=json.loads(original); s=j(f'Q{n}-SUMMARY.json');assert d['status']==s['status']=='open-connected-owner-current-alias-qualification' and d['implementation_gate_complete'] is s['implementation_gate_complete'] is False
 related=j('RELATED-SELECTION-IDENTITIES.json')
 for x in related['queue']:
  assert sha((P/x['capture']).read_bytes())==x['sha256']
 for n in (67,87,2701,2934):assert (P/f'RELATED-Q{n}-ORIGINAL.json').read_bytes()==(P/f'RELATED-Q{n}-PUBLIC_RECONSTRUCTION.json').read_bytes()
 assert all(x['structural_original_reconstruction_equal'] for x in j('LEDGER-COMPARISON.json'))
 for i in range(3): assert j(f'LEDGER-{i:02}-ORIGINAL.json')==j(f'LEDGER-{i:02}-PUBLIC_RECONSTRUCTION.json')
 for n in (2701,2934): assert 'status' not in j(f'RELATED-Q{n}-ORIGINAL.json') and not j(f'RELATED-Q{n}-SUMMARY.json')['status_key_present']
 actual=set()
 for n in (2355,2356):refs(j(f'Q{n}-ORIGINAL-EXACT.json'),actual)
 registered={(x['path'],x['expected_sha256']) for x in j('RECOVERY-REGISTRATION.json')['references']};assert actual==registered and len(actual)==9,(len(actual),len(registered))
 allrec=[]
 for regn,resn in [('RECOVERY-REGISTRATION.json','RECOVERY-RESULT.json'),('NESTED-REGISTRATION.json','NESTED-RESULT.json'),('RELATED-RECOVERY-REGISTRATION.json','RELATED-RECOVERY-RESULT.json')]:
  reg=j(regn)['references'];res=j(resn);assert len(reg)==len(res)
  for x in res:
   assert any(z['path']==x['reference'] and z.get('expected_sha256',z.get('sha256'))==x['expected_sha256'] for z in reg)
   b=(P/x['capture']).read_bytes();assert x['matched'] and sha(b)==x['expected_sha256'] and len(b)==x['bytes'];allrec.append(x)
 assert len(allrec)==13 and j('TRANSPORT-RESULT.json')['direct_matches']==9 and j('TRANSPORT-RESULT.json')['nested_matches']==2 and j('RELATED-TRANSPORT-RESULT.json')['matched']==2
 print('PASS manifest; 42 source captures, 45 registered displays and coverage; selected queue and ledger comparisons; 13 hash-cited historical records.')
 if a.git_repo:
  env=dict(os.environ,GIT_NO_LAZY_FETCH='1')
  def git(*args):return subprocess.check_output(['git','-C',str(a.git_repo),'--no-replace-objects',*args],env=env)
  for c in caps:
   b=git('show',f'{REV}:{c["path"]}');assert b==(P/c['capture']).read_bytes()
   old=git('show',f'{OLD}:{c["path"]}')
   if 'e275_same_bytes' in c: assert (old==b)==c['e275_same_bytes']
  for x in j('SOURCE-IDENTITIES.json'):
   for rev,v in x['versions'].items():
    b=git('show',f'{rev}:{x["path"]}');assert sha(b)==v['sha256'] and blob(b)==v['blob'] and len(b)==v['bytes']
   assert x['frozen_matches'] and x['e275_95e_equal']==(x['versions'][OLD]['sha256']==x['versions'][REV]['sha256'])
  streams={}
  for x in ids:
   key=(x['revision'],x['path'])
   if key not in streams:streams[key]=git('show',f'{key[0]}:{key[1]}')
   b=streams[key];assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and blob(b)==x['git_blob'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==(P/x['capture']).read_bytes()
  for x in related['queue']:
   b=streams[(x['revision'],'baseline-059/queue.json')];assert b[x['start']:x['end']]==(P/x['capture']).read_bytes()
  for x in related['ledger']:
   key=(x['revision'],'baseline-059/ledger.json')
   if key not in streams:streams[key]=git('show',f'{key[0]}:{key[1]}')
   b=streams[key];assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'];assert json.loads(b)['discovery_findings_and_dependencies'][x['key']]==j(x['capture'])
  print('PASS Git object store: pinned source bodies, historic identities and 12 queue slices plus 6 selected ledger objects from their respective streams.')
 else: print('NOT REPEATED: Git source and stream membership (supply --git-repo).')
 if a.transport_manifest:
  data=json.load(gzip.open(a.transport_manifest,'rt'));assert data['transport_only'] and not data['source_read_credit']
  for x in allrec:
   suffix=x['reference'].removeprefix('target/726-audit-adler-20261027/')
   assert any(e['path'].endswith(suffix) and e['sha256']==x['expected_sha256'] and e['bytes']==x['bytes'] for e in data['entries'])
  print('PASS supplied transport metadata: 13/13 records match; this is not independent custody or source reading.')
 else: print('NOT REPEATED: transport metadata correspondence (supply --transport-manifest).')
 print('LIMITS: hashes, timestamps and registrations do not establish comprehension, full producer admission, policy, reachability, product behavior, global closure or independent custody.')
if __name__=='__main__':main()
