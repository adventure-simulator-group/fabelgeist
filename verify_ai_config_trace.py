"""Verify preserved AI config, dependency source, and historical relationships."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-AI-CONFIG-95E.json'
doc = json.loads((root / name).read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
if doc['source_revision'] != revision or doc['lens'] != 1:
    raise ValueError('wrong source or lens')
def tree(name):
    result = {}
    for line in (root / name).read_text().splitlines():
        header, path = line.split('\t', 1)
        _, _, oid = header.split()
        if path in result:
            raise ValueError('duplicate tree path')
        result[path] = oid
    return result
current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    if not path.is_relative_to(root) or item['content_encoding'] != 'gzip' or item['revision'] != revision:
        raise ValueError('unsafe or incorrect source reference')
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if (len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
        hashlib.sha256(raw).hexdigest() != item['sha256'] or oid != item['git_blob'] or
        current.get(item['path']) != oid or previous.get(item['path']) != oid):
        raise ValueError('revision-bound source mismatch: ' + item['path'])
    if item['path'] in sources:
        raise ValueError('duplicate source')
    for first, last in item['ranges']:
        if not 1 <= first <= last <= item['lines']:
            raise ValueError('invalid read range')
    sources[item['path']] = raw.decode()
if len(sources) != 16:
    raise ValueError('incorrect source count')
config = sources['crates/adventuresim-tactical-core/src/combat_config.rs']
validation = config.split('pub fn validate(&self)', 1)[1].split('pub fn digest(&self)', 1)[0]
for marker in ('!ai.offense.has_valid_intervals()', 'self.targeting.reported_hit_precision.is_finite()',
               'let ai = &self.ai.ordinary;'):
    if marker not in validation:
        raise ValueError('parent config admission marker missing')
if 'ai.offense.hit_precision' in validation or 'ranged_standoff_slop_metres' in validation:
    raise ValueError('parent now validates the candidate fields')
if 'let bytes = serde_json::to_vec(self)' not in config or 'self.validate()?' not in config.split('pub fn digest(&self)', 1)[1].split('const fn body_part_index', 1)[0]:
    raise ValueError('digest serialization marker missing')
ai = sources['crates/adventuresim-tactical-core/src/combat_config/ai_offense.rs']
check = ai.split('fn has_valid_intervals(&self)', 1)[1].split('impl Default for AiOffenseConfig', 1)[0]
for marker in ('pub hit_precision: f32', 'pub ranged_standoff_slop_metres: f32',
               'pub ranged_reach_fraction: f32'):
    if marker not in ai:
        raise ValueError('AI source field marker missing')
if 'hit_precision' in check or 'ranged_standoff_slop_metres' in check or 'ranged_reach_fraction' in check:
    raise ValueError('AI child now validates the candidate fields')
startup = sources['crates/adventuresim-tactical-server/src/startup.rs']
for marker in ('serde_saphyr::from_str(&text)', '.validate()', '.install_runtime_snapshot()', '.digest()'):
    if marker not in startup:
        raise ValueError('server startup admission marker missing')
main = sources['crates/adventuresim-tactical-server/src/main.rs']
if 'combat_config: Option<PathBuf>' not in main or 'app.insert_resource(combat_config)' not in main:
    raise ValueError('server config input marker missing')
runtime = sources['crates/adventuresim-tactical-core/src/combat_config/runtime.rs']
if 'self.validate()?' not in runtime or '= self.clone();' not in runtime:
    raise ValueError('runtime config snapshot marker missing')
offense = sources['crates/adventuresim-tactical-server/src/bot/offense.rs']
if '&combat_config.ai.ordinary.offense' not in offense or 'drive_offensive_phase(' not in offense:
    raise ValueError('server AI resource marker missing')
phase = sources['crates/adventuresim-tactical-server/src/bot/offense/phase.rs']
if phase.count('ReportedPrecision::new(config.hit_precision)') != 2 or phase.count('.expect("AI precision is finite")') != 2:
    raise ValueError('AI precision assertion markers changed')
if 'drive_ranged_pursuit(' not in phase or 'if facts.use_ranged' not in phase:
    raise ValueError('ranged phase marker missing')
authority = sources['crates/adventuresim-tactical-server/src/combat/authority.rs']
if 'value.is_finite().then_some(Self(value))' not in authority:
    raise ValueError('reported precision finite contract changed')
ranged = sources['crates/adventuresim-tactical-server/src/bot/offense/ranged.rs']
for marker in ('facts.weapon_reach * config.ranged_reach_fraction',
               'pursuit.distance > standoff + config.ranged_standoff_slop_metres',
               'pursuit.distance + config.ranged_standoff_slop_metres < standoff',
               'cmd.trigger(RangedAttackStartedIntent'):
    if marker not in ranged:
        raise ValueError('ranged deadband marker missing')
facts = sources['crates/adventuresim-tactical-server/src/bot/offense/facts.rs']
if 'use_ranged: ranged && reach > 0.0 && has_ammo' not in facts:
    raise ValueError('ranged equipment safeguard marker missing')
yaml = sources['content/tactical/combat.yaml']
for marker in ('hit_precision: 1.0', 'ranged_standoff_min_metres: 1.5',
               'ranged_standoff_max_metres: 12.0', 'ranged_standoff_slop_metres: 0.5',
               'ranged_reach_fraction: 0.5'):
    if marker not in yaml:
        raise ValueError('authored default marker missing')
lock = sources['Cargo.lock']
native_text = {}
for package in doc['native_sources']:
    manifest_path = (root / package['manifest']).resolve()
    if not manifest_path.is_relative_to(root):
        raise ValueError('unsafe native manifest')
    manifest = json.loads(manifest_path.read_text())
    archive = (manifest_path.parent / manifest['archive']).read_bytes()
    if (len(archive) != manifest['bytes'] or hashlib.sha256(archive).hexdigest() != manifest['sha256'] or
        package['archive_sha256'] != manifest['sha256'] or manifest['sha256'] not in lock):
        raise ValueError('native archive or lock mismatch')
    with tarfile.open(manifest_path.parent / manifest['archive'], 'r:gz') as tar:
        for member in package['inspected_members']:
            raw = tar.extractfile(member['path']).read()
            if len(raw) != member['bytes'] or hashlib.sha256(raw).hexdigest() != member['sha256']:
                raise ValueError('native member mismatch')
            for first, last in member['ranges']:
                if not 1 <= first <= last <= len(raw.splitlines()):
                    raise ValueError('invalid native range')
            native_text[member['path']] = raw.decode()
if 'from_str_with_options(input, Options::default())' not in native_text['serde-saphyr-1.2.0/src/de/api.rs']:
    raise ValueError('parser default options marker missing')
if 'angle_conversions: false' not in native_text['serde-saphyr-1.2.0/src/de/options.rs']:
    raise ValueError('parser angle option marker missing')
if 'let v: f32 = parse_yaml12_float' not in native_text['serde-saphyr-1.2.0/src/de/deserializer.rs']:
    raise ValueError('typed scalar parser marker missing')
parse = native_text['serde-saphyr-1.2.0/src/parse_scalars.rs']
if '".nan" | "+.nan" | "-.nan" => Ok(T::nan())' not in parse or '".inf" | "+.inf" => Ok(T::infinity())' not in parse:
    raise ValueError('nonfinite parser branch marker missing')
serde_core = native_text['serde_core-1.0.229/src/de/impls.rs']
if 'num_self!(f32:visit_f32);' not in serde_core or 'macro_rules! num_self' not in serde_core:
    raise ValueError('native f32 visitor marker missing')
serde_json = native_text['serde_json-1.0.151/src/ser.rs']
serializer = serde_json.split('fn serialize_f32(self, value: f32)', 1)[1].split('fn serialize_f64', 1)[0]
if 'FpCategory::Nan | FpCategory::Infinite' not in serializer or '.write_null(&mut self.writer)' not in serializer:
    raise ValueError('native JSON nonfinite encoding marker missing')
historical = doc['historical_source']
ledger = json.loads((root / historical['ledger']).read_text())['discovery_findings_and_dependencies']
if historical['checkpoint'] != '059' or len(historical['contract_ids']) != 2:
    raise ValueError('incorrect historical scope')
for identity in historical['contract_ids']:
    if ledger[identity]['accepted_unique_count_change'] != 0:
        raise ValueError('historical unique-count qualification changed')
old = ledger['SEGOVB-L12-AI-PRECISION-ADMISSION-GAP']
if old['current_reconciled'] is not False or old['source_sha256'] != hashlib.sha256(authority.encode()).hexdigest():
    raise ValueError('historical precision attribution changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for item in historical['matching_roles'] + historical['different_physical_roles']:
        index, sha = item['index'], item['sha256']
        if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
            raise ValueError('historical source relationship mismatch')
matched = {item['index']: item['sha256'] for item in historical['matching_roles']}
for index, text in [(1861, ai), (2022, phase), (2023, ranged), (2027, authority), (2075, startup)]:
    if matched[index] != hashlib.sha256(text.encode()).hexdigest():
        raise ValueError('current/pr-working source relationship changed')
different = {item['index']: item['sha256'] for item in historical['different_physical_roles']}
if different[351] == hashlib.sha256(config.encode()).hexdigest() or different[352] == hashlib.sha256(ai.encode()).hexdigest():
    raise ValueError('different physical-p16 source qualification changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 2:
    raise ValueError('ledger count mismatch')
for candidate in doc['candidates']:
    matches = [item for item in entries if item['id'] == candidate['id']]
    if (len(matches) != 1 or matches[0]['record'] != name or candidate['repair'] != 'not started' or
        candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
        raise ValueError('unqualified candidate')
print('verified sixteen Git-bound AI sources, three native archives, and separately attributed historical records')
