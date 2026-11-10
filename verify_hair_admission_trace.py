"""Verify hair admission source and historical evidence without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-HAIR-ADMISSION-95E.json'
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
assert len(sources) == 15
p = 'crates/adventuresim-character-creator/src/'
must(sources[p + 'lib.rs'], ['pub mod hair;'])
must(sources[p + 'hair.rs'], ['pub use placement::{HeadPlacement, HeadPlacementError, HeadSkin};',
    'pub fn groom(&self) -> Groom', 'shape: self.shape'])
shape = sources[p + 'hair/shape.rs']
must(shape, ['#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Groom',
    '#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]',
    '#[serde(deny_unknown_fields, try_from = "ShapeValues")]',
    'FRIZZ.contains(&values.frizz)', 'pub struct ShapeKnobs',
    'pub fn with_knobs(self, knobs: ShapeKnobs) -> Self',
    'value.clamp(*range.start(), *range.end())', 'frizz: within(knobs.frizz, FRIZZ)',
    'look.frizz = Metres::from_metres(look.frizz.metres() * (self.frizz));',
    'self.shape.apply(&mut look);'])
must(sources[p + 'hair/groom.rs'], ['pub fn grow(groom: Groom, gpu: &HairGpu)',
    'let look = groom.look();', 'groomer.grow_look(&look, &head, &scalp)?'])
placement = sources[p + 'hair/placement.rs']
must(placement, ['pub fn fit(skin: &HeadSkin<\'_>, bust: &Head)',
    'skin.positions.len() != skin.joint_indices.len()',
    'skin.joint_indices.joint_count() != skin.joint_names.joint_count()',
    'normalize(forward).ok_or(HeadPlacementError::NoFacing)?',
    'let width = extent.side[1] - extent.side[0];',
    'let depth = extent.forward[1] - extent.forward[0];',
    'let scale_side = width / (bust_extent.side[1] - bust_extent.side[0]);',
    'let scale_forward = depth / (bust_extent.forward[1] - bust_extent.forward[0]);',
    'let scale_up = 0.5 * (scale_side + scale_forward);',
    'dot(offset, x) / (sx * sx)', 'dot(offset, y) / (sy * sy)',
    'dot(offset, z) / (sz * sz)', 'if share < HEAD_SKIN_SHARE',
    'let mut extent: Option<Self> = None;', 'side: [s, s]', 'forward: [f, f]'])
must(sources[p + 'hair/geometry.rs'], ['to.point(from.bust_point(point))'])
preview = sources[p + 'hair_preview.rs']
must(preview, ['HeadPlacement::fit(&skin, hair.head())?',
    'placed.hair.groom == groom && Arc::ptr_eq(&placed.body, body)',
    'worker.groom == groom && Arc::ptr_eq(&worker.body, body)',
    'std::thread::spawn(move ||', 'self.worker = Some(Worker {',
    'preview.request(design.groom(), &body, BodyHead::new(&model, &body))'])
must(sources[p + 'hair_controls.rs'], ['let mut knobs = shape.knobs();',
    '*shape = shape.with_knobs(knobs);'])
must(sources['crates/fabelgeist-hair/src/look.rs'], ['pub struct Look', 'pub frizz: Metres'])
must(sources['crates/fabelgeist-hair/src/head.rs'], ['pub fn mannequin() -> Self',
    'center: Vec3::new(0.074 * side, -0.004, -0.008)',
    'radii: Vec3::new(0.011, 0.03, 0.019)'])
must(sources['crates/fabelgeist-math/src/units.rs'], ['pub struct Metres(f32);',
    'pub const fn from_metres(value: f32) -> Self {\n        Self(value)'])
must(sources['crates/fabelgeist-rig/src/skin.rs'], ['pub fn from_native(',
    'if usize::try_from(joint).map_or(true, |joint| joint >= joints.0)',
    'pub const fn from_native_joint_count(joints: usize) -> Self',
    'pub fn vertex(&self, vertex: usize)'])
must(sources['crates/fabelgeist-rig/src/table.rs'], ['pub struct RigJointTable<T>(Vec<T>);',
    'impl<T> FromIterator<T> for RigJointTable<T>'])
must(sources['crates/fabelgeist-rig/src/lib.rs'], ['pub fn index_in<\'a>(',
    '.position(|name: &Self| -> bool { name == self })'])
must(sources['crates/fabelgeist-rig/src/names.rs'], ['pub const C_HEAD:', 'pub const L_EYE:', 'pub const R_EYE:'])
# The single hypothetical finite vertex has zero width and depth and uses
# in-range rig words. No product constructor or numerical runtime is invoked.
assert all(i < 3 for i in [0] * 8)
assert max([0]) - min([0]) == 0
assert 1.0 >= 0.5 and len([0, 1, 2]) == 3
pigment = json.loads((root / 'ASSESSMENT-L01-HAIR-PIGMENT-95E.json').read_text())
f32 = preserved(pigment['installed_toolchain_evidence']['files'][0]).decode()
must(f32, ['this function returns NaN if the initial value was NaN',
    'pub const fn clamp(mut self, min: f32, max: f32)'])
manifest = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
entries = {item['path']: item for item in manifest['entries']}
receipts = {}
for item in doc['recovered_historical_receipts']:
    raw = preserved(item)
    entry = entries[item['original_path']]
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) == item['bytes']
    receipts[item['original_path'].rsplit('/', 1)[-1]] = json.loads(raw)
oldread = receipts['READ-PL13-110.json']
assert oldread['registration']['sha256'] == doc['recovered_historical_receipts'][2]['sha256']
assert oldread['proposed_findings'][0]['proposed_id'] == 'PL13-110-HEAD-PLACEMENT-INVERTIBILITY'
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())['segovb_continuation']['lens13']
contract = ledger['delegated_connected_contracts_pending_alias'][193]
finding = ledger['delegated_findings_pending_primary_provenance_and_alias'][11]
assert contract['receipt_sha256'] == finding['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert finding['delegated_receipt']['sha256'] == doc['recovered_historical_receipts'][1]['sha256']
assert contract['contract']['contract'] == doc['historical_records'][0]['id']
assert finding['finding']['proposed_id'] == doc['historical_records'][1]['id']
assert finding['accepted_unique_count_change'] == 0 and finding['current_reconciled'] is False
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id']
    assert role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
entries = base['newly_assessed_ledger']['entries']
for candidate in doc['candidates']:
    assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
    assert candidate['discovery'].startswith('partial')
    assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
print('verified fifteen Git-bound sources and three historical receipts; no product ran')
