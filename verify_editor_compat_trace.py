"""Verify editor rule and retained source trace without running editor or product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-EDITOR-COMPAT-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        header, name = line.split('\t', 1)
        _, _, oid = header.split()
        assert name not in result
        result[name] = oid
    return result

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip' and item['revision'] == revision
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert oid == item['git_blob'] == current[item['path']] == previous[item['path']]
    assert all(1 <= a <= b <= item['lines'] for a,b in item['ranges']) and item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 11
p = 'crates/adventuresim-building-generator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)
must(sources['AGENTS.md'], ['Backward compatibility is intentionally unsupported.', 'compatibility\n  shims or fields', 'Remove any such path encountered within the\n  work\'s scope'])
must(sources['crates/AGENTS.md'], ['compatibility scaffolding'])
must(sources[p+'model.rs'], ['pub use player_build::*;'])
model = sources[p+'model/player_build.rs']
must(model, ['Compatibility-only working data for callers compiled against the first', '#[serde(skip)]\n    pub parts: Vec<PlayerBuildPart>', 'parts: Vec::new()', 'PlayerBuildEdit::Place { part: _ }', 'generic freeform parts were replaced by semantic building assemblies', 'PlayerBuildEdit::Move {', 'let part = next.part_mut(id)?;', 'PlayerBuildEdit::SetMaterial { id, material }'])
must(sources[p+'model/editor.rs'], ['pub struct EditableBuildingAssembly', 'pub storeys: Vec<StoreyPlan>', 'pub roofs: Vec<RoofPiece>', 'pub stairs: Vec<Stair>'])
commands = sources[p+'viewer/editor_commands.rs']
must(commands, ['.map(|document| document.parts.clone())', 'EditorCommand::PlacePart { part }', 'PlayerBuildEdit::Place { part }', 'Some(PlayerBuildDocument::empty())', 'serde_json::to_string_pretty(&snapshots)'])
must(sources[p+'viewer/editor_types.rs'], ['pub(crate) enum EditorCommand', 'PlacePart {', 'MovePart {', 'pub parts: Vec<PlayerBuildPart>'])
must(sources[p+'viewer/editor_updates.rs'], ['match document.apply(edit)', 'Err(error) => runtime.error = Some(error)'])
must(sources[p+'viewer/editor_ui.rs'], ['.map(|document| document.parts.clone())', 'for part in &player_parts', 'EditorUiAction::MovePlayerPart', 'PlayerBuildEdit::Move'])
must(sources[p+'viewer/player_build_scene.rs'], ['document.assembly.storeys', 'document.assembly.stairs'])
must(sources[p+'main.rs'], ['editor_script: Option<PathBuf>', 'viewer::run_editor_script(&script)'])
for f in ('baseline-059/ledger.json', 'baseline-059/queue.json'):
    raw = (root / f).read_text().lower()
    for term in ('playerbuildpart', 'playerbuildedit', 'player_build.rs', 'freeform parts', 'compatibility-only working data'):
        assert term not in raw
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = baseline['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified eleven Git-bound rules and editor sources; no editor or product ran')
