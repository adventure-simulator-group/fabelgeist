#!/usr/bin/env python3
"""Verify bot behavior recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-BOT-BEHAVIOR-SNAPSHOT-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2018.json'
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
lengths = {2018:1248}
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
bot=current[2018]
assert b'pub struct CombatantBehaviorPackages(pub Vec<CombatantBehaviorPackage>)' in bot
assert b'Added<CombatantBehaviorPackages>' in bot and b'entity.insert(OffensiveCombatAi::default())' in bot
assert b'CombatantBehaviorPackage::RecoverToUpright' in bot
assert b'Ok(resolve_fixture_path(' in bot and b'pub(crate) fn resolve_scene_fixture(selector: &str) -> Result<PathBuf, String>' in bot
assert b'fn apply_enemy_fixture' in bot and b'fixture.enemy_count()' in bot
assert b'.after(CombatSet::Condition)' in bot
controller=sources['crates/adventuresim-tactical-server/src/bot/offense.rs']
assert b'pub struct OffensiveCombatAi {' in controller
assert b'target: None' in controller and b'phase: OffensiveCombatPhase::Pursuing' in controller
assert b'pub(super) fn drive_offensive_combat_ai' in controller
phase=sources['crates/adventuresim-tactical-server/src/bot/offense/phase.rs']
assert b'pub(super) enum OffensiveCombatPhase {' in phase and b'MeleeWindup {' in phase
main=sources['crates/adventuresim-tactical-server/src/main.rs']
assert b'(bot::BotPlugin, openings::BuildingOpeningsPlugin)' in main
assert b'load_world_dump.after(on_server_started)' in main
assert b'WorldDeserializer {' in main and b'scene.write_to_world(world, &mut entity_map)' in main
assert b'world.flush()' in main
assert b'#[arg(long, value_parser = bot::resolve_scene_fixture)]' in main
assert b'#[arg(long, value_parser = bot::load_enemy_fixture)]' in main
snapshot=sources['crates/adventuresim-tactical-server/src/debug_dump.rs']
assert b'.allow_component::<crate::bot::CombatantBehaviorPackages>()' in snapshot
assert b'.allow_component::<crate::bot::OffensiveCombatAi>()' in snapshot
spawn=sources['crates/adventuresim-tactical-server/src/player_projection/spawn_admission.rs']
assert b'CombatantBehaviorPackages::from_fixture(enemy.behavior, combat)' in spawn
assert b'CombatantBehaviorPackages::standard_combat(combat)' in spawn
assert b'.expect("fixture path resolution is infallible")' in sources['crates/adventuresim-tactical-server/src/startup.rs']
fixture=sources['crates/adventuresim-core/src/tactical_fixture.rs']
assert b'#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]' in fixture
assert b'pub struct TacticalEnemyFixture {' in fixture and b'enemies: Vec<TacticalEnemySpec>' in fixture
assert b'let fixture: Self = serde_saphyr::from_str(yaml)' in fixture and b'fixture.validate()?' in fixture
assert b'if self.enemies.is_empty()' in fixture
assert b'pub fn enemy_count(&self) -> u32' in fixture and b'.expect("validated enemy fixtures fit in u32")' in fixture
assert b'pub const MAX_MOB_COUNT: u32 = 30' in sources['crates/adventuresim-core/src/threat_escalation.rs']
assert b'Path::new(env!("CARGO_MANIFEST_DIR"))' in sources['crates/adventuresim-core/src/fixture_path.rs']
lock_bytes=contents(a['lock_source']);lock=tomllib.loads(lock_bytes.decode())
assert oid('blob',lock_bytes)==tree['Cargo.lock']
native={}
for item in a['pinned_native_source_ranges']:
 archive=(root/item['archive_path']).read_bytes();assert sha(archive)==item['archive_sha256'] and len(archive)==item['archive_bytes']
 package=next(x for x in lock['package'] if x['name']==item['package'] and x['version']==item['version'])
 assert package['checksum']==item['archive_sha256']
 with tarfile.open(root/item['archive_path'],'r:gz') as ar:body=ar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and len(body.splitlines())==item['member_lines']
 assert all(1<=lo<=hi<=item['member_lines'] for lo,hi in item['ranges'])
 native[item['member']]=body
assert b'only retains results the first time after they have been added' in native['bevy_ecs-0.19.1/src/query/filter.rs']
assert b'This will overwrite any previous value(s) of the same component type.' in native['bevy_ecs-0.19.1/src/system/commands/mod.rs']
assert b'F: Fn(&str) -> Result<T, E>' in native['clap_builder-4.6.6/src/builder/value_parser.rs']
assert b'fn map<T, F>(self, func: F)' in native['clap_builder-4.6.6/src/builder/value_parser.rs']
assert b'type Value = DynamicWorld' in native['bevy_world_serialization-0.19.1/src/serde.rs']
assert b'.or_insert_with(|| world.spawn_empty().id())' in native['bevy_world_serialization-0.19.1/src/dynamic_world.rs']
assert b'reflect_component.apply_or_insert_mapped(' in native['bevy_world_serialization-0.19.1/src/dynamic_world.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
 assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:
  assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
p='segovb/READ-L12-676.json';z=json.loads(records[p]);check=json.loads(records['segovb/VERIFY-L12-676.json'])
assert [row['original_index'] for row in z['rows']]==[2018] and z['current_source_reconciled'] is False
assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
assert z['rows'][0]['fresh_personal_body_reads']==[[1,1248]]
assert z['rows'][0]['registration'] in records and z['rows'][0]['registration_sha256']==lookup[z['rows'][0]['registration']]['sha256']
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['segovb/STATE-L12-676.json']['sha256']
p='adler/READ-L11-313.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-313.json'])
assert [r['original_index'] for r in z['sources']]==[2018,2019,2020,2021,2022,2023,2024,2025] and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-313.json']['sha256']
for row in z['sources']:
 rp='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert rp in records and row['registration_sha256']==lookup[rp]['sha256']
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (1688,1689):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len(a['conditional_findings'])==5
for f in a['conditional_findings']:assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
assert 'unwrap or expect outside tests' in a['rust_rule_evidence']['rule']
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical bot body, qualified original receipts, lock-bound Bevy source and bounded fixture contracts; no product execution.')
