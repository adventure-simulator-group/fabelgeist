#!/usr/bin/env python3
"""Verify a bounded source-only q2359 packet without product execution."""
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
 for name,v in m['files'].items():
  assert pathlib.Path(name).name==name
  b=(P/name).read_bytes();assert len(b)==v['bytes'] and sha(b)==v['sha256']
def refs(v,out):
 if isinstance(v,dict):
  for a,b in [('path','sha256'),('verification','verification_sha256'),('receipt','receipt_sha256'),('registration','registration_sha256')]:
   if isinstance(v.get(a),str) and isinstance(v.get(b),str) and len(v[b])==64:out.add((v[a],v[b]))
  for x in v.values():refs(x,out)
 elif isinstance(v,list):
  for x in v:refs(x,out)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=pathlib.Path);ap.add_argument('--transport-manifest',type=pathlib.Path);a=ap.parse_args();check_manifest()
 caps=[]
 for f in sorted(P.glob('SOURCE-CAPTURES*.json')):caps.extend(json.loads(f.read_text()))
 assert len(caps)==6 and len({x['path'] for x in caps})==6;bypath={x['path']:x for x in caps}
 for x in caps:
  b=(P/x['capture']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and len(b.splitlines())==x['eof']
 regs={f.name:json.loads(f.read_text()) for f in P.glob('SOURCE*REGISTRATION.json')};receipt=j('SOURCE-READ-RECEIPT.json');assert len(receipt['displays'])==7
 spans={x['path']:[] for x in caps}
 for x in receipt['displays']:
  lo,hi=x['range'];r=regs[x['registration']];c=bypath[x['path']];assert r['revision']==x['revision']==REV and x['registered_before_display']
  assert any(z['path']==x['path'] and z['range'][0]<=lo<=hi<=z['range'][1] for z in r['sources'])
  lines=(P/c['capture']).read_text().splitlines();assert 1<=lo<=hi<=len(lines);b=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1)).encode()
  assert (P/x['display']).read_bytes()==b and sha(b)==x['sha256'];spans[x['path']].append((lo,hi))
 for x in receipt['coverage']:
  merged=[]
  for lo,hi in sorted(spans[x['path']]):
   if merged and lo<=merged[-1][1]+1:merged[-1][1]=max(merged[-1][1],hi)
   else:merged.append([lo,hi])
  assert x['personally_displayed_union']==merged and x['personal_displayed_lines']==sum(b-a+1 for a,b in merged) and x['unread_lines']==x['eof']-x['personal_displayed_lines']
 assert sum(x['personal_displayed_lines'] for x in receipt['coverage'])==820 and sum(x['unread_lines'] for x in receipt['coverage'])==4436
 ids=j('HISTORY-IDENTITIES.json');assert len(ids)==2
 for x in ids:
  b=(P/x['capture']).read_bytes();assert len(b)==x['entry_bytes'] and sha(b)==x['entry_sha256']
 original=(P/'Q2359-ORIGINAL-EXACT.json').read_bytes();assert original==(P/'Q2359-RECONSTRUCTED-EXACT.json').read_bytes();q=json.loads(original)
 assert q['contract_id']=='L2-EXT-GPU-READINESS-NATIVE-IDENTITIES' and q['status']=='scoped-field-classification-with-current-alias-qualification' and q['implementation_gate_complete'] is False
 g=q['granular_contract_views'][0]['contract_update'];assert g['audit_discovery_closed_within_named_scope'] is True and g['existing_canonical_identity'] is None and g['new_accepted_unique_count']==0
 rows=q['exact_source_identities_and_assessed_ranges'];assert len(rows)==16 and rows[0]['fresh_personal_ranges']==[[216,240]] and rows[0]['inherited_same_lens_qualification']['assessed_ranges']==[[1,1095]]
 assert rows[6]['source_sha256']==caps[[x['capture'] for x in caps].index('SOURCE-000.dat')]['sha256']
 ad=q['current_own_lens_source_adoptions'][0];assert ad['global_or_native_chain_closed'] is False and ad['body_or_population_credit']==0
 related=j('RELATED-IDENTITIES.json');assert len(related)==4
 for x in related:assert sha((P/x['capture']).read_bytes())==x['sha256']
 for n in (1214,2303):assert (P/f'RELATED-Q{n}-ORIGINAL.json').read_bytes()==(P/f'RELATED-Q{n}-RECONSTRUCTED.json').read_bytes()
 assert 'status' not in j('RELATED-Q2303-ORIGINAL.json') and j('RELATED-Q1214-ORIGINAL.json')['status']=='open-evidence-or-disposition'
 for i in range(3):assert j(f'LEDGER-{i:02}-ORIGINAL.json')==j(f'LEDGER-{i:02}-RECONSTRUCTED.json') and j('LEDGER-COMPARISON.json')[i]['structural_original_reconstruction_equal']
 direct=set();refs(q,direct);registered={(x['path'],x['expected_sha256']) for x in j('RECOVERY-REGISTRATION.json')['references']};assert direct==registered and len(direct)==9
 recovered=[]
 for regn,resn in [('RECOVERY-REGISTRATION.json','RECOVERY-RESULT.json'),('NESTED-REGISTRATION.json','NESTED-RESULT.json')]:
  reg=j(regn)['references'];res=j(resn);assert len(reg)==len(res)
  for x in res:
   assert any(z['path']==x['reference'] and z.get('expected_sha256',z.get('sha256'))==x['expected_sha256'] for z in reg)
   b=(P/x['capture']).read_bytes();assert x['matched'] and len(b)==x['bytes'] and sha(b)==x['expected_sha256'];recovered.append(x)
 assert len(recovered)==11 and j('TRANSPORT-RESULT.json')['matched']==j('TRANSPORT-RESULT.json')['total']==11
 print('PASS packet manifest, six source identities, seven registered displays/coverage, selected historical scopes and 11 recovered records.')
 if a.git_repo:
  env=dict(os.environ,GIT_NO_LAZY_FETCH='1')
  def git(*args):return subprocess.check_output(['git','-C',str(a.git_repo),'--no-replace-objects',*args],env=env)
  for x in caps:
   b=git('show',f'{REV}:{x["path"]}');assert b==(P/x['capture']).read_bytes()
   if 'e275_same_bytes' in x:assert (git('show',f'{OLD}:{x["path"]}')==b)==x['e275_same_bytes']
   if 'versions' in x:
    for rev,v in x['versions'].items():
     b=git('show',f'{rev}:{x["path"]}');assert sha(b)==v['sha256'] and blob(b)==v['blob'] and len(b)==v['bytes']
  streams={}
  for x in ids:
   key=(x['revision'],x['path']);b=git('show',f'{key[0]}:{key[1]}');streams[key]=b;assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and blob(b)==x['git_blob'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==(P/x['capture']).read_bytes()
  for x in related:
   b=streams[(x['revision'],'baseline-059/queue.json')];assert b[x['start']:x['end']]==(P/x['capture']).read_bytes()
  for x in j('LEDGER-IDENTITIES.json'):
   if not x['present']:continue
   key=(x['revision'],'baseline-059/ledger.json')
   if key not in streams:streams[key]=git('show',f'{key[0]}:{key[1]}')
   b=streams[key];assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'];assert json.loads(b)['discovery_findings_and_dependencies'][x['key']]==j(x['capture'])
  print('PASS Git: pinned source, frozen/e275/95e primary source identity, six queue slices and six reserialized ledger selections.')
 else:print('NOT REPEATED: source/stream membership (supply --git-repo).')
 if a.transport_manifest:
  d=json.load(gzip.open(a.transport_manifest,'rt'));assert d['transport_only'] and not d['source_read_credit']
  for x in recovered:
   suffix=x['reference'].removeprefix('target/726-audit-adler-20261027/');assert any(e['path'].endswith(suffix) and e['sha256']==x['expected_sha256'] and e['bytes']==x['bytes'] for e in d['entries'])
  print('PASS 11 supplied transport-metadata matches; not independent custody or source read credit.')
 else:print('NOT REPEATED: transport metadata (supply --transport-manifest).')
 print('LIMITS: integrity and timestamps do not prove comprehension, readiness behavior, supported producer coverage, policy, global alias disposition or independent custody.')
if __name__=='__main__':main()
