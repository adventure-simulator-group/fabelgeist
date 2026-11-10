"""Verify a bounded source trace and provenance, without product execution."""
import gzip,hashlib,json,tarfile,tomllib
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-PROTECTION-TOPOLOGY-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==16

def contains(path,*markers):
 text=source[path]
 for m in markers:
  if m not in text: raise ValueError('missing '+path+': '+m)
 return text
contains('crates/adventuresim-tactical-client/src/equipment.rs','Some(GrabSelection::Slot { location, depth }) => EquipmentAction::Slot','commands.client_trigger(EquipmentActionRequest {','expected_revision: action_state.revision')
contains('crates/adventuresim-tactical-netcode/src/message.rs','pub struct EquipmentActionRequest {','pub enum EquipmentAction {','expected_destination: Option<Entity>')
server=contains('crates/adventuresim-tactical-server/src/equipment.rs','if request.actor != controlled','if request.expected_revision != state.revision','if authoritative_hand_item != request.expected_hand_item','ordered_at_location(controlled, location, &items)','transfer_slot(','fn has_children(','channel: EquipmentChannel::Held')
transfer=contains('crates/adventuresim-tactical-server/src/equipment/transfer.rs','None => {','Some(ReachableTarget::Occupied(item)) = destination','if has_children(item, items)','insert((hand_topology(hand), hand.slot()))','insert(proposed)','.remove::<EquipSlot>()'); assert 'ArmorItem' not in transfer and 'remove::<ItemOf>' not in transfer
contains('crates/adventuresim-tactical-core/src/inventory/ordering.rs','fn root_order','TacticalEquipmentAnchor::CharacterLocation(location)')
contains('crates/adventuresim-stdb-module/src/tactical.rs','let placement = worn.as_ref().and_then','placement.protection.clone()','definition.kind == CatalogItemKind::Armor')
contains('crates/adventuresim-tactical-server/src/player_projection.rs','TacticalInventoryItemId(item.inventory_item_id)','ItemOf(entity),','if let Some(armor) = projected_armor(item)','item_cmd.insert(armor)')
contains('crates/adventuresim-tactical-server/src/player_projection/projection_equipment.rs','let part = item.protected_body_parts.first()?;','Some(ArmorItem {','covered_parts: tactical_covered_parts(&item.protected_body_parts)')
inv=contains('crates/adventuresim-tactical-core/src/inventory.rs','pub struct EquipmentTopology {','pub slot: Option<&\'static EquipSlot>','pub armor: Option<&\'static ArmorItem>','self.armor_layer_chain(part, sample)','fn on_equip_slot_replaced','fn on_equip_slot_removed'); hooks=inv.split('fn on_equip_slot_replaced',1)[1].split('#[cfg(test)]',1)[0]; assert 'ArmorItem' not in hooks
contains('crates/adventuresim-tactical-core/src/inventory/armor.rs','pub struct ArmorItem {','pub covered_parts: [bool; 7]')
contains('crates/adventuresim-tactical-core/src/inventory/query.rs','.flat_map(|inventory| inventory.iter())','self.q_item.iter_many(items)')
contains('crates/adventuresim-tactical-core/src/inventory_armor.rs','armor.covered_parts[index] && armor.coverage > 0.0','armor.coverage_geometry[index]','surface: adventuresim_core::equipment::ArmorSurface')
contains('crates/adventuresim-tactical-server/src/combat/consequence.rs','&\'static EquipSlot,','impact.surface.inventory_item_id,','provenance.0 == inventory_item_id')
contains('crates/adventuresim-core/src/starting_character.rs','item("linen_breeches", 1, Some(LoadoutSlot::LeftLeg))')
items=json.loads(source['content/items/catalog.yaml'])['items']; row=next(x for x in items if x['id']=='linen_breeches'); placement=next(x for x in row['equipment']['placements'] if x['id']=='worn'); assert 'left_leg' in placement['protection'] and {x['channel'] for x in placement['occupancy']}=={'base_clothing'}
native=doc['native_sources'][0]; mp=(root/native['manifest']).resolve(); assert mp.is_relative_to(root); m=json.loads(mp.read_text()); archive=mp.parent/m['archive']; assert hashlib.sha256(archive.read_bytes()).hexdigest()==native['archive_sha256']==m['sha256']; lock=tomllib.loads(source['Cargo.lock']); packages=[p for p in lock['package'] if p['name']=='bevy_ecs' and p['version']==m['version']]; assert len(packages)==1 and packages[0]['checksum']==m['sha256']; members={}
with tarfile.open(archive) as tar:
 for item in native['inspected_members']:
  raw=tar.extractfile(item['path']).read(); assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'] and all(1<=a<=b<=len(raw.splitlines()) for a,b in item['ranges']); members[item['path']]=raw.decode()
cmd=members['bevy_ecs-0.19.1/src/system/commands/mod.rs']; ec=members['bevy_ecs-0.19.1/src/system/commands/entity_command.rs']; assert 'pub fn insert(&mut self, bundle: impl Bundle)' in cmd and 'overwrit' in cmd and 'pub fn remove<B: Bundle>' in cmd and 'entity.insert_with_caller(bundle, mode, caller' in ec and 'entity.remove_with_caller::<T>(caller)' in ec
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); assert h['checkpoint']=='059' and h['related_id'] in json.dumps(ledger); roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in manifest['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial'); assert any(x['id']=='REVIEW-SLOTLESS-PROTECTIVE-CONTACT-OMISSION-001' for x in entries)
print('verified 16 Git sources, lock-bound Bevy command source and separately attributed role records')
