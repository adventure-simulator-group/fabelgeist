"""Verify preserved wedding evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-WEDDING-CAPACITY-95E.json'
doc = json.loads((root / name).read_text())
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
            raise ValueError('invalid read range')
    sources[item['path']] = raw.decode()
if len(sources) != 12:
    raise ValueError('incorrect source count')
markers = {
    'crates/adventuresim-stdb-module/src/relationship/marriage.rs': [
        'pub fn settle_due_weddings', 'DowryOutcomeKind::Paid',
        'crate::item::credit_personal_currency', 'ctx.db.dowry_escrow().commitment_id().delete',
        'ctx.db.household().insert(Household', 'join_household(',
        'crate::residence::move_residence_occupant_at(', 'CommitmentTerminalReason::WeddingCompleted'],
    'crates/adventuresim-stdb-module/src/relationship/mod.rs': [
        'pub fn settle_due_lifecycle_events_global', 'if let Err(error) = settle_due_weddings',
        'CommitmentTerminalReason::ResidenceUnavailable', 'record_lifecycle_failure', 'Ok(count)'],
    'crates/adventuresim-stdb-module/src/residence.rs': [
        'pub(crate) fn move_residence_occupant_at', 'occupancy::admit('],
    'crates/adventuresim-stdb-module/src/residence/occupancy.rs': [
        'pub(super) fn admit(', 'properties::require_property_room_at(ctx, &occupant, minute)?',
        'home.property_id == occupant.property_id && home.holding_id == occupant.holding_id',
        'pub(super) fn residents_at('],
    'crates/adventuresim-stdb-module/src/residence/properties.rs': [
        'pub(super) fn require_property_room_at(', 'named + aggregate >= u64::from(property.resident_capacity)',
        'Residence property has no remaining occupancy capacity'],
    'crates/adventuresim-stdb-module/src/relationship/family.rs': [
        'fn leave_household(', 'fn join_household(', 'leave_household(ctx, character_id);'],
    'crates/adventuresim-stdb-module/src/relationship/commitments.rs': [
        'fn transition_commitment_terminal(', 'status != CommitmentStatus::Fulfilled',
        'ctx.db.dowry_escrow().commitment_id().find(&commitment.id)',
        'pub(crate) fn reserve_wedding'],
    'crates/adventuresim-stdb-module/src/relationship/lifecycle.rs': [
        'fn processable(&self', 'canonical_now(ctx, character_id)'],
    'crates/adventuresim-stdb-module/src/npc_causal.rs': [
        'pub fn run_npc_causal_tick', 'settle_due_lifecycle_events_global(', 'Ok(())'],
    'crates/adventuresim-stdb-module/src/item.rs': ['pub fn credit_personal_currency'],
    'crates/adventuresim-stdb-module/src/relationship/tests/marriage.rs': [
        'fn wedding_contract_uses_effective_history_and_records_one_dowry_outcome'],
    'crates/adventuresim-stdb-module/src/relationship/tests/lifecycle.rs': [
        'fn global_lifecycle_selection_is_stable_non_starving_and_poison_tolerant'],
}
for path, values in markers.items():
    if any(value not in sources[path] for value in values):
        raise ValueError('source marker missing: ' + path)
wedding = sources['crates/adventuresim-stdb-module/src/relationship/marriage.rs'].split('pub fn settle_due_weddings(', 1)[1].split('pub fn settle_due_weddings_global(', 1)[0]
for left, right in [
    ('ctx.db.dowry_escrow().commitment_id().delete', 'ctx.db.household().insert(Household'),
    ('ctx.db.household().insert(Household', 'join_household('),
    ('join_household(', 'crate::residence::move_residence_occupant_at('),
    ('crate::residence::move_residence_occupant_at(', 'CommitmentTerminalReason::WeddingCompleted'),
]:
    if wedding.index(left) >= wedding.index(right):
        raise ValueError('wedding write/error order changed')

native = doc['native_source']
manifest_path = (root / native['package_manifest']).resolve()
if not manifest_path.is_relative_to(root):
    raise ValueError('unsafe native path')
manifest = json.loads(manifest_path.read_text())
raw = (manifest_path.parent / manifest['archive']).read_bytes()
if len(raw) != manifest['bytes'] or hashlib.sha256(raw).hexdigest() != manifest['sha256']:
    raise ValueError('native archive mismatch')
with tarfile.open(manifest_path.parent / manifest['archive'], 'r:gz') as tar:
    source = tar.extractfile(native['archive_member']).read()
expected = next(x for x in manifest['inspected_members'] if x['path'] == native['archive_member'])
if len(source) != expected['bytes'] or hashlib.sha256(source).hexdigest() != expected['sha256']:
    raise ValueError('native member mismatch')
if b'only committed if the reducer returns successfully' not in source:
    raise ValueError('native transaction documentation missing')
for first, last in native['ranges']:
    if not 1 <= first <= last <= len(source.splitlines()):
        raise ValueError('invalid native range')

historical = doc['historical_source']
ledger = json.loads((root / historical['ledger']).read_text())
old = ledger['discovery_findings_and_dependencies']['CONT-WEDDING-CAPACITY-POSTWRITE-ERROR']
if (historical['checkpoint'] != '059' or old['current_reconciled'] is not False or
    old['validated'] is not False or old['accepted_unique_count_change'] != 0 or old['source_lenses'] != [11]):
    raise ValueError('historical record qualification changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for item in historical['matching_pr_working_roles']:
        role = roles[item['original_index']]
        if (role['source_id'] != item['source_id'] or role['registry_pointer'] != item['registry_pointer'] or
            role['historical_sha256'] != item['sha256'] or archive_hashes[item['original_index']] != item['sha256']):
            raise ValueError('historical role mismatch')
        raw = tar.extractfile(item['sha256']).read()
        current_path = item['source_id'].removeprefix('pr-working:')
        if hashlib.sha256(raw).hexdigest() != item['sha256'] or raw != sources[current_path].encode():
            raise ValueError('historical/current byte relationship mismatch')
if len(historical['matching_pr_working_roles']) != 5:
    raise ValueError('incorrect historical role count')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [x for x in entries if x['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or
    candidate['repair'] != 'not started' or candidate['validation'] != 'not performed' or
    not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified twelve Git-bound sources, transaction documentation and five separately attributed historical role matches')
