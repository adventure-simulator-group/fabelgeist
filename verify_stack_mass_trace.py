"""Verify preserved audit source evidence, not product execution."""
import gzip, hashlib, json, tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-STACK-MASS-95E.json'
doc=json.loads((root/name).read_text()); baseline=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 result={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in result; result[p]=oid
 return result
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); sources={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
 assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]
 assert s['path'] not in sources and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); sources[s['path']]=raw.decode()
assert len(sources)==21

def contains(path,*markers):
 value=sources[path]
 for marker in markers:
  if marker not in value: raise ValueError('missing marker '+path+' '+marker)
 return value
contains('crates/adventuresim-stdb-module/src/tactical.rs','let items = connected_player_items(ctx, character.id).collect();','let mut item = ctx.db.item().id().find(&inventory_item.item_id)?;','quantity: inventory_item.quantity,','weapon_appearance,','weapon_holder_appearance,')
contains('crates/adventuresim-stdb-module/src/item.rs','weight: definition.weight_kg,')
contains('crates/adventuresim-stdb-module/src/capability/mass.rs','row_is_fireplace_rooted','return Some(lot.mass_kg.max(0.0));','personal_fraction(ctx, inventory.id)','map_or(inventory.quantity as f32, |fraction| fraction.as_unit_f32())','unit_mass * effective_quantity')
contains('crates/adventuresim-stdb-module/src/inventory_amount.rs','pub fn personal_fraction(','.inventory_item_amount()')
contains('crates/adventuresim-stdb-module/src/weapon_instance/appearance.rs','if !matches!(item_id, "scabbard" | "weapon_loop")','return None;')
contains('crates/adventuresim-stdb-module/src/weapon_instance.rs','pub(crate) fn fitted_holder_mass(','.weapon_holder_instance()','return Ok(None);')
contains('crates/adventuresim-tactical-server/src/player_projection.rs','TacticalItemQuantity::new(item.quantity)','ItemOf(entity),\n            quantity,','weight: instance_geometry','projected_holder_mass(item)','unwrap_or(item.item.weight)')
contains('crates/adventuresim-tactical-server/src/player_projection/projection_equipment.rs','let Some(appearance) = item.weapon_appearance.as_ref() else','default_design(&item.item.id).is_none()','let appearance = item.weapon_holder_appearance.as_ref()?;')
contains('crates/adventuresim-tactical-core/src/inventory.rs','pub quantity: &\'static TacticalItemQuantity','pub properties: &\'static ItemProperties','self.iter().map(|item| item.properties.weight).sum()','fn weapon_weight(&self) -> f32')
contains('crates/adventuresim-tactical-core/src/inventory/query.rs','.flat_map(|inventory| inventory.iter())','self.q_item.iter_many(items)')
contains('crates/adventuresim-tactical-core/src/inventory/quantity.rs','pub const fn new(value: u32) -> Option<Self>','ItemQuantity::new(value)')
contains('crates/adventuresim-tactical-core/src/player.rs','let inventory = self.inventory.get(entity);','.with_equipment(inventory)')
contains('crates/adventuresim-tactical-core/src/physics.rs','let burden = player.body_weight() + player.inventory_weight();','mass.0 = mass_kg;','insert(Mass(mass_kg))')
contains('crates/adventuresim-tactical-core/src/physics/movement_speed.rs','let burden = player.body_weight() + player.inventory_weight();','let sprint = tactical_sprint_speed(')
contains('crates/adventuresim-tactical-server/src/combat/condition.rs','let burden = view.body_weight() + view.inventory_weight();','combat_movement_fatigue_per_second(','view.inventory_weight(),')
contains('crates/adventuresim-tactical-server/src/combat.rs','CoreItemQuantity::new(quantity.get() - 1)')
contains('crates/adventuresim-tactical-server/src/combat/ranged.rs','properties.id == ARROW_ID','remaining: remaining_ammo_after_shot(quantity.0)','insert(TacticalItemQuantity(remaining))','commands.entity(self.entity).despawn()','authority.authorize_shot(validated, now, cooldown)','ammunition.consume(&mut cmd, &q_ids, &mut consequences, &shot);')
contains('crates/adventuresim-core/src/starting_character.rs','item("arrow", if adult { 28 } else { 44 }, None)')
contains('crates/adventuresim-weapon-model/src/catalog.rs','include_str!("../catalog/gameplay.json")','.find(|design| design.catalog_id == catalog_id)')
items=json.loads(sources['content/items/catalog.yaml'])['items']; arrow=[i for i in items if i['id']=='arrow']; assert len(arrow)==1 and arrow[0]['weight_kg']==0.05 and arrow[0]['kind']=='simple'
gameplay=json.loads(sources['crates/adventuresim-weapon-model/catalog/gameplay.json']); assert all(x['catalog_id']!='arrow' for x in gameplay)
assert abs(28*arrow[0]['weight_kg']-1.40)<1e-12 and abs((28-1)*arrow[0]['weight_kg']-1.35)<1e-12
h=doc['historical_source']; assert h['checkpoint']=='059'; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['ADLER-L13-STACK-MASS-RECEIVER']; assert old['accepted_unique_count_change']==0 and old['repair_status'].startswith('not started') and 'PL13-STACK-MASS-CONNECTION' in json.dumps(ledger)
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; manifest=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in manifest['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  i=x['index']; r=roles[i]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[i] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']
  path=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(sources[path].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=baseline['newly_assessed_ledger']['entries']; assert progress['new_current_ledger_candidates']==len(entries) and len(doc['candidates'])==1
c=doc['candidates'][0]; matches=[x for x in entries if x['id']==c['id']]; assert len(matches)==1 and matches[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified 21 Git-bound sources and separately attributed stack-mass evidence')
