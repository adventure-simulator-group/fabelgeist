"""Verify historical and current authority evidence; never execute product code."""
import argparse
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
sha = lambda raw: hashlib.sha256(raw).hexdigest()
blob = lambda raw: hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
doc = load('ASSESSMENT-L02-GENERATED-AUTHORITY-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
assert doc['source_revision'] == rev and doc['lens'] == 2
assert [x['id'] for x in doc['candidates']] == [
    'SEGOVB-L12-GENERATED-INVESTIGABILITY-TUPLE-SLOT',
    'REVIEW-DEVELOPER-AUTHORITY-CONTEXT-DECODE-001',
]
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta, path = line.split('\t', 1)
    tree[path] = meta.split()[2]
contents = {}
for item in doc['current_sources']:
    raw = gzip.decompress((root / item['preserved_bytes']).read_bytes())
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert sha(raw) == item['sha256'] and blob(raw) == item['git_blob'] == tree[item['path']]
    assert item['revision'] == rev and all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    contents[item['path']] = raw

def source(suffix):
    matches = [v for k, v in contents.items() if k.endswith(suffix)]
    assert len(matches) == 1
    return matches[0]

lock = source('Cargo.lock')
assert b'name = "serde"\nversion = "1.0.229"' in lock
assert b'4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba' in lock

def member(package, name):
    directory = root / 'native-evidence' / package
    manifest = json.loads((directory / 'MANIFEST.json').read_text())
    raw = (directory / manifest['archive']).read_bytes()
    assert len(raw) == manifest['bytes'] and sha(raw) == manifest['sha256']
    entry = next(x for x in manifest['inspected_members'] if x['path'].endswith('/' + name))
    with tarfile.open(directory / manifest['archive'], 'r:gz') as archive:
        data = archive.extractfile(entry['path']).read()
    assert len(data) == entry['bytes'] and sha(data) == entry['sha256']
    return data

derive = member('serde-derive-1.0.229', 'src/de.rs')
assert b'fn expr_is_missing(' in derive and b'de::missing_field' in derive
struct = member('serde-derive-1.0.229', 'src/de/struct_.rs')
assert b'expr_is_missing' in struct
missing = member('serde-1.0.229', 'src/private/de.rs')
assert b'pub fn missing_field' in missing and b'Err(Error::missing_field(self.0))' in missing
assert b'fn deserialize_option' in missing and b'forward_to_deserialize_any!' in missing
string = member('serde_core-1.0.229', 'src/de/impls.rs')
assert b"impl<'de> Deserialize<'de> for String" in string and b'deserializer.deserialize_string(StringVisitor)' in string

model = source('quest_generation/model.rs')
context = model.split(b'pub struct GenerationContext {', 1)[1].split(b'\n}', 1)[0]
manifest = model.split(b'pub struct GeneratedCase {', 1)[1].split(b'\n}', 1)[0]
assert b'pub seed:' in context and b'pub settlement_id: String' in context and b'pub catalog_revision:' not in context
assert b'pub catalog_revision: String' in manifest
wrapper = source('developer_quest.rs').split(b'pub struct DeveloperGenerationContext {', 1)[1].split(b'\n}', 1)[0]
assert all(x in wrapper for x in (b'pub base:', b'pub definition:', b'pub allow_implausible:'))
assert b'pub seed:' not in wrapper and b'pub settlement_id:' not in wrapper and b'pub catalog_revision:' not in wrapper
bootstrap = source('mission_bootstrap.rs')
assert b'serde_json::to_string(&context)' in bootstrap and b'serde_json::to_string(&developer_context)' in bootstrap
assert b'Some(&context_snapshot_json)' in bootstrap and b'require_development_gateway(ctx)?' in bootstrap
authority = source('authority_model.rs')
assert b'developer.base.clone()' in authority and b'Ok(ValidatedQuestGenerationAuthority { context, manifest })' in authority
proj = source('investigation/projections.rs')
candidate = proj.split(b'fn validated_generated_authority_candidate(', 1)[1].split(b'\n}', 1)[0]
assert b'validate_quest_generation_authority(&authority).map_err(|_| ())?;' in candidate
assert candidate.index(b'authority.manifest_json') < candidate.index(b'authority.context_snapshot_json')
unique = proj.split(b'fn exactly_one_generated_authority(', 1)[1].split(b'\n}', 1)[0]
assert b'for (case_id, manifest, context)' in unique and b'(manifest.clone(), context.clone())' in unique
inv = proj.split(b'fn generated_investigability(', 1)[1].split(b'\n}', 1)[0]
assert b'let (_, manifest_json) = generated_authority_reducer' in inv and b'GeneratedCase>(&manifest_json)' in inv
pattern = proj.split(b'fn generated_pattern_authority(', 1)[1].split(b'\nfn exactly_one_generated_authority', 1)[0]
assert b'let Some((manifest_json, context_json))' in pattern and b'quest_generation::GenerationContext>(\n        context_json' in pattern
assert b'GeneratedPatternAuthority::Invalid' in pattern
assert b'GeneratedPatternAuthority::Invalid => return false' in proj
assert b'!capability_has_live_support_view(ctx, &capability, kind)' in proj
capabilities = source('investigation/capabilities.rs')
assert b'GeneratedPatternAuthority::Invalid => return false' in capabilities
assert b'if !capability_has_live_pattern_support_reducer(ctx, capability)' in capabilities
assert b'validate_capability_blueprint_reducer(ctx, root)?;' in capabilities
assert b'let context = serde_json::from_str::<adventuresim_core::quest_generation::GenerationContext>' in capabilities
assert b'let generation_context = validated.context;' in capabilities
assert b'validate_newly_issued_action_route_graph(ctx, owner_character_id, case_id)' in capabilities
acts = source('investigation/actions.rs')
assert b'The current journal no longer supports this investigation route' in acts
assert b'if let Some(investigability) = generated_investigability(ctx, &capability)' in acts
assert b'apply_investigability_to_route_skills(route_skills, investigability)' in acts
existing_test = source('investigation/tests/action_authority.rs').split(
    b'fn generated_pattern_authority_fails_closed_and_manual_actions_remain_permissive()', 1)[1].split(b'\n#[test]', 1)[0]
assert b'let context = GenerationContext {' in existing_test and b'serde_json::to_string(&context)' in existing_test
assert b'DeveloperGenerationContext' not in existing_test
assert doc['selected_existing_test_source']['historical_role'] == 1133

monsters = json.loads(source('content/quests/bestiary.yaml'))['monsters']
profiles = {m['id']: m['investigation']['investigability'] for m in monsters}
assert profiles['wolf'] == 75 and profiles['bandit'] == 60
inventory = doc['authored_content_inventory']
assert len(monsters) == inventory['monster_count'] and inventory['wolf_investigability'] == 75
relations = json.loads(source('content/quests/generation.yaml'))['relations']
selected = {r['id']: [c for c in r['candidates'] if c['id'] in ('bandit', 'wolf')]
            for r in relations if r['id'] in ('cause.recurring_depredation', 'cause.disappearance_or_loss')}
assert selected == inventory['selected_family_cause_candidates']
assert all(len(v) == 2 and all(c['plausibility'] > 0 and c['curation'] > 0 and
           c['hard_zero_reason'] is None and c['required_bridge'] is None for c in v) for v in selected.values())
assert b'profile.investigation.investigability = authored.investigation.investigability;' in source('bestiary.rs')
assert b'((i16::from(investigability.min(100)) - 50) * 30)' in source('threat_escalation.rs')
assert (75 - 50) * 30 * 2 == 1500 and 3000 < 3500 < 3000 + 1500

transport = {x['path']: x for x in json.loads(gzip.decompress(
    (root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_records']
assert len(records) == 103
for item in records:
    raw = gzip.decompress((root / item['preserved_bytes']).read_bytes())
    source_entry = transport[item['original_path']]
    assert item['sha256'] == source_entry['sha256'] and item['git_blob'] == source_entry['git_blob']
    assert item['bytes'] == source_entry['bytes'] and sha(raw) == item['published_sha256']
    assert len(raw) == item['published_bytes']
    if not item['redacted_local_path_occurrences']:
        assert sha(raw) == item['sha256'] and blob(raw) == item['git_blob'] and len(raw) == item['bytes']
    else:
        assert (b'/' + b'home/') not in raw and b'<LOCAL_PATH_' in raw
        assert 'pinned checkpoint 059 transport bundle' in item['original_recovery']
assert sum(bool(x['redacted_local_path_occurrences']) for x in records) == 4
by_path = {x['original_path']: x for x in records}
alias_info = doc['historical_l11_alias_receipt']
alias = json.loads(gzip.decompress((root / by_path[alias_info['original_path']]['preserved_bytes']).read_bytes()))
assert alias['canonical'] == 'SEGOVB-L12-GENERATED-INVESTIGABILITY-TUPLE-SLOT'
assert alias['accepted_unique_count_change'] == 0 and alias['global_alias_inventory_complete'] is False
for item in alias['evidence']:
    assert item['path'] in by_path and by_path[item['path']]['sha256'] == item['sha256']
queue = load('baseline-059/queue.json')['entries']
view = queue[216]['granular_contract_views'][1]
for row in view['exact_source_identities_and_assessed_ranges']:
    ref = row['selected_root_adoption']
    assert by_path[ref['path']]['sha256'] == ref['sha256']
for ref in view['proofs']:
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert by_path[path]['sha256'] == ref['sha256']

old = doc['historical_behavior_qualification']
old_manifest = gzip.decompress((root / old['historical_package_manifest']).read_bytes())
assert sha(old_manifest) == old['manifest_sha256']
old_entries = {x['path']: x for x in json.loads(old_manifest)['files']}
assert old_entries[old['original_path']]['sha256'] == old['original_sha256']
redacted = gzip.decompress((root / old['published_path_redacted_bytes']).read_bytes())
assert sha(redacted) == old['published_sha256'] and redacted.count(b'${HISTORICAL_WORKSPACE_ROOT}/') == 8
old_claim = json.loads(redacted)
assert 'CAM-OWNER-F011' in old_claim['aliases'] and old_claim['id'] == 'CONT08-QUAL012'
assert old_claim['source_proofs'][0]['after_sha256'] == sha(proj)
cam = doc['historical_camera_finding']
for kind, rel in (('findings', cam['original_findings_path']), ('contracts', cam['original_contracts_path'])):
    rec = by_path['root-l2-extension-20261030/INHERITED-L2-CAMERA-' + kind + '.jsonl']
    assert old_entries[rel]['sha256'] == rec['sha256'] and old_entries[rel]['bytes'] == rec['bytes']
    raw = gzip.decompress((root / rec['preserved_bytes']).read_bytes())
    rows = [json.loads(line) for line in raw.splitlines()]
    if kind == 'findings':
        finding = rows[cam['finding_line'] - 1]
        assert finding['finding_id'] == 'CAM-OWNER-F011' and '(_,manifest_json)' in finding['actual_defect']
    else:
        assert rows[92]['contract_id'] == 'CAM-OWN-INVESTIGATION-PROJECTIONS-090'
        assert rows[93]['contract_id'] == 'CAM-OWN-INVESTIGATION-ACTIONS-091'
parser = argparse.ArgumentParser()
parser.add_argument('--historical-workspace-root')
args = parser.parse_args()
if args.historical_workspace_root:
    prefix = args.historical_workspace_root.rstrip('/').encode() + b'/'
    original = redacted.replace(b'${HISTORICAL_WORKSPACE_ROOT}/', prefix)
    assert len(original) == old['original_bytes'] and sha(original) == old['original_sha256']

baseline = load('BASELINE-95E-20261110.json')
for candidate in doc['candidates']:
    assert len([x for x in baseline['newly_assessed_ledger']['entries'] if x['id'] == candidate['id']]) == 1
reconciliation = load('RECONCILIATION-059.json')['entries']
for i in (215, 216, 936, 942):
    assert reconciliation[i]['current_reconciliation'].startswith('partial ')
progress = load('CONTINUATION-PROGRESS-95E.json')
assert progress['queue_records_semantically_reconciled'] == 0
assert progress['lenses_complete'] == 0
assert progress['new_current_ledger_candidates'] == len(baseline['newly_assessed_ledger']['entries'])
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified source, archive, historical and redaction evidence; no product ran')
