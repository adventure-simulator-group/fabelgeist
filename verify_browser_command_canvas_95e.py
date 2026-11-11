#!/usr/bin/env python3
"""Verify static browser, native-source and historical-evidence connections."""
import difflib,gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-BROWSER-COMMAND-CANVAS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1457-1460.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1457,1458,1459,1460]
assert [x['lines'] for x in h['historical_source_roles']]==[206,507,44,185]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1457-1460.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index'] in (1458,1460):
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-'+str(x['original_index'])+'/'+comp['path'],tofile='95e/'+comp['path'])).encode()
   assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256']
  else: assert raw==cur
assert h['historical_source_roles'][0]['prior_role_status']=='bounded-current-trace-only'
assert h['historical_source_roles'][0]['prior_current_observations']==[{'assessment':'ASSESSMENT-L1-AUDIO-CONFIG.json','ranges':[[10,64],[66,76],[79,185]]}]
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
native={}
for x in a['native_connected_sources']:
 archive=(r/x['archive']).read_bytes();assert len(archive)==x['archive_bytes'] and sha(archive)==x['archive_sha256']
 with tarfile.open(r/x['archive'],'r:gz') as ar: raw=ar.extractfile(x['member']).read()
 assert len(raw)==x['member_bytes'] and sha(raw)==x['member_sha256'] and all(1<=lo<=hi<=len(raw.splitlines()) for lo,hi in x['ranges']);native[x['member']]=raw
bevy=native['bevy_ecs-0.19.1/src/system/commands/mod.rs'];assert b'all queued commands are automatically applied in sequence' in bevy and b'world\n                .spawn_at_with_caller(entity, bundle, caller)' in bevy
serde=native['serde_core-1.0.229/src/de/impls.rs'];assert b'macro_rules! int_to_uint' in serde and b'if 0 <= v' in serde and b'u32, NonZeroU32, deserialize_u32' in serde
j=native['serde_json-1.0.151/src/de.rs'];assert b'ParserNumber::F64(x) => visitor.visit_f64(x)' in j and b"b'e' | b'E' => ParserNumber::F64" in j
browser=current[1458];assert b'let pending = pending_commands();' in browser and b'for command in pending {' in browser and b'clients: Query<Entity, With<AdventureSimulatorClient>>' in browser
assert b'if clients.is_empty() {' in browser and b'commands.spawn(AdventureSimulatorClient {' in browser and b'for entity in &clients {' in browser
assert b'despawn_forge_preview(&mut commands, &scene_entities);' in browser and b'self.yaw += delta_x * 0.008;' in browser and b'Quat::from_euler(EulerRot::YXZ, view.yaw' in browser
js=s['crates/strategic-web/static/strategic-renderer.js'];assert b'wasm_command(JSON.stringify(payload))' in js and b'type: "enter-tactical"' in js and b'type: "exit-tactical"' in js
scene=s['crates/strategic-web/static/strategic-scene.js'];assert b'if (right <= left || bottom <= top) return null;' in scene and b'width: Math.floor(right * ratio) - x' in scene
assert b'lastPayload = payload;' in scene and b'type: "sync-strategic-view"' in scene and b'state.revision === revision' in scene
canvas=s['crates/adventuresim-tactical-client/src/presentation/canvas_view.rs'];assert b'pub width: u32,' in canvas and b'pub height: u32,' in canvas
left=Fraction(1,5);right=Fraction(4,5);ceiling=lambda x:-((-x.numerator)//x.denominator);floor=lambda x:x.numerator//x.denominator
assert right>left and floor(right)-ceiling(left)==-1
max_f32=(2**24-1)*2**104;assert Fraction(200)*3*10**38*Fraction(78,10000)>max_f32
assert b'fn animation_capture_camera_offset(' in current[1460] and b'distance: authored_metres(3.75)' in s['crates/adventuresim-tactical-client/src/camera/config.rs']
assert b'if value.is_finite() && value > 0.0' in current[1457] and b'serde_saphyr::from_str(text)' in current[1457]
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {428:[1457],429:[1458],430:[1459,1460,1461]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
for n in (206,212):
 receipt=json.loads(records[f'adler/READ-L11-{n}.json']);state=json.loads(records[f'adler/STATE-L11-{n}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
 assert state['receipt_sha256']==m[f'adler/READ-L11-{n}.json']['sha256']
check=json.loads(records['adler/VERIFY-L11-212.json']);assert check['receipt_sha256']==m['adler/READ-L11-212.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-212.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];indices=[1256,1257,1258,1259];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in indices]==[1457,1458,1459,1460];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in [50,52]+indices)
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];historical=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
for x in a['conditional_findings']:
 assert sum(y['id']==x['id'] for y in ledger)==1
 if x['id'].startswith('ADLER-'):assert x['id'] in historical
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical bodies, bounded native-source links and symbolic command/canvas cases; no product execution.')
