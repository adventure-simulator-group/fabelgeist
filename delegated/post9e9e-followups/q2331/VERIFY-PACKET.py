#!/usr/bin/env python3
"""Privacy-qualified q2331 flat packet verifier; never execute product or recovered code."""
import argparse,datetime as dt,gzip,hashlib,json,os,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parent
INDEX=json.loads((ROOT/'INDEX.json').read_text())['files']
SEALED={x['logical_path']:x for x in json.loads((ROOT/'ENCRYPTED-ORIGINALS.json').read_text())['originals']}
_KEY=None
_CACHE={}
class Virtual:
 def __init__(self,name=''): self.name=name
 def __truediv__(self,other): return Virtual((self.name+'/' if self.name else '')+str(other))
 def __eq__(self,other): return isinstance(other,Virtual) and self.name==other.name
 def __str__(self): return self.name
 def relative_to(self,other): assert not other.name; return self.name
 def is_symlink(self): return False
 def is_file(self): return self.name=='MANIFEST.json' or self.name in INDEX
 def rglob(self,pattern):
  assert not self.name and pattern=='*'
  return [Virtual(x) for x in INDEX]+[Virtual('MANIFEST.json')]
 def read_bytes(self):
  if self.name=='MANIFEST.json':return (ROOT/'PREDECESSOR-MANIFEST.json').read_bytes()
  x=INDEX[self.name]
  if self.name in SEALED and _KEY is not None:
   if self.name not in _CACHE:
    from cryptography.hazmat.primitives.ciphers.aead import AESGCM
    v=SEALED[self.name];enc=(ROOT/v['ciphertext']).read_bytes();assert hashlib.sha256(enc).hexdigest()==v['ciphertext_sha256'];plain=AESGCM(_KEY).decrypt(enc[:12],enc[12:],self.name.encode());assert hashlib.sha256(plain).hexdigest()==v['original_sha256'] and len(plain)==v['original_bytes'];_CACHE[self.name]=plain
   return _CACHE[self.name]
  return (ROOT/x['object']).read_bytes()
 def read_text(self):return self.read_bytes().decode()
P=Virtual()
def check_original(name,digest,size=None):
 x=INDEX[name];assert x['original_sha256']==digest and (size is None or x['original_bytes']==size);b=(P/name).read_bytes()
 if name in SEALED and _KEY is None:
  assert hashlib.sha256(b).hexdigest()==x['public_sha256'] and len(b)==x['public_bytes'];return
 assert hashlib.sha256(b).hexdigest()==digest and (size is None or len(b)==size)
def check_safe_manifest():
 m=json.loads((ROOT/'MANIFEST.json').read_text());actual={str(f.relative_to(ROOT)) for f in ROOT.rglob('*') if f.is_file() and f!=ROOT/'MANIFEST.json'};assert actual==set(m['files'])
 for n,x in m['files'].items():
  f=pathlib.Path(n);assert not f.is_absolute() and '..' not in f.parts and not (ROOT/f).is_symlink();b=(ROOT/f).read_bytes();assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256'],n
 prev=json.loads((ROOT/'PREDECESSOR-MANIFEST.json').read_text());assert set(INDEX)==set(prev['files']) and len(SEALED)==3
 for n,x in INDEX.items():
  assert prev['files'][n]['sha256']==x['original_sha256'] and prev['files'][n]['bytes']==x['original_bytes'];b=(ROOT/x['object']).read_bytes();assert hashlib.sha256(b).hexdigest()==x['public_sha256'] and len(b)==x['public_bytes']
  if n not in SEALED:assert x['transformation']=='byte-identical' and x['public_sha256']==x['original_sha256'] and x['public_bytes']==x['original_bytes']
 for n,v in SEALED.items():
  enc=(ROOT/v['ciphertext']).read_bytes();x=INDEX[n];assert len(enc)==v['ciphertext_bytes'] and hashlib.sha256(enc).hexdigest()==v['ciphertext_sha256'] and v['original_sha256']==x['original_sha256'] and v['redacted_sha256']==x['public_sha256'] and x['transformation']=='local-path redaction'


def load(n):return json.loads((P/n).read_text())
def sha(b):return hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def time(s):return dt.datetime.fromisoformat(s)
def safe(s):return not pathlib.Path(s).is_absolute() and '..' not in pathlib.Path(s).parts
def git(repo,*args):return subprocess.check_output(['git','--no-replace-objects','-C',str(repo),*args],env=dict(os.environ,GIT_NO_LAZY_FETCH='1'))
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

def collect_refs(v,refs):
 if isinstance(v,dict):
  for a,b in [(v.get('path'),v.get('sha256')),(v.get('receipt'),v.get('receipt_sha256')),(v.get('registration'),v.get('registration_sha256')),(v.get('windows_locator_qualification'),v.get('windows_locator_qualification_sha256'))]:
   if isinstance(a,str) and isinstance(b,str):
    if a in refs:assert refs[a]==b,(a,refs[a],b)
    refs[a]=b
  for x in v.values():collect_refs(x,refs)
 elif isinstance(v,list):
  for x in v:collect_refs(x,refs)

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=pathlib.Path);ap.add_argument('--publication',type=pathlib.Path);ap.add_argument('--private-key',type=pathlib.Path);a=ap.parse_args();global _KEY;check_safe_manifest();_KEY=a.private_key.read_bytes() if a.private_key else None;assert _KEY is None or len(_KEY)==32;print('PASS safe flat manifest, redacted/ciphertext crosswalk and public hashes.');print('Original-byte recovery:', 'authenticated private key supplied' if _KEY is not None else 'NOT REPEATED for three redacted originals; supply --private-key.')
 m=load('MANIFEST.json');actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file() and f!=P/'MANIFEST.json'};assert actual==set(m['files'])
 for n,x in m['files'].items():
  assert safe(n) and not (P/n).is_symlink();check_original(n,x['sha256'],x['bytes'])
 rows=load('SOURCE-CAPTURES.json')+[load('SOURCE-SHADER-CAPTURE.json'),load('SOURCE-MESH-CAPTURE.json')];captures={x['path']:x for x in rows};assert len(rows)==len(captures)==14
 for x in rows:
  b=(P/x['capture']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes'] and len(b.splitlines())==x['eof']
 rr=load('SOURCE-READ-RECEIPT.json');seen={s:set() for s in captures}
 for x in rr['chunks']:
  r=load(x['registration']);assert time(r['registered_utc'])<=time(x['saved_utc']);assert any(v['path']==x['source_path'] and v['range'][0]<=x['start']<=x['end']<=v['range'][1] for v in r['sources']);lines=(P/captures[x['source_path']]['capture']).read_text().splitlines();b=''.join(f'{i:4d} {lines[i-1]}\n' for i in range(x['start'],x['end']+1)).encode();assert b==(P/x['display']).read_bytes() and sha(b)==x['sha256'];seen[x['source_path']].update(range(x['start'],x['end']+1))
 assert len(rr['coverage'])==len(captures)
 for x in rr['coverage']:
  s=x['source_path'];assert x['eof']==captures[s]['eof'] and x['personally_displayed']==spans(seen[s]) and x['unread']==spans(set(range(1,x['eof']+1))-seen[s])
 assert sum(map(len,seen.values()))==rr['unique_lines_personally_displayed']
 for x in load('OTHER-READ-RECEIPT.json')['chronology']:assert time(load(x['registration'])['registered_utc'])<=time(x['saved_utc']) and (P/x['selection']).is_file()
 h=load('HISTORY-IDENTITIES.json');parts=[(P/x['packet_path']).read_bytes() for x in h];assert len(parts)==2 and parts[0]==parts[1]
 for x,b in zip(h,parts):assert sha(b)==x['entry_sha256'] and len(b)==x['entry_bytes']
 q=json.loads(parts[0]);assert all(q[k]==v for k,v in load('Q2331-SUMMARY.json').items()) and q['status']==load('Q2331-STATUS.json')['status']
 for i,v in enumerate(q['granular_contract_views']):
  s=load(f'Q2331-VIEW-{i}-SUMMARY.json');assert all(v[k]==z for k,z in s.items());d=load(f'Q2331-VIEW-{i}-DISPOSITION.json');assert all(v[k]==z for k,z in d.items() if k!='contract_update_as_proposed') and all(v['contract_update_as_proposed'][k]==z for k,z in d['contract_update_as_proposed'].items());items=[]
  for lo in range(0,len(v['exact_source_identities_and_assessed_ranges']),3):items+=load(f'Q2331-VIEW-{i}-SOURCES-{lo}-{min(lo+2,len(v["exact_source_identities_and_assessed_ranges"])-1)}.json')
  assert items==v['exact_source_identities_and_assessed_ranges'];bundle=load('historical-originals/'+v['bundle']['path']);assert pointer(bundle,v['bundle']['row_pointer'])==v['contract_update_as_proposed']
 refs={};collect_refs(q,refs);direct=load('RECOVERY-REGISTRATION.json');dr={x['reference']:x for x in load('RECOVERY-RESULT.json')};assert len(refs)==len(direct['references'])==len(dr)==27
 def verify_refs(reg,res,expected):
  assert len(reg['references'])==len(res)==len(expected)
  for x in reg['references']:
   v=res[x['path']];assert v['matched'] and expected[x['path']]==x['expected_sha256']==v['actual_sha256'];check_original(v['packet_path'],v['actual_sha256'],v['bytes'])
 verify_refs(direct,dr,refs)
 more={}
 for e in q['granular_contract_views'][0]['exact_source_identities_and_assessed_ranges']:
  rec=load('historical-originals/'+e['receipt']['path']);collect_refs(pointer(rec,e['row_pointer']),more)
 more={k:v for k,v in more.items() if k not in refs};mr={x['reference']:x for x in load('RECOVERY-EXTENSION-RESULT.json')};assert len(more)==8;verify_refs(load('RECOVERY-EXTENSION-REGISTRATION.json'),mr,more)
 ad=load('RECOVERY-ADOPTION-RESULT.json')['candidates'];assert len(ad)==8
 for x in ad:
  check_original(x['packet_path'],x['sha256'],x['bytes']);assert x['suffix_matches']>0
 extra={}
 for n in [865,867,873,880]:
  root=load(f'historical-originals/segovb/READ-L12-{n}.json');ver=load(f'historical-originals/segovb/VERIFY-L12-{n}.json');check_original(f'historical-originals/segovb/READ-L12-{n}.json',ver['receipt_sha256']);assert root['proposal_sha256']==ver['proposal_sha256'];check_original('historical-originals/'+root['proposal'],root['proposal_sha256'])
  for path,digest in [(root['proposal'],root['proposal_sha256'])]+[(ver[k]['path'],ver[k]['sha256']) for k in ['ruleset','root_adapter']]:
   if path not in refs and path not in more:
    if path in extra:assert extra[path]==digest
    extra[path]=digest
 er={x['reference']:x for x in load('RECOVERY-ADOPTION-EXTENSION-RESULT.json')};assert len(extra)==6;verify_refs(load('RECOVERY-ADOPTION-EXTENSION-REGISTRATION.json'),er,extra)
 for x in load('HISTORICAL-ADOPTIONS-SELECT.json'):
  root=load('historical-originals/'+x['read_record']);ver=load('historical-originals/'+x['verify_record']);assert all(root[k]==v for k,v in x['root_fields'].items()) and ver==x['verification']
  for row in x['matching_source_rows']:assert any(all(raw.get(k)==v for k,v in row.items()) for raw in root['rows'])
 first=load('HISTORICAL-NEW-READERS.json')
 for e,x in zip(q['granular_contract_views'][0]['exact_source_identities_and_assessed_ranges'],first):
  rec=load('historical-originals/'+e['receipt']['path']);row=pointer(rec,e['row_pointer']);assert x['receipt']==e['receipt']['path'] and x['receipt_at']==rec.get('at') and all(row[k]==v for k,v in x['selected'].items())
 owners=load('HISTORICAL-OWNER-READERS-0-2.json')+load('HISTORICAL-OWNER-READERS-3-5.json')+load('HISTORICAL-OWNER-READERS-6-8.json')
 for e,x in zip(q['granular_contract_views'][1]['exact_source_identities_and_assessed_ranges'],owners):
  reg=load('historical-originals/'+e['prospective_registration']['path']);rec=load('historical-originals/'+e['original_agent_receipt']['path']);root=load('historical-originals/'+e['root_adoption']['path']);d=pointer(rec,e['row_pointer']);z=pointer(root,e['root_row_pointer']);assert d['source_id']==z['source_id']==reg['source_id']==e['source_id'] and d['source_sha256']==z['source_sha256']==reg['source_sha256']==e['source_sha256'];assert all(d[k]==v for k,v in x['delegated_selected'].items()) and all(z[k]==v for k,v in x['root_row_selected'].items());assert x['registered_at']==reg.get('registered_at') and x['receipt_at']==rec.get('at') and x['root_at']==root.get('at') and time(reg['registered_at'])<=time(root['at']) and x['new_personal_reads']==e['new_personal_reads']
 mesh=load('HISTORICAL-MESH-REUSE-SELECT.json');old=load('historical-originals/parallel-l12-20261029/PROPOSAL-L12-865.json')['rows'][1];connected=load('historical-originals/parallel-l12-20261029/READ-CONNECTED-PAR-L12-769-COLLISION-OWNERS.json');root=load('historical-originals/segovb/READ-L12-CONNECTED-TEAM-006.json');reg=load('historical-originals/parallel-l12-20261029/REGISTRATION-PAR-L12-297-CONNECTED-ORIGINAL2551.json');assert all(reg[k]==v for k,v in mesh['registration'].items()) and all(old[k]==v for k,v in mesh['ordered_proposal_selected'].items()) and all(connected['rows'][3][k]==v for k,v in mesh['connected_read_selected'].items()) and root['qualified_future_original_proofs'][3]==mesh['root_future_proof'] and all(root['checked_rows'][3][k]==v for k,v in mesh['root_checked_row'].items())
 assert load('HISTORICAL-748-FINDING.json')==load('historical-originals/parallel-l12-20261029/PROPOSAL-L12-748.json')['new_findings'][0]
 for sel,src in [('HISTORICAL-QUALIFIED-SELECT.json','QUALIFIED-REUSE-PAR-L12-873-GPU-PARAMETER-OWNERS.json'),('HISTORICAL-FASTPATH-SELECT.json','QUALIFICATION-PAR-L12-873-FASTPATH-EXACT-REUSE.json')]:
  d=load('historical-originals/parallel-l12-20261029/'+src);assert all(d[k]==v for k,v in load(sel).items())
 for x in load('HISTORICAL-WINDOWS-SELECT.json'):
  d=load('historical-originals/'+x['path']);assert all(d[k]==v for k,v in x['selected'].items())
 d=load('historical-originals/parallel-l12-20261029/QUALIFICATION-PAR-L12-603-WINDOWS-ORIGINAL2558.json');assert all(d[k]==v for k,v in load('HISTORICAL-PREFIX-WINDOWS-SELECT.json').items())
 related=load('RELATED-HISTORY-IDENTITIES.json')+[load('Q370-IDENTITY.json')]
 for x in related:
  b=(P/x['packet_path']).read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes']==x['end_byte_exclusive']-x['start_byte'];obj=json.loads(b);n=x['selector'].split('/')[-1];assert all(obj[k]==v for k,v in load('Q'+n+'-SELECT.json').items())
 for x in load('RELATED-IDENTITIES.json'):
  b=(P/x['packet_path']).read_bytes();assert sha(b)==x['sha256'] and blob(b)==x['blob'] and len(b)==x['bytes']
 for label,filename in [('BIND','ASSESSMENT-GPU-BIND-BUFFER-95E.json'),('PASS','ASSESSMENT-GPU-PASS-PIPELINE-95E.json'),('KERNEL','ASSESSMENT-KERNEL-95E.json')]:
  d=load('related/'+filename);s=load('RELATED-'+label+'-SELECT.json');assert d['source_revision']==s['source_revision'] and d['scope']==s['scope'] and all(d['observations'][int(i)]==v for i,v in s['observations'].items()) and all(d[k]==s[k] for k in ['remaining','repair','validation'])
  if label=='BIND':assert all(d['reconnected_historical_findings'][int(i)]==v for i,v in s['reconnected'].items()) and d['explicit_historical_subcontract_aliases']==s['explicit_historical_subcontract_aliases'] and d['new_current_candidates'][0]==s['candidate_0']
  if label=='PASS':assert d['reconnected_historical_findings'][18]==s['sparse_group_historical'] and d['new_current_candidates'][2]==s['multiple_uniform_candidate']
  if label=='KERNEL':assert d['reconnected_historical_findings'][2]==s['uniform_historical'] and d['new_current_candidates']==s['candidates'] and d['connected_historical_shader_family']==s['connected_historical_shader_family']
 d=load('related/delegated/bvh-shaders-report.json');s=load('RELATED-BVH-SELECT.json');assert d['findings'][5]==s['finding_5'] and all(d[k]==v for k,v in s.items() if k!='finding_5')
 for x in load('DIFF-IDENTITIES.json'):
  b=(P/x['diff']).read_bytes();assert len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and sha(b)==x['sha256']
 print('PASS virtual packet selections, 14 source captures/displays, q2331, historical metadata/roles and related assessments; see original-byte recovery status above.')
 if a.git_repo:
  for x in rows:assert git(a.git_repo,'show',rr['source_revision']+':'+x['path'])==(P/x['capture']).read_bytes()
  for x in load('SOURCE-IDENTITIES.json'):
   vals=[]
   for rev,v in x['revisions'].items():
    b=git(a.git_repo,'show',rev+':'+x['path']);assert sha(b)==v['sha256'] and blob(b)==v['blob'] and len(b)==v['bytes'] and len(b.splitlines())==v['eof'];vals.append(b)
   assert x['old_matches_historical_record'] and sha(vals[0])==x['historical_expected_sha256'] and (vals[0]==vals[1])==x['old_current_sha_equal']
  streams={}
  for x,part in zip(h,parts):
   b=git(a.git_repo,'show',x['revision']+':'+x['path']);streams[x['role']]=b;assert sha(b)==x['stream_sha256'] and blob(b)==x['git_blob'] and len(b)==x['stream_bytes'] and b[x['entry_start_byte']:x['entry_end_byte_exclusive']]==part and json.loads(b)['entries'][2331]==q
  assert streams['original']!=streams['public_reconstruction'];original=json.loads(streams['original'])
  for x in related:
   b=streams['original'];part=(P/x['packet_path']).read_bytes();assert sha(b)==x['stream_sha256'] and len(b)==x['stream_bytes'] and b[x['start_byte']:x['end_byte_exclusive']]==part and pointer(original,x['selector'])==json.loads(part)
  r=load('RELATED-REGISTRATION.json')
  for x in load('RELATED-IDENTITIES.json'):assert git(a.git_repo,'show',r['revision']+':'+x['path'])==(P/x['packet_path']).read_bytes()
  r=load('DIFF-REGISTRATION.json')
  for x in load('DIFF-IDENTITIES.json'):assert git(a.git_repo,'diff','--no-ext-diff','--no-color','--no-renames','--unified=2',r['old_revision'],r['current_revision'],'--',x['path'])==(P/x['diff']).read_bytes()
  r=load('TREE-REGISTRATION.json');names=git(a.git_repo,'ls-tree','-r','--name-only',r['revision'],'--',*r['prefixes']).decode().splitlines();s=load('TREE-RESULT.json');assert len(names)==s['total_paths'] and [x for x in names if '/shader' in x or x.endswith('.wgsl') or '/param' in x or '/macro' in x]==s['selected_paths']
  def locate(rev,scopes,pat):
   c=['git','--no-replace-objects','-C',str(a.git_repo),'grep','-n','-i','-F',pat,rev,'--',*scopes];z=subprocess.run(c,capture_output=True,text=True,env=dict(os.environ,GIT_NO_LAZY_FETCH='1'));out=[]
   for line in z.stdout.splitlines():
    t=line.split(':',3)
    if len(t)==4:
     try:out.append({'path':t[1],'line':int(t[2])})
     except ValueError:pass
   return z.returncode,out
  for prefix in ['LOCATOR-','LOCATOR-MESH-','LOCATOR-CALLERS-','RELATED-LOCATOR-','RELATED-DETAIL-LOCATOR-']:
   r=load(prefix+'REGISTRATION.json')
   for k,v in load(prefix+'RESULT.json').items():
    status,found=locate(r['revision'],r['scopes'],k);assert status==v['status'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted'],(prefix,k)
  r=load('RELATED-HISTORY-LOCATOR-REGISTRATION.json')
  for pattern,v in zip(r['patterns'],load('RELATED-HISTORY-LOCATOR-RESULT.json')):
   found=[]
   for i,e in enumerate(original['entries']):
    text=json.dumps({k:e.get(k) for k in ['contract_id','canonical_owner','first_failed_contract_or_unresolved_question']}).lower()
    if all(z in text for z in pattern['all']):found.append({'index':i,'contract_id':e.get('contract_id')})
   assert pattern==v['pattern'] and len(found)==v['matches_total'] and found[:100]==v['retained'] and max(0,len(found)-100)==v['omitted']
  print('PASS fixed source memberships, 6533/95e comparison and observed diffs, two full 059 streams and 13 exact slices, selected prior assessments and bounded locators.')
 else:print('NOT REPEATED Git memberships, whole queue streams, diffs, related assessments or locators: supply --git-repo.')
 if a.publication:
  t=load('TRANSPORT-IDENTITY.json');b=(a.publication/t['path']).read_bytes();assert sha(b)==t['sha256'] and len(b)==t['bytes'];nodes=[]
  def walk(v):
   if isinstance(v,dict):
    if 'sha256' in v:nodes.append(v)
    for x in v.values():walk(x)
   elif isinstance(v,list):
    for x in v:walk(x)
  walk(json.loads(gzip.decompress(b)));res={**dr,**mr,**er}
  for x in load('TRANSPORT-MATCHES.json'):
   v=res[x['reference']];rel=x['reference'].removeprefix('target/726-audit-adler-20261027/');ms=[n for n in nodes if n.get('sha256')==v['actual_sha256'] and n.get('bytes')==v['bytes']];assert len(ms)==x['digest_size_matches'] and sum(str(n.get('path','')).endswith(rel) for n in ms)==x['path_suffix_matches'] and x['path_suffix_matches']>0
  for x in ad:
   ms=[n for n in nodes if n.get('sha256')==x['sha256'] and n.get('bytes')==x['bytes'] and str(n.get('path','')).endswith(x['path'])];assert len(ms)==x['suffix_matches'] and len(ms)>0
  print('PASS transport metadata for 41 cited records and eight qualified candidate root files; independent custody not established.')
 else:print('NOT REPEATED transport comparison: supply --publication.')
 print('LIMITS: timestamps attest process order; hashes and scans do not prove comprehension, native admission, supported reach, product behavior or independent custody. Diff formatting depends on Git configuration.')
if __name__=='__main__':main()
