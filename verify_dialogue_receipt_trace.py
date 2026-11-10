"""Verify pinned dialogue receipt evidence without executing product code."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-DIALOGUE-RECEIPT-95E.json'
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
if len(sources) != 5:
    raise ValueError('incorrect source count')
prompts = sources['crates/adventuresim-stdb-module/src/strategic/dialogue_prompts.rs']
topic = prompts.split('pub fn choose_dialogue_topic(', 1)[1].split('fn dialogue_answer_is_committed_retry(', 1)[0]
for text in ('require_strategic_gateway(ctx)?', 'require_dialogue_revision(&catalog_revision)?',
             'validate_dialogue_action_id(&action_id)?', 'require_session_member(ctx, &session_id, character_id)?',
             'if ctx.db.dialogue_action().id().find(&action_row_id).is_some() {\n        return Ok(());',
             'session.catalog_revision != catalog_revision', 'session.revision != expected_revision'):
    if text not in topic:
        raise ValueError('topic evidence marker missing')
if not (topic.index('require_session_member') < topic.index('dialogue_action().id().find') <
        topic.index('session.catalog_revision != catalog_revision') < topic.index('session.revision != expected_revision')):
    raise ValueError('topic receipt ordering changed')
helper = prompts.split('fn dialogue_answer_is_committed_retry(', 1)[1].split('#[reducer]', 1)[0]
answer = prompts.split('pub fn answer_dialogue_prompt(', 1)[1]
if ('receipt.action_kind == format!("answer:{prompt_row_id}")' not in helper or
    'Err("Dialogue action ID conflicts with another action".into())' not in helper or
    not (answer.index('participant.role != prompt.respondent_role') <
         answer.index('dialogue_answer_is_committed_retry') <
         answer.index('serde_json::from_str(&choice_ids_json)'))):
    raise ValueError('answer receipt behavior changed')
schema = sources['crates/adventuresim-stdb-module/src/strategic/dialogue_schema.rs']
action = schema.split('pub struct DialogueAction {', 1)[1].split('\n}', 1)[0]
answer_row = schema.split('pub struct DialogueAnswer {', 1)[1].split('\n}', 1)[0]
if any(x in action for x in ('character_id', 'choice_ids_json', 'topic_id')) or any(
    x not in answer_row for x in ('character_id', 'choice_ids_json', 'prompt_row_id')):
    raise ValueError('receipt or answer source shape changed')
provenance = sources['crates/adventuresim-stdb-module/src/strategic/dialogue_provenance.rs']
for text in ('pub fn join_dialogue_session(', 'action_kind: "join".into()',
             'pub(crate) fn require_session_member(', 'DialogueSessionState::Active',
             'require_live_dialogue_presence(ctx, &session, character_id)?',
             'fn validate_dialogue_action_id('):
    if text not in provenance:
        raise ValueError('membership or join marker missing')
web = sources['crates/strategic-web/src/routes/dialogue.rs']
for text in ('async fn topic(', 'session.character_id_u64()', '"choose_dialogue_topic"',
             'json!(request.action_id)', 'build_view(&state, character_id, &request.session_id).await?',
             'async fn answer(', 'dialogue_prompt_row_belongs_to_session'):
    if text not in web:
        raise ValueError('web source marker missing')
client = sources['crates/strategic-web/static/dialogue-client.js']
for text in ('const createRetriableAction = (createId)', 'const actionId = () =>',
             'const chooseTopic = (binding)', 'const answerPrompt = (choices)',
             'action_id: actionId(), expected_revision: binding.revision',
             'const acceptance = createRetriableAction(actionId)'):
    if text not in client:
        raise ValueError('client source marker missing')

historical = doc['historical_source']
ledger = json.loads((root / historical['ledger']).read_text())
old = ledger['discovery_findings_and_dependencies']['ADLER-L11-DIALOGUE-ANSWER-RECEIPT']
fact = ledger['discovery_findings_and_dependencies']['ADLER-L11-CASE-FACT-SOURCE-REPLAY']
if (historical['checkpoint'] != '059' or not old['source_key'].startswith(historical['matching_source_sha256'] + ':') or
    old['accepted_unique_count_change'] != 0 or fact['accepted_unique_count_change'] != 0 or
    old['validation_status'] != 'not run' or fact['validation_status'] != 'not run'):
    raise ValueError('historical record qualification changed')
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    for key, sha_key in [('matching_role_index', 'matching_source_sha256'),
                         ('different_physical_p16_role_index', 'different_physical_p16_sha256')]:
        index = historical[key]
        sha = historical[sha_key]
        if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
            raise ValueError('historical source relationship mismatch')
if (historical['matching_source_sha256'] != hashlib.sha256(prompts.encode()).hexdigest() or
    historical['different_physical_p16_sha256'] == historical['matching_source_sha256']):
    raise ValueError('historical current/different distinction changed')
entries = baseline['newly_assessed_ledger']['entries']
if progress['new_current_ledger_candidates'] != len(entries) or len(doc['candidates']) != 1:
    raise ValueError('ledger count mismatch')
candidate = doc['candidates'][0]
matches = [x for x in entries if x['id'] == candidate['id']]
if (len(matches) != 1 or matches[0]['record'] != name or candidate['repair'] != 'not started' or
    candidate['validation'] != 'not performed' or not candidate['discovery'].startswith('partial')):
    raise ValueError('unqualified candidate')
print('verified five Git-bound dialogue sources and separately attributed historical receipt records')
