#!/usr/bin/env python3
"""Verify historical equipment identities and bounded current source connections."""
import difflib,gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
def obj(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
blob=lambda b:obj('blob',b)
def contents(p):return gzip.decompress((r/p).read_bytes())
def entries(b):
 result={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);result[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return result
rev='95e63628f9d70b8dd6e6fede64334696c6995009';oldrev='36c97923da8ee84d0ab8994e7cfedf6a0cdea641';head='6533e444fa37552927070cc8d8190907fd3b413e'
a=load('ASSESSMENT-EQUIPMENT-PRESENTATION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1474-1478.json');w=load('COVERAGE-L01-ROLES.json');proof=load('EQUIPMENT-HISTORICAL-GIT-PROOFS.json')
tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 x,path=line.split('\t',1);mode,kind,oid=x.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
nodes={}
for oid,info in proof['tree_objects'].items():
 b=contents(info['preserved_bytes']);assert sha(b)==info['sha256'] and len(b)==info['bytes'] and obj('tree',b)==oid==info['git_object'];nodes[oid]=entries(b)
for revision,x in proof['revisions'].items():
 c=x['commit'];b=contents(c['preserved_bytes']);assert sha(b)==c['sha256'] and len(b)==c['bytes'] and obj('commit',b)==revision==c['git_object']
 assert b.splitlines()[0]==b'tree '+x['root_tree'].encode()
 for path,p in x['paths'].items():
  current=x['root_tree'];assert [step['name'] for step in p['chain']]==path.split('/')
  for step in p['chain']:
   assert step['tree']==current and nodes[current][step['name']]==(step['mode'],step['child']);current=step['child']
  assert current==p['git_blob']
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1474,1479))
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1474-1478.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  original=proof['revisions'][x['original_exact_git_revision']]['paths'][x['current_comparison']['path']]
  assert sha(raw)==original['sha256'] and blob(raw)==original['git_blob'] and len(raw)==original['bytes']
  comp=x['current_comparison'];cur=contents(comp['preserved_bytes']);assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes'] and len(cur.splitlines())==comp['lines']
  assert blob(cur)==comp['git_blob']==tree[comp['path']];current[i]=cur
  if raw!=cur:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile=f'historical-{i}/'+comp['path'],tofile='95e/'+comp['path'])).encode();assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256']
  if i in (1474,1476):
   assert x['original_exact_git_revision']==oldrev;later=x['later_committed_variant'];b=contents(later['preserved_bytes']);p=proof['revisions'][head]['paths'][later['path']]
   assert sha(b)==later['sha256']==p['sha256'] and blob(b)==later['git_blob']==p['git_blob'] and len(b)==later['bytes'] and len(b.splitlines())==later['lines'] and raw!=b
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),b.decode().splitlines(keepends=True),fromfile=f'original-{i}/'+later['path'],tofile='6533/'+later['path'])).encode();assert d==(r/later['diff']).read_bytes() and sha(d)==later['diff_sha256'] and b'fn icon_uv(' in d
  else:assert x['original_exact_git_revision']==head and x['later_committed_variant'] is None
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and blob(b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
server=s['crates/adventuresim-tactical-server/src/equipment.rs'];grab=current[1475];visual=current[1478]
assert b'transform.transform_point(-physical.anchor_offset_m)' in grab and b'Transform::from_translation(-physical.anchor_offset_m)' in visual
assert b'item_box_center(item_transform.translation, physical)' in server and b'grip - physical.anchor_offset_m' in server and b'RigidBody::Dynamic' in server
assert b'Added<TacticalEquipmentPhysical>' in current[1474] and b'Changed<TacticalEquipmentPhysical>' not in current[1474]
sockets=s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment/sockets.rs'];rig=s['crates/adventuresim-tactical-client/src/animation/procedural/rig.rs'];names=s['crates/fabelgeist-rig/src/names.rs']
assert b'RigJointName::ROOT.index_in(&body.joint_names)' in sockets and b'world.reparented_to(&bind)' in sockets and b'Self::Pelvis => RigJointName::ROOT' in rig and b'pub const ROOT: Self = Self(Cow::Borrowed("root"));' in names
assert b'EquipmentLocation::LeftBelt' in current[1474] and b'EquipmentLocation::BackBelt' in current[1474] and b'=> BoneRole::Pelvis' in current[1474]
catalog=json.loads(s['content/items/catalog.yaml']);belt=next(x for x in catalog['items'] if x['id']=='belt');worn=next(x for x in belt['equipment']['placements'] if x['id']=='worn')
assert {x['location'] for x in worn['occupancy']}=={'left_belt','right_belt','front_belt','back_belt'}
assert {loc for pt in belt['equipment']['attachment_points'] for loc in pt['locations']}=={'left_belt','right_belt','front_belt','back_belt'}
assert b'if has_children(item, items)' in s['crates/adventuresim-tactical-server/src/equipment/transfer.rs']
bridge=s['crates/adventuresim-tactical-client/src/hud.rs'];slot=s['crates/adventuresim-tactical-client/src/equipment/slot_hud.rs'];js=s['crates/adventuresim-stdb-module/static/tactical-hud.js'];icons=current[1476]
assert b'#[cfg(not(target_family = "wasm"))]\n    let _ = (view, json);' in bridge and b'hud::view(HudView::Slots, json)' in slot
assert b'the native client has no HUD' in s['crates/adventuresim-tactical-client/README.md'] and b'performance playtesting' in s['justfile']
assert b'installTacticalHud({' in s['crates/adventuresim-stdb-module/static/tactical.html'] and b'installTacticalHud({ container: surface, command });' in s['crates/strategic-web/static/strategic-renderer.js']
assert icons.index(b'hud::icon(&page_key, size, size, &rgba);')<icons.index(b'self.icons.insert(key, page_key.clone());') and b'new Uint8ClampedArray(rgba)' in js and b'icons.set(key, canvas.toDataURL());' in js
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 b=contents(x['preserved_bytes']);assert len(b)==x['published_bytes'] and sha(b)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and blob(b)==x['git_blob']);records[x['original_path']]=b
catalogue=json.loads(records['segovb/FROZEN774-CATALOG.json'])
for i in (1474,1476):
 q=json.loads(records[f'segovb/QUALIFICATION-L12-ORIGINAL{i}-REVISION.json']);x=h['historical_source_roles'][i-1474]
 assert q['canonical_git_revision']==oldrev and q['source_sha256']==x['sha256'] and q['actual_EOF']==x['lines'] and q['new_original_overlay_merged_shared_extension_credit']==0
 row=catalogue['rows'][i];assert row['scope_index']==i and row['expected_sha256']==x['sha256'] and row['origin'].startswith('git:'+oldrev+':')
qual=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1478-LOCATOR.json']);assert qual['historical_locator_proof_body_not_read'] is True and qual['new_population_credit']==0 and '\\' in qual['recorded_path']
for batch,index in zip(range(439,444),range(1474,1479)):
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==[index] and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
for batch in (319,323):
 receipt=json.loads(records[f'adler/READ-L11-{batch}.json']);check=json.loads(records[f'adler/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and check['receipt_sha256']==m[f'adler/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'adler/STATE-L11-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in range(1273,1278)]==list(range(1474,1479)) and all(q[i]['current_reconciliation'].startswith('partial ') for i in range(1273,1278))
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']=='REVIEW-PICKUP-ANCHOR-WORLD-PROJECTION-001' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five original equipment bodies, revision/tree proofs, earlier qualifications and bounded current links; no product execution.')
