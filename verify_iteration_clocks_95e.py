#!/usr/bin/env python3
"""Verify iteration source history, clock qualification and evidence only."""
import difflib,gzip,hashlib,json,tarfile
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';pre='c100638b720a7281a5fc96f6a3c5c23d5dda85da';name='ASSESSMENT-ITERATION-CLOCKS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2054-2060.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2054:60,2055:648,2056:29,2057:163,2058:427,2059:89,2060:167};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={};historic={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status'] in ('unassessed','bounded-current-trace-only') and item['prior_current_observations']==row.get('current_observations',[]) and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and sha(body)==c['sha256'] and len(body)==c['bytes'] and len(body.splitlines())==c['lines'] and oid('blob',body)==c['git_blob']==tree[c['path']]
  chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes)
  if i==2055:
   assert before!=body and c['byte_identical'] is False and c['current_personal_ranges']==[[1,104]] and c['current_content_read_via_identical_historical_ranges']==[]
   d=''.join(difflib.unified_diff(before.decode().splitlines(keepends=True),body.decode().splitlines(keepends=True),fromfile='6533/'+c['path'],tofile='95e/'+c['path'])).encode();assert contents(c['read_only_unified_diff']['path'])==d and sha(d)==c['read_only_unified_diff']['sha256']
  else:assert before==body and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  if i in (2056,2058):assert 'c100 predecessor' in item['prior_worklist_observation_revision_qualification'] and load('ASSESSMENT-L1-ITERATION-TIMEOUT.json')['current_revision']==pre
  current[i]=body;historic[i]=before
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
assert all((root/p).is_file() for p in a['prior_current_evidence']);assert a['repair']=='not started' and a['validation']=='not performed'
old=historic[2055];now=current[2055];clock=current[2056];equipment=current[2057];net=sources['crates/adventuresim-tactical-netcode/src/lib.rs'];timing=sources['crates/adventuresim-tactical-server/src/combat/ingress/timing.rs'];loc=sources['crates/adventuresim-tactical-core/src/animation/locomotion.rs'];config=sources['content/tactical/combat.yaml'];build=sources['crates/adventuresim-core/src/autoresolve/melee_iteration.rs'];lib=sources['crates/adventuresim-tactical-server/src/lib.rs'];catalog=sources['crates/adventuresim-core/src/item_catalog.rs']
assert b'const TACTICAL_TICK_HZ: u64 = 64;' in old and b'pub const TACTICAL_TICK_HZ: u32 = 30;' in net and b'u64::from(TACTICAL_TICK_HZ)' in now
assert b'let mut final_tick = maximum_ticks;' in old and b'for tick in 0..maximum_ticks' in now and b'simulated_ticks: final_tick + 1' in now and b'TacticalElapsed::from_tick_count(final_tick + 1)' in now
assert b'pub(super) fn from_tick_count(ticks: u64)' in clock and b'ticks as f32 * TACTICAL_TICK_SECONDS' in clock
assert b'  locomotion:\n    sample_hz: 64.0' in config and b'.locomotion\n        .sample_hz' in loc and b'(time.elapsed_secs_f64() * locomotion_sample_rate().hertz() as f64).round() as u64' in timing
assert b'pub struct MeleeIterationBuild' in build and b"pub weapon_id: &'static str" in build and b'pub combatant: Combatant' in build and b'authored_equipment(weapon_id, armor_ids, shield_id)?' in build
assert b'pub fn resolve_tactical_server_melee_duel(' in now and b'pub mod iteration;' in lib and b'if let Some(weapon) = build.combatant.equipment.melee_weapon' in equipment and b'.expect("iteration roster validates its weapon")' in equipment and b'pub fn definition(id: &str) -> Option' in catalog
assert b'if !valid_id(id)' in sources['crates/adventuresim-core/src/item_catalog_validation.rs'] and b'!id.is_empty()' in sources['crates/adventuresim-core/src/item_catalog_validation.rs']
assert b'.insert_resource(IterationLog::default())' in now and b'remove_resource::<IterationLog>().unwrap()' in now and b'Never use `unwrap` or `expect` outside tests' in sources['crates/AGENTS.md']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((691,[2055]),(692,[2056,2057,2058]),(693,[2059,2060])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['current_source_reconciled'] is False and check['receipt_sha256']==lookup[path]['sha256']
 for row in data['rows']:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]] and row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
 for row in data.get('qualified_supporting_same_lens_reuse',[]):assert row['fresh_personal_body_reads_this_batch']==[] and row['receipt'] in records and row['registration'] in records
assert next(x for x in json.loads(records['segovb/READ-L12-690.json'])['rows'] if x['original_index']==2054)['fresh_personal_body_reads']==[[1,60]]
data=json.loads(records['adler/READ-L11-320.json']);assert [x['original_index'] for x in data['sources']]==list(range(2054,2062)) and data['current_reconciled'] is False
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (118,274,1726,1733,1742,1751,1752,1753,1754,1757,1758,1759,1760):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
followup=load('ASSESSMENT-MELEE-REJECTION-LIFECYCLE-95E.json')['timing_qualification_followup'];assert followup['assessment']==name and 'not30Hz fixed updates' in followup['qualification'] and 'Original trace preserved' in followup['qualification']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['conditional_findings']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert next(x for x in ledger if x['id']=='REVIEW-MELEE-REJECTED-CONTACT-STALE-TRADE-001')['timing_qualification_followup']==name
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified seven historical iteration roles, changed current body, original attribution and distinct clocks; no product execution.')
