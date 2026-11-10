"""Verify source-bound loot pairing and qualified historical attribution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; rev='95e63628f9d70b8dd6e6fede64334696c6995009'; name='ASSESSMENT-L01-LOOT-PAIR-95E.json'; doc=json.loads((root/name).read_text()); base=json.loads((root/'BASELINE-95E-20261110.json').read_text()); progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text()); assert doc['source_revision']==rev and doc['lens']==1

def tree(path):
 d={}
 for line in (root/path).read_text().splitlines():
  header,p=line.split('\t',1); _,_,oid=header.split(); assert p not in d; d[p]=oid
 return d
cur=tree('main-advance-95e/current-tree.tsv'); prev=tree('main-advance-95e/previous-tree.tsv'); source={}
for s in doc['current_sources']:
 p=(root/s['preserved_bytes']).resolve(); assert p.is_relative_to(root) and s['content_encoding']=='gzip' and s['revision']==rev; raw=gzip.decompress(p.read_bytes()); oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(); assert len(raw)==s['bytes'] and len(raw.splitlines())==s['lines'] and hashlib.sha256(raw).hexdigest()==s['sha256'] and oid==s['git_blob']==cur[s['path']]==prev[s['path']]; assert s['path'] not in source and all(1<=a<=b<=s['lines'] for a,b in s['ranges']); source[s['path']]=raw.decode()
assert len(source)==4
route=source['crates/strategic-web/src/routes/quests.rs']; template=source['crates/strategic-web/src/templates/quest.rs']; js=source['crates/strategic-web/static/party-trade.js']; native=source['crates/adventuresim-stdb-module/src/strategic/inventory_trade.rs']
for marker in ('struct StoreLootForm {', '#[serde(default)]\n    item_ids: String', '#[serde(default)]\n    quantities: String', 'session.character_id_u64()', '.filter_map(|v| v.parse::<u64>().ok())', '.filter_map(|v| v.parse::<u32>().ok())', '"store_battle_loot"'): assert marker in route
for marker in ('data-count=(entry.quantity)', 'data-loot-stage=(entry.id)', 'id="loot-transfer-offer"', 'button type="submit" disabled'): assert marker in template
for marker in ('const draft = strategicTradeUi.state.lootTransferDraft ||= new Map()', 'draft.set(id, staged + amount)', 'item_ids: [...draft.keys()], quantities: [...draft.values()]', 'values.join(",")'): assert marker in js
start=native.index('pub fn store_battle_loot('); end=native.index('\n#[reducer]',start); body=native[start:end]
for marker in ('require_strategic_character_authority(ctx, character_id)?', 'loot_item_ids.len() != quantities.len()', 'Duplicate battle loot IDs are not allowed', 'if result.party_id != party_id', 'let loot: Vec<_> = if loot_item_ids.is_empty() {\n        available', '.zip(&quantities)', '*quantity == 0 || *quantity > entry.quantity', 'if participants.is_empty()', 'add_to_party_inventory(ctx, &party_id, &entry.item_id, entry.quantity)', 'battle_loot_item().id().delete(entry.id)', 'credit_party_reserve(ctx, &party_id'): assert marker in body
assert 'let _ = add_to_party_inventory_checked' in native
h=doc['historical_source']; ledger=json.loads((root/h['ledger']).read_text()); old=ledger['discovery_findings_and_dependencies']['ADLER-L11-LOOT-PAIR-ADMISSION']; assert h['checkpoint']=='059' and old['accepted_unique_count_change']==0 and old['current_reconciled'] is False
for key in h['contract_ids'][1:]: assert key in json.dumps(ledger)
roles=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; reg=json.loads((root/'recovered/original-registry-source-archive/MANIFEST.json').read_text()); hashes={x['original']:x['sha256'] for x in reg['sources']}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as tar:
 for x in h['historical_source_roles']:
  r=roles[x['index']]; assert r['source_id']==x['source_id'] and r['historical_sha256']==x['sha256']==hashes[x['index']] and hashlib.sha256(tar.extractfile(x['sha256']).read()).hexdigest()==x['sha256']; p=x['source_id'].split(':',1)[1]; sha=hashlib.sha256(source[p].encode()).hexdigest(); assert x['current_sha256']==sha and x['relation']==('byte-match' if x['sha256']==sha else 'different')
entries=base['newly_assessed_ledger']['entries']; c=doc['candidates'][0]; found=[x for x in entries if x['id']==c['id']]; assert progress['new_current_ledger_candidates']==len(entries) and len(found)==1 and found[0]['record']==name and c['repair']=='not started' and c['validation']=='not performed' and c['discovery'].startswith('partial')
print('verified four Git sources, native selection branch and qualified 059 attribution')
