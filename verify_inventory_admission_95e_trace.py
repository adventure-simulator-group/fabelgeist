"""Verify archived source and prior read receipts; never execute the product."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
revision='95e63628f9d70b8dd6e6fede64334696c6995009'
name='ASSESSMENT-INVENTORY-ACTOR-ADMISSION-95E.json';doc=load(name)
ids=['REVIEW-INVENTORY-DISCARD-ACTOR-ADMISSION-001','REVIEW-PARTY-TRANSFER-ACTOR-ADMISSION-001',
 'REVIEW-PARTY-OFFER-ACTOR-ADMISSION-001','REVIEW-PARTY-DEPOSIT-ACTOR-ADMISSION-001',
 'REVIEW-PARTY-WITHDRAW-ACTOR-ADMISSION-001','REVIEW-PARTY-LIQUIDATE-ACTOR-ADMISSION-001',
 'REVIEW-PARTY-REMOVAL-ACTOR-ADMISSION-001','REVIEW-PARTY-DISBAND-ACTOR-ADMISSION-001',
 'REVIEW-STOREFRONT-TRADE-ACTOR-ADMISSION-001','SEGOVB-L12-PARTY-GRANT-ERROR-ERASURE',
 'SEGOVB-L12-STOREFRONT-ROUTE-PRIVATE-READMISSION','SEGOVB-L12-COND-TEMP-COMPANION-STAKE-SETTLEMENT']
assert doc['source_revision']==revision and doc['lenses']==[3,12]
assert [r['id'] for r in doc['candidates']]==ids

tree={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta,path=line.split('\t',1);tree[path]=meta.split()[2]
contents={}
for row in doc['current_sources']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes())
    assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']==tree[row['path']]
    assert len(raw)==row['bytes'] and len(raw.splitlines())==row['lines']
    assert row['revision']==revision and all(1<=a<=b<=row['lines'] for a,b in row['ranges'])
    contents[row['path']]=raw
trade=contents['crates/adventuresim-stdb-module/src/strategic/inventory_trade.rs']
def section(marker):
    return trade.split(b'\n'+marker,1)[1].split(b'\n}',1)[0]
names=('discard_inventory_items','transfer_party_item','finalize_party_offer','deposit_party_inventory_item',
 'withdraw_party_inventory_item','liquidate_party_inventory','leave_party','remove_party_member',
 'disband_party','finalize_storefront_trade','finalize_merchant_trade')
for fn in names:
    body=section(b'pub fn '+fn.encode()+b'(')
    for guard in (b'ctx.sender()',b'require_strategic_gateway',b'require_strategic_character_authority'):
        assert guard not in body,(fn,guard)
    client=contents[f'crates/adventuresim-stdb-client/src/{fn}_reducer.rs']
    assert fn.encode() in client and b'RemoteReducers' in client
impl=section(b'fn finalize_storefront_trade_impl(')
assert b'require_strategic_character_authority' not in impl and b'ctx.sender()' not in impl
for part in (b'require_living_character(ctx, character_id)',b'enforce_temporal_scope(',
             b'current_settlement_id.as_deref() != Some(&settlement_id)',b'npc_is_present(',
             b'storefront_stocks(',b'physical_object_row_is_atomic(',b'Not enough coin'):
    assert part in impl, part
purchase=section(b'pub fn purchase_personal_storefront_with_party_stake(')
assert b'require_strategic_character_authority(ctx, character_id)?' in purchase
discard=section(b'pub fn discard_inventory_items(')
for part in (b'require_character_no_unresolved_encounter',b'require_living_character',
             b'!seen.insert(inventory_item_id)',b'item.character_id != character_id',
             b'inventory_item_is_equipped',b'physical_object_row_is_atomic',
             b'delete_subtree(ctx, object.id)',b'inventory_item().id().delete(item.id)'):
    assert part in discard
transfer=section(b'pub fn transfer_party_item(')
for part in (b'require_living_character(ctx, from_character_id)',b'require_living_character(ctx, to_character_id)',
             b'from.party_id != to.party_id',b'source_item.character_id != from_character_id',
             b'rehome_subtree',b'split_lot'):
    assert part in transfer
offer=section(b'pub fn finalize_party_offer(')
for part in (b'parse_party_offer(',b'from.party_id != to.party_id',b'item.character_id != from_id',b'transfer_party_item('):assert part in offer
deposit=section(b'pub fn deposit_party_inventory_item(')
for part in (b'inventory.character_id != character_id',b'rehome_subtree',b'credit_party_stake',b'add_to_party_inventory('):assert part in deposit
withdraw=section(b'pub fn withdraw_party_inventory_item(')
for part in (b'inventory.party_id != party_id',b'transfer_personal_currency_to_party',b'party_stake()',b'rehome_subtree'):assert part in withdraw
liquidate=section(b'pub fn liquidate_party_inventory(')
for part in (b'current_settlement_id.as_deref() != Some(&settlement_id)',b'SettlementService::Market',
             b'SettlementService::GeneralStore',b'!seen.insert(id)',b'entry.party_id != party_id',
             b'delete_subtree',b'credit_party_currency'):assert part in liquidate
leave=section(b'pub fn leave_party(');remove=section(b'pub fn remove_party_member(')
assert b'remove_party_member(ctx, character_id, character_id)' in leave
for part in (b'actor.party_id.as_deref() != Some(&party_id)',b'party.leader_id != actor_character_id',
             b'require_members_clear_current_camp_fireplace',b'settle_temporary_member_stake',
             b'stake.character_id == member_character_id && stake.value > 0',b'normalize_and_elect_party_leader'):
    assert part in remove
disband=section(b'pub fn disband_party(')
for part in (b'party.leader_id != leader_id',b'party.current_case_site_id.is_some()',
             b'stake.value > 0',b'reserve_value',b'create_solo_party_for_character'):
    assert part in disband
wrapper=section(b'pub(crate) fn add_to_party_inventory(')
assert b'let _ = add_to_party_inventory_checked' in wrapper
checked=section(b'pub(crate) fn add_to_party_inventory_checked(')
assert b'quantity == 0' in checked and b'insert_party_object(ctx, &row)?' in checked
assert b'initialize_party_weapon(ctx, &row)?' in checked
preflight=section(b'fn validate_personal_storefront_purchase(')
assert b'MerchantStorefrontRoute::try_from(service_id)' in preflight
assert b'Ok((' in preflight and b'party_id' in preflight
assert b'service_id: String' in trade.split(b'struct StorefrontTradeExecution',1)[1].split(b'\n}',1)[0]
assert b'MerchantStorefrontRoute::try_from(' in impl
settle=section(b'fn settle_temporary_member_stake(')
assert b'transfer_party_currency_to_personal(ctx, party_id, member_character_id, stake_value)' in settle
assert b'party_stake().id().update' not in settle and b'party_stake().id().delete' not in settle
currency=section(b'fn transfer_party_currency_to_personal(')
assert b'party_inventory_item()' in currency and b'add_inventory_item(' in currency
assert b'party_stake()' not in currency
camp=contents['crates/adventuresim-stdb-module/src/strategic/journey_camp.rs']
encounter=camp.split(b'pub(crate) fn require_character_no_unresolved_encounter(',1)[1].split(b'\n}',1)[0]
assert b'require_no_unresolved_encounter(ctx, &party_id)' in encounter and b'ctx.sender()' not in encounter
living=contents['crates/adventuresim-stdb-module/src/character.rs'].split(b'pub fn require_living_character(',1)[1].split(b'\n}',1)[0]
assert b'if !character.alive' in living and b'ctx.sender()' not in living
assert b'run.owner == ctx.sender()' in contents['crates/adventuresim-stdb-module/src/simulation.rs']
auth=contents['crates/adventuresim-stdb-module/src/strategic/authority_model.rs']
assert b'require_strategic_gateway(ctx).is_ok()' in auth and b'sender_owns_simulation_character(ctx, character_id)' in auth
assert b'accessor = inventory_item, public' in contents['crates/adventuresim-stdb-module/src/item.rs']
web=contents['crates/strategic-web/src/routes/settlements/party/transfers.rs']
assert b'session.character_id_u64() == Some(character_id)' in web
assert b'form.from_character_id != active_character.id && recipient_id != active_character.id' in web
assert b'*id == active.id || *id == character_id' in web and b'get_active_character(&state, session.character_id_u64())' in web
webpool=contents['crates/strategic-web/src/routes/settlements/party/inventory_medical.rs']
for word in (b'"deposit_party_inventory_item"',b'"withdraw_party_inventory_item"',b'"liquidate_party_inventory"',b'session.character_id_u64()'):
    assert word in webpool
webparty=contents['crates/strategic-web/src/routes/parties.rs']
assert b'"leave_party", &[json!(character_id)]' in webparty and b'PartyAction::DisbandParty' in webparty
core=contents['crates/adventuresim-core/src/strategic_inventory.rs']
assert b'pub struct MerchantStorefrontRoute' in core and b'pub fn parse_party_offer(' in core
assert b'to.len() != len || inventory.len() != len || quantities.len() != len' in core
rel=contents['crates/adventuresim-stdb-module/src/relationship/model.rs']
assert b'let actor_minute = canonical_now(ctx, actor_id)?' in rel
assert b'ctx.sender()' not in rel.split(b'pub fn enforce_temporal_scope(',1)[1].split(b'\n}',1)[0]
chron=contents['crates/adventuresim-stdb-module/src/relationship/chronology.rs']
assert b'character_time()' in chron.split(b'pub fn canonical_now(',1)[1].split(b'\n}',1)[0]

transport={r['path']:r for r in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']}
records=doc['recovered_historical_records'];assert len(records)==17;by={}
for row in records:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes());m=transport[row['original_path']]
    assert (row['sha256'],row['git_blob'],row['bytes'])==(m['sha256'],m['git_blob'],m['bytes'])
    assert sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if row['redacted_local_path_occurrences']:
        assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    else:assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    by[row['original_path']]=(row,json.loads(raw))
def original_sha(path):return by[path][0]['sha256']
for number,index in ((299,1225),(300,1226)):
    key=f'segovb/READ-L12-{number}.json';r=by[key][1]
    assert r['lens']==12 and [x['original_index'] for x in r['rows']]==[index]
    assert r['accepted_unique_count_change']==0 and r['current_source_reconciled'] is False
    assert r['implementation_gate_complete'] is False and original_sha(r['plan'])==r['plan_sha256']
    for x in r['rows']:
        assert original_sha(x['registration'])==x['registration_sha256']
        assert x['fresh_personal_body_reads']==x['assessed_ranges'] and x['other_lens_verdict_transfer'] is False
    check=by[f'segovb/VERIFY-L12-{number}.json'][1]
    assert check['receipt_sha256']==original_sha(key)
    assert check['state_sha256']==original_sha(f'segovb/STATE-L12-{number}.json')
r=by['segovb/READ-L12-299.json'][1]
assert len(r['partial_sequence'])==5
for idx,(ref,expected) in enumerate(zip(r['partial_sequence'],[[1,600],[601,1200],[1201,1800],[1801,2400],[2401,3250]])):
    assert original_sha(ref['path'])==ref['sha256'];part=by[ref['path']][1]
    assert part['original_index']==1225 and part['fresh_personal_body_reads']==[expected]
    assert part['other_lens_coverage_transferred'] is False and part['current_source_reconciled'] is False
    assert part['unread_tail']==([[expected[1]+1,3250]] if idx<4 else [])
    if idx:assert part['predecessor']['sha256']==r['partial_sequence'][idx-1]['sha256']
for support in r['supporting_reads']:
    assert original_sha(support['registration'])==support['registration_sha256']
    assert support['new_original_population_credit']==0

histname='ASSESSMENT-L01-HISTORICAL-1225-1226.json';history=load(histname)['historical_source_roles']
assert [r['original_index'] for r in history]==[1225,1226];roles=load('COVERAGE-L01-ROLES.json')['roles']
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read()
        assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']];assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
        assert role.get('current_observations',[])==row['prior_current_observations']
q=load('RECONCILIATION-059.json')['entries']
for i in (16,238,239,240,990):assert q[i]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches=[x for x in ledger if x['id']==identity];assert len(matches)==1 and matches[0]['record']==name
p=load('CONTINUATION-PROGRESS-95E.json')
assert name in p['post_transition_bounded_admission_traces'] and name in p['post_transition_bounded_string_traces']
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified inventory actor traces and earlier partial-read qualifications; no product ran')
