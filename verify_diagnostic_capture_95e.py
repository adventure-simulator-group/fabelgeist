#!/usr/bin/env python3
"""Verify diagnostic source and historical evidence without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-DIAGNOSTIC-CAPTURE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1472-1473.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1472,1473] and [x['lines'] for x in h['historical_source_roles']]==[376,876]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']]
  assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1472-1473.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert raw==cur and comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
native={}
for x in a['native_connected_sources']:
 archive=(r/x['archive']).read_bytes();assert sha(archive)==x['archive_sha256']
 package=x['package'];expected=f'name = "{package}"\nversion = "0.19.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "{x["archive_sha256"]}"'.encode();assert expected in s['Cargo.lock']
 with tarfile.open(r/x['archive'],'r:gz') as t:raw=t.extractfile(x['member']).read()
 assert sha(raw)==x['member_sha256'] and len(raw)==x['member_bytes'] and len(raw.splitlines())==x['member_lines']
 assert all(1<=lo<=hi<=x['member_lines'] for lo,hi in x['personally_inspected_ranges']);native[package]=raw
script=current[1473];debug=current[1472];main=s['crates/adventuresim-tactical-client/src/main.rs'];net=s['crates/adventuresim-tactical-netcode/src/message.rs']
assert b'pending_captures.0 += 1;' in script and b'commands.spawn(Screenshot::primary_window()).observe(' in script and b'pending.0 = pending.0.saturating_sub(1);' in script
assert b'exit_after_script && *elapsed >= 0.25 && pending_captures.0 == 0' in script
assert script.index(b'pending_captures.0 += 1;')<script.index(b'commands.spawn(Screenshot::primary_window()).observe(')<script.index(b'script.command_index += 1;',script.index(b'commands.spawn(Screenshot::primary_window()).observe('))
assert b'if let ScriptCommand::Screenshot { path } = &command' in script and b'continue;\n        }\n\n        if let ScriptCommand::WaitForSignal' in script
assert b'File::create(path)' in script and script.index(b'.map(|path| create_diagnostic_file(path, "animation log"))')<script.index(b'if frame_timing_seconds.is_some_and(')
assert b'file.try_clone().ok()' in script and b'let command_start = script.command_elapsed == 0.0;' in script and b'script.command_elapsed += delta;' in script
assert b'"dive" => MovementPace::Walk' in script and b'"slide" => MovementPace::Sprint' in script and b'pace: diagnostic_command_pace(kind, script.pace)' in script
assert b'primary_window: None,' in main and b'.disable::<bevy::winit::WinitPlugin>()' in main and b'commands.insert_resource(debug::HeadlessScreenshotTarget(image));' in main
assert b'Some(target) => Screenshot::image(target.0.clone())' in debug and b'None => Screenshot::primary_window()' in debug and b'if self.quarter_speed { 0.25 } else { 1.0 }' in net
render=native['bevy_render'];camera=native['bevy_camera'];window=native['bevy_window']
assert b'#[derive(EntityEvent, Reflect, Deref, DerefMut, Debug)]' in render and b'pub struct ScreenshotCaptured {' in render
assert b'Self(RenderTarget::Window(WindowRef::Primary))' in render and b'Self(RenderTarget::Image(image.into()))' in render
assert b'if seen_targets.contains(&render_target)' in render and b'commands.entity(entity).despawn();\n            continue;' in render
assert b'let Some(render_target) = render_target.normalize(primary_window) else {' in render and b'Unknown render target for screenshot, skipping:' in render
assert b'RenderTarget::Window(window_ref) => window_ref\n                .normalize(primary_window)' in camera and b'Self::Primary => primary_window' in window
assert b'pub fn save_to_disk(path: impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)' in render and b'Cannot save screenshot, IO error:' in render
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,index in [(437,1472),(438,1473)]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==[index] and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
receipt=json.loads(records['segovb/READ-L12-438.json']);reuse=receipt['qualified_supporting_same_lens_reuse'][0]
assert reuse['original_index']==1322 and reuse['fresh_personal_body_reads_this_batch']==[] and reuse['receipt_sha256']==m[reuse['receipt']]['sha256'] and reuse['registration_sha256']==m[reuse['registration']]['sha256']
for x in receipt['partial_sequence']:assert x['sha256']==m[x['path']]['sha256'] and x['path'] in records
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries']
assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1271,1272)]==[1472,1473] and all(q[i]['current_reconciliation'].startswith('partial ') for i in (1271,1272))
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];historical=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
assert a['conditional_findings'][0]['id']=='REVIEW-DIAGNOSTIC-CAPTURE-UNSETTLED-001' and a['conditional_findings'][0]['id'] not in historical
for x in a['conditional_findings']:
 assert sum(y['id']==x['id'] for y in ledger)==1 and x['repair']=='not started' and x['validation']=='not performed'
for x in a['conditional_findings'][1:]:assert x['id'] in historical
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified two historical debug/diagnostic bodies, prior receipts, pinned screenshot sources and bounded current links; no product execution.')
