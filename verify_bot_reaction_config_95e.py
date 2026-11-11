#!/usr/bin/env python3
"""Verify bot reaction recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-BOT-REACTION-CONFIG-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2019-2024.json'
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
lengths = {2019:287,2020:287,2021:198,2022:370,2023:43,2024:48}
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
import tomllib
assert all((root/p).is_file() for p in a['prior_current_evidence'])
defense, offense, facts, phase, ranged, tactics=(current[i] for i in lengths)
assert b'if attempt.chances.dodge_chance >= 1.0' in defense
assert b'(attempt.windup_seconds.seconds() - 0.16).clamp(0.02, 0.12)' in defense
assert b'config.reaction_delay_min_seconds.seconds()' in defense and b'config.reaction_delay_max_seconds.seconds()' in defense
assert b'.insert(PendingBotReaction { timer, choice })' in defense
assert b'.remove::<PendingBotReaction>()' in defense and b'cmd.trigger(DefendIntent {' in defense
assert b'ReportedPrecision::new(config.hit_precision)' in phase
assert phase.count(b'.expect("AI precision is finite")') == 2
assert b'if matches!(controller.phase, OffensiveCombatPhase::Pursuing)' in phase and b'unreachable!("pursuit was handled above")' in phase
assert b'config.ranged_standoff_slop_metres' in ranged and b'input.0 = Some(Vec2::Y)' in ranged
assert b'config.ranged_reach_fraction' in ranged
assert b'viewer.inventory.get(entity).has_item_id(ARROW_ID)' in facts
assert b'compare_target(' in offense and b'initiative_delay_seconds(' in tactics
config=sources['crates/adventuresim-tactical-core/src/combat_config.rs']
assert b'#[derive(Clone, Debug, PartialEq, Resource, Serialize, Deserialize)]' in config
assert b'!(0.0..=1.0).contains(&ai.defense.dodge_chance)' in config
assert b'!ai.offense.has_valid_intervals()' in config
assert b'self.targeting.reported_hit_precision.is_finite()' in config
assert b'serde_json::to_vec(self)' in config and b'Sha256::digest(bytes)' in config
body=sources['crates/adventuresim-tactical-core/src/combat_config/ai_offense.rs']
assert b'pub hit_precision: f32' in body and b'pub ranged_standoff_slop_metres: f32' in body
segment=body[body.index(b'pub(super) fn has_valid_intervals'):body.index(b'impl Default')]
assert b'hit_precision' not in segment and b'ranged_standoff_slop_metres' not in segment and b'ranged_reach_fraction' not in segment
assert b'self.ranged_standoff_max_metres >= self.ranged_standoff_min_metres' in segment
assert b'value.is_finite().then_some(Self(value))' in sources['crates/adventuresim-tactical-server/src/combat/authority.rs']
assert b'self.validate()?' in sources['crates/adventuresim-tactical-core/src/combat_config/runtime.rs']
startup=sources['crates/adventuresim-tactical-server/src/startup.rs']
assert b'serde_saphyr::from_str(&text)' in startup and b'.install_runtime_snapshot()' in startup and b'.digest()' in startup
bot=sources['crates/adventuresim-tactical-server/src/bot.rs']
assert b'dodge_chance: defense.dodge_chance' in bot
assert b'pub(crate) fn unit_f64' in bot and b'self.0.unit_f64()' in bot
assert b'half-open unit interval `[0, 1)`' in sources['crates/fabelgeist-determinism/src/lib.rs']
assert b'minimum + self.unit_f32() * (maximum - minimum)' in sources['crates/fabelgeist-determinism/src/stream.rs']
yaml=sources['content/tactical/combat.yaml']
for s in (b'ranged_standoff_min_metres: 1.5',b'ranged_standoff_max_metres: 12.0',b'ranged_reach_fraction: 0.5',b'reaction_delay_min_seconds: 0.20',b'reaction_delay_max_seconds: 0.27'):assert s in yaml
lock_bytes=contents(a['lock_source']);lock=tomllib.loads(lock_bytes.decode());assert oid('blob',lock_bytes)==tree['Cargo.lock'];native={}
for item in a['pinned_native_source_ranges']:
 archive=(root/item['archive_path']).read_bytes();assert sha(archive)==item['archive_sha256'] and len(archive)==item['archive_bytes']
 package=next(x for x in lock['package'] if x['name']==item['package'] and x['version']==item['version']);assert package['checksum']==item['archive_sha256']
 with tarfile.open(root/item['archive_path'],'r:gz') as ar:body=ar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and len(body.splitlines())==item['member_lines']
 assert all(1<=lo<=hi<=item['member_lines'] for lo,hi in item['ranges']);native[item['member']]=body
assert b'from_str_with_options(input, Options::default())' in native['serde-saphyr-1.2.0/src/de/api.rs']
assert b'let v: f32 = parse_yaml12_float' in native['serde-saphyr-1.2.0/src/de/deserializer.rs']
assert b'".nan" | "+.nan" | "-.nan" => Ok(T::nan())' in native['serde-saphyr-1.2.0/src/parse_scalars.rs']
assert b'".inf" | "+.inf" => Ok(T::infinity())' in native['serde-saphyr-1.2.0/src/parse_scalars.rs']
assert b'Concrete `f32`/`f64` targets are unaffected' in native['serde-saphyr-1.2.0/src/de/options.rs']
assert b'FpCategory::Nan | FpCategory::Infinite' in native['serde_json-1.0.151/src/ser.rs'] and b'.write_null(&mut self.writer)' in native['serde_json-1.0.151/src/ser.rs']
assert b'This will overwrite any previous value(s) of the same component type.' in native['bevy_ecs-0.19.1/src/system/commands/mod.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
 assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:
  assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in [(677,[2019,2020,2021]),(678,[2022,2023,2024,2025]),(680,[2027,2028,2029])]:
 p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
 assert [row['original_index'] for row in z['rows']]==indices and z['current_source_reconciled'] is False
 assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
 for row in z['rows']:
  assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
  if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
 for row in z.get('qualified_supporting_same_lens_reuse',[]):
  assert row['receipt'] in records and row['receipt_sha256']==lookup[row['receipt']]['sha256']
  assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
p='segovb/READ-L12-698.json';z=json.loads(records[p]);check=json.loads(records['segovb/VERIFY-L12-698.json'])
assert z['delegated_attribution_preserved'] is True and z['root_personal_rereads']==[]
assert all('Subagent /root/lens12_frontier' in row['attribution'] for row in z['rows'])
assert z['proposal'] in records and z['proposal_sha256']==lookup[z['proposal']]['sha256']
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['segovb/STATE-L12-698.json']['sha256']
for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
for n,indices in [(282,[1855,1856,1857,1858,1859,1860,1861]),(313,[2018,2019,2020,2021,2022,2023,2024,2025]),(314,[2026,2027,2028]),(322,[2066])]:
 p=f'adler/READ-L11-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json'])
 assert [row['original_index'] for row in z['sources']]==indices and z['current_reconciled'] is False
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
 for row in z['sources']+z.get('connected_reinspection',[]):
  rp='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert rp in records and row['registration_sha256']==lookup[rp]['sha256']
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (106,269,1696,1697,1699,1700):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for x in a['recovered_findings']:
 assert x['id']==base[x['historical_queue_index']]['contract_id']
 assert x['id'] in load('baseline-059/ledger.json')['discovery_findings_and_dependencies']
 assert sum(y['id']==x['id'] and y['record']==name for y in ledger)==1
assert len(a['conditional_findings'])==3
for f in a['conditional_findings']:assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historical AI roles, original and delegated attribution, pinned YAML/JSON source and conditional config traces; no product execution.')
