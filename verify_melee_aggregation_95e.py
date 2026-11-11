#!/usr/bin/env python3
"""Verify melee aggregation recovered receipt provenance without executing product code."""
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
name = 'ASSESSMENT-MELEE-REPORT-AGGREGATION-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2013-2017.json'
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
lengths = {2013:213,2014:263,2015:270,2016:85,2017:325}
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
from fractions import Fraction as F
assert all((root/p).is_file() for p in a['prior_current_evidence'])
measurement, command, report, reviewer, runner=(current[i] for i in lengths)
assert b'Skill::Sword.training_rank(primary_hours)' in reviewer and b'.weighted_check(|skill| skills.skill_hours_trained(skill))' in reviewer
assert b'pub(super) effective_weapon_contacts: u64' in report
assert b'defender.effective_weapon_contacts += 1' in command
assert b'MeleeResponseChoice::Block | MeleeResponseChoice::Parry' in command
assert b'entry.outcome == BattleAttackOutcome::Blocked' in command
assert b'defender_contact_item_id == defender_weapon_id' in runner and b'causal.shield_contacts += 1' in runner
assert b'vec![john.combatant.clone()]' in runner and b'vec![opponent.combatant.clone()]' in runner
assert b'outcome.allies[0]' in runner and b'outcome.enemies[0]' in runner
assert command.index(b'"acceptance-evidence.json"') < command.index(b'"reviewer-index.json"')
assert b'File::create(directory.join("tactical-traces.ndjson"))' in runner
skill=sources['crates/adventuresim-core/src/skill.rs']
for variant in (b'Polearm',b'Axe',b'Bludgeon',b'Sword',b'Knife'):
 assert b'max_hours = 8000.0, kind = SkillKind::Physical, is_trained = false)]\n    '+variant+b',' in skill
for variant,h in ((b'Dodge',b'20000.0'),(b'Block',b'12000.0'),(b'Will',b'5000.0'),(b'Balance',b'30000.0')):
 assert b'max_hours = '+h+b',' in skill and b'    '+variant+b',' in skill
assert b'MAX_CHECK * (hours / (hours + self.max_hours() * 0.5))' in skill
assert b'rank * self.max_hours() * 0.5 / (MAX_CHECK - rank)' in skill
assert b'skill.capped_training_rank(hours_trained, attr)' in skill
assert b'let hours_trained = self.effective_skill_hours(skill)' in skill
assert F(4)*8000*F(1,2)/(5-4)==16000
assert 5*F(32000,3)/(F(32000,3)+4000)==F(40,11)
assert (F(4)+0+4)/3==F(8,3)
assert 5*F(16000,16000+10000)==F(40,13)
assert 5*F(16000,16000+6000)==F(40,11)
assert 5*F(16000,16000+2500)==F(160,37)
assert 5*F(16000,16000+15000)==F(80,31)
roster=sources['crates/adventuresim-core/src/autoresolve/melee_iteration.rs']
assert b'let hours = Skill::Sword.hours_for_rank(skill_rank)' in roster
segment=roster[roster.index(b'let skills = StartingSkills {'):roster.index(b'let spec = StartingCharacterSpec {')]
for field in (b'sword: hours',b'polearm: hours',b'bludgeon: hours',b'knife: hours',b'dodge: hours',b'block: hours',b'will: hours',b'balance: hours'):
 assert field in segment
assert b'axe:' not in segment and b'..StartingSkills::default()' in segment
assert b'axe_hours: s.axe' in roster and b'equipment.defense_item_id = Some(stable_id(shield_id))' in roster
assert b'"polearm_veteran"' in roster and b'"halberd"' in roster and b'Some("buckler")' in roster
catalog=sources['content/items/catalog.yaml']
pos=catalog.index(b'"id": "halberd"');halberd=catalog[pos:catalog.index(b'"capabilities"',pos)]
for field in (b'"polearm": 0.3333333333333333',b'"axe": 0.3333333333333333',b'"bludgeon": 0.3333333333333333'):assert field in halberd
assert b'let skill_distribution: crate::equipment::WeaponSkillDistribution = (*skills).into()' in sources['crates/adventuresim-core/src/autoresolve/authored_weapon.rs']
equipment=sources['crates/adventuresim-core/src/equipment.rs']
assert b'axe: skills.axe' in equipment and b'.map(|(skill, weight)| check(skill) * weight)' in equipment
assert b'compile_items(&root)' in sources['crates/adventuresim-core/build.rs']
assert b'pub const ITEM_CATALOG_JSON: &str' in sources['crates/adventuresim-core/build.rs']
assert b'serde_json::from_str(ITEM_CATALOG_JSON)' in sources['crates/adventuresim-core/src/item_catalog.rs']
core=sources['crates/adventuresim-core/src/autoresolve.rs']
assert b'physical_contact: true' in core and b'=> BattleAttackOutcome::Blocked' in core
assert b'equipment.defense_item_id.or(equipment.melee_weapon_id)' in core
assert b'allies: allies.into_iter().map(outcome).collect()' in core and b'enemies: enemies.into_iter().map(outcome).collect()' in core
assert b'allies: &mut [Combatant]' in core and b'enemies: &mut [Combatant]' in core
assert b'melee_defender_contact_item_id(result, response, &defender.equipment)' in sources['crates/adventuresim-core/src/autoresolve/melee_round/mod.rs']
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes'])
 assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:
  assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in [(674,[2011,2012,2013,2014]),(675,[2015,2016,2017])]:
 p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
 assert [row['original_index'] for row in z['rows']]==indices and z['current_source_reconciled'] is False
 assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256']
 assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
 for row in z['rows']:
  assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
  if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
p='adler/READ-L11-312.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-312.json'])
assert [r['original_index'] for r in z['sources']]==[2013,2014,2015,2016,2017] and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-312.json']['sha256']
for row in z['sources']:
 rp='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert rp in records and row['registration_sha256']==lookup[rp]['sha256']
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries']
for i in (1675,1680,1682,1685,1686,1687):assert q[i]['contract_id']==base[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert len(a['conditional_findings'])==4
for f in a['conditional_findings']:assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical report roles, original receipt attribution, source paths and ideal rational examples; no product execution.')
