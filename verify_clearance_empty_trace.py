"""Verify clearance-station evidence without compiling or running a shader."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-CLEARANCE-EMPTY-95E.json'
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
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 13
p = 'crates/adventuresim-character-creator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

must(sources[p + 'lib.rs'], ['pub mod device_clearance;', 'pub mod device_fit;', 'pub mod device_frames;'])
must(sources['crates/fabelgeist-rig/src/skin.rs'], ['pub fn from_native(', 'joint >= joints.0'])
must(sources['crates/fabelgeist-armor/src/gpu/body.rs'], ['pub struct BodySurface',
     'surface.joint_weights.len() != vertices', 'surface.joint_indices.joint_count() != surface.joints.joint_count()'])
must(sources[p + 'device_fit.rs'], ['pub struct FitBody', 'pub struct Realization',
     'let mut recording = record(&wearer, &mut batch, target)', 'batch.submit();',
     'readback::read(gpu, &recordings, &skin, &names).await', 'wearer.record_fitted_limb(batch, limb, region)'])
must(sources[p + 'device_recipe_plan.rs'], ['_ => Ok(Self::None)'])
must(sources[p + 'armor_frames.rs'], ['const SKIN_SUPPORT_THRESHOLD: f32 = 0.30;'])
must(sources[p + 'device_frames.rs'], ['pub fn record_frame(', 'status: gpu.scratch(4, "armor frame status")?'])
must(sources[p + 'device_frames/passes.rs'], ['if (weight < SKIN_SUPPORT_THRESHOLD)', 'fail(NO_ENVELOPE);'])
must(sources[p + 'device_limb.rs'], ['let frame = self.record_frame(batch, region)?;',
     'self.record_clearance_fit(batch, &frame, &fit, part.carriers(), part.status())?'])
must(sources[p + 'device_extremities.rs'], ['fn record_mitten(', 'self.record_clearance_fit(batch, &frame, &fit, part.carriers(), part.status())?'])
must(sources[p + 'device_clearance.rs'], ['const STATIONS: u32 = 13;', 'const STATION_CAPACITY: u32 = 8192;',
     'self.record_support(batch, fit.region, frame, &support)?;',
     '.dispatch(&*compile(SECTIONS)?, &parameters, [STATIONS, 1, 1])'])
shader = sources[p + 'device_clearance/wgsl.rs']
must(shader, ['var lists = 0u;', 'support[i] = lists;', 'var count = 0u;',
     'if (best_key == 0xffffffffu)', 'var unique = 0u;', 'var hull = 0u;',
     'for (var half = 0u; half < 2u; half = half + 1u)', 'hull = hull - 1u;',
     'for (var i = 0u; i < hull; i = i + 1u)'])
must(sources[p + 'device_fit_readback.rs'], ['let results = gpu.read_staged_async(staging).await?;',
     'DeviceFrame::check_status(results.status(frame), *region)?;'])
assert ((0 - 1 - 1) % (1 << 32)) == 4294967294
receipt = doc['recovered_historical_receipts'][0]
path = (root / receipt['preserved_bytes']).resolve()
assert path.is_relative_to(root) and receipt['content_encoding'] == 'gzip'
raw = gzip.decompress(path.read_bytes())
assert hashlib.sha256(raw).hexdigest() == receipt['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][158]
assert old['receipt_sha256'] == receipt['sha256']
assert old['contract']['contract'] == doc['historical_records'][0]['id']
assert old['contract']['sources'] == [719, 720, 721]
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
assert doc['external_reference']['integer_types'].endswith('#integer-types')
assert doc['external_reference']['out_of_bounds_access'].endswith('#out-of-bounds-access')
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified thirteen source identities and one recovered receipt; no shader or product ran')
