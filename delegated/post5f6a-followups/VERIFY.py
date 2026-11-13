#!/usr/bin/env python3
"""Verify public evidence identities and disclosed finite-scan classification."""
from pathlib import Path
import argparse,hashlib,io,json,re,subprocess,sys,tarfile
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();h=lambda b:hashlib.sha256(b).hexdigest();load=lambda x:json.loads(x.read_bytes())
for n in ['transport','late-transport','later-transport','encrypted-originals','supplemental-encrypted']:subprocess.run([sys.executable,'-B',str(p/n/'VERIFY.py')],check=True)
assert subprocess.check_output(['git','-C',str(root),'rev-parse','5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0^{tree}']).decode().strip()=='84c63be7bd8e5b00b33bd63f480fe8ebb7b900c7';q=json.loads(subprocess.check_output(['git','-C',str(root),'show','5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0:RECONCILIATION-059.json']))['entries'];z=load(p/'PENDING-SOURCE-REVIEWS.json');assert z['queue_indices']==[0,19,2497,2501,2503,2504,2510,2521,2534] and len(z['rows'])==9
for r in z['rows']:
 x=q[r['queue_index']];assert (x['contract_id'],x['historical_lens'],x['current_reconciliation'])==(r['historical_contract_id'],r['historical_lens'],r['central_state_at_5f6a']) and r['central_state_at_5f6a']=='unassessed'
c=load(p/'PRIVACY-CLASSIFICATION.json');assert c['preliminary_wrapper_exit']==1 and c['forbidden_digest_total']==c['scan_exceptions']==0 and c['other_candidate_counts']=={'email':4} and c['transport_manifest_sha256']==h((p/'transport/MANIFEST.json').read_bytes())
with tarfile.open(p/'transport/q19-source.tar.gz','r:gz') as t:lib=t.extractfile('q19-source/native/0').read();crate=t.extractfile('q19-source/native/spacetimedb-2.6.1.crate').read()
assert h(crate)==c['sdk_crate_sha256']=='b6eff5017248ff2e35e03564df947fdf3e963f7198be6ae1d9e894c2efb2ea15' and h(lib)==c['member_sha256']
with tarfile.open(fileobj=io.BytesIO(crate),mode='r:*') as t:assert t.extractfile('spacetimedb-2.6.1/src/lib.rs').read()==lib
pattern=re.compile(rb'(?<![A-Za-z0-9_.+%-])[A-Za-z0-9_.+%-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![A-Za-z0-9_.-])');found=pattern.findall(lib);assert len(found)==2 and len(set(found))==1 and found[0].rsplit(b'@',1)[1].lower() in {b'example.com',b'example.org',b'example.net'}
prescan=load(p/'STAGE-PRESCAN.json');assert prescan['scanned_paths']==98 and prescan['preliminary_wrapper_exit']==1 and prescan['forbidden_digest_total']==prescan['exceptions']==prescan['tar_pattern_issues']==0 and prescan['other_candidate_counts']=={'email':4}
late=load(p/'LATE-SCAN.json');assert late['transport_manifest_sha256']==h((p/'late-transport/MANIFEST.json').read_bytes()) and late['scanned_paths']==5 and late['forbidden_digest_total']==late['exceptions']==late['tar_pattern_issues']==0 and not late['other_candidate_counts'] and all(x['exit']==0 for x in late['checks'])
capture=load(p/'ADDITIONAL-CAPTURE.json');scan=load(p/'ADDITIONAL-SCAN.json');assert len(capture['rows'])==7 and scan['capture_sha256']==h((p/'ADDITIONAL-CAPTURE.json').read_bytes()) and scan['scanned_paths']==7 and scan['forbidden_digest_total']==scan['exceptions']==scan['tar_pattern_issues']==0 and not scan['other_candidate_counts'] and all(x['exit']==0 for x in scan['checks'])
for row in capture['rows']:
 head,tail=row['path'].split('/',1);assert head in ('later-transport','application-review');f=(p/'later-transport'/tail) if head=='later-transport' else (p/'application/review-transport'/tail);b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
r=load(p/'PRIMARY-RESTORATION.json');assert r['restore_exit']==0 and r['collections']==33 and r['matched_file_bytes_and_modes']==2069 and r['matched_directory_modes']==159 and r['selected_replay_exits']==[0]*5
s=load(p/'SUPPLEMENTAL-RESTORATION.json');assert s['restore_exit']==0 and s['collections']==17 and s['matched_file_bytes_and_modes']==2137 and s['matched_directory_modes']==272 and s['selected_replay_exits']==[0]*6
rr=load(p/'REMOTE-RECOVERY.json');pr=load(p/'PREDECESSOR-REPLAY.json');assert rr['commit']==pr['commit']=='5f6a6e829dca0f8b2a5efff9b6f17fab5db71ae0' and rr['captured_paths_matched']==179 and all(x['exit']==0 for x in rr['checks']) and len(pr['checks'])==3 and all(x['exit']==0 for x in pr['checks'])
print('Verified public archive identities, 5f6a pending snapshot, classified SDK example, encrypted identities and same-machine receipts. Full private replay requires the separately held key and documented dependencies; no semantic closure or product validation is established.')
