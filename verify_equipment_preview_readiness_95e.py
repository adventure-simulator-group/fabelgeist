#!/usr/bin/env python3
"""Verify preview/readiness evidence and historical qualifications without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
contents=lambda p:gzip.decompress((r/p).read_bytes())
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-EQUIPMENT-PREVIEW-READINESS-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1479-1481.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 x,path=line.split('\t',1);mode,kind,oid=x.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1479,1480,1481] and [x['lines'] for x in h['historical_source_roles']]==[437,72,234]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1479-1481.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]];raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert raw==cur and c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and blob(cur)==c['git_blob']==tree[c['path']];current[x['original_index']]=cur
proof=load('PREVIEW-HISTORICAL-GIT-PROOFS.json');objects=load(proof['commit_and_tree_objects']);commit=objects['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes'])
assert hashlib.sha1(b'commit '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
def tree_entries(raw):
 result={};pos=0
 while pos<len(raw):
  end=raw.index(b'\0',pos);mode,name=raw[pos:end].split(b' ',1);result[name.decode()]=(mode.decode(),raw[end+1:end+21].hex());pos=end+21
 assert pos==len(raw);return result
for x in h['historical_source_roles']:
 path=x['current_comparison']['path'];p=proof['paths'][path];oid=proof['root_tree'];assert [step['name'] for step in p['chain']]==path.split('/')
 for step in p['chain']:
  raw=contents(objects['tree_objects'][oid]['preserved_bytes']);assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==oid==step['tree']
  assert tree_entries(raw)[step['name']]==(step['mode'],step['child']);oid=step['child']
 assert oid==p['git_blob']==blob(current[x['original_index']]) and p['sha256']==x['sha256']
s={}
for x in a['current_supporting_sources']:
 raw=contents(x['preserved_bytes']);assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines'] and blob(raw)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
preview=current[1479];ready=current[1480];binding=current[1481];parent=s['crates/adventuresim-tactical-client/src/equipment.rs'];slot=s['crates/adventuresim-tactical-client/src/equipment/slot_selection.rs']
assert b'None => Self::Empty' in preview and b'Self::Unknown, Self::Defined' in preview
assert b'HeldPreviewItem::from(held.and_then(|entity| properties.get(entity).ok()))' in parent and b'HeldPreviewItem::Empty => outermost_occupied_depth(&layers)' in slot and b'HeldPreviewItem::Unknown => None' in slot
assert b'if worn && skin.is_none()' in binding and b'unbound.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX)' in binding and b'unbound.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)' in binding
assert b'if failed {\n                return Err("Could not generate character equipment");' in ready and b'Ok(ready == items.len()' in ready
scene=s['crates/adventuresim-tactical-client/src/strategic_scene/equipment.rs'];status=s['crates/adventuresim-tactical-client/src/strategic_scene/status.rs'];protocol=s['crates/adventuresim-tactical-client/src/strategic_scene/protocol.rs'];appearance=s['crates/adventuresim-core/src/equipment_presentation.rs'];browser=s['crates/adventuresim-tactical-client/src/browser_runtime.rs']
assert b'pub equipment: Vec<adventuresim_core::equipment_presentation::EquipmentAppearance>' in protocol and b'pub item: String' in appearance
assert b'let command = serde_json::from_str(json)' in browser and b'BrowserCommand::SyncStrategicView { view } => {\n                commands.insert_resource(view);' in browser
assert b'let Some(definition) = item_catalog::definition(&item.item) else {\n        commands.entity(entity).despawn();\n        return;' in scene
assert b'let Some(equipment) = &definition.equipment else {\n        commands.entity(entity).despawn();\n        return;' in scene and b'if current.0 == person.equipment {\n                continue;' in scene
assert b'items: Query<Entity, With<super::equipment::SceneEquipment>>' in status and b'let equipment_state = equipment.check(items.iter());' in status and b'&& equipment_ready' in status
schema=s['crates/adventuresim-stdb-module/src/character.rs'];route=s['crates/strategic-web/src/routes/scene_equipment.rs']
assert b'#[primary_key]\n    pub inventory_item_id: u64' in schema and b'item: item.item_id.clone()' in route and b'.ok_or(StatusCode::SERVICE_UNAVAILABLE)?' in route
fit=s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment.rs'];gen=s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment/generation.rs']
assert b'cache.failed = true;' in fit and b'models: HashMap<FitKey, Result<CachedEquipment, String>>' in fit and b'.map_err(|error| format!("{error:#}"));' in gen and b'.insert((ProceduralEquipmentResolved, ProceduralEquipmentFailed))' in gen
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=contents(x['preserved_bytes']);assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256'];assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in [(444,[1479]),(445,[1480,1481,1482])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
receipt=json.loads(records['adler/READ-L11-210.json']);row=next(x for x in receipt['sources'] if x['original_index']==1481);assert row['fresh_reads']==[]
future={x['original_index']:x for x in receipt['connected_reinspection']};assert all(future[i]['additional_original_frontier_credit']==0 for i in (1493,1495))
check=json.loads(records['adler/VERIFY-L11-210.json']);assert check['receipt_sha256']==m['adler/READ-L11-210.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-210.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (60,61,1278,1902));old=load('baseline-059/queue.json')['entries'];assert old[1278]['first_failed_contract_or_unresolved_question']['source_original_index']==1479
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];historical=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
for x in a['conditional_findings']:assert sum(y['id']==x['id'] for y in ledger)==1 and x['repair']=='not started' and x['validation']=='not performed'
assert all(x['id'] in historical for x in a['conditional_findings'][:2]) and a['conditional_findings'][2]['id'] not in historical
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified three original preview/readiness bodies, prior no-credit qualifications and bounded producer/status links; no product execution.')
