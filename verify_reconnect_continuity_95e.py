#!/usr/bin/env python3
"""Verify reconnect source evidence and attribution without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
 d={};i=0
 while i<len(body):
  end=body.index(b'\0',i);mode,n=body[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),body[end+1:end+21].hex());i=end+21
 assert i==len(body);return d
def chain(path,proof,body,start,nodes):
 assert [x['name'] for x in proof['chain']]==path.split('/');current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';earlier='e2755a732c84e67296b5e3441a4259a84f461b2f';pre='c100638b720a7281a5fc96f6a3c5c23d5dda85da';name='ASSESSMENT-RECONNECT-CONTINUITY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2067-2071.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
eproof=load(proof['earlier_git_comparison']['commit_and_shared_trees']);nodes.update(eproof['additional_tree_objects']);nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2067:3497,2068:101,2069:162,2070:139,2071:108};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status'] in ('unassessed','bounded-current-trace-only') and item['prior_current_observations']==row.get('current_observations',[]) and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
  if i in (2067,2070):
   assert 'c100 predecessor' in item['prior_worklist_observation_revision_qualification']
   for prior in item['prior_current_observations']:
    doc=load(prior['assessment']);assert doc.get('revision',doc.get('current_revision',doc.get('source_revision')))==pre
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
comparison=proof['earlier_git_comparison'];commit=contents(eproof['commit']['preserved_bytes']);assert oid('commit',commit)==comparison['revision']==earlier and commit.splitlines()[0]==b'tree '+comparison['root_tree'].encode();body=contents(comparison['current_preserved_bytes']);assert body==current[2067] and oid('blob',body)==tree[comparison['path']];chain(comparison['path'],comparison['proof'],body,comparison['root_tree'],nodes)
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
projection=current[2067];facing=current[2069];ingress=sources['crates/adventuresim-tactical-server/src/combat/ingress.rs'];start=sources['crates/adventuresim-tactical-server/src/combat/ingress/attack_start.rs'];defense=sources['crates/adventuresim-tactical-server/src/combat/defense.rs'];ranged=sources['crates/adventuresim-tactical-server/src/combat/ranged.rs'];melee=sources['crates/adventuresim-tactical-server/src/combat/melee.rs'];turn=sources['crates/adventuresim-tactical-core/src/animation/state/turn_deadline.rs'];enrollment=sources['crates/adventuresim-tactical-server/src/mission/enrollment.rs'];systems=sources['crates/adventuresim-tactical-server/src/mission/systems.rs']
struct=projection.split(b'pub(crate) struct ProjectedPlayerSnapshot {',1)[1].split(b'\n}',1)[0];assert b'quickstep_push: QuickstepPush' in struct and b'melee_authority: MeleeAttackAuthority' in struct and b'pending_melee_contact: Option<PendingMeleeContact>' in struct and b'AttackFacing' not in struct and b'PendingDefenderResponse' not in struct
assert b'projection.insert(&mut commands, client)' in projection and b'commands.entity(disconnected).despawn()' in projection and b'pending_melee_contact: motion.13.copied()' in projection
assert b'cmd.entity(event.defender).insert(PendingDefenderResponse' in ingress and b'commands.entity(attacker).remove::<PendingMeleeContact>()' in ingress and b'target: event.target' in start and b'begin_attack_facing(' in start
assert b'let Ok(defender_view) = viewer.get(event.target)' in melee and b'let Some(completion) = MeleeCompletion::prepare(' in melee
assert b'let Some(pending) = pending else' in defense and b'guard == WeaponGuardState::Raised && action == SkeletonAction::None' in defense and b'DefenderResponse::None' in defense and b'pending.get(target).ok()' in ranged
assert b'if let Some(attack_facing) = self.attack' in facing and b'attack_facing.advance_rotation' in facing and b'advance_body_facing_with_turn(' in facing and b'controller_yaw(controller_orientation) * Vec3::NEG_Z' in turn
assert b'const RECONNECT_GRACE_SECS: f32 = 30.0' in projection and b'PARTY_RECONNECT_GRACE: Duration = Duration::from_secs(10)' in systems and b'if loaded_party == 0 && !has_loading_player' in enrollment and b'EnrollmentEffect::Abandoned => {' in systems
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((696,[2067]),(697,[2068,2069,2070,2071])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((323,[2067]),(324,[2068,2069,2070,2071])):
 data=json.loads(records[f'adler/READ-L11-{n}.json'])
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
r=json.loads(records['root-l1-current-e275-20261029/READ-ROOT-L1-CURRENT-027.json']);row=next(x for x in r['rows'] if x['git_path']==comparison['path']);reuse=row['qualified_same_lens_reuse'];assert row['fresh_personal_body_ranges']==[[1,250]] and reuse['qualified_inherited_tail_ranges']==[[251,3497]] and reuse['inherited_assessed_ranges']==[[1,3497]] and r['historical_partial'] in records and r['registration'] in records
native=a['older_native_evidence'];raw=contents(native['preserved_bytes']);older=load(native['path_proof']);assert sha(raw)==native['sha256']==older['path_proof']['sha256']==reuse['raw_proof_sha256'] and oid('blob',raw)==older['path_proof']['git_blob'];line=raw.splitlines()[native['line']-1];trace=json.loads(line);assert sha(line)==native['line_sha256'] and trace['id']==native['receipt_id']==reuse['receipt_id'] and trace['source_reads'][0]['sha256']==sha(current[2067]) and trace['source_reads'][0]['read_ranges']==[[1,3497]] and 'workspace revision' in native['qualification']
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (762,763,764,1777,1778,1946,1965,2005,2007):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];finding=a['conditional_findings'][0];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed';assert next(x for x in ledger if x['id']=='REVIEW-MELEE-REJECTED-CONTACT-STALE-TRADE-001')['reconnect_missing_target_followup']==name
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical roles, native026 and delegated attributions, and bounded reconnect source links; no product execution.')
