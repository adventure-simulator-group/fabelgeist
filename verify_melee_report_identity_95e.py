#!/usr/bin/env python3
"""Verify melee report recovered receipt provenance without executing product code."""
import gzip, hashlib, json, tarfile, tomllib
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
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end+1:end+21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def check_chain(path, proof, body, start, nodes):
    assert [x['name'] for x in proof['chain']] == path.split('/')
    current = start
    for part in proof['chain']:
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == part['tree'] and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body) and sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
old = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-MELEE-REPORT-IDENTITY-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2007-2012.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:
    nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][old]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == old and commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {2007:55,2008:7,2009:197,2010:330,2011:185,2012:62}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']; body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed','bounded-current-trace-only') and item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1,n]] and len(before.splitlines()) == n and sha(before) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert before == body and c['byte_identical'] and c['current_personal_ranges'] == [] and c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        check_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes'] and len(body.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo,hi in item['ranges']) and oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body

assert all((root / p).is_file() for p in a['prior_current_evidence'])
manifest, entry, audit, causal, auto, error = (current[i] for i in lengths)
assert b'required-features = ["iteration"]' in manifest and b'melee_combat_iteration::run()' in entry
assert b'.movement_displacement_metres.unwrap_or_default().abs()' in auto
assert b'.map_or(0.0, |(before, after)| (after - before).abs())' in auto
assert b'event.attacker == john_name' in causal and b'wound.combatant == john_name' in causal
assert b'format!("{:?}", telemetry.contact_classification)' in auto
assert b'war_hammer' in audit and b'all_weapon_swept_contact_contract_holds' in audit
assert b'MissingMovementObservation' in error and b'UnknownTacticalVictor' in error
movement=sources['crates/adventuresim-core/src/autoresolve/melee_round/mobility.rs']
for field in (b'combatant_id',b'engagement_distance_before_metres',b'engagement_distance_after_metres',b'movement_elapsed_seconds',b'movement_displacement_metres',b'movement_velocity_before_metres_per_second',b'movement_velocity_after_metres_per_second',b'movement_speed_limit_metres_per_second'):
 assert b'event.'+field+b' = Some(' in movement
assert b'event.sequence = self.next_timeline_sequence' in sources['crates/adventuresim-core/src/autoresolve.rs']
assert b'timeline: recorder.timeline' in sources['crates/adventuresim-core/src/autoresolve.rs']
assert b'pub combatant_id: Option<u64>' in sources['crates/adventuresim-core/src/autoresolve/timeline.rs']
roster=sources['crates/adventuresim-core/src/autoresolve/melee_iteration.rs']
for weapon in (b'longsword',b'arming_sword',b'halberd',b'war_hammer',b'baselard'):assert b'"'+weapon+b'"' in roster
assert b'let opponents = vec![' in roster and b'Ok((john, opponents))' in roster
proofs=sources['crates/adventuresim-core/src/autoresolve/melee_iteration/evidence.rs']
segment=proofs[proofs.index(b'fn all_weapon_contact_band_evidence('):proofs.index(b'fn polearm_contact_revalidation_evidence(')]
for token in (b'(john, "longsword",',b'(hammer, "war_hammer",',b'(veteran, "halberd",'):assert token in segment
assert b'"arming_sword"' not in segment and b'"baselard"' not in segment
assert b'all_weapon_contact_band_evidence(&john, hammer, veteran)' in proofs
iteration=sources['crates/adventuresim-tactical-server/src/iteration.rs']
assert b'TacticalCombatSide::Party' in iteration and b'TacticalCombatSide::Enemy' in iteration
assert b'victor: left_name.to_owned()' in iteration and b'victor: right_name.to_owned()' in iteration
resolved=sources['crates/adventuresim-tactical-server/src/iteration/logging/resolution.rs']
assert b'attacker: attacker.name.clone()' in resolved and b'anatomical_subregion: serialized_name(event.anatomical_subregion)' in resolved
assert b'combat diagnostic value serializes as a string' in resolved
assert b'pub attacker: Entity' in sources['crates/adventuresim-tactical-server/src/combat.rs']
assert b'pub anatomical_subregion: AnatomicalSubregion' in sources['crates/adventuresim-tactical-server/src/combat.rs']
assert b'pub enum AnatomicalSubregion' in sources['crates/adventuresim-core/src/combat/targeting.rs']
assert b'#[serde(rename_all = "snake_case")]' in sources['crates/adventuresim-core/src/combat/targeting.rs']
assert b'pub enum MeleeContactClassification' in sources['crates/adventuresim-core/src/combat/contact_geometry.rs']
report=sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/report.rs']
assert b'contact_classifications: BTreeMap<String, u64>' in report
assert b'anatomical_subregions_received: BTreeMap<String, u64>' in report
runner=sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/runner.rs']
assert b'UnknownTacticalVictor' in runner and b'let Some(id) = event.combatant_id else' in runner
command=sources['crates/adventuresim-tactical-server/src/bin/melee_combat_iteration/mod.rs']
assert b'args.tactical_seeds == 0 || args.autoresolve_seeds == 0' in command
assert command.index(b'args.tactical_seeds == 0') < command.index(b'fs::create_dir_all(&args.output)')
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']:x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
 assert (item['sha256'],item['bytes'],item['git_blob']) == (original['sha256'],original['bytes'],original['git_blob'])
 assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:
  assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in [(673,[2007,2008,2009,2010]),(674,[2011,2012,2013,2014]),(693,[2059,2060])]:
 p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
 assert [row['original_index'] for row in z['rows']]==indices and z['current_source_reconciled'] is False
 assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
 for row in z['rows']:
  assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
  if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
 for row in z.get('qualified_supporting_same_lens_reuse',[]) + z.get('qualified_connected_original_reuse',[]):
  assert row['receipt'] in records and row['receipt_sha256']==lookup[row['receipt']]['sha256']
  assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
for n,indices in [(310,[2003,2004,2005,2006,2007,2008]),(311,[2009,2010,2011,2012])]:
 p=f'adler/READ-L11-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
 assert [row['original_index'] for row in z['sources']]==indices and z['current_reconciled'] is False
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
 for row in z['sources']+z.get('connected_reinspection',[]):
  rp='adler/'+row['registration'].replace('\\','/').split('/')[-1]
  assert rp in records and row['registration_sha256']==lookup[rp]['sha256']
for suffix in ('676-CORE-TIMELINE','693-CORE-OWNERS'):
 p=f'segovb/READ-REVISIT-L12-{suffix}.json';z=json.loads(records[p]);v=json.loads(records[f'segovb/VERIFY-REVISIT-L12-{suffix}.json'])
 assert v['receipt_sha256']==lookup[p]['sha256'] and v['product_execution'] is False
 for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
 if suffix.startswith('676'):
  assert z['new_original_overlay_merged_shared_or_extension_credit']==0
  assert any(x['finding']=='ADLER-L11-MELEE-MOVEMENT-EVIDENCE-ADMISSION' for x in z['connected_contract_resolutions'])
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (115,268,275,276,277,1672,1673,1674,1675,1676,1677,1678,1679,1680,1682,1683,1684,1690,1692):
 assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
 assert q[i]['contract_id']==base[i]['contract_id']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for x in a['recovered_findings']:
 assert x['id']==base[x['historical_queue_index']]['contract_id']
 assert x['id'] in load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
 assert sum(y['id']==x['id'] and y['record']==name for y in ledger)==1
assert len(a['conditional_findings'])==1 and a['conditional_findings'][0]['id']=='REVIEW-MELEE-ALL-WEAPON-BAND-COVERAGE-001'
assert sum(x['id']==a['conditional_findings'][0]['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historical melee report roles, qualified original receipts, producer counterevidence and bounded current identity and sample scope; no product execution.')
