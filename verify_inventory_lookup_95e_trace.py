"""Verify selected audit evidence and symbolic comparison, never product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
doc = load('ASSESSMENT-L08-INVENTORY-LOOKUP-95E.json')
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
assert doc['source_revision'] == rev and doc['lens'] == 8
assert doc['candidates'][0]['id'] == 'NEW-INV-LOOKUP-001'
assert load('ASSESSMENT-INVENTORY-LOOKUP.json')['id'] == doc['candidates'][0]['id']
assert load('BASELINE-20261110.json')['repair_ledger']['entries'][0]['id'] == doc['candidates'][0]['id']
entry = [e for e in load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'] if e['id'] == doc['candidates'][0]['id']]
assert len(entry) == 1 and entry[0]['record'] == 'ASSESSMENT-L08-INVENTORY-LOOKUP-95E.json'
tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    metadata, path = line.split('\t', 1)
    tree[path] = metadata.split()[2]
contents = {}
for source in doc['current_sources']:
    raw = gzip.decompress((root / source['preserved_bytes']).read_bytes())
    assert sha(raw) == source['sha256'] and blob(raw) == source['git_blob'] == tree[source['path']]
    assert len(raw) == source['bytes'] and len(raw.splitlines()) == source['lines']
    assert source['revision'] == rev and all(1 <= a <= b <= source['lines'] for a,b in source['ranges'])
    contents[source['path'].split('/src/', 1)[-1]] = raw
assert b'.filter(item_id)' in contents['inventory_container/object_lookup.rs']
assert b'None => Box::new(ctx.db.inventory_object().iter().filter(is_row_object))' in contents['inventory_container/object_lookup.rs']
assert b'if object.item_id != item_id || object.location != location' in contents['food/lot_inventory.rs']
assert b'Ok(ctx.db.inventory_object().insert(crate::InventoryObject {' in contents['food/lot_inventory.rs']
assert b'.filter(|candidate| same_backing_row' in contents['object_custody.rs']
assert b'.filter(""..)' in contents['food/projections.rs']
assert b'.ok()\n                .flatten()' in contents['food/ingredient_preparation.rs']
assert b'if let Some(object) = container_object' in contents['strategic/inventory_trade.rs']
assert b'insert_personal_object(ctx, &item)' in contents['item.rs']
historical = doc['historical_source']
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    raw = archive.extractfile(historical['sha256']).read()
assert sha(raw) == historical['sha256']
old = raw.split(b'pub(crate) fn object_for_row(',1)[1].split(b'fn insert_carried_object(',1)[0]
assert b'.inventory_object()\n            .iter()' in old and b'.filter(item_id)' not in old
assert b'if matches.next().is_some()' in old
assert sha(contents['inventory_container.rs']) != historical['sha256']
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert sha(raw) == search['sha256'] and len(raw) == search['bytes']
assert len(raw.splitlines()) == search['lines'] == 45
manifest = {e['path']: e for e in json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records = doc['recovered_historical_receipts'] + doc['recovered_historical_support']
assert len(records) == 10
for record in records:
    raw = gzip.decompress((root / record['preserved_bytes']).read_bytes())
    m = manifest[record['original_path']]
    assert sha(raw) == record['sha256'] == m['sha256']
    assert blob(raw) == record['git_blob'] == m['git_blob'] and len(raw) == record['bytes'] == m['bytes']
oldreceipts = [json.loads(gzip.decompress((root / r['preserved_bytes']).read_bytes())) for r in doc['recovered_historical_receipts']]
assert [x['rows'][0]['original_index'] for x in oldreceipts] == [1105,1119]
assert all(x['current_source_reconciled'] is False and x['accepted_unique_count_change'] == 0 for x in oldreceipts)
queue = load('RECONCILIATION-059.json')['entries']
for i, cid in ((904,'Q-L12-b4d507c50e82a55abe7a'), (918,'Q-L12-67cc48fef45cac7ee3a0')):
    assert queue[i]['contract_id'] == cid and queue[i]['current_reconciliation'].startswith('partial ')
assert load('CONTINUATION-PROGRESS-95E.json')['queue_records_semantically_reconciled'] == 0
assert 'ASSESSMENT-L08-INVENTORY-LOOKUP-95E.json' in load('CONTINUATION-PROGRESS-95E.json')['post_transition_bounded_behavior_traces']
# A symbolic set comparison follows the inspected predicates, not executed product code.
rows = [('apple', 'R')]
objects = [('pear', 'R')]
old_matches = [x for x in objects if x[1] == 'R']
new_matches = [x for x in objects if x[0] == rows[0][0] and x[1] == 'R']
assert old_matches == [('pear','R')] and not new_matches
objects.append(('apple','R'))
assert len([x for x in objects if x[1] == 'R']) == 2
assert [x for x in objects if x[0] == rows[0][0] and x[1] == 'R'] == [('apple','R')]
print('verified selected lookup evidence and symbolic predicate comparison; no product ran')
