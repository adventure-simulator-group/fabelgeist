"""Verify preserved presence evidence without executing product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-PRESENCE-SAFEGUARD-95E.json'
doc = json.loads((root / name).read_text())
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
if len(sources) != 7:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/settlement_population.rs': [
        'pub fn npc_is_present(', 'npc_presence_remaining_minutes_at(ctx, presence, minute).is_some()',
        'pub fn npc_presence_remaining_minutes_at_view(', 'patient_presence_suppression_at_view'],
    'crates/adventuresim-stdb-module/src/outbreak.rs': [
        'pub(crate) fn patient_presence_suppression_at(',
        'let alive_at_observer = crate::relationship::character_alive_at(ctx, character_id, minute)',
        'health_suppressed: !alive_at_observer', 'pub(crate) fn patient_presence_suppression_at_view(',
        '.is_none_or(|birth| minute.is_at_or_after_signed_birth(birth.birth_minute))'],
    'crates/adventuresim-stdb-module/src/relationship/chronology.rs': [
        'pub(crate) fn character_alive_at(',
        '.is_some_and(|birth| minute.is_at_or_after_signed_birth(birth.birth_minute))'],
    'crates/adventuresim-core/src/strategic_presence.rs': [
        'pub struct DailyPresenceWindow', 'if context_suppressed || health_suppressed',
        'return Err(PresenceError::Suppressed)'],
    'crates/adventuresim-stdb-module/src/character.rs': [
        'pub(crate) fn insert_character_with_origin(', 'crate::relationship::record_character_birth('],
    'crates/adventuresim-stdb-module/src/investigation/projections.rs': [
        'pub fn backend_investigation_actions(', 'fn projected_contact_schedule_wait_minutes(',
        'npc_presence_remaining_minutes_at_view(ctx, presence, minute)',
        'fn projected_contact_presence_availability('],
    'crates/adventuresim-stdb-module/src/investigation/actions.rs': [
        'fn validate_action_position(', 'npc_is_present(ctx, &presence, minute)'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
ledger = json.loads((root / doc['historical_source']['ledger']).read_text())
old = ledger['segovb_continuation']['lens12']['pending_connected_contracts'][78]
if (doc['historical_source']['checkpoint'] != '059' or old['source_original'] != 1188 or
    not any('Presenceviewmissingbirth' in s for s in old['contracts'])):
    raise ValueError('historical question qualification changed')
if name not in progress['new_nonfinding_assessments'] or len(doc['dispositions']) != 2:
    raise ValueError('missing qualified disposition')
print('verified seven Git-bound sources and separately attributed historical presence question')
