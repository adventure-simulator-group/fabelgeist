#!/usr/bin/env python3
"""Source/evidence-only q2341/42 verification; no product execution."""
import argparse,gzip,hashlib,json,os,pathlib,subprocess
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
P=pathlib.Path(__file__).resolve().parent
REV='95e63628f9d70b8dd6e6fede64334696c6995009';EARLIER='e2755a732c84e67296b5e3441a4259a84f461b2f'
def j(name):return json.loads((P/name).read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def refs(v,out):
 if isinstance(v,dict):
  for p,h in [('path','sha256'),('verification','verification_sha256'),('receipt','receipt_sha256'),('registration','registration_sha256')]:
   if isinstance(v.get(p),str) and isinstance(v.get(h),str) and len(v[h])==64:out.add((v[p],v[h]))
  for x in v.values():refs(x,out)
 elif isinstance(v,list):
  for x in v:refs(x,out)
def check_manifest():
 m=j('MANIFEST.json');files={x.name for x in P.iterdir() if x.is_file() and x.name!='MANIFEST.json'};assert files==set(m['files']) and all(not x.is_symlink() for x in P.iterdir())
 for name,x in m['files'].items():
  assert pathlib.Path(name).name==name;b=(P/name).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'],name

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=pathlib.Path);ap.add_argument('--transport-manifest',type=pathlib.Path);ap.add_argument('--private-key',type=pathlib.Path);ap.add_argument('--digest-scope',type=pathlib.Path,action='append',default=[]);a=ap.parse_args();check_manifest()
 encrypted=j('ENCRYPTED-ORIGINALS.json')['originals'];assert len(encrypted)==3;enc={x['file']:x for x in encrypted};key=a.private_key.read_bytes() if a.private_key else None;assert key is None or len(key)==32;originals={};removed=[]
 def compare(old,new,file):
  if new=='<redacted-local-path>':assert isinstance(old,str) and (old.lower().startswith('c:\\users\\') or old.lower().startswith('c:/users/'));removed.append(old);return
  if new=='<redacted-local-path-context>':assert file=='PRIVACY-CLASSIFICATION.json' and isinstance(old,str);return
  if isinstance(old,dict) and isinstance(new,dict):
   assert old.keys()==new.keys()
   for k in old:compare(old[k],new[k],file)
  elif isinstance(old,list) and isinstance(new,list):
   assert len(old)==len(new)
   for x,y in zip(old,new):compare(x,y,file)
  else:assert old==new
 for name,x in enc.items():
  b=(P/name).read_bytes();c=(P/x['ciphertext']).read_bytes();assert len(b)==x['public_bytes'] and sha(b)==x['public_sha256'] and len(c)==x['ciphertext_bytes'] and sha(c)==x['ciphertext_sha256']
  if key:
   raw=AESGCM(key).decrypt(c[:12],c[12:],name.encode());assert len(raw)==x['original_bytes'] and sha(raw)==x['original_sha256'];compare(json.loads(raw),json.loads(b),name);originals[name]=raw
 if key:assert len(removed)==2
 caps=[]
 for n in ('SOURCE-CAPTURES.json','SOURCE-CAPTURES-SUPPORT.json','SOURCE-CAPTURES-EXT.json','SOURCE-CAPTURES-CLOCK.json'):caps+=j(n)
 assert len(caps)==18;by={x['path']:x for x in caps};assert len(by)==18
 for c in caps:
  b=(P/c['capture']).read_bytes();assert sha(b)==c['sha256'] and blob(b)==c['blob'] and len(b)==c['bytes'] and len(b.splitlines())==c['eof']
 regs={n:j(n) for n in ('SOURCE-REGISTRATION.json','SOURCE-SUPPORT-REGISTRATION.json','SOURCE-EXT-REGISTRATION.json','SOURCE-CLOCK-REGISTRATION.json')};receipt=j('SOURCE-READ-RECEIPT-V2.json');assert len(receipt['displays'])==37;spans={p:[] for p in by}
 for row in receipt['displays']:
  lo,hi=row['range'];reg=regs[row['registration']];assert row['revision']==reg['revision']==REV and row['registered_before_display'];assert any(x['path']==row['path'] and x['range'][0]<=lo<=hi<=x['range'][1] for x in reg['sources']);lines=(P/by[row['path']]['capture']).read_text().splitlines();assert 1<=lo<=hi<=len(lines);b=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1)).encode();assert (P/row['display']).read_bytes()==b and sha(b)==row['sha256'];spans[row['path']].append((lo,hi))
 for row in receipt['coverage']:
  merged=[]
  for lo,hi in sorted(spans[row['path']]):
   if merged and lo<=merged[-1][1]+1:merged[-1][1]=max(merged[-1][1],hi)
   else:merged.append([lo,hi])
  total=sum(hi-lo+1 for lo,hi in merged);assert merged==row['personally_displayed_union'] and total==row['personal_displayed_lines'] and row['eof']-total==row['unread_lines']
 for n in (2341,2342):
  b=(P/f'Q{n}-ORIGINAL-EXACT.json').read_bytes();assert b==(P/f'Q{n}-PUBLIC_RECONSTRUCTION-EXACT.json').read_bytes();v=json.loads(b);assert v['status']=='open-producer-owner-current-alias-reconciliation' and v['new_body_or_population_credit']==0 and v['implementation_gate_complete'] is False;assert all(k not in v for k in ('audit_discovery_closed','repair_or_runtime_passed','body_or_population_credit'))
  assert len(v['exact_source_identities_and_assessed_ranges'])==7 and all(x['attribution'].startswith('Root personal Lens2') and x['new_credit_from_adoption']==0 for x in v['exact_source_identities_and_assessed_ranges'])
  assert v['exact_source_identities_and_assessed_ranges']==j(f'Q{n}-SOURCES.json')
 refs_expected=set()
 for n in (2341,2342):refs(j(f'Q{n}-ORIGINAL-EXACT.json'),refs_expected)
 recorded={(x['path'],x['expected_sha256']) for x in j('RECOVERY-REGISTRATION.json')['references']};assert len(recorded)==22 and refs_expected==recorded
 recovered=j('RECOVERY-RESULT.json');assert {(x['reference'],x['expected_sha256']) for x in recovered}==recorded
 for x in recovered:
  assert x['matched'];name=x['capture'];b=(P/name).read_bytes()
  if name in enc:
   assert enc[name]['original_sha256']==x['expected_sha256'] and enc[name]['original_bytes']==x['bytes']
   if key:assert sha(originals[name])==x['expected_sha256']
  else:assert sha(b)==x['expected_sha256'] and len(b)==x['bytes']
 related=j('RELATED-RECOVERY-RESULT.json');assert len(related)==3
 for x in related:
  assert x['matched'];b=(P/x['capture']).read_bytes();assert sha(b)==x['expected_sha256'] and len(b)==x['bytes']
 assert j('LEDGER-ROW-RECONSTRUCTED.json')==j('LEDGER-ROW-PUBLIC-RECONSTRUCTION.json') and j('LEDGER-RECON-IDENTITY.json')['selected_structural_equality_to_original']
 old=j('HISTORICAL-008.dat');assert old['attribution']==j('HISTORICAL-FOCUS-008.json')['attribution'] and old['source']['source_sha256']==j('Q2341-SOURCES.json')[0]['source_sha256'];same=j('HISTORICAL-002.dat');assert same['fresh_body_reads']==[] and same['product_execution'] is False
 print('PASS manifest, 18 source captures, 37 registered displays/coverage, q2341/42 exact slices, 22 referenced records and 3 linked-record references.')
 if key:print('PASS authenticated restoration and structural redaction of all 3 exact originals; no key bytes or removed values displayed.')
 else:print('NOT REPEATED: original byte hashes and private-value checks for 3 redacted files; supply --private-key.')
 if a.git_repo:
  env=dict(os.environ,GIT_NO_LAZY_FETCH='1')
  def git(*args):return subprocess.check_output(['git','-C',str(a.git_repo),'--no-replace-objects',*args],env=env)
  for c in caps:
   b=git('show',f'{REV}:{c["path"]}');assert b==(P/c['capture']).read_bytes() and git('show',f'{EARLIER}:{c["path"]}')==b
  for x in j('SOURCE-IDENTITIES.json'):
   for rev,d in x['versions'].items():
    b=git('show',f'{rev}:{x["path"]}');assert sha(b)==d['sha256'] and blob(b)==d['blob'] and len(b)==d['bytes']
   assert x['frozen_matches'] and x['e275_95e_equal']
  streams={}
  for x in j('HISTORY-IDENTITIES.json'):
   k=(x['revision'],x['path'])
   if k not in streams:streams[k]=git('show',f'{k[0]}:{k[1]}')
   b=streams[k];assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and blob(b)==x['git_blob'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==(P/x['capture']).read_bytes() and sha((P/x['capture']).read_bytes())==x['entry_sha256']
  orig='56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e';raw=streams[(orig,'baseline-059/queue.json')];sel=j('RELATED-SELECTION-IDENTITIES.json')
  for x in sel['queue_selections']:assert raw[x['start']:x['end']]==(P/x['capture']).read_bytes() and sha((P/x['capture']).read_bytes())==x['sha256']
  rawled=git('show',f'{orig}:baseline-059/ledger.json');assert sha(rawled)==sel['ledger_original_stream_sha256'];k=sel['ledger_selected_key'];assert json.loads(rawled)['discovery_findings_and_dependencies'][k]==j('LEDGER-ROW-RECONSTRUCTED.json')
  recon='26e360920b7b2ec32d5c76238623eaf6d6617d7e';rawled2=git('show',f'{recon}:baseline-059/ledger.json');assert sha(rawled2)==j('LEDGER-RECON-IDENTITY.json')['stream_sha256'] and json.loads(rawled2)['discovery_findings_and_dependencies'][k]==j('LEDGER-ROW-PUBLIC-RECONSTRUCTION.json')
  print('PASS supplied Git store: 18 e275/95e file comparisons, 6 frozen identities, two queue streams/slices and original/reconstructed selected ledger.')
 else:print('NOT REPEATED: Git source, queue and ledger membership; supply --git-repo.')
 if a.transport_manifest:
  transport=json.load(gzip.open(a.transport_manifest,'rt'));assert transport['transport_only'] and not transport['source_read_credit'];entries=transport['entries']
  for x in recovered+related:
   suffix=x['reference'].removeprefix('target/726-audit-adler-20261027/').removeprefix('../../');assert any(e['path'].endswith(suffix) and e['sha256']==x['expected_sha256'] and e['bytes']==x['bytes'] for e in entries)
  print('PASS supplied same-machine transport metadata for 22 direct and 3 linked references; not independent custody.')
 else:print('NOT REPEATED: transport metadata correspondence; supply --transport-manifest.')
 if key:
  vals=set(removed);candidates=set(vals)
  for v in vals:
   sep='\\' if '\\' in v else '/';parts=v.split(sep)
   for n in range(3,len(parts)):candidates.add(sep.join(parts[:n]))
  scopes=[P]+a.digest_scope;counts=[]
  for scope in scopes:
   files=[x for x in scope.iterdir() if x.is_file() and not x.is_symlink()];hits=[]
   for f in files:
    b=f.read_bytes()
    for v in candidates:
     for alg in ('sha256','sha1','md5'):
      d=hashlib.new(alg,v.encode()).digest()
      for representation,needle in (('raw',d),('lowerhex',d.hex().encode()),('upperhex',d.hex().upper().encode())):
       if needle in b:hits.append((f.name,alg,representation))
   assert not hits,('path-value digest match in bounded scope',len(hits));counts.append(len(files))
  print('PASS private-input removed-value and directory-prefix digest search for packet and',len(a.digest_scope),'additional receipt scopes; files per scope',counts,'occurrences',len(removed),'distinct values',len(vals),'candidates',len(candidates),'.')
 else:print('NOT REPEATED: individual removed-path digest search; supply --private-key and optional --digest-scope.')
 print('LIMITS: hashes/scans do not establish semantic comprehension, full owner search, supported reach, product behavior, global alias clearance, or independent key custody.')
if __name__=='__main__':main()
