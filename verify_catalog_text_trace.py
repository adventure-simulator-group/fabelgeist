"""Verify catalog source bytes and marker inventory; not product behavior."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-CATALOG-TEXT-95E.json'; doc=json.loads((root/name).read_text()); assert doc['lens']==1 and doc['source_revision']==rev

def tree(path):
 out={}
 for line in (root/path).read_text().splitlines():
  h,p=line.split('\t',1); _,_,oid=h.split(); assert p not in out; out[p]=oid
 return out
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev
 raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']] and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); assert s['path'] not in source; source[s['path']]=raw.decode()
assert len(source)==10
catalog=source['content/items/catalog.yaml']; build=source['crates/adventuresim-core/build.rs']; validation=source['crates/adventuresim-core/src/item_catalog_validation.rs']; schema=source['crates/adventuresim-core/src/item_catalog_schema.rs']; core=source['crates/adventuresim-core/src/item_catalog.rs']; currency=source['crates/adventuresim-core/src/strategic_currency.rs']; components=source['crates/strategic-web/src/templates/components.rs']; inventory=source['crates/strategic-web/src/templates/settlement/trade/inventory.rs']; js=source['crates/strategic-web/static/inventory-browser.js']; merchant=source['crates/strategic-web/src/templates/settlement/trade/merchant.rs']
for marker in ('let content = root.join("content/items");','let value = item_catalog_validation::parse_document(&text)', 'item_catalog_validation::validate_documents_with_sources', 'let json = serde_json::to_string(&documents).unwrap();','pub const ITEM_CATALOG_JSON: &str'): assert marker in build
for marker in ('serde_json::from_str::<StrictValue>(text)', 'required_string(item, "display_name", &path, errors);', 'is_some_and(|value| value.len() > MAX_STRING_BYTES)', '.is_none_or(str::is_empty)'): assert marker in validation
assert 'pub display_name: String,' in schema and 'serde_json::from_str(ITEM_CATALOG_JSON)' in core and 'binary_search_by_key(&id' in core
assert 'pub fn currency_name(item_id: &str)' in currency and '.map(|item| item.display_name.as_str())' in currency
assert 'return item.display_name.clone();' in components and 'data-currency-name=(currency_name)' in inventory and 'expect("Lübeck mark remains an authored currency")' in inventory
assert 'label.textContent = label.dataset.currencyName' in js and 'label.dataset.itemName = label.dataset.currencyName' in js
assert 'let display_name = item_display_name(&item.id)' in merchant and 'item_name_with_display(&item.id, &display_name, Some(item))' in merchant
h=doc['historical_source']; roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; role=roles[h['role_index']]; assert role['source_id']==h['source_id'] and role['historical_sha256']==h['sha256'] and h['personal_read_ranges']==[[1,3000]]
with tarfile.open(root/h['evidence_archive']) as t: old=t.extractfile(h['sha256']).read()
assert hashlib.sha256(old).hexdigest()==h['sha256'] and hashlib.sha256(catalog.encode()).hexdigest()!=h['sha256']
def markers(raw): return {x['id']:x['display_name'] for x in json.loads(raw)['items'] if any(ch in x['display_name'] for ch in ('\u00c3','\u00c2','\ufffd'))}
current=markers(catalog); historical=markers(old); inv=doc['mechanical_marker_inventory']; assert current==inv['current'] and historical==inv['historical'] and len(current)==len(historical)==9 and set(current)==set(historical) and current['lubeck_mark']!=historical['lubeck_mark'] and current['lubeck_mark']!='Lübeck mark'
for f in ['baseline-059/ledger.json','baseline-059/queue.json']:
 text=(root/f).read_text().casefold(); assert not any(needle in text for needle in ('lubeck','lübeck','mojibake'))
c=doc['candidates'][0]; entries=json.loads((root/'BASELINE-95E-20261110.json').read_text())['newly_assessed_ledger']['entries']; progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert sum(x['id']==c['id'] and x['record']==name for x in entries)==1 and progress['new_current_ledger_candidates']==len(entries) and c['discovery'].startswith('partial') and c['repair']=='not started' and c['validation']=='not performed'
print('verified ten Git sources, separately attributed catalog bytes and bounded marker inventories')
