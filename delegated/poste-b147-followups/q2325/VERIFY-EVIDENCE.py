#!/usr/bin/env python3
"""Evidence-only q2325 integrity, provenance, and display checks; no product execution."""
import argparse,datetime as dt,gzip,hashlib,json,os,pathlib,subprocess
P=pathlib.Path(__file__).resolve().parent

def load(n): return json.loads((P/n).read_text())
def sha(b): return hashlib.sha256(b).hexdigest()
def blob(b): return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def time(s): return dt.datetime.fromisoformat(s)
def safe(s): return not pathlib.Path(s).is_absolute() and '..' not in pathlib.Path(s).parts
def git(repo,*args): return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args],env=dict(os.environ,GIT_NO_LAZY_FETCH='1'))
def pointer(o,p):
 for k in p.split('/')[1:]:
  k=k.replace('~1','/').replace('~0','~');o=o[int(k)] if isinstance(o,list) else o[k]
 return o
def spans(ns):
 out=[]
 for i in sorted(ns):
  if not out or out[-1][1]+1!=i:out.append([i,i])
  else:out[-1][1]=i
 return out

def main():
 a=argparse.ArgumentParser();a.add_argument('--git-repo',type=pathlib.Path);a.add_argument('--publication',type=pathlib.Path);a=a.parse_args()
 m=load('MANIFEST.json');fs={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'};assert fs==set(m['files']),(len(fs),len(m['files']))
 for n,x in m['files'].items():
  assert safe(n) and not (P/n).is_symlink();b=(P/n).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'],n
 rows=load('SOURCE-CAPTURES.json')+load('SOURCE-SUPPORT-CAPTURES.json')+[load('SOURCE-FITTING-CAPTURE.json')];captures={x['path']:x for x in rows};assert len(rows)==len(captures)==13
 for x in rows:
  b=(P/x['capture']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and len(b.splitlines())==x['eof']
 rr=load('SOURCE-READ-RECEIPT.json');seen={p:set() for p in captures}
 for x in rr['chunks']:
  r=load(x['registration']);assert time(r['registered_utc'])<=time(x['saved_utc']);assert any(v['path']==x['source_path'] and v['range'][0]<=x['start']<=x['end']<=v['range'][1] for v in r['sources']);lines=(P/captures[x['source_path']]['capture']).read_text().splitlines();b=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(x['start'],x['end']+1)).encode();assert b==(P/x['display']).read_bytes() and sha(b)==x['sha256'];seen[x['source_path']].update(range(x['start'],x['end']+1))
 assert len(rr['coverage'])==len(captures)
 for x in rr['coverage']:
  s=x['source_path'];assert x['eof']==captures[s]['eof'] and x['personally_displayed']==spans(seen[s]) and x['unread']==spans(set(range(1,x['eof']+1))-seen[s])
 assert sum(map(len,seen.values()))==rr['unique_lines_personally_displayed']
 for x in load('OTHER-READ-RECEIPT.json')['chronology']:assert time(load(x['registration'])['registered_utc'])<=time(x['saved_utc']) and (P/x['selection']).is_file()
 h=load('HISTORY-IDENTITIES.json');parts=[(P/x['packet_path']).read_bytes() for x in h];assert len(parts)==2 and parts[0]==parts[1]
 for x,b in zip(h,parts):assert sha(b)==x['entry_sha256'] and len(b)==x['entry_bytes']
 q=json.loads(parts[0]);assert all(q[k]==v for k,v in load('Q2325-DISPLAY-SUMMARY.json').items());assert load('Q2325-SOURCES-0-3.json')+load('Q2325-SOURCES-4-7.json')==q['exact_source_identities_and_assessed_ranges']
 refs={}
 def collect(v):
  if isinstance(v,dict):
   for p,d in [(v.get('path'),v.get('sha256')),(v.get('verification'),v.get('verification_sha256'))]:
    if isinstance(p,str) and isinstance(d,str):
     if p in refs:assert refs[p]==d
     refs[p]=d
   for x in v.values():collect(x)
  elif isinstance(v,list):
   for x in v:collect(x)
 collect(q);reg=load('RECOVERY-REGISTRATION.json');res={x['reference']:x for x in load('RECOVERY-RESULT.json')};assert len(refs)==len(reg['references'])==len(res)==13
 for x in reg['references']:
  v=res[x['path']];b=(P/v['packet_path']).read_bytes();assert v['matched'] and refs[x['path']]==x['expected_sha256']==v['actual_sha256']==sha(b) and len(b)==v['bytes']
 readers=[]
 for e in q['exact_source_identities_and_assessed_ranges']:
  rp=e['body_receipt']['path'].removeprefix('target/726-audit-adler-20261027/');pp=e['registration_ref_and_sha256']['path'].removeprefix('target/726-audit-adler-20261027/');d=load('historical-originals/'+rp);plan=load('historical-originals/'+pp);read=pointer(d,e['receipt_pointer']);pr=pointer(plan,e['registration_ref_and_sha256']['json_pointer']);root=load('historical-originals/'+e['root_adoption']['path']);mapping=[x for x in root['adopted_registration_mapping'] if x['registration']==read['registration']];assert len(mapping)==1;readers.append({'source_row_key':e['source_row_key'],'delegated_receipt':read,'plan':pr,'root_adoption_mapping':mapping[0]})
 assert readers==load('HISTORICAL-READERS-0-3.json')+load('HISTORICAL-READERS-4-7.json')
 for x in load('HISTORICAL-ROOT-VERIFICATION.json'):
  root=load('historical-originals/segovb/'+x['record']);v=load('historical-originals/segovb/'+x['verification_record']);assert all(root[k]==z for k,z in x['root_fields'].items()) and v==x['verification']
 u=load('historical-originals/parallel-l11-20261029/PROPOSAL-BUNDLE-L11-SOURCE-005.json')['contract_updates'][2];assert all(u[k]==v for k,v in load('HISTORICAL-PROPOSAL-SELECT.json').items())
 for x in load('RELATED-HISTORY-IDENTITIES.json'):
  b=(P/x['packet_path']).read_bytes();assert sha(b)==x['entry_sha256'] and len(b)==x['entry_end_byte_exclusive']-x['entry_start_byte'];j=json.loads(b);n=x['selector'].split('/')[-1];assert all(j[k]==v for k,v in load('Q'+n+'-HISTORICAL-SELECT.json').items())
 rels=[('RELATED-EQUIPMENT-SELECT.json','related/ASSESSMENT-EQUIPMENT-MESH-RIG-95E.json',{'observation_0':'/observations/0','conditional_finding_0':'/conditional_findings/0'}),('RELATED-RECON-SELECT.json','related/ASSESSMENT-DELEGATED-RECONCILIATION-95E.json',{'delegated_observation_20':'/delegated_observations/20'}),('RELATED-RECON2-SELECT.json','related/ASSESSMENT-DELEGATED-RECONCILIATION-2-95E.json',{'delegated_observation_20':'/delegated_observations/20'}),('RELATED-RIG-SELECT.json','related/delegated/rig-skin-report.json',{'revision':'/current_authoritative_revision','finding_0':'/findings_and_open_questions/0'})]
 for s,source,aliases in rels:
  r=load(source)
  for k,v in load(s).items():assert pointer(r,aliases.get(k,'/'+k))==v,(s,k)
 for x in load('RELATED-IDENTITIES.json')+[load('RELATED-RIG-IDENTITY.json')]:
  b=(P/x['packet_path']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes']
 print('PASS packet exact files and hashes; 13 source captures, registered displays and unread complements; q2325 13 cited originals, separately attributed rows and selected proposal; four related original entries and separately attributed 95e selections.')
 if a.git_repo:
  rev=rr['source_revision']
  for x in rows:assert git(a.git_repo,'show',rev+':'+x['path'])==(P/x['capture']).read_bytes()
  for x in load('SOURCE-IDENTITIES.json'):
   vals=[]
   for rev,v in x['revisions'].items():
    b=git(a.git_repo,'show',rev+':'+x['path']);assert sha(b)==v['sha256'] and blob(b)==v['blob'] and len(b)==v['bytes'];vals.append(b)
   assert x['old_matches_historical_record'] and x['old_current_sha_equal'] and len(vals)==2 and vals[0]==vals[1]
  streams={}
  for x,part in zip(h,parts):
   b=git(a.git_repo,'show',x['revision']+':'+x['path']);streams[x['role']]=b;assert sha(b)==x['stream_sha256'] and blob(b)==x['git_blob'] and len(b)==x['stream_bytes'] and b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==part and json.loads(b)['entries'][2325]==q
  assert streams['original']!=streams['public_reconstruction']
  for x in load('RELATED-HISTORY-IDENTITIES.json'):
   b=streams['original'];part=(P/x['packet_path']).read_bytes();assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==part and pointer(json.loads(b),x['selector'])==json.loads(part)
  ar=load('RELATED-REGISTRATION.json')
  for x in load('RELATED-IDENTITIES.json'):
   b=git(a.git_repo,'show',ar['revision']+':'+x['path']);assert b==(P/x['packet_path']).read_bytes()
  ar=load('RELATED-RIG-REGISTRATION.json');x=load('RELATED-RIG-IDENTITY.json');assert git(a.git_repo,'show',ar['revision']+':'+ar['path'])==(P/x['packet_path']).read_bytes()
  def locate(rev,scopes,pat):
   c=['git','--no-replace-objects','-C',str(a.git_repo),'grep','-n','-i','-F',pat,rev,'--',*scopes];z=subprocess.run(c,capture_output=True,text=True,env=dict(os.environ,GIT_NO_LAZY_FETCH='1'));out=[]
   for line in z.stdout.splitlines():
    t=line.split(':',3)
    if len(t)==4:
     try:out.append({'path':t[1],'line':int(t[2])})
     except ValueError:pass
   return z.returncode,out
  for tag in ['','REDUCTION-']:
   r=load('LOCATOR-'+tag+'REGISTRATION.json');results=load('LOCATOR-'+tag+'RESULT.json')
   for k,v in results.items():
    group,pat=k.split(':',1);status,found=locate(r[group+'_revision'],r[group+'_scopes'],pat);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted'],(tag,k)
  for tag in ['CLOTHING','FITTING','HELPER','HELPER2']:
   r=load('LOCATOR-'+tag+'-REGISTRATION.json');results=load('LOCATOR-'+tag+'-RESULT.json')
   for k,v in results.items():
    status,found=locate(r['revision'],r['scopes'],k);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted'],(tag,k)
  print('PASS fixed 95e source memberships and eight e275 identities; both whole 059 streams and five exact entries; current related records and bounded locators.')
 else:print('NOT REPEATED Git memberships, whole 059 streams, current related records or locators: supply --git-repo.')
 if a.publication:
  t=load('TRANSPORT-IDENTITY.json');b=(a.publication/t['path']).read_bytes();assert sha(b)==t['sha256'] and len(b)==t['bytes'];nodes=[]
  def walk(v):
   if isinstance(v,dict):
    if 'sha256' in v:nodes.append(v)
    for x in v.values():walk(x)
   elif isinstance(v,list):
    for x in v:walk(x)
  walk(json.loads(gzip.decompress(b)))
  for x in load('TRANSPORT-MATCHES.json'):
   v=res[x['reference']];rel=x['reference'].removeprefix('target/726-audit-adler-20261027/');ms=[n for n in nodes if n.get('sha256')==v['actual_sha256'] and n.get('bytes')==v['bytes']];assert len(ms)==x['digest_size_matches'] and sum(str(n.get('path','')).endswith(rel) for n in ms)==x['path_suffix_matches'] and x['path_suffix_matches']>0
  print('PASS transport metadata digest, size and suffix for 13 originals; independent custody is not established.')
 else:print('NOT REPEATED transport inventory comparison: supply --publication.')
 print('LIMITS: timestamps attest process order; neither hashes nor scans prove semantics, comprehension, supported reach, rendered output, product behavior or independent custody.')
if __name__=='__main__':main()
