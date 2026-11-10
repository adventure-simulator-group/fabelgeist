"""Verify source and archive provenance for a bounded audit observation."""
import gzip,hashlib,json,tarfile,tomllib
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-PROTECTIVE-CONTACT-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==20

def contains(path,*markers):
 text=source[path]
 for m in markers:
  if m not in text: raise ValueError('missing '+path+': '+m)
 return text
contains('crates/adventuresim-stdb-module/src/tactical.rs','quantity: inventory_item.quantity,','protected_body_parts,','occupancies,','definition.kind == CatalogItemKind::Armor','crate::repair::apply_impact(ctx, contact.inventory_item_id, contact.contact_stress)')
contains('crates/adventuresim-stdb-module/src/item.rs','item.coverage = protection.coverage;','item.kind = CatalogItemKind::Clothing;')
contains('crates/adventuresim-stdb-module/src/repair.rs','pub(crate) fn apply_impact(','damage_from_impact(','inventory_item_id()')
projection=contains('crates/adventuresim-tactical-server/src/player_projection.rs','cmd.spawn_empty().id()','TacticalInventoryItemId(item.inventory_item_id)','ItemOf(entity),','if let Some(armor) = projected_armor(item)','occupancy.channel == adventuresim_stdb_client::EquipmentChannel::Held','item_cmd.insert(EquipSlot::HoldingLeft)','item_cmd.insert(EquipSlot::HoldingRight)')
projected=projection.split('let tactical_items: BTreeMap',1)[1].split('[startup] Player {entity:?} is fully loaded',1)[0]; assert projected.count('item_cmd.insert(EquipSlot::')==2
contains('crates/adventuresim-tactical-server/src/player_projection/projection_equipment.rs','let part = item.protected_body_parts.first()?;','coverage_geometry[index] = Some(geometry);','Some(ArmorItem {','covered_parts: tactical_covered_parts(&item.protected_body_parts)')
inv=contains('crates/adventuresim-tactical-core/src/inventory.rs','#[require(ItemProperties, TacticalItemQuantity)]','pub slot: Option<&\'static EquipSlot>','pub armor: Option<&\'static ArmorItem>','pub inventory_item_id: Option<&\'static TacticalInventoryItemId>','self.armor_layer_chain(part, sample)'); assert 'EquipSlot' not in inv.split('#[require(ItemProperties, TacticalItemQuantity)]',1)[0][-10:]
contains('crates/adventuresim-tactical-core/src/inventory/query.rs','.flat_map(|inventory| inventory.iter())','self.q_item.iter_many(items)')
contains('crates/adventuresim-tactical-core/src/inventory_armor.rs','armor.covered_parts[index] && armor.coverage > 0.0','position(|(_, armor)|','inventory_item_id: item.inventory_item_id.map(|id| id.0)','surface: adventuresim_core::equipment::ArmorSurface')
contains('crates/adventuresim-core/src/item_catalog.rs','ItemKind::Clothing =>','protection.coverage = coverage;')
coverage=contains('crates/adventuresim-core/src/combat/coverage.rs','coordinate >= self.start && coordinate < self.end','SurfaceAnchor::Proximal => (0.0, coverage)','Region::LeftThigh => (BodyPart::LeftLeg, 0.0, 0.5','Region::LeftLowerLeg => (BodyPart::LeftLeg, 0.5, 1.0')
contains('crates/adventuresim-core/src/combat.rs','defender_equip.armor_surface(defender_body_part, 0.5)','let energy = armor::resolve_contact_energy(armor_surface, attack, attack_force, precision);','armor_impact: energy.armor_impact')
contains('crates/adventuresim-core/src/combat/armor.rs','let Some(surface) = surface else','let impact = ArmorImpact {','armor_impact: Some(impact)')
contains('crates/adventuresim-tactical-server/src/combat/ranged.rs','let result = attacker_view.resolve_ranged_attack(','outcome::RangedContact {')
contains('crates/adventuresim-tactical-server/src/combat/ranged/outcome.rs','commands.trigger(ApplyMeleeAttackResult {','result: response.result,')
cons=contains('crates/adventuresim-tactical-server/src/combat/consequence.rs','&\'static ItemOf,','&\'static EquipSlot,','&\'static TacticalInventoryItemId,','Option<&\'static ArmorItem>','impact.surface.inventory_item_id,','owner.0 == event.target','provenance.0 == inventory_item_id','record_equipment_contact(')
contains('crates/adventuresim-tactical-server/src/mission/receipt.rs','equipment_contacts: accumulated','TacticalEquipmentContactRole::DefenderEquipment')
contains('crates/adventuresim-tactical-server/src/iteration/equipment.rs','fn spawn_armor(','EquipSlot::from_armor_body_part(part)')
contains('crates/adventuresim-core/src/starting_character.rs','item("linen_breeches", 1, Some(LoadoutSlot::LeftLeg))')
items=json.loads(source['content/items/catalog.yaml'])['items']; row=next(x for x in items if x['id']=='linen_breeches'); assert row['kind']=='clothing' and row['capabilities']['durability']['wear']>0; placement=next(x for x in row['equipment']['placements'] if x['id']=='worn'); assert {x['channel'] for x in placement['occupancy']}=={'base_clothing'} and 'left_leg' in placement['protection']; surface=placement['surface'][0]; assert surface['regions']==['left_thigh','left_lower_leg'] and surface['anchor']=='proximal' and surface['coverage']==0.94; assert 0.5<=0.5<surface['coverage']
native=doc['native_sources'][0]; manifest_path=(root/native['manifest']).resolve(); assert manifest_path.is_relative_to(root); m=json.loads(manifest_path.read_text()); archive=manifest_path.parent/m['archive']; raw_archive=archive.read_bytes(); assert hashlib.sha256(raw_archive).hexdigest()==native['archive_sha256']==m['sha256']; lock=tomllib.loads(source['Cargo.lock']); packages=[p for p in lock['package'] if p['name']=='bevy_ecs' and p['version']==m['version']]; assert len(packages)==1 and packages[0]['checksum']==m['sha256']; members={}
with tarfile.open(archive) as tar:
 for item in native['inspected_members']:
  raw=tar.extractfile(item['path']).read(); assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'] and all(1<=a<=b<=len(raw.splitlines()) for a,b in item['ranges']); members[item['path']]=raw.decode()
fetch=members['bevy_ecs-0.19.1/src/query/fetch.rs']; wq=members['bevy_ecs-0.19.1/src/query/world_query.rs']; assert 'unsafe impl<T: Component> WorldQuery for &T' in fetch and 'set_contains_id(state)' in fetch and 'true $(&& $name::matches_component_set($name, set_contains_id))*' in wq
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); assert h['checkpoint']=='059'
def find_record(value):
 if isinstance(value,dict):
  if value.get('id')==h['related_id']: yield value
  for v in value.values(): yield from find_record(v)
 elif isinstance(value,list):
  for v in value: yield from find_record(v)
records=list(find_record(ledger)); assert len(records)==1 and records[0]['accepted_unique_count_change']==0 and 'Native validate_tactical_receipt' in records[0]['first_failed_contract'] and 'projected/selected/recorded' in records[0]['first_failed_contract']
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in manifest['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified 20 Git sources, lock-bound Bevy query source and separately attributed historical roles')
