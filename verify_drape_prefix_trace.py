"""Verify bounded drape-prefix evidence without running the application."""
import gzip
import hashlib
import json
from pathlib import Path

root=Path(__file__).resolve().parent
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-L01-DRAPE-PREFIX-95E.json'
doc=json.loads((root/name).read_text())
assert doc['lens']==1 and doc['source_revision']==revision

def tree(path):
    result={}
    for line in (root/path).read_text().splitlines():
        header,filename=line.split('\t',1)
        _,kind,oid=header.split()
        assert filename not in result and kind in ('blob','commit')
        result[filename]=oid
    return result

current=tree('main-advance-95e/current-tree.tsv')
previous=tree('main-advance-95e/previous-tree.tsv')
sources={}
for item in doc['current_sources']:
    path=(root/item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding']=='gzip'
    raw=gzip.decompress(path.read_bytes())
    oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
    assert item['revision']==revision and oid==item['git_blob']
    assert current[item['path']]==previous[item['path']]==oid
    assert hashlib.sha256(raw).hexdigest()==item['sha256']
    assert len(raw)==item['bytes'] and len(raw.splitlines())==item['lines']
    assert all(1<=a<=b<=item['lines'] for a,b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']]=raw.decode()
assert len(sources)==3
p='crates/adventuresim-character-creator/src/'
def must(source,markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: '+marker)
worker=sources[p+'drape_worker.rs']
must(worker,['self.cancel.store(true, Ordering::Relaxed);',
    'self.pending = (!input.is_empty()).then_some(input);',
    'preview: if self.cancel.load(Ordering::Relaxed)',
    'if !self.cancel.load(Ordering::Relaxed)',
    'let (tags, input): (Vec<_>, Vec<_>) = pending.into_iter().unzip();',
    'self.draping = tags;', 'drape_outfit(input, previous, &cancel, |garments|'])
assert worker.index('let (tags, input):') < worker.index('self.draping = tags;') < worker.index('drape_outfit(input, previous')
outfit=sources[p+'garment/outfit.rs']
must(outfit,['for (index, mut input) in inputs.into_iter().enumerate()',
    'let mut snapshot = finished.clone();', 'snapshot.push(current);',
    'snapshot.push(garment.clone());', 'finished.push(garment);',
    'Err(error) => {', 'garments: finished,', 'return OutfitOutcome {'])
preview=sources[p+'drape_preview.rs']
must(preview,['self.worker.request(input);', 'self.ready = None;',
    'job.ready = Some(outcome.garments);',
    'for (&id, garment) in job.worker.draping().iter().zip(garments)',
    '(CharacterMesh, DrapeMesh(id))'])
work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_sources']:
    role=work['roles'][reference['original_role_index']]
    assert role['source_id']==reference['source_id'] and role['historical_sha256']==reference['sha256']
    assert reference['byte_identical']==(reference['sha256']==reference['current_sha256'])
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['new_nonfinding_assessments']
assert progress['queue_records_semantically_reconciled']==0
assert doc['dispositions'][0]['validation'].startswith('not performed')
print('verified three Git-bound drape sources and historical identities; no product ran')
