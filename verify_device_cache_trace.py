"""Verify body/device cache evidence without invoking product or GPU code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-DEVICE-CACHE-95E.json'
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
assert len(sources) == 9
p = 'crates/adventuresim-character-creator/src/'
t = 'crates/adventuresim-tactical-client/src/equipment/runtime_equipment/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

must(sources[p + 'device_body.rs'], ['pub struct DeviceBody(OnceLock<GpuBody>);',
     'if let Some(body) = self.0.get()', 'return Ok(body);', 'let body = upload()?;'])
must(sources[p + 'runtime_equipment.rs'], ['pub struct RuntimeBody {', 'pub positions: Vec<[f32; 3]>',
     'pub device: DeviceBody,', 'fn validate(&self) -> Result<()>', 'device: &self.device,',
     'device_fit::fit_recipe_async(', 'generate_vambrace(gpu, body, selection, bracer_design).await',
     'generate_breastplate(gpu, body, breastplate_design).await'])
must(sources[p + 'device_fit.rs'], ['self.device.get_or_upload(|| {',
     'positions: self.positions,', 'pub fn wearer(&self, body: &FitBody',
     'let hosts = realizations', 'let bodies = realizations'])
must(sources[p + 'device_frames.rs'], ['pub struct DeviceWearer', 'pub body: &\'a GpuBody', 'pub host: &\'a Wearer'])
must(sources[p + 'device_equipment.rs'], ['device: &generated.device', 'device: &sample.device'])
must(sources[t + 'body.rs'], ['let body = RuntimeBody {', 'device: Default::default()'])
generation = sources[t + 'generation.rs']
must(generation, ['let mut body = canonical.body.clone();', 'key.apply(&mut body, &canonical.targets)?;',
     'canonical.rig.fit(&mut body, key.proportions())?', 'body: Arc::new(body)',
     'runtime_equipment::generate('])
assert generation.index('key.apply(&mut body, &canonical.targets)?;') < generation.index('canonical.rig.fit(&mut body, key.proportions())?') < generation.index('body: Arc::new(body)')
must(sources[t + 'morphs.rs'], ['body.positions.iter_mut()', 'body.device = Default::default();'])
must(sources[t + 'rig.rs'], ['body: &mut RuntimeBody', '.global_joint_states\n            .iter_mut()'])
receipt = doc['recovered_historical_receipts'][0]
path = (root / receipt['preserved_bytes']).resolve()
assert path.is_relative_to(root) and receipt['content_encoding'] == 'gzip'
raw = gzip.decompress(path.read_bytes())
assert hashlib.sha256(raw).hexdigest() == receipt['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias']
assert old[208]['receipt_sha256'] == receipt['sha256']
assert old[208]['contract']['contract'] == doc['historical_records'][0]['id']
assert old[157]['contract']['contract'] == doc['historical_records'][1]['id']
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified nine Git-bound body-cache sources and one historical receipt; no product or device ran')
