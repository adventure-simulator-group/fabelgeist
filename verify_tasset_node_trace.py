"""Verify the source-only station-count trace, without running the product."""
import gzip
import hashlib
import json
import struct
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-TASSET-NODE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in out and kind in ('blob', 'commit')
        out[filename] = oid
    return out

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob']
    assert current[item['path']] == previous[item['path']] == oid
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 15
p = 'crates/adventuresim-character-creator/src/'
a = 'crates/fabelgeist-armor/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

support = sources[p + 'device_wrapped_tasset.rs']
must(support, ['const SECTION_HALF_WIDTH_M: f32 = 0.025;',
    'top.is_finite() && bottom.is_finite() && top > bottom',
    'points_for(FitRegion::Thigh(side))?', 'points.extend(points_for(FitRegion::Hips)?);',
    'low.is_finite()', 'high.is_finite()', 'high > low', 'thigh_end >= 3',
    'waist_end > thigh_end + 2',
    '((high - low) / SECTION_HALF_WIDTH_M).ceil() as u32 + 1',
    'face.iter().any(|&i| owned[i as usize])',
    'design.validate()?;', 'Support::new(self.host, design, &shape, Side::Left',
    'Support::new(self.host, design, &shape, Side::Right',
    'validate_shell_support(&part.positions[component.vertices.clone()], span)?'])
assert support.index('let left_support = Support::new') < support.index('let right_support =') < support.index('let left = self.record_tasset_support')
fit=sources[p + 'device_fit.rs']
must(fit, ['pub struct FitBody', 'pub struct Realization', 'pub async fn fit_recipe_async',
    'RecipePlan::prepare(gpu, body, wearer, design).await?',
    '.map(|r| r.upload(gpu, body))', 'record_recipe(device, batch, design, placement, &surfaces, &plan)',
    'fabelgeist_armor::GarmentPlateShape::WrappedTassets(_)',
    'wearer.record_wrapped_tassets(batch, garment, layers, None)'])
must(sources[p + 'device_recipe_plan.rs'], ['_ => Ok(Self::None)'])
frames=sources[p + 'armor_frames.rs']
must(frames, ['RigJointName::ROOT,', 'RigJointName::L_UPLEG,',
    'Thigh(s) => sided(s, &[RigJointPart::Upleg])',
    'weight >= SKIN_SUPPORT_THRESHOLD', 'pub(crate) const SKIN_SUPPORT_THRESHOLD: f32 = 0.30'])
must(sources[p + 'clothing/rig_selection.rs'], ['name == owner'])
must(sources[p + 'lib.rs'], ['pub mod device_fit;', 'pub mod armor_frames;'])
must(sources['crates/fabelgeist-rig/src/skin.rs'], ['pub fn from_native(', 'joint >= joints.0'])
must(sources['crates/fabelgeist-rig/src/mhr.rs'], ['RigSide::Left => "l"', 'format!("{prefix}_{part}")'])
must(sources['crates/fabelgeist-rig/src/part.rs'], ['serialize = "upleg"', 'serialize = "lowleg"'])
must(sources['crates/fabelgeist-rig/src/names.rs'], ['L_UPLEG: Self = Self(Cow::Borrowed("l_upleg"))', 'L_LOWLEG: Self = Self(Cow::Borrowed("l_lowleg"))', 'ROOT: Self = Self(Cow::Borrowed("root"))'])
must(sources[a + 'gpu/body.rs'], ['surface.normals.len() != vertices',
    'surface.joint_indices.len() != vertices', 'surface.joint_weights.len() != vertices',
    '.any(|i| *i as usize >= vertices)'])
must(sources[a + 'garment_armor.rs'], ['pub fn new(kind: GarmentArmorKind)',
    'length: Permille(1_000)', 'lame_count: 4', 'shape.section_break >= self.lame_count'])
must(sources[a + 'garment_plate_design.rs'], ['Self::WrappedTassets(shape)',
    'kind == GarmentArmorKind::Tassets'])
must(sources[a + 'wrapped_tassets.rs'], ['knee_reach: Permille(880)',
    'section_break: 6', 'self.section_break > 11'])
must(sources[a + 'design.rs'], ['f32::from(self.0) / 1_000.0'])
# The f32 divisor lies strictly between zero and 1/32. 2^30/(1/32)=2^35,
# far above u32::MAX, so any correctly rounded f32 quotient retains the bound.
divisor = struct.unpack('>f', struct.pack('>f', 0.025))[0]
assert 0 < divisor < 1 / 32
assert (1 << 30) * 32 == 1 << 35 and 1 << 35 > (1 << 32) - 1
assert ((1 << 32) - 1 + 1) % (1 << 32) == 0
# Three whole-left-upleg vertices are also hip-owned; Hips is appended to
# the original thigh vector. This verifies the source witness's count only.
thigh_end=3
waist_end=thigh_end+3
assert thigh_end >= 3 and waist_end > thigh_end+2
receipt=doc['recovered_historical_receipts'][0]
path=(root / receipt['preserved_bytes']).resolve()
assert path.is_relative_to(root) and receipt['content_encoding']=='gzip'
raw=gzip.decompress(path.read_bytes())
assert hashlib.sha256(raw).hexdigest()==receipt['sha256']
ledger=json.loads((root / 'baseline-059/ledger.json').read_text())
old=ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
assert old[157]['receipt_sha256']==receipt['sha256']
assert old[157]['contract']['contract']==doc['historical_records'][0]['id']
assert old[208]['contract']['contract']==doc['historical_records'][1]['id']
work=json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
role=work['roles'][761];hist=doc['historical_source_roles'][0]
assert role['source_id']==hist['source_id'] and role['historical_sha256']==hist['historical_sha256']
assert hist['byte_identical']==(hist['historical_sha256']==hist['current_sha256'])
base=json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress=json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate=doc['candidates'][0];entries=base['newly_assessed_ledger']['entries']
assert sum(x['id']==candidate['id'] and x['record']==name for x in entries)==1
assert progress['new_current_ledger_candidates']==len(entries) and progress['queue_records_semantically_reconciled']==0
assert candidate['discovery'].startswith('partial') and candidate['repair']=='not started' and candidate['validation']=='not performed'
print('verified 15 Git-bound sources, recovered receipt and arithmetic bounds; no product or device ran')
