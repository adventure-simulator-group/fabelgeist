#!/usr/bin/env python3
"""Verify the static design packet; never execute product or old verifiers."""
import argparse,datetime,gzip,hashlib,json,re,subprocess
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--git-repo',type=Path);args=ap.parse_args()
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda n:json.loads((root/n).read_bytes())
receipt=load('READ-RECEIPT.json');rev=receipt['publication_revision']
for n,h in receipt['registrations'].items():assert sha((root/n).read_bytes())==h
for n,row in receipt['displays'].items():
 b=(root/n).read_bytes();assert (len(b),sha(b))==(row['bytes'],row['sha256'])
for n in receipt['registrations']:
 d=load(n)
 if 'registered_utc' in d:assert datetime.datetime.fromisoformat(d['registered_utc'])<datetime.datetime.fromisoformat(receipt['recorded_utc'])
assert 'truncated' in receipt['chronology_and_limits'][0]
assert 'general scope' in receipt['chronology_and_limits'][1]
old=(root/'REPORT.md').read_bytes();new=(root/'REPORT-v2.md').read_bytes();correction=load('REPORT-CORRECTION.json');assert (sha(old),sha(new))==(correction['initial_report_sha256'],correction['successor_report_sha256'])
a=b'All 13 lenses and 3,713 indexed 059 queue records are still incomplete; the\n1,031 ledger entries are entries and relationships, not independent defect\ncounts.'
b=b'All 13 current lenses remain incomplete, and none of the 3,713 indexed 059\nqueue records is counted as semantically reconciled at 95e. Their historical\nnamed-scope dispositions remain intact. The 1,031 ledger entries are entries\nand relationships, not independent defect counts.'
assert old.count(a)==1 and old.replace(a,b)==new
listing=gzip.decompress((root/'TREE-LISTING.z.gz').read_bytes());tree=load('TREE-INVENTORY.json');assert len(listing)==tree['listing_raw_bytes'] and sha(listing)==tree['listing_raw_sha256'] and sha((root/'TREE-LISTING.z.gz').read_bytes())==tree['listing_gzip_sha256']
if args.git_repo:
 def git(*a):return subprocess.check_output(['git','--no-replace-objects','-C',str(args.git_repo),*a])
 assert git('rev-parse',rev+'^{tree}').decode().strip()==tree['tree']
 assert git('ls-tree','-r','-l','-z',rev)==listing
 catalog={};allow={}
 for regname in ('REGISTRATION.json','DISCOVERY-EXT-REGISTRATION.json','DEPENDENCY-EXT-REGISTRATION.json','SCRATCH-REGISTRATION.json'):
  for e in load(regname)['entries']:
   path=e['path'];raw=git('show',rev+':'+path);assert sha(raw)==e['sha256']
   if 'bytes' in e:assert len(raw)==e['bytes']
   if 'git_blob' in e:assert hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==e['git_blob']
   catalog[path]=raw.decode().splitlines();allow.setdefault(path,[]).extend(e.get('ranges',[]))
 for regname in ('SUCCESSOR-SOURCE-REGISTRATION.json','SUCCESSOR-CONTEXT-REGISTRATION.json','PRIVACY-EXACT-REGISTRATION.json'):
  data=load(regname);entries=data.get('entries',[data])
  for e in entries:
   path=e.get('path',e.get('source'));raw=git('show',rev+':'+path);assert sha(raw)==e['sha256'];catalog[path]=raw.decode().splitlines();allow.setdefault(path,[]).extend(e.get('ranges',e.get('registered_ranges',[])))
 for regname in ('LANDING-RECORDS-REGISTRATION.json','SUBSTITUTION-REGISTRATION.json','PRIVACY-REGISTRATION.json'):
  for e in load(regname)['entries']:
   raw=git('show',rev+':'+e['path']);assert sha(raw)==e['sha256']
 def in_scope(path,n):return any(a<=n<=b for a,b in allow.get(path,[]))
 for name in receipt['displays']:
  if not name.endswith('.txt'):continue
  for line in (root/name).read_text().splitlines():
   m=re.match(r'^(.*):([0-9]{4}): (.*)$',line)
   if not m:continue
   path,n,text=m.group(1),int(m.group(2)),m.group(3);assert path in catalog and in_scope(path,n) and catalog[path][n-1]==text
 names=[x.decode() for x in git('ls-tree','-r','--name-only','-z',rev).split(b'\0') if x];nav=load('NAVIGATION-RESULTS.json');assert sorted(n for n in names if re.fullmatch(r'verify_[^/]+\.py',n))==nav['root_verifier_files'];assert sorted(n for n in names if re.fullmatch(r'native-evidence/[^/]+/verify\.py',n))==nav['native_verifier_files']
 for label,pattern in load('HEURISTIC-RESULT.json')['patterns'].items():
  run=subprocess.run(['git','--no-replace-objects','-C',str(args.git_repo),'grep','-l','-E',pattern,rev,'--','verify_*.py'],stdout=subprocess.PIPE,check=False);assert run.returncode in (0,1);names=[x.decode().split(':',1)[1] for x in run.stdout.splitlines()];assert sorted(x for x in names if re.fullmatch(r'verify_[^/]+\.py',x))==load('HEURISTIC-RESULT.json')['root_verifier_files'][label]
 base=json.loads(git('show',rev+':BASELINE-95E-20261110.json'));progress=json.loads(git('show',rev+':CONTINUATION-PROGRESS-95E.json'));queue=json.loads(git('show',rev+':RECONCILIATION-059.json'));coverage=json.loads(git('show',rev+':COVERAGE-L01-ROLES.json'));qual=json.loads(git('show',rev+':EVIDENCE-VERIFIER-EVOLUTION-QUALIFICATION.json'));expected={'evolution_qualification':qual,'baseline':{k:base[k] for k in ('source_revision','status','authority_rule','lens_names','lens_status','historical_queue_records_indexed','historical_queue_records_semantically_reconciled','recovery_qualification','latest_main_recheck')},'ledger_count':len(base['newly_assessed_ledger']['entries']),'progress':{k:progress[k] for k in ('source_revision','lens_count','lenses_complete','queue_records_indexed','queue_records_semantically_reconciled','new_current_ledger_candidates','repair','product_validation')},'queue_count':len(queue['entries']),'role_count':len(coverage['roles']),'role_status_counts':coverage['status_counts']};assert load('STATE-DISPLAY.json')==expected
 print('Pinned Git tree, selected source displays, static heuristic files and state selections rechecked against supplied local Git objects.')
else:print('No Git repository supplied; pinned Git source and tree comparisons were not repeated.')
print('Packet identities and chronology checked; no equivalence, privacy completeness, product behavior or historical replay is established.')
