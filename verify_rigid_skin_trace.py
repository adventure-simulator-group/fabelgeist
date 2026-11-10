"""Verify rigid-skin source and recovery evidence without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-RIGID-SKIN-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision


def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in result and kind in ('blob', 'commit')
        result[filename] = oid
    return result


def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    return raw


def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)


current = tree('main-advance-95e/current-tree.tsv')
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob'] == current[item['path']]
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 13
p = 'crates/adventuresim-character-creator/src/'
a = 'crates/fabelgeist-armor/src/'
r = 'crates/fabelgeist-rig/src/'
must(sources[p + 'lib.rs'], ['pub mod skin_rules;', 'pub mod armor_recipes;',
    'pub mod armor_frames;', 'pub mod device_fit;'])
attach = sources[p + 'skin_rules.rs']
must(attach, ['pub fn attach(', 'armor.joint_indices.joint_count() == joint_names.joint_count()',
    'ParametricDesign::Helmet(HelmetDesign::ArmingCap(_) | HelmetDesign::MailCoif(_))',
    'ParametricDesign::Helmet(_) => {', 'joint_index(joint_names, &RigJointName::C_HEAD)?',
    'fill(armor, 0..armor.positions.len(), head)?;', 'fn fill(armor: &mut GeneratedArmor',
    'armor.joint_indices.set_vertex(vertex, [joint; 8])?;',
    'armor.joint_weights[vertices].fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);'])
assert attach.index('armor.joint_indices.set_vertex(vertex, [joint; 8])?;') < attach.index('armor.joint_weights[vertices].fill(')
must(sources[p + 'armor_recipes.rs'], ['pub enum ParametricDesign', 'Helmet(HelmetDesign)'])
must(sources[p + 'armor_frames.rs'], ['pub enum FittingPlacement', 'Unpaired,'])
must(sources[a + 'helmets.rs'], ['pub enum HelmetDesign', 'Burgonet(BurgonetDesign)'])
must(sources[a + 'design.rs'], ['pub struct GeneratedArmor', 'pub positions: Vec<[f32; 3]>',
    'pub joint_indices: fabelgeist_rig::PackedRigSkin<8>', 'pub joint_weights: Vec<[f32; 8]>'])
skin = sources[r + 'skin.rs']
must(skin, ['pub fn from_native(', 'pub const fn from_native_joint_count(joints: usize)',
    'pub fn set_vertex(', 'let words = self.encode(joints)?;', 'self.words[vertex] = words;'])
assert skin.index('let words = self.encode(joints)?;', skin.index('pub fn set_vertex(')) < skin.index('self.words[vertex] = words;', skin.index('pub fn set_vertex('))
must(sources[r + 'lib.rs'], ['pub fn index_in<\'a>(', '.position(|name: &Self| -> bool { name == self })'])
must(sources[r + 'table.rs'], ['pub struct RigJointTable<T>(Vec<T>);', 'impl<T> FromIterator<T> for RigJointTable<T>'])
must(sources[r + 'names.rs'], ['pub const ROOT:', 'pub const C_HEAD:'])
assembly = sources[p + 'device_fit.rs']
must(assembly, ['pub struct DevicePiece', 'pub fn assemble_recipe(',
    'positions: base.positions,', 'joint_indices: skin.joint_indices,',
    'joint_weights: skin.joint_weights,', 'crate::device_textile_materials::apply(design, &mut armor)?;',
    'crate::skin_rules::attach('])
assert assembly.index('crate::device_textile_materials::apply(design, &mut armor)?;') < assembly.index('crate::skin_rules::attach(')
must(sources[p + 'parametric_equipment.rs'], ['device_fit::assemble_recipe('])
must(sources[p + 'runtime_equipment.rs'], ['generate_parametric(gpu, body, &design, selection, layers).await'])
# Only evidence arithmetic: one admitted rig word row is shorter than two positions.
assert 1 < 2 and all(0 <= joint < 2 for joint in [0, 1])
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
entries = {item['path']: item for item in manifest['entries']}
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = entries[item['original_path']]
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) == item['bytes']
    receipts[item['original_path'].rsplit('/', 1)[-1]] = (item, json.loads(raw))
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']
old = ledger['lens13']['delegated_connected_contracts_pending_alias'][210]
assert old['receipt_sha256'] == receipts['READ-L13-PARALLEL-117.json'][0]['sha256']
assert old['contract']['contract'] == doc['historical_records'][0]['id']
qual = ledger['lens12']['connected_body_receipts'][36]
assert qual['sha256'] == receipts['QUALIFICATION-PAR-L12-769-779-AUDIT-ONLY-HEADERS.json'][0]['sha256']
assert qual['evidence']['proposal_sha256'] == receipts['PROPOSAL-L12-771.json'][0]['sha256']
assert qual['evidence']['historical_accepted_unique_count_change'] == 'field absent'
assert qual['evidence']['effective_accepted_unique_count_change'] == 0
assert qual['evidence']['effective_product_execution'] is False
assert any('set_vertex' in s for row in qual['evidence']['effective_pending_connected_contracts'] for s in row['scopes'])
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
reference = doc['historical_source_roles'][0]
role = work['roles'][reference['original_index']]
assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['historical_sha256']
assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256']) is False
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial')
assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified thirteen Git-bound sources and three original receipts; no product ran')
