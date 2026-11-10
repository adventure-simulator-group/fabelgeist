"""Verify wardrobe and installed Rust source evidence without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-WARDROBE-SAVE-95E.json'
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
assert len(sources) == 7
p = 'crates/adventuresim-character-creator/src/'
must(sources[p + 'lib.rs'], ['pub mod wardrobe;'])
wardrobe = sources[p + 'wardrobe.rs']
must(wardrobe, ['pub fn load(path: &Path) -> Result<Self>',
    'serde_json::from_slice(&std::fs::read(path)?)', '.validate()',
    'pub fn save(&self, path: &Path) -> Result<()>', 'self.validate()?;',
    'std::fs::write(path, serde_json::to_vec_pretty(self)?)'])
assert wardrobe.index('self.validate()?;', wardrobe.index('pub fn save(')) < wardrobe.index('std::fs::write(path, serde_json::to_vec_pretty(self)?)')
tab = sources[p + 'wardrobe_tab.rs']
must(tab, ['pub(super) fn save(&mut self, name: &str, garment: SettledGarment)',
    'let mut library = self.library.clone();', 'library.insert(name.clone(), garment)?;',
    'library.save(std::path::Path::new(&self.path))?;', 'self.library = library;',
    'pub(super) fn delete(&mut self, name: &LibraryName)', 'library.remove(name);'])
assert tab.index('library.save(std::path::Path::new(&self.path))?;') < tab.index('self.library = library;')
must(sources[p + 'wardrobe_ui.rs'], ['studio_theme::primary_button("Save to wardrobe")',
    'wardrobe.save(model, &mut studio.wardrobe)', 'studio.wardrobe.delete(name)',
    'Could not save the garment: {error:#}'])
must(sources[p + 'main.rs'], ['wardrobe: adventuresim_character_creator::wardrobe::Wardrobe::load(&args.wardrobe)?'])
must(sources[p + 'cli.rs'], ['default_value = "assets_src/equipment/wardrobe.json"'])
must(sources['rust-toolchain.toml'], ['channel = "nightly-2026-07-13"'])
installed = doc['installed_toolchain_evidence']
assert installed['requested_channel'] == 'nightly-2026-07-13'
files = {}
for item in installed['files']:
    raw = preserved(item)
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    files[item['installed_relative_path']] = raw.decode()
assert len(files) == 2
fs = files['lib/rustlib/src/rust/library/std/src/fs.rs']
must(fs, ['File::create(path)?.write_all(contents)',
    'OpenOptions::new().write(true).create(true).truncate(true).open(path.as_ref())',
    'impl Write for File {'])
file_write = fs.split('impl Write for File {', 1)[1].split('impl Seek for File {', 1)[0]
assert 'fn write_all(' not in file_write
write = files['lib/rustlib/src/rust/library/core/src/io/write.rs']
must(write, ['fn write_all(&mut self, mut buf: &[u8]) -> Result<()>',
    'Ok(n) => buf = &buf[n..],', 'Err(e) => return Err(e),'])
assert 'not recovered or independently verified' in installed['qualification']
pigment = json.loads((root / 'ASSESSMENT-L01-HAIR-PIGMENT-95E.json').read_text())
assert pigment['installed_toolchain_evidence']['requested_channel'] == installed['requested_channel']
assert pigment['installed_toolchain_evidence']['installed_manifest_version'] == installed['installed_manifest_version']
for item in pigment['installed_toolchain_evidence']['files'][1:]:
    preserved(item)
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id']
    assert role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
assert work['roles'][851]['current_observations']
assert work['roles'][849]['status'] == 'assessed-nonapplicable'
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial')
assert candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified seven Git-bound sources and pinned installed I/O source evidence; no product ran')
