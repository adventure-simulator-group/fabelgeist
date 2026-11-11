"""Verify dialogue source and historical evidence only; no product code runs."""
import collections
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L05-DIALOGUE-PRIORITY-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 5 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
    assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 10
build = sources['crates/adventuresim-dialogue/build.rs']
must(build, ['other.priority == response.priority', 'other.conditions == response.conditions',
             'Condition::All { conditions } | Condition::Any { conditions }',
             'validate_document(&typed, &relative, &mut global_ids);',
             'fs::read_dir(&content)', 'p.extension().is_some_and(|e| e == "yaml")'])
schema = sources['crates/adventuresim-dialogue/src/authoring_schema.rs']
must(schema, ['pub enum Condition {', '    Always,', '    All {', 'conditions: Vec<Condition>'])
condition = sources['crates/adventuresim-dialogue/src/condition_validation.rs']
must(condition, ['Condition::All { conditions } | Condition::Any { conditions }',
                 'for child in conditions {', 'Condition::Always => {}'])
runtime = sources['crates/adventuresim-dialogue/src/lib.rs']
must(runtime, ['Condition::Always => true', 'conditions.iter().all(|c| self.matches(c))',
               'other.priority == response.priority && other.conditions == response.conditions',
               '.filter(|r| facts.matches(&r.conditions))', '.map(|r| r.priority)',
               'if winners.next().is_some()', 'return Err(DialogueError::AmbiguousPriority'])
must(sources['crates/adventuresim-stdb-module/src/strategic/dialogue_sessions.rs'],
     ['adventuresim_dialogue::validate(adventuresim_dialogue::catalog())',
      'adventuresim_dialogue::select_start_response(conversation, &facts)',
      'No unambiguous eligible conversation greeting'])
for path in ('crates/adventuresim-stdb-module/src/strategic/dialogue_bindings.rs',
             'crates/adventuresim-stdb-module/src/strategic/dialogue_prompts.rs'):
    must(sources[path], ['adventuresim_dialogue::select_response(topic, &facts)',
                         'No unambiguous eligible dialogue response'])
search = doc['search_evidence']
raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
paths = set()
for line in raw.decode().splitlines():
    assert line.startswith(revision + ':')
    paths.add(line[len(revision)+1:].split(':', 1)[0])
assert paths == {'crates/adventuresim-dialogue/src/lib.rs',
                 'crates/adventuresim-stdb-module/src/strategic/dialogue_bindings.rs',
                 'crates/adventuresim-stdb-module/src/strategic/dialogue_prompts.rs',
                 'crates/adventuresim-stdb-module/src/strategic/dialogue_sessions.rs'}
# Recreate only the recorded structural index; this does not evaluate conditions.
evidence = doc['corpus_evidence']
raw = (root / evidence['path']).read_bytes()
assert len(raw) == evidence['bytes'] and hashlib.sha256(raw).hexdigest() == evidence['sha256']
groups = []
for path in ('content/dialogue/examples.yaml', 'content/dialogue/organizations.yaml',
             'content/dialogue/services.yaml'):
    authored = json.loads(sources[path])
    for conversation in authored['conversations']:
        sets = [('start', 'on_start', conversation.get('on_start', []))]
        sets += [('topic', topic['id'], topic['responses']) for topic in conversation['topics']]
        for kind, label, responses in sets:
            by_priority = collections.defaultdict(list)
            for response in responses:
                by_priority[response['priority']].append({'id': response['id'],
                    'conditions': response.get('conditions', {'op': 'always'})})
            for priority, group in by_priority.items():
                if len(group) > 1:
                    groups.append({'file': path, 'conversation': conversation['id'],
                                   'kind': kind, 'label': label, 'priority': priority,
                                   'responses': group})
assert len(groups) == 7 and json.loads(raw)['groups'] == groups
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 5
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any(x['id'] == 'PL13-AUTHORED-DIALOGUE-NUMERIC-ROLES' and 'pending' in x['status']
           for x in old['evidence']['connected_contracts'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration']):
    path = ref['path'].removeprefix('target/726-audit-adler-20261027/')
    assert support[path][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][91]
assert prior['receipt'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert prior['contract']['id'] == 'PL13-AUTHORED-DIALOGUE-NUMERIC-ROLES'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_aggregate_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified dialogue source, structural index and 5 historical records; no product ran')
