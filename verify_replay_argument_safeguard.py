"""Verify preserved source for the bounded replay-argument disposition."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-REPLAY-ARGUMENT-SAFEGUARD-95E.json'
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
if len(sources) != 4:
    raise ValueError('incorrect source count')
report = sources['crates/adventuresim-strategic-sim/src/investigation_eval/report.rs']
promotion = report.split('pub fn promote_replay_candidate(', 1)[1].split('fn observable_state_digest(', 1)[0]
for marker in ('choice_id: event.choice_id.clone()', 'arguments: Default::default()',
               'Termination::StepLimit | Termination::DeadEnd'):
    if marker not in promotion:
        raise ValueError('replay promotion marker missing')
env = sources['crates/adventuresim-strategic-sim/src/investigation_eval/environment.rs']
if env.count('choices.push(LegalChoice {') != 1 or env.count('self.frame.legal_choices = choices;') != 1:
    raise ValueError('choice construction scope changed')
builder = env.split('fn push_choice(', 1)[1].split('fn validate_arguments(', 1)[0]
validation = env.split('fn validate_arguments(', 1)[1].split('fn choice_id(', 1)[0]
for marker in ('allowed: Vec::<ArgumentValue>::new()', 'self.capabilities.insert(id.clone(), capability)'):
    if marker not in builder:
        raise ValueError('empty choice argument construction changed')
if 'None if allowed.allowed.is_empty() => Ok(())' not in validation or 'Err("typed choice arguments are not legal for this capability".into())' not in validation:
    raise ValueError('argument admission changed')
if 'validate_arguments(&legal.typed_arguments, &decision.arguments)?' not in env or 'pub fn frame(&self) -> &PlayerFrame' not in env:
    raise ValueError('environment admission boundary changed')
types = sources['crates/adventuresim-strategic-sim/src/investigation_eval/types.rs']
for marker in ('pub struct DecisionArguments', 'pub selection: Option<String>', 'pub struct PublicTraceEvent', 'pub choice_id: ChoiceId'):
    if marker not in types:
        raise ValueError('public type marker missing')
policy = sources['crates/adventuresim-strategic-sim/src/investigation_eval/policy.rs']
if 'arguments: decision.arguments' not in policy:
    raise ValueError('policy wire marker missing')
historical = doc['historical_source']
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
manifest = json.loads((root / 'recovered/original-registry-source-archive/MANIFEST.json').read_text())
archive_hashes = {item['original']: item['sha256'] for item in manifest['sources']}
index, sha = historical['original_role_index'], historical['sha256']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz') as tar:
    if roles[index]['historical_sha256'] != sha or archive_hashes[index] != sha or hashlib.sha256(tar.extractfile(sha).read()).hexdigest() != sha:
        raise ValueError('historical source relationship mismatch')
if sha != hashlib.sha256(report.encode()).hexdigest() or name not in progress['new_nonfinding_assessments'] or len(doc['dispositions']) != 1:
    raise ValueError('nonfinding qualification changed')
print('verified four Git-bound sources and separately attributed historical report')
