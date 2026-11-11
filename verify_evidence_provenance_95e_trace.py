"""Verify evidence provenance audit bytes and source relationships; no product runs."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
obj = lambda kind,b: hashlib.sha1(kind.encode() + b' ' + str(len(b)).encode() + b'\0' + b).hexdigest()
doc = load('ASSESSMENT-L05-EVIDENCE-PROVENANCE-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
cid = 'REVIEW-EVIDENCE-ACTION-INSPECTION-PROVENANCE-001'
assert doc['source_revision'] == rev and doc['lens'] == 5 and doc['candidates'][0]['id'] == cid
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    metadata, path = line.split('\t',1); tree[path] = metadata.split()[2]
contents = {}
for source in doc['current_sources']:
    raw = gzip.decompress((root / source['preserved_bytes']).read_bytes())
    assert sha(raw) == source['sha256'] and obj('blob',raw) == source['git_blob'] == tree[source['path']]
    assert len(raw) == source['bytes'] and len(raw.splitlines()) == source['lines']
    assert source['revision'] == rev and all(1 <= a <= b <= source['lines'] for a,b in source['ranges'])
    contents[source['path'].split('/src/',1)[-1]] = raw
model = contents['investigation/model.rs']
knowledge = model.split(b'pub(crate) fn record_evidence_knowledge(',1)[1].split(b'fn inspection_stat_milli',1)[0]
assert b'"evidence-knowledge"' in knowledge and b'existing.source_id == source_id' in knowledge
assert b'Evidence knowledge has conflicting provenance' in knowledge
inspection = model.split(b'pub fn inspect_physical_evidence(',1)[1].split(b'pub struct InvestigationBelief',1)[0]
assert b'if inspection.passed && reveals_clue' in inspection
assert b'evidence-inspection:{evidence_id}:{topic_id}' in inspection
assert inspection.index(b'record_evidence_knowledge(') < inspection.index(b'physical_evidence_inspection_action_receipt().insert')
actions = contents['investigation/actions.rs']
assert b'.unwrap_or_else(|| attempt_id.to_owned())' in actions and b'record_evidence_knowledge(' in actions
assert b'let attempt_id = inv::compound_id(&[' in actions and b'"attempt",' in actions
assert b'else {\n        return Ok(None);\n    };' in contents['outbreak.rs']
assert b'let Some(outbreak) = &generated.outbreak else' in contents['outbreak.rs']
assembly = contents['quest_generation/assembly.rs']
assert b'pattern_evidence_id.clone(),\n            EvidenceKind::LedgerEntry' in assembly
assert b'evidence_site.clone(),\n            format!("Corroborated accounts show:' in assembly
assert b'exact_location_initially_known: true' in assembly
solver = contents['quest_generation/solver.rs']
assert solver.count(b'evidence_id: pattern_evidence_id.clone()') >= 2
assert b'evidence_presentation(kind, &id, investigability)' in contents['quest_generation/projection.rs']
assert b'presentation_kind: crate::investigation::EvidencePresentationKind::Physical' in contents['strategic/mission_bootstrap.rs']
assert b'backend_case_site_pins(ctx)' in contents['investigation/projections.rs']
assert b'StatusCode::CONFLICT' in contents['routes/evidence.rs']
content = json.loads(contents['content/quests/investigation.yaml'])
entries = [{'id':e['id'],'revealing_topics':[t['id'] for t in e['topics'] if (t.get('check') or {}).get('reveals_clue') is True]} for e in content['evidence']]
assert entries == doc['authored_content_inventory']['entries'] and len(entries) == 11
assert all(len(e['revealing_topics']) == 1 for e in entries)
assert next(e for e in entries if e['id']=='ledger_entry')['revealing_topics'] == ['correction']
assert sha(contents['content/quests/investigation.yaml']) == doc['authored_content_inventory']['source_sha256']
# Source-defined prefixes are unequal for a symbolic actor/action/evidence;
# no generated source or product function is invoked.
assert '7.attempt:...' != 'evidence-inspection:evidence:correction'
search = doc['search_evidence'];raw=(root/search['path']).read_bytes()
assert sha(raw) == search['sha256'] and len(raw) == search['bytes'] and len(raw.splitlines()) == search['lines'] == 4
manifest = {e['path']:e for e in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_receipts'] + doc['recovered_historical_support']
assert len(records) == 11
for record in records:
    raw = gzip.decompress((root / record['preserved_bytes']).read_bytes()); m=manifest[record['original_path']]
    assert sha(raw) == record['sha256'] == m['sha256']
    assert obj('blob',raw) == record['git_blob'] == m['git_blob'] and len(raw) == record['bytes'] == m['bytes']
old = [json.loads(gzip.decompress((root/r['preserved_bytes']).read_bytes())) for r in doc['recovered_historical_receipts']]
assert old[0]['source_revision'] == 'e2755a732c84e67296b5e3441a4259a84f461b2f'
assert old[0]['accepted_unique_count_change'] == old[1]['accepted_unique_count_change'] == 0
assert old[1]['current_source_reconciled'] is False
assert old[0]['rows'][0]['sha256'] == old[1]['rows'][0]['source_sha256'] == sha(model)
# Verify the historic commit and each raw Git tree along its path.
proof = load(doc['historical_current_read']['git_object_proof'])
objects = proof['objects']
assert objects[0]['object_type'] == 'commit' and objects[0]['git_oid'] == proof['commit']
raw_objects = []
for record in objects:
    raw = gzip.decompress((root / record['preserved_bytes']).read_bytes())
    assert len(raw) == record['bytes'] and sha(raw) == record['sha256']
    assert obj(record['object_type'],raw) == record['git_oid']
    raw_objects.append(raw)
assert proof['commit'] == doc['historical_current_read']['source_revision']
next_oid = next(line.split()[1].decode() for line in raw_objects[0].splitlines() if line.startswith(b'tree '))
for component, record, raw in zip(proof['components'],objects[1:],raw_objects[1:]):
    assert record['object_type'] == 'tree' and record['git_oid'] == next_oid
    position=0; found=None
    while position < len(raw):
        end=raw.index(b'\0',position); mode,name=raw[position:end].split(b' ',1)
        target=raw[end+1:end+21].hex(); position=end+21
        if name.decode() == component: found=(mode,target); break
    assert found is not None; mode,next_oid=found
assert next_oid == proof['leaf_git_blob'] == obj('blob',model) and proof['leaf_sha256'] == sha(model)
historical=load('ASSESSMENT-L01-HISTORICAL-1126-1127.json')['historical_source_roles']
assert [r['original_index'] for r in historical] == [1126,1127]
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for role in historical:
        raw=archive.extractfile(role['sha256']).read()
        assert sha(raw)==role['sha256'] and len(raw.splitlines())==role['lines']
        assert role['new_personal_read_ranges']==[[1,role['lines']]] and role['current_comparison']['byte_identical'] is True
        assert obj('blob',raw)==tree[role['source_id'].split(':',1)[1]]
b=load('BASELINE-95E-20261110.json');assert len([e for e in b['newly_assessed_ledger']['entries'] if e['id']==cid and e['record']=='ASSESSMENT-L05-EVIDENCE-PROVENANCE-95E.json'])==1
q=load('RECONCILIATION-059.json')['entries'][934];assert q['contract_id']=='Q-L12-92782b71b10de599664f' and q['current_reconciliation'].startswith('partial ')
p=load('CONTINUATION-PROGRESS-95E.json');assert p['queue_records_semantically_reconciled']==0 and p['new_current_ledger_candidates']==len(b['newly_assessed_ledger']['entries'])
print('verified action/inspection evidence, content inventory and historical Git path; no product ran')
