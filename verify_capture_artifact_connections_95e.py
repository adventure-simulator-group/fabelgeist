#!/usr/bin/env python3
"""Verify preserved capture and native source evidence, without product execution."""
import gzip
import hashlib
import json
import tarfile
from fractions import Fraction
from pathlib import Path

r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-CAPTURE-ARTIFACT-CONNECTIONS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1410-1416.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1410,1417))
assert [x['lines'] for x in h['historical_source_roles']]==[1524,71,1021,168,57,58,59]
for index in (1412,1415):
 x=next(x for x in h['historical_source_roles'] if x['original_index']==index)
 assert x['prior_role_status']=='bounded-current-trace-only' and x['prior_current_observations']==w['roles'][index]['current_observations']
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1410-1416.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
capture=current[1412];assert capture.index(b'save_to_disk(&path)(captured)')<capture.index(b'sequence.capture_in_flight = false',capture.index(b'save_to_disk(&path)(captured)'))
assert b'visual_fingerprint(&captured.image)' in capture and b'write_completed_capture(completed)' in capture
report=s['crates/adventuresim-tactical-client/src/animation_viewer/report/mod.rs'];assert b'fn capture_artifacts_written' in report and b'fs::metadata(output.join(name)).ok()' in report and b'metadata.len() > 0' in report
assert b'"manifest.json"' in report and b'"global-bone-transforms.jsonl"' in report
validation=s['crates/adventuresim-tactical-client/src/animation_viewer/report/validation.rs'];assert b'capture_artifacts_written(&output, &frames)' in validation and b'validation.all_artifacts_written' in validation
schedule=current[1414];assert schedule.index(b'reject_capture,')<schedule.index(b'capture_frame\n') and b'.run_if(harness::ready)' in schedule
admission=current[1413];assert b'left_support_weight\n            .zip(diagnostics.right_support_weight)' in admission and b'return Some(cadence)' in admission
lock=s['Cargo.lock'];assert b'fc86a32b2eef9859e1e7d323c6d1a9150fe77aa17c30fd05abebc12f645929dc' in lock
with tarfile.open(r/'native-evidence/bevy_render-0.19.1/bevy_render-0.19.1.crate','r:gz') as ar:
 v=ar.extractfile('bevy_render-0.19.1/src/view/window/screenshot.rs').read();assert b'pub fn save_to_disk(path: impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)' in v and b'Cannot save screenshot, IO error' in v
# Exact rational inequality only; no f32 kernel or product simulation ran.
a0=Fraction(0);b0=Fraction(75,10**6);c0=Fraction(150,10**6);tol=Fraction(1,10**4)
assert abs(a0-b0)<tol and abs(b0-c0)<tol and abs(a0-c0)>=tol
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));original={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=original[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {403:[1410],404:[1411],405:[1412],406:[1413,1414,1415],407:[1416,1417]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids
 assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==original[f'segovb/READ-L12-{batch}.json']['sha256']
native=json.loads(records['segovb/READ-L11-535-NATIVE-001.json']);assert native['current_main_revision']=='7a9d92ad7357779e9280146c521bd8588b86fcdb' and native['source_sha256']=='f49e4ca4f0929fc51a5c55fc002294c6009ae7a4c1c07dc6ef6c8ff737df0c5b'
q=load('RECONCILIATION-059.json')['entries'];oldq=load('baseline-059/queue.json')['entries']
assert q[206]['contract_id']==a['existing_finding_connections'][0]['id']=='SEGOVB-L11-CAPTURE-RUN-ARTIFACT-CERTIFICATION'
assert [oldq[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in range(1205,1212)]==list(range(1410,1417))
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [206,*range(1205,1212)])
p=load('CONTINUATION-PROGRESS-95E.json');ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified seven historical reads and qualified capture persistence sources; no product execution.')
