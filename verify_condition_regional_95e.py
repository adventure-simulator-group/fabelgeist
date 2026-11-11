#!/usr/bin/env python3
"""Verify exact audit provenance and rational illustrations, not product execution."""
import gzip, hashlib, json, tarfile
from fractions import Fraction
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, body):
    return hashlib.sha1(kind.encode() + b' ' + str(len(body)).encode() + b'\0' + body).hexdigest()
def entries(body):
    result, pos = {}, 0
    while pos < len(body):
        end = body.index(b'\0',pos); mode,name = body[pos:end].split(b' ',1)
        result[name.decode()] = (mode.decode(),body[end+1:end+21].hex()); pos=end+21
    assert pos==len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree',raw) == current == part['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'],part['child']); current=part['child']
    assert current == proof['git_blob'] == oid('blob',body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009'; old='6533e444fa37552927070cc8d8190907fd3b413e'; older='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
name='ASSESSMENT-CONDITION-REGIONAL-95E.json'; hist='ASSESSMENT-L01-HISTORICAL-1891-1894.json'
a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']); shared=load(proof['shared_commit_and_tree_objects']); nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']: nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']); commit=contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1891:41,1892:211,1893:33,1894:104};current={}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=worklist['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']==('bounded-current-trace-only' if i==1892 else 'unassessed')
        assert item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==[]
        assert c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes); current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes'])
    assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges'])
    assert oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
state=current[1892];tick=current[1893];region=current[1894]
assert b'apply_combat_workload(&mut self.fatigue, workload, endurance, parameters)' in state
assert b'self.incapacitation + self.fatigue - before' in state
assert b'total if total >= 1.0 => IncapacitationStatus::Incapacitated' in state
assert b'total if total > 0.5 => IncapacitationStatus::Staggered' in state
assert b'self.0.wrapping_add(1)' in tick and b'distance != 0 && distance <= u32::MAX / 2' in tick
assert b'points.dedup()' in region and b'if points.len() < 2' in region
assert b'#[serde(try_from = "RegionalConnectionWire", deny_unknown_fields)]' in region
assert b'self.calories_used_today()\n            / (attr.attr_by_parts(SimpleAttribute::Endurance, body) * CALORIES_PER_ENDURANCE)' in sources['crates/adventuresim-core/src/essential.rs']
morale=sources['crates/adventuresim-core/src/morale.rs']
assert b'((fatigue_ratio - 0.5).max(0.0) / 0.5).powi(2)' in morale
assert b'let fatigue_ratio = stats.fatigue_by_parts(&attributes, &limbs)' in sources['crates/adventuresim-stdb-module/src/condition.rs']
assert b'stats.calories_used += elapsed_minutes as f32' in sources['crates/adventuresim-stdb-module/src/condition.rs']
strategic=sources['crates/adventuresim-stdb-module/src/tactical.rs']
assert b'strategic_incapacitation: strategic.incapacitation' in strategic and b'strategic_fatigue: strategic.fatigue' in strategic
assert b'crate::character::require_living_character(ctx, character_id)?' in strategic
projection=sources['crates/adventuresim-tactical-server/src/player_projection.rs']
assert b'(starting_incapacitation - player.strategic_fatigue).max(0.0)' in projection
assert b'fatigue: player.strategic_fatigue' in projection and b'incapacitation: player.strategic_incapacitation' in projection
assert b'input_tick: *motion.3' in projection and b'snapshot.input_tick' in projection
condition=sources['crates/adventuresim-tactical-server/src/combat/condition.rs']
assert b'if !active_action && !moving' in condition and b'recover_combat_fatigue(&mut state.fatigue' in condition
assert b') * elapsed_seconds.seconds())\n    .clamp(0.0, 1.0)' in condition
assert b') + state.acute_trauma\n            + state.fatigue\n            + state.encumbrance' in condition
fatigue=sources['crates/adventuresim-core/src/combat/fatigue.rs']
assert b'parameters.recovery_watts_per_endurance' in fatigue and b'*fatigue = (*fatigue - recovered).clamp(0.0, 1.0)' in fatigue
assert b'recovery_watts_per_endurance: 55.0' in sources['content/tactical/combat.yaml']
assert b'capacity_joules_per_endurance: 16000.0' in sources['content/tactical/combat.yaml']
autoresolve=sources['crates/adventuresim-core/src/autoresolve.rs']
assert b'combatant.fatigue = state.fatigue.clamp(0.0, 1.0)' in autoresolve
assert b'recover_combat_fatigue(' in autoresolve
client=sources['crates/adventuresim-tactical-netcode/src/client.rs']
assert b'.init_resource::<ReconnectCredential>()' in client and b'.init_resource::<LocalInputTick>()' in client
assert b'input_tick.0 = input_tick.0.wrapping_next()' in client
connection=sources['crates/adventuresim-tactical-client/src/presentation/regional_map/connections.rs']
assert b'presented.source == pose.source' in connection and b'presented.window == environment.terrain().request()' in connection
assert b'state.environment = Some(environment)' in sources['crates/adventuresim-tactical-client/src/presentation/regional_map/mod.rs']
# Exact rational illustration only; not native floating-point or product execution.
ratio=Fraction(4500,3000); strategic_fatigue=((ratio-Fraction(1,2))/Fraction(1,2))**2
assert strategic_fatigue==4
post_idle=1-Fraction(55,16000*64)
assert post_idle==Fraction(204789,204800) and Fraction(1,2)<post_idle<1
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
    assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob'])
    if item['redacted_local_path_occurrences']: assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(625,[1891,1892,1893]),(626,[1894])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]] and row['same_lens_reuse']==[]
reg=json.loads(records['segovb/REGISTRATION-SEGOVB-L12-801.json']);qual=json.loads(records[reg['locator_qualification']])
assert reg['locator_qualification_sha256']==lookup[reg['locator_qualification']]['sha256']
assert qual['body_coverage']==qual['new_population_credit']==0 and qual['historical_locator_proof_body_not_read'] is True
assert qual['original_index']==1894 and qual['exact_source_sha256']==worklist['roles'][1894]['historical_sha256']
path='segovb/READ-L11-609.json';adopt=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L11-609.json'])
assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup['segovb/STATE-L11-609.json']['sha256']
assert check['agent_attribution_preserved'] and check['other_lens_credit']==0 and not check['product_execution']
assert adopt['proposal'] in records and adopt['proposal_sha256']==lookup[adopt['proposal']]['sha256']
assert adopt['agent_verification'] in records and adopt['agent_verification_sha256']==lookup[adopt['agent_verification']]['sha256']
for plan in adopt['plans']: assert plan['path'] in records and plan['sha256']==lookup[plan['path']]['sha256']
assert all(r['revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' and 'parallel lens11_reconcile agent' in r['attribution'] and 'root adopts without claiming personal reread' in r['attribution'] for r in adopt['rows'])
prior=load('ASSESSMENT-INVENTORY-CACHE-IDENTITY-95E.json');olditems=prior['recovered_older_774_records'];base='handoff/774/adler-original/'
pkg=next(x for x in olditems if x['path_in_774']==base+'PACKAGE-MANIFEST.json');package=json.loads(contents(pkg['preserved_bytes']))
oldloc=next(x for x in olditems if x['path_in_774'].endswith('source-scope-reconciliation-044.json'))
assert oldloc['package_original_sha256']!=oldloc['package_exported_sha256']
locator=json.loads(contents(oldloc['preserved_bytes']));loc=locator['new_variant_identity_and_range_map'][108]
assert loc['source_id']==worklist['roles'][1894]['source_id'] and loc['recorded_sha256']==worklist['roles'][1894]['historical_sha256']
native=next(x for x in olditems if x['path_in_774'].endswith('/native/traces.jsonl'));nativebody=contents(native['preserved_bytes'])
native_record=next(json.loads(x) for x in nativebody.splitlines() if json.loads(x).get('id')=='NATIVE-PS-ROOT-TERRAIN-SOURCE-PACKET-015')
assert native_record['related_current_reads'][5]==qual['raw_producer_observations'][1]['source_context']
op=load(a['older_774_git_membership_evidence']);os=load(op['shared_commit_and_tree_objects']);onodes={x['git_object']:x for x in os['objects'] if x['type']=='tree'}
for p in op['additional_shared_tree_objects']:onodes.update(load(p)['additional_tree_objects'])
onodes.update(op['additional_tree_objects']);oc=next(x for x in os['objects'] if x['type']=='commit' and x['git_object']==older);raw=contents(oc['preserved_bytes'])
assert oid('commit',raw)==older==op['revision'] and raw.splitlines()[0]==b'tree '+op['root_tree'].encode()
item=a['recovered_older_774_records'][0];body=contents(item['preserved_bytes']);path=item['path_in_774']
assert item['older_publication_revision']==older and sha(body)==item['sha256'] and len(body)==item['bytes']
check_chain(path,op['paths'][path],body,op['root_tree'],onodes)
row=next(x for x in package['entries'] if x['path']==path[len(base):])
assert row['original_sha256']==row['exported_sha256']==item['sha256']==item['package_original_sha256']==item['package_exported_sha256']
root_record=next(json.loads(x) for x in body.splitlines() if json.loads(x).get('id')=='ROOT-PPS-TERRAIN-DISPATCH-WEB-037')
assert root_record['source_proofs'][5]==qual['raw_producer_observations'][0]['source_context']
prior_adler=load('ASSESSMENT-PRONE-EFFORT-95E.json');old_adler=next(x for x in prior_adler['recovered_historical_records'] if x['original_path']=='adler/READ-L11-288.json')
assert sha(contents(old_adler['preserved_bytes']))==old_adler['published_sha256'] and old_adler['sha256']==lookup[old_adler['original_path']]['sha256']
prior_disposition=load('baseline-059/ledger.json')['segovb_continuation']['current_connected_dispositions'][157]
assert prior_disposition['scope_revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f'
assert 'exact recovery/policy not claimedclosed' in prior_disposition['disposition']['trace'][1]
q=load('RECONCILIATION-059.json')['entries']
for i in (1563,1564):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f=a['conditional_findings'][0];assert f['id']=='REVIEW-STRATEGIC-OVERFATIGUE-TRANSFER-001'
assert f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, earlier delegated and producer records, and conditional condition-transfer evidence; no product execution.')
