#!/usr/bin/env python3
"""Verify mission source histories and attribution without executing product code."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';earlier='e2755a732c84e67296b5e3441a4259a84f461b2f';pre='c100638b720a7281a5fc96f6a3c5c23d5dda85da';name='ASSESSMENT-MISSION-ENROLLMENT-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2061-2066.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
eproof=load(proof['earlier_git_comparison']['commit_and_shared_trees']);nodes.update(eproof['additional_tree_objects']);nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2061:51,2062:726,2063:362,2064:320,2065:303,2066:99};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status'] in ('unassessed','bounded-current-trace-only') and item['prior_current_observations']==row.get('current_observations',[]) and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
  if i==2062:
   assert 'c100 predecessor' in item['prior_worklist_observation_revision_qualification']
   for prior in item['prior_current_observations']:
    doc=load(prior['assessment']);assert doc.get('revision',doc.get('current_revision',doc.get('source_revision')))==pre
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
comparison=proof['earlier_git_comparison'];commit=contents(eproof['commit']['preserved_bytes']);assert oid('commit',commit)==comparison['revision']==earlier and commit.splitlines()[0]==b'tree '+comparison['root_tree'].encode() and len(comparison['paths'])==5
for p,item in comparison['paths'].items():
 body=contents(item['current_preserved_bytes']);assert oid('blob',body)==tree[p] and item['historical_fresh_personal_ranges']==[] and item['historical_qualified_reuse_ranges']==item['historical_assessed_ranges'] and item['frozen_source_adoption_pending_as_recorded'] is True;chain(p,item['proof'],body,comparison['root_tree'],nodes)
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
main=current[2062];facade=current[2063];systems=current[2064];terminal=current[2065];openings=current[2066];enrollment=sources['crates/adventuresim-tactical-server/src/mission/enrollment.rs'];bot=sources['crates/adventuresim-tactical-server/src/bot.rs']
assert b'party_reconnect_grace: f32' in main and b'Duration::from_secs_f32(args.party_reconnect_grace)' in main and b'args.required_enemy_kills = fixture.enemy_count();' in bot
assert b'reconnect_grace: Duration' in facade and b'reconnect_grace,' in facade
assert b'if loaded_party == 0 && !has_loading_player' in enrollment and b'*empty_for = Duration::ZERO;' in enrollment and b'if *empty_for >= reconnect_grace' in enrollment and b'EnrollmentEffect::Abandoned' in enrollment
assert b'state.observe_loaded_party_member(*player_id)' in systems and b'state.begin_enrollment();' in systems and b'EnrollmentEffect::Abandoned => {' in systems and b'TacticalMissionResolution::Failed' in systems
assert b'let frozen = FrozenTerminal {' in systems and b'TerminalSubmissionResult::Accepted' in terminal and b'Self::Presenting {' in terminal
assert b'#[path = "doors.rs"]' in openings and b'pub(crate) use doors::DoorGrabber;' in openings and b'pub(crate) use windows::WindowGrabber;' in openings
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((694,[2061,2062]),(695,[2063,2064,2065,2066])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
 if n==695:
  r3,r4,r5,r6=data['rows'];assert r3['original_requested_ranges']==[[1,158]] and r3['fresh_personal_body_reads']==[[1,158]] and r3['connected_gap_personal_reads']['ranges']==[[159,362]] and r3['connected_gap_personal_reads']['new_population_credit']==0
  assert r4['original_requested_ranges']==[[52,320]] and r4['fresh_personal_body_reads']==[[52,320]] and r4['connected_gap_personal_reads']['ranges']==[[1,51]] and r4['connected_gap_personal_reads']['new_population_credit']==0
  for row in (r3,r4,r5):assert row['exact_locator_qualification'] in records and row['qualification_sha256']==lookup[row['exact_locator_qualification']]['sha256']
  assert r6['fresh_personal_body_reads']==[] and len(r6['same_lens_reuse'])==1 and r6['same_lens_reuse'][0]['fresh_personal_body_reads_this_batch']==[] and r6['same_lens_reuse'][0]['receipt'] in records
adler=json.loads(records['adler/READ-L11-321.json']);item=next(x for x in adler['sources'] if x['original_index']==2063);assert item['fresh_reads']==[[1,158],[159,220]]
adler=json.loads(records['adler/READ-L11-322.json']);item=next(x for x in adler['connected_reinspection'] if x['original_index']==2814);assert item['fresh_reads']==[[1,125]] and item['additional_original_frontier_credit']==0 and work['roles'][2814]['status']=='unassessed'
old=json.loads(records['root-l3-extension-20261030/READ-ROOT-L3-CURRENT-002.json']);assert old['fresh_personal_lines']==0 and old['qualified_reuse_lines']==1067 and len(old['rows'])==5
for row in old['rows']:
 assert row['revision']==earlier and row['fresh_personal_ranges']==[] and row['qualified_reuse_ranges']==row['assessed_ranges'] and row['frozen_source_adoption_pending'] is True
 for k in ('registration_ref','prior_own_lens_receipt','prior_registration_ref'):assert row[k]['path'] in records and row[k]['sha256']==lookup[row[k]['path']]['sha256']
state=json.loads(records['segovb/STATE-L3-EXT-CURRENT-003.json']);assert state['conditional_finding']=='SEGOVB-L3-PARTY-ZERO-GRACE-ABANDONMENT' and state['current_fresh_personal_reads']==0 and state['whole_lens_or_global_current_complete'] is False and state['proposal']['path'] in records
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (1747,1755,1775,3134):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];finding=a['conditional_findings'][0];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historical mission roles, five e275 source paths, zero-fresh reuse and delegated receipt qualifications; no product execution.')
