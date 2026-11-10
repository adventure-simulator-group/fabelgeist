"""Verify current source, native pose evidence and historical pickup attribution."""
import gzip,hashlib,json,tarfile,tomllib
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-PICKUP-FRAME-95E.json'; doc=json.loads((root/name).read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); out[p]=oid
 return out
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root); raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert oid==s['git_blob']==cur[s['path']]==prev[s['path']] and s['revision']==rev and len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==11
server=source['crates/adventuresim-tactical-server/src/equipment.rs']; projection=source['crates/adventuresim-tactical-server/src/player_projection.rs']; core=source['crates/adventuresim-tactical-core/src/inventory.rs']; grab=source['crates/adventuresim-tactical-client/src/equipment/grab_world.rs']; ui=source['crates/adventuresim-tactical-client/src/equipment.rs']; targeting=source['crates/adventuresim-tactical-client/src/targeting.rs']; visual=source['crates/adventuresim-tactical-client/src/equipment/placeholder_visual.rs']; net=source['crates/adventuresim-tactical-netcode/src/replication.rs']; catalog=json.loads(source['content/items/catalog.yaml']); halberd=next(x for x in catalog['items'] if x['id']=='halberd')['equipment']['physical']
assert halberd['dimensions_m']==[0.16,2.1,0.08] and halberd['grip_to_tip_m']==1.9 and halberd['anchor_offset_m']==[0.0,-0.2,0.0]
for x in ('const PICKUP_RANGE_M: f32 = 2.0;', 'Collider::compound(vec![(-physical.anchor_offset_m, Quat::IDENTITY, shape)])','Transform::from_translation(position)','RigidBody::Dynamic','grip - physical.anchor_offset_m','item_box_center(item_transform.translation, physical)','item_position.distance_squared(actor_transform.translation) > PICKUP_RANGE_M * PICKUP_RANGE_M','SpatialQueryFilter::from_mask(TACTICAL_TERRAIN_LAYER)'): assert x in server
catalog_owner=source['crates/adventuresim-core/src/item_catalog.rs']
for x in ('let physical = adventuresim_weapon_model::derive_properties(&design)', 'equipment.physical.grip_to_tip_m = physical.grip_to_tip_m;', 'equipment.physical.dimensions_m[1] = physical.length_m;'): assert x in catalog_owner
assert 'equipment.physical.anchor_offset_m' not in catalog_owner
for x in ('let mut physical = equipment.physical','if let Some(instance) = instance_geometry','physical.anchor_offset_m[1] = -(instance.total_length_m - instance.grip_to_tip_m)','anchor_offset_m: Vec3::from_array(physical.anchor_offset_m)'): assert x in projection
for x in ('self.dimensions_m.cmpgt(Vec3::ZERO).all()','self.anchor_offset_m.is_finite()'): assert x in core
for x in ('transform.transform_point(-physical.anchor_offset_m)','self.visible(origin, position)','PICKUP_RANGE_M'): assert x in grab
assert 'Some(GrabSelection::SceneItem(item)) => EquipmentAction::Pickup { item }' in ui and 'commands.client_trigger(EquipmentActionRequest' in ui and 'position.distance_squared(actor_origin)' in targeting and 'Transform::from_translation(-physical.anchor_offset_m)' in visual and 'add_mapped_client_event::<EquipmentActionRequest>' in net
lock=gzip.decompress((root/'main-advance-95e/basis-wire-sources/01.source.gz').read_bytes()); assert hashlib.sha1(b'blob '+str(len(lock)).encode()+b'\0'+lock).hexdigest()==cur['Cargo.lock']; pkgs=tomllib.loads(lock.decode())['package']; bodies={}
for path in doc['native_evidence']:
 m=json.loads((root/path).read_text()); d=(root/path).parent; raw=(d/m['archive']).read_bytes(); assert len(raw)==m['bytes'] and hashlib.sha256(raw).hexdigest()==m['sha256']; p=[x for x in pkgs if x['name']==m['package'] and x['version']==m['version']]; assert len(p)==1 and p[0]['checksum']==m['sha256']
 with tarfile.open(d/m['archive']) as t:
  for x in m['inspected_members']:
   b=t.extractfile(x['path']).read(); assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256']; bodies[x['path']]=b.decode()
assert 'c.into().shape_scaled().clone()' in bodies['avian3d-0.7.0/src/collision/collider/parry/mod.rs'] and 'shape.compute_aabb(delta)' in bodies['parry3d-0.27.0/src/shape/compound.rs'] and 'self.rotation * p + self.translation' in bodies['glamx-0.2.0/src/pose3.rs'] and 'self.0.transform_point3(point)' in bodies['bevy_transform-0.19.1/src/components/global_transform.rs']
h=doc['historical_source']; old=json.loads((root/h['ledger']).read_text())['discovery_findings_and_dependencies'][h['contract_id']]; assert old['accepted_unique_count_change']==0 and source['crates/adventuresim-tactical-server/src/equipment.rs'] and doc['current_sources'][0]['sha256'] in old['source_key']; roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as t:
 for item in h['source_roles']:
  r=roles[item['index']]; assert r['source_id']==item['source_id'] and r['historical_sha256']==item['sha256'] and hashlib.sha256(t.extractfile(item['sha256']).read()).hexdigest()==item['sha256']; path=item['source_id'].split(':',1)[1]; current=hashlib.sha256(source[path].encode()).hexdigest(); assert item['current_relation']==('byte-match' if item['sha256']==current else 'different')
client_distance=Fraction(39,20)**2; server_distance=Fraction(43,20)**2+Fraction(1,5)**2; assert client_distance<4<server_distance
correction=json.loads((root/doc['correction']).read_text()); assert correction['record']==name and 'dimensions are [0.16,2.1,0.08]' in correction['previous_catalog_case'] and correction['current_qualification']==doc['candidates'][0]['catalog_case'] and 'not established final projected dimensions' in correction['current_qualification']
c=doc['candidates'][0]; b=json.loads((root/'BASELINE-95E-20261110.json').read_text())['newly_assessed_ledger']['entries']; p=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert sum(x['id']==c['id'] and x['record']==name for x in b)==1 and p['new_current_ledger_candidates']==len(b) and c['discovery'].startswith('partial') and c['repair']=='not started' and c['validation']=='not performed'
print('verified source-bound pickup frame chain, conditional geometry and qualified 059 record')
