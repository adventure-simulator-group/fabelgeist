#!/usr/bin/env python3
"""Verify source-only q2334/35 packet; never import or execute product code."""
import argparse,datetime,gzip,hashlib,json,os,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent
REV='95e63628f9d70b8dd6e6fede64334696c6995009'
def j(name):return json.loads((P/name).read_text())
def sha(data):return hashlib.sha256(data).hexdigest()
def blob(data):return hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
def check_manifest():
 m=j('MANIFEST.json');actual={x.name for x in P.iterdir() if x.is_file() and x.name!='MANIFEST.json'}
 assert all(not x.is_symlink() for x in P.iterdir()) and actual==set(m['files'])
 for name,info in m['files'].items():
  assert pathlib.Path(name).name==name
  data=(P/name).read_bytes();assert len(data)==info['bytes'] and sha(data)==info['sha256'],name
 return m

def refs(v,out):
 if isinstance(v,dict):
  for p,h in [('path','sha256'),('verification','verification_sha256'),('receipt','receipt_sha256'),('registration','registration_sha256')]:
   if isinstance(v.get(p),str) and isinstance(v.get(h),str) and len(v[h])==64:out.add((v[p],v[h]))
  for x in v.values():refs(x,out)
 elif isinstance(v,list):
  for x in v:refs(x,out)

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=pathlib.Path);ap.add_argument('--transport-manifest',type=pathlib.Path);a=ap.parse_args();check_manifest()
 caps=j('SOURCE-CAPTURES.json')+j('SOURCE-SUPPORT-CAPTURES.json');assert len(caps)==11 and len({c['path'] for c in caps})==11
 bypath={c['path']:c for c in caps}
 for c in caps:
  b=(P/c['capture']).read_bytes();assert sha(b)==c['sha256'] and blob(b)==c.get('blob',c.get('git_blob')) and len(b)==c['bytes'] and len(b.splitlines())==c.get('eof',c.get('lines'))
 regs={n:j(n) for n in ['SOURCE-REGISTRATION.json','SOURCE-SUPPORT-REGISTRATION.json']};receipt=j('SOURCE-READ-RECEIPT.json');assert len(receipt['displays'])==26
 spans={x['path']:[] for x in caps}
 for r in receipt['displays']:
  lo,hi=r['range'];reg=regs[r['registration']];assert reg['revision']==r['revision']==REV and r['registered_before_display'];assert any(x['path']==r['path'] and x['range'][0]<=lo<=hi<=x['range'][1] for x in reg['sources'])
  c=bypath[r['path']];lines=(P/c['capture']).read_text().splitlines();assert 1<=lo<=hi<=len(lines);out=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1)).encode();assert (P/r['display']).read_bytes()==out and sha(out)==r['sha256'];spans[r['path']].append((lo,hi))
 for row in receipt['coverage']:
  merged=[]
  for lo,hi in sorted(spans[row['path']]):
   if merged and lo<=merged[-1][1]+1:merged[-1][1]=max(merged[-1][1],hi)
   else:merged.append([lo,hi])
  assert row['personally_displayed_union']==merged and row['personal_displayed_lines']==sum(b-a+1 for a,b in merged) and row['unread_lines']==row['eof']-row['personal_displayed_lines']
 ids=j('HISTORY-IDENTITIES.json');assert len(ids)==4
 for x in ids:
  data=(P/x['packet_path']).read_bytes();assert len(data)==x['entry_bytes'] and sha(data)==x['entry_sha256']
 for n in (2334,2335):
  o=(P/f'Q{n}-ORIGINAL-EXACT-SLICE.json').read_bytes();recon=(P/f'Q{n}-PUBLIC_RECONSTRUCTION-EXACT-SLICE.json').read_bytes();assert o==recon
  d=json.loads(o);f=j(f'Q{n}-FLAGS.json');assert d['status']==f['status']=='open-connected-source-current-alias-qualification' and d['audit_discovery_closed'] is f['audit_discovery_closed'] is False and d['repair_or_runtime_passed'] is f['repair_or_runtime_passed'] is False
  assert d['body_or_population_credit']==0
 for n in (2299,2321,2392):
  o=(P/f'RELATED-Q{n}-ORIGINAL.json').read_bytes();r=(P/f'RELATED-Q{n}-PUBLIC_RECONSTRUCTION.json').read_bytes();assert o==r;d=json.loads(o);summary=j(f'RELATED-Q{n}-SUMMARY.json');assert all(summary[k]==d.get(k) for k in summary)
 registered={(x['path'],x['expected_sha256']) for x in j('RECOVERY-REGISTRATION.json')['references']};actual=set()
 for n in (2334,2335):refs(j(f'Q{n}-ORIGINAL-EXACT-SLICE.json'),actual)
 assert actual==registered and len(actual)==36,(len(actual),len(registered))
 rec=j('RECOVERY-RESULT.json');assert {(x['reference'],x['expected_sha256']) for x in rec}==registered
 for x in rec:
  b=(P/x['packet_path']).read_bytes();assert x['matched'] and len(b)==x['bytes'] and sha(b)==x['actual_sha256']==x['expected_sha256']
 historical=j('HISTORICAL-027.dat');registration=j('HISTORICAL-035.dat');root=j('HISTORICAL-011.dat');focus=j('ATTRIBUTION-FOCUS-PRIMARY.json');assert historical['rows']==focus['delegated']['rows']==root['evidence']['rows'];assert registration['sources']==focus['registration']['sources'];assert root['delegated_receipt']==focus['root']['delegated_receipt'];assert historical['rows'][0]['source_revision'].endswith('6533e444fa37552927070cc8d8190907fd3b413e')
 assert len(j('RELATED-RECORDS-IDENTITY.json'))==6 and j('TRANSPORT-RESULT.json')['matched']==36 and j('TRANSPORT-RESULT.json')['total']==36
 print('PASS flat manifest, 11 source identities, 26 registered displays/coverage, original/reconstructed queue slices, 36 hash-cited records and focused historic attribution.')
 if a.git_repo:
  env=dict(os.environ,GIT_NO_LAZY_FETCH='1')
  def git(*args):return subprocess.check_output(['git','-C',str(a.git_repo),'--no-replace-objects',*args],env=env)
  for c in caps:
   b=git('show',f'{REV}:{c["path"]}');assert b==(P/c['capture']).read_bytes()
  for x in j('SOURCE-IDENTITIES.json'):
   for rev,rec2 in x['revisions'].items():
    b=git('show',f'{rev}:{x["path"]}');assert sha(b)==rec2['sha256'] and blob(b)==rec2['blob'] and len(b)==rec2['bytes']
   assert x['old_matches_historical_record'] and x['e275_current_equal']==(x['revisions']['e2755a732c84e67296b5e3441a4259a84f461b2f']['sha256']==x['revisions'][REV]['sha256'])
  streams={}
  for x in ids:
   key=(x['revision'],x['path'])
   if key not in streams:streams[key]=git('show',f'{key[0]}:{key[1]}')
   b=streams[key];assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and blob(b)==x['git_blob'];assert b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==(P/x['packet_path']).read_bytes()
  for x in j('RELATED-RECORDS-IDENTITY.json'):
   b=streams[(x['revision'],'baseline-059/queue.json')];assert b[x['byte_start']:x['byte_end']]==(P/x['packet_path']).read_bytes() and sha((P/x['packet_path']).read_bytes())==x['sha256']
  print('PASS supplied Git object store: 95e source captures, 6533/e275/95e identities and both 059 streams and ten exact slices.')
 else:print('NOT REPEATED: Git source and queue membership (supply --git-repo).')
 if a.transport_manifest:
  data=json.load(gzip.open(a.transport_manifest,'rt'));assert data['transport_only'] and not data['source_read_credit'];entries=data['entries']
  for x in rec:
   suffix=x['reference'].removeprefix('target/726-audit-adler-20261027/');assert any(e['path'].endswith(suffix) and e['sha256']==x['expected_sha256'] and e['bytes']==x['bytes'] for e in entries)
  print('PASS supplied transport metadata: 36/36 records match. This does not establish independent custody or reading.')
 else:print('NOT REPEATED: transport metadata correspondence (supply --transport-manifest).')
 print('LIMITS: integrity checks and timestamps do not prove comprehension, full producer admission, reachability, global alias disposition, policy, product behavior or independent custody.')
if __name__=='__main__':main()
