"""Verify preserved religious-demand source evidence, not product behavior."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
doc = json.loads((root / 'ASSESSMENT-L01-RELIGIOUS-DEMAND-95E.json').read_text())
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
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
            raise ValueError('invalid source range')
    sources[item['path']] = raw.decode()
if len(sources) != 13:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/condition.rs': [
        '#[table(accessor = religious_demand, public)]', 'fn ensure_holy_day_demand',
        'ensure_holy_day_demand(ctx, &requested)?', 'pub fn resolve_religious_demand',
        'if character.server != ctx.sender()', 'demand.status = "resolved".into()'],
    'crates/adventuresim-stdb-module/src/character.rs': [
        'Sole tactical assignment authority; zero means unassigned', 'server: Identity::ZERO'],
    'crates/adventuresim-stdb-module/src/tactical.rs': [
        'character.server = server.identity;', 'character.server = Identity::ZERO;'],
    'crates/adventuresim-stdb-module/src/strategic/authority_model.rs': [
        'if ctx.sender() == Identity::ZERO', 'pub fn register_strategic_gateway',
        'pub(crate) fn require_strategic_character_authority'],
    'crates/strategic-web/src/routes/settlements/religion_party.rs': [
        'session.character_id_u64() == Some(character_id)',
        '"resolve_religious_demand"', '&[json!(demand_id), json!(form.choice)]',
        'tracing::warn!', 'Redirect::to('],
    'crates/strategic-web/src/templates/settlement/religion.rs': [
        'fn religious_demand_rail', 'name="choice" value="observe"', 'name="choice" value="refuse"'],
    'crates/strategic-web/src/routes/settlements/party/location_personal.rs': [
        'SELECT * FROM religious_demand WHERE character_id = {character_id}',
        "status = 'pending'"],
    'crates/strategic-web/src/spacetimedb/client.rs': [
        'pub fn with_token', 'pub async fn call', 'request.header("Authorization"',
        'if !response.status().is_success()'],
    'crates/strategic-web/src/main.rs': [
        '.with_token(config.spacetimedb_token.clone())', '"register_strategic_gateway"',
        'let state = AppState {'],
    'crates/strategic-web/src/session.rs': ['pub fn character_id_u64(&self)'],
    'crates/strategic-web/src/routes/settlements/router.rs': [
        'paths::RESOLVE_RELIGIOUS_DEMAND.pattern()', 'post(resolve_religious_demand)'],
    'crates/strategic-web/src/location_urls/patterns.rs': [
        '/party/{character_id}/religious-demand/{demand_id}'],
    'crates/strategic-web/src/routes/app_state.rs': ['pub db: SpacetimeClient'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
route = sources['crates/strategic-web/src/routes/settlements/religion_party.rs']
if 'demand.character_id' in route:
    raise ValueError('route ownership observation changed')

candidate = doc['candidate']
entries = baseline['newly_assessed_ledger']['entries']
matches = [item for item in entries if item['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != 'ASSESSMENT-L01-RELIGIOUS-DEMAND-95E.json' or
    progress['new_current_ledger_candidates'] != len(entries) or
    candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
    not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate or ledger mismatch')
print('verified 13 Git-bound sources and qualified religious-demand candidate')
