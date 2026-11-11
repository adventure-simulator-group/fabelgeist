#!/usr/bin/env python3
"""Verify source evidence for combat effect ownership, without product execution."""
import difflib,gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009'
a=load('ASSESSMENT-COMBAT-EFFECTS-SURFACE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1470-1471.json');w=load('COVERAGE-L01-ROLES.json');tree={}
for line in (r/w['current_tree_inventory']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,oid=head.split();tree[path]=oid
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==[1470,1471] and [x['lines'] for x in h['historical_source_roles']]==[308,337]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  role=w['roles'][x['original_index']];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1470-1471.json'
  assert x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==role['historical_sha256']==x['sha256'] and len(raw.splitlines())==x['lines']
  comp=x['current_comparison'];cur=gzip.decompress((r/comp['preserved_bytes']).read_bytes())
  assert (raw==cur)==comp['byte_identical'] and sha(cur)==comp['sha256'] and len(cur)==comp['bytes']
  assert blob(cur)==comp['git_blob']==tree[comp['path']] and len(cur.splitlines())==comp['lines'];current[x['original_index']]=cur
  if x['original_index']==1470:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-1470/'+comp['path'],tofile='95e/'+comp['path'])).encode()
   assert d==(r/comp['diff']).read_bytes() and sha(d)==comp['diff_sha256'] and b'+fn preload_impact_sounds()' in d and len(cur.splitlines())==321
  else: assert raw==cur
s={}
for x in a['current_supporting_sources']:
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert x['revision']==rev and len(raw)==x['bytes'] and sha(raw)==x['sha256'] and len(raw.splitlines())==x['lines']
 assert blob(raw)==x['git_blob']==tree[x['path']] and all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=raw
native={};nm=load('native-evidence/bevy-replicon-0.41.1/MANIFEST.json');lookup={x['path']:x for x in nm['files']}
for x in a['native_connected_sources']:
 raw=(r/x['path']).read_bytes();rel=x['path'].split('bevy-replicon-0.41.1/',1)[1];assert sha(raw)==x['sha256']==lookup[rel]['sha256'] and len(raw)==x['bytes']==lookup[rel]['bytes'];assert all(1<=lo<=hi<=len(raw.splitlines()) for lo,hi in x['ranges']);native[rel]=raw
assert b'name = "bevy_replicon"\nversion = "0.41.1"' in s['Cargo.lock'] and nm['crate_sha256'].encode() in s['Cargo.lock']
ctx=native['src/shared/message/ctx.rs'];receiver=native['src/shared/message/server_message.rs'];assert b'self.invalid_entities.push(source)' in ctx and b'if ctx.invalid_entities.is_empty()' in receiver and b'ctx.invalid_entities.clear();\n            Err(msg.into())' in receiver
assert b'message.map_entities(ctx);' in receiver
combat=current[1470];assert combat.index(b'let Some(sound_seed) = combat_sound_seed(')<combat.index(b'if event.impact_effects.metal_sparks {')<combat.index(b'if event.impact_effects.blood {')
assert b'let character_id = identities.get(attacker).ok()?.0;' in combat and b'Local<std::collections::BTreeMap<u64, u64>>' in combat
surface=current[1471];assert b'global.affine().inverse().transform_point3(world_point)' in surface and b'.min_by(|left, right| left.distance_squared.total_cmp(&right.distance_squared))' in surface
assert b'closest.distance_squared(point)' in surface and b'is_descendant_of(*entity, target, parents)' in surface
assert Fraction(1,5)**2<Fraction(3,10)**2 and 10*Fraction(1,5)>Fraction(3,10)
net=s['crates/adventuresim-tactical-netcode/src/message.rs'];assert b'#[entities]\n    pub attacker: Entity' in net and b'#[entities]\n    pub hit: Vec<Entity>' in net
rep=s['crates/adventuresim-tactical-netcode/src/replication.rs'];assert b'.replicate::<CharacterId>()' in rep and b'.add_mapped_server_event::<SuccessfulAttackResponse>(Channel::Ordered)' in rep
server=s['crates/adventuresim-tactical-server/src/combat.rs'];assert b'physical_contact: false,' in server and b'=> ImpactSound::None' in server and b'blood: matches!(result, AttackResult::ToDefender' in server
assert b'CharacterId(player.character.id)' in s['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'commands.entity(player).with_children' in s['crates/adventuresim-tactical-client/src/animation/loading.rs'] and b'insert(ChildOf(rig_scene))' in s['crates/adventuresim-tactical-client/src/equipment/placeholder_update.rs']
assert b'commands.entity(entity).with_child(part.render_bundle(' in s['crates/adventuresim-tactical-client/src/equipment/runtime_equipment/generation.rs'] and b'.insert(SkinnedMesh {' in s['crates/adventuresim-tactical-client/src/equipment/skin.rs']
manifest=json.loads(gzip.decompress((r/'recovered/original-transport-manifest.json.gz').read_bytes()));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 orig=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(orig['sha256'],orig['bytes'],orig['git_blob'])
 raw=gzip.decompress((r/x['preserved_bytes']).read_bytes());assert len(raw)==x['published_bytes'] and sha(raw)==x['published_sha256']
 assert x['redacted_local_path_occurrences'] or (sha(raw)==x['sha256'] and blob(raw)==x['git_blob']);records[x['original_path']]=raw
for batch,ids in {435:[1470],436:[1471],687:[2042,2043,2044,2045]}.items():
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==ids and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256']
receipt=json.loads(records['adler/READ-L11-208.json']);state=json.loads(records['adler/STATE-L11-208.json']);check=json.loads(records['adler/VERIFY-L11-208.json'])
assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert state['receipt_sha256']==check['receipt_sha256']==m['adler/READ-L11-208.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-208.json']['sha256']
for n in (2,3):
 row=json.loads(records[f'root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-{n:03}.json']);assert row['accepted_unique_count_change']==0 and row['original_overlay_merged_shared_credit']==0 and row['other_lens_credit']==0
qualification=json.loads(records['root-l1-current-e275-20261029/QUALIFICATION-ROOT-L1-CURRENT-003-001.json']);assert qualification['coverage_change']==0 and qualification['accepted_unique_count_change']==0 and qualification['receipt_sha256']==m['root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-003.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1269,1270)]==[1470,1471];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (56,57,289,1269,1270))
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];historical=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
for x in a['conditional_findings']:assert x['id'] in historical and sum(y['id']==x['id'] for y in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified two historical combat bodies, prior qualifications and bounded event/surface links; no product execution.')
