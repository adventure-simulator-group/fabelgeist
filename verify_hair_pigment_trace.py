"""Verify preserved pigment sources and provenance without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-HAIR-PIGMENT-95E.json'
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
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob']
    assert current[item['path']] == previous[item['path']] == oid
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 7
p = 'crates/adventuresim-character-creator/src/'
must(sources[p + 'lib.rs'], ['pub mod hair;'])
hair = sources[p + 'hair.rs']
must(hair, ['#[serde(deny_unknown_fields, try_from = "PigmentShares")]',
    'let share = 0.0..=1.0;', 'share.contains(&shares.melanin)',
    'share.contains(&shares.redness)', 'shares.dye.iter().all(|channel| share.contains(channel))',
    'return Err(PigmentRangeError);', 'pub fn new(melanin: f32, redness: f32)',
    'melanin: melanin.clamp(0.0, 1.0)', 'redness: redness.clamp(0.0, 1.0)',
    'pub fn dyed(self, dye: [f32; 3])', 'dye: dye.map(|channel| channel.clamp(0.0, 1.0))',
    'pub fn color(self) -> [f32; 3]', 'tint: self.dye', '.base_color()'])
material = sources['crates/fabelgeist-hair/src/material.rs']
must(material, ['pub fn base_color(&self) -> [f32; 3]', 'tint: [1.0; 3]',
    '..*self', '.absorption()', '* self.tint[k].clamp(0.0, 1.0)'])
must(sources[p + 'hair_color_controls.rs'], ['HairPigment::new(', '.dyed(',
    'let [red, green, blue] = pigment.color();'])
must(sources[p + 'character_export/hair.rs'], ['let color = design.pigment.color();'])
must(sources[p + 'hair_preview/assets.rs'], ['let [red, green, blue] = design.pigment.color();'])
must(sources['rust-toolchain.toml'], ['channel = "nightly-2026-07-13"'])
installed = doc['installed_toolchain_evidence']
assert installed['requested_channel'] == 'nightly-2026-07-13'
files = {}
for item in installed['files']:
    raw = preserved(item)
    assert len(raw) == item['bytes'] and item['installed_relative_path'] not in files
    files[item['installed_relative_path']] = raw.decode()
assert len(files) == 5
f32 = files['lib/rustlib/src/rust/library/core/src/num/f32.rs']
must(f32, ['pub const fn clamp(mut self, min: f32, max: f32) -> f32',
    'if self < min {', 'if self > max {', 'this function returns NaN if the initial value was NaN'])
assert f32.index('if self < min {', f32.index('pub const fn clamp(mut self')) < f32.index('if self > max {', f32.index('pub const fn clamp(mut self'))
manifest = files['lib/rustlib/multirust-channel-manifest.toml']
must(manifest, ['date = "2026-07-13"', '[pkg.rust-src]', installed['installed_manifest_version']])
assert 'not recovered or independently verified' in installed['qualification']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
old = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][187]
receipt = doc['recovered_historical_receipts'][0]
assert hashlib.sha256(preserved(receipt)).hexdigest() == old['receipt_sha256']
assert doc['historical_records'][0]['id'] == old['contract']['contract']
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id']
    assert role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial')
assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified seven Git-bound sources, five installed-toolchain files and one recovered receipt; no product ran')
