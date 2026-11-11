#!/usr/bin/env python3
"""Verify player and quickstep source evidence; never execute product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-PLAYER-QUICKSTEP-TRACE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1498-1501.json');w=load('COVERAGE-L01-ROLES.json');proof=load('PLAYER-TRACE-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']]==[1498,1499,1500,1501] and [x['lines'] for x in h['historical_source_roles']]==[944,201,453,128]
assert h['historical_source_roles'][0]['prior_role_status']=='bounded-current-trace-only' and h['historical_source_roles'][0]['prior_current_observations']==[{'assessment':'ASSESSMENT-L1-CAMERA.json','ranges':'432-445, 485-487'}]
current={};historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1498-1501.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];historical[i]=raw
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert raw==cur and c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
 old=load('ASSESSMENT-L01-HISTORICAL-333-338.json')['historical_source_roles'][5];prior=ar.extractfile(old['sha256']).read();assert old['original_index']==338 and old['snapshot']=='physical-p16' and prior==historical[1501] and w['roles'][338]['historical_sha256']==old['sha256']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
player=current[1498];clock=current[1499];start=current[1500];trace=current[1501];physics=s['crates/adventuresim-tactical-core/src/physics.rs'];quick=s['crates/adventuresim-tactical-core/src/physics/quickstep.rs'];projection=s['crates/adventuresim-tactical-server/src/player_projection.rs'];lunge=s['crates/adventuresim-tactical-server/src/combat/ingress/lunge.rs'];ingress=s['crates/adventuresim-tactical-server/src/combat/ingress.rs'];net=s['crates/adventuresim-tactical-netcode/src/message.rs'];coreplayer=s['crates/adventuresim-tactical-core/src/player.rs'];equipment=s['crates/adventuresim-core/src/equipment.rs'];composite=s['crates/adventuresim-core/src/composite.rs'];inventory=s['crates/adventuresim-tactical-core/src/inventory.rs'];elapsed=s['crates/adventuresim-core/src/elapsed.rs']
assert b'#[require(\n    CharacterId,' in coreplayer and b'register_required_components_with::<Player' in player
assert b'weapon_ranged_windup_secs()' in start and b'let windup_secs = windup_secs?;' in start and b'#[delegate(PlayerEquipment' in composite and b'InventoryView' in coreplayer and b'fn weapon_ranged_windup_secs' not in inventory and b'fn weapon_ranged_windup_secs(&self)' in equipment and b'AnimationElapsed::from_seconds(0.3)' in equipment and b'if !seconds.is_finite()' in elapsed
assert b'cmd.client_trigger(MeleeActionRequest' in start and b'event.client_id.entity()' in ingress and b'attack_preparation_secs(&view' in ingress and b'pub enum RangedActionRequest' in net
assert trace.index(b'if !changed {') < trace.index(b'let attack_lunge =') and b'previous.action == SkeletonAction::Dodge' in trace and b'self.actuator.start_tick() != previous.actuator.start_tick()' in trace
assert b'snapshot.melee_lunge.is_some_and(|lunge| lunge.quickstep)' in trace and b'(dodge_action || attack_lunge) != push.active' in trace
assert b'!quickstep_action' in quick and b'AnimationSampleSpan::from(2u64)' in quick and b'forward_velocity.abs() > 0.02' in quick
assert b'if push.active && (quickstep_action || attack_quickstep || bounded_recovery)' in physics and b'velocity.xz().dot(world_direction).abs() <= 0.02' in physics and b'quickstep_push: *quickstep_push' in projection and b'melee_lunge: melee_lunge.copied()' in projection
assert b'commands.entity(request.attacker).insert(movement)' in lunge and b'if movement.quickstep {' in lunge and b'push.cancel();\n                    commands.entity(entity).remove::<MeleeLungeMovement>();' in physics
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(453,[1498]),(454,[1499,1500]),(455,[1501])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
receipt=json.loads(records['adler/READ-L11-213.json']);check=json.loads(records['adler/VERIFY-L11-213.json']);assert [x['original_index'] for x in receipt['sources']]==[1498,1499,1500,1501] and receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256']==m['adler/READ-L11-213.json']['sha256'] and check['state_sha256']==m['adler/STATE-L11-213.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1287,1288,1289));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1287,1288,1289)]==[1498,1500,1501]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==2 and all(sum(x['id']==c['id'] and x['record']=='ASSESSMENT-PLAYER-QUICKSTEP-TRACE-95E.json' for x in ledger)==1 for c in a['conditional_findings']) and all(c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical player bodies, prior attribution and bounded quickstep diagnostic source connections; no product execution.')
