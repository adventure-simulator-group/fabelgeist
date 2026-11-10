"""Verify preserved creator-save source evidence, without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-CREATOR-SAVE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    entries = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in entries and kind in ('blob', 'commit')
        entries[filename] = oid
    return entries

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
assert len(sources) == 5
p = 'crates/adventuresim-character-creator/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

output = sources[p + 'src/armor_design_output.rs']
must(output, ['let paths = [self.catalog, self.bracer, self.breastplate];',
              'let resolved = paths.map(resolve);', 'design files must have distinct paths',
              'for (path, bytes) in paths.into_iter().zip(bytes)',
              'std::fs::write(path, bytes).with_context(|| format!("saving {}", path.display()))?;',
              'let resolved = if path.exists() {', 'path.canonicalize()?',
              'aliases_are_rejected_before_any_write_and_distinct_designs_reload'])
assert output.index('let catalog_bytes = armor_design_input::encode(catalog)?;') < output.index('let resolved = paths.map(resolve);') < output.index('for (path, bytes) in paths.into_iter().zip(bytes)')
must(sources[p + 'src/armor_design_input.rs'], ['pub fn encode(designs: &ArmorDesigns)', 'parse(&bytes)?;'])
must(sources[p + 'src/item_design.rs'], ['pub fn save(&self, paths: &DesignPaths', 'paths.save(&self.overrides, &self.vambrace, &self.breastplate)'])
must(sources[p + 'src/studio_ui.rs'], ['egui::TextEdit::singleline(path)', 'if ui.button("Save all catalog designs").clicked()', 'Err(error) => format!("Could not save catalog designs: {error:#}")'])
must(sources[p + 'README.md'], ['Save all catalog designs', 'Pass all three files back through their corresponding options to'])
for filename in ('baseline-059/ledger.json', 'baseline-059/queue.json'):
    raw = (root / filename).read_text().lower()
    for term in ('armor_design_output', 'designpaths', 'save all catalog designs'):
        assert term not in raw
historical = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-682-690.json').read_text())
role = next(x for x in historical['historical_source_roles'] if x['original_index'] == 689)
assert role['source_id'] == doc['historical_source_roles'][0]['source_id']
assert role['sha256'] == doc['historical_source_roles'][0]['sha256']
assert role['sha256'] == hashlib.sha256(sources[p + 'src/armor_design_output.rs'].encode()).hexdigest()
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries)
assert progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started'
assert candidate['validation'] == 'not performed'
print('verified five Git-bound creator sources and separately attributed historical role; no save or product ran')
