"""Verify selected source, old receipts and arithmetic, not product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
files = ['ASSESSMENT-L04-BELIEF-REVISION-95E.json','ASSESSMENT-L05-TESTIMONY-CORRECTION-95E.json']
docs = [load(x) for x in files]
ids = ['PL13-260-BELIEF-REVISION-COUNT-NARROWING-PENDING','PL13-260-INVESTIGATION-CORRECTION-PROPOSITION-IDENTITY-PENDING']
assert [d['lens'] for d in docs] == [4,5] and [d['candidates'][0]['id'] for d in docs] == ids
assert all(d['source_revision'] == rev and d['repair'] == 'not started' and d['validation'] == 'not performed' for d in docs)
assert docs[0]['current_sources'] == docs[1]['current_sources']
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    metadata, path = line.split('\t', 1); tree[path] = metadata.split()[2]
contents = {}
for source in docs[0]['current_sources']:
    raw = gzip.decompress((root / source['preserved_bytes']).read_bytes())
    assert sha(raw) == source['sha256'] and blob(raw) == source['git_blob'] == tree[source['path']]
    assert len(raw) == source['bytes'] and len(raw.splitlines()) == source['lines']
    assert source['revision'] == rev and all(1 <= a <= b <= source['lines'] for a,b in source['ranges'])
    contents[source['path'].split('/src/',1)[-1]] = raw
claims = contents['investigation/claims.rs']
assert claims.count(b'.saturating_add(1) as u16') == 2
assert b'belief.proposition_id != receipt.proposition_id' in claims
assert b'.corrects_proposition_id' in claims and b'serde_json::to_string(&pipeline)' in claims
assert b'if let Some(existing_share)' in claims and b'&source.current_revision_id' in claims
assert b'pub revision: u16' in contents['investigation/model.rs']
projection = contents['quest_generation/projection.rs']
assert b'PropositionId::new(draft.proposition_id.clone())' in projection
assert b'&format!("{character_id}:{}:{index}", witness.id.0)' in projection
assert b'proposition_id: input.proposition.id' in contents['investigation.rs']
developer = contents['developer_quest.rs']
assert b'MAX_DEVELOPER_COLLECTION_ITEMS: usize = 64' in developer
assert b'MAX_DEVELOPER_TOTAL_ITEMS: usize = 512' in developer
assert b'validate_recursive_bounds(definition, &mut diagnostics)' in developer
assert b'item.tier == DiagnosticTier::Structural' in developer
assert b'&testimony.proposition_id' in developer and b'&testimony.corrects_proposition_id' in developer
assert b'remap(&mut testimony.proposition_id, &replacements)' in developer
assert b'remap(proposition_id, &replacements)' in developer
validation = contents['quest_generation/validation.rs']
assert validation.count(b'corrects_proposition_id') == 1
assert b'qg::generate(&context)' in contents['strategic/authority_model.rs']
assert b'developer_quest::compile(developer)' in contents['strategic/authority_model.rs']
assembly = contents['quest_generation/assembly.rs']
assert b'let mut primary_testimony = vec![' in assembly and b'primary_testimony.push(TestimonyDraft {' in assembly
assert b'corrects_proposition_id: secondary_corrects_proposition_id' in assembly
# Symbolic arithmetic and identity comparison, with no product execution.
assert (65535 + 1) % (2 ** 16) == 0 and (65536 + 1) % (2 ** 16) == 1
prior_proposition, new_proposition = 'A', 'B'
assert prior_proposition != new_proposition
search = docs[0]['search_evidence']; raw = (root / search['path']).read_bytes()
assert sha(raw) == search['sha256'] and len(raw) == search['bytes'] and len(raw.splitlines()) == search['lines']
manifest = {e['path']: e for e in json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = docs[0]['recovered_historical_receipts'] + docs[0]['recovered_historical_support']
assert len(records) == 11 and docs[1]['recovered_historical_support'] == docs[0]['recovered_historical_support']
for record in records:
    raw = gzip.decompress((root / record['preserved_bytes']).read_bytes()); m = manifest[record['original_path']]
    assert sha(raw) == record['sha256'] == m['sha256']
    assert blob(raw) == record['git_blob'] == m['git_blob'] and len(raw) == record['bytes'] == m['bytes']
old = [json.loads(gzip.decompress((root / r['preserved_bytes']).read_bytes())) for r in docs[0]['recovered_historical_receipts']]
assert all(x['accepted_unique_count_change'] == 0 and x['current_source_reconciled'] is False for x in old)
historical = load('ASSESSMENT-L01-HISTORICAL-1123-1125.json')['historical_source_roles']
assert [r['original_index'] for r in historical] == [1123,1124,1125]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for role in historical:
        raw = archive.extractfile(role['sha256']).read()
        assert sha(raw) == role['sha256'] and len(raw.splitlines()) == role['lines']
        assert role['new_personal_read_ranges'] == [[1,role['lines']]]
        assert role['current_comparison']['byte_identical'] is True and role['current_comparison']['sha256'] == role['sha256']
        assert blob(raw) == tree[role['source_id'].split(':',1)[1]]
baseline = load('BASELINE-95E-20261110.json')
for cid,name in zip(ids,files):
    assert len([e for e in baseline['newly_assessed_ledger']['entries'] if e['id'] == cid and e['record'] == name]) == 1
q = load('RECONCILIATION-059.json')['entries']
assert q[2339]['contract_id'] == ids[0] and q[2338]['contract_id'] == ids[1]
assert q[2339]['current_reconciliation'].startswith('partial ') and q[2338]['current_reconciliation'].startswith('partial ')
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['queue_records_semantically_reconciled'] == 0 and p['new_current_ledger_candidates'] == len(baseline['newly_assessed_ledger']['entries'])
obs = load(baseline['latest_main_recheck'])
assert obs['observed_revision'] == rev and obs['method'] in ('GitHub connector fetch_commit with commit_sha main', 'GitHub connector fetch branches/main API')
print('verified selected belief sources, historical receipts and symbolic arithmetic; no product ran')
