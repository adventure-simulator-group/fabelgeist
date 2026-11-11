"""Verify published source bytes, selector text and historical read receipts only."""
import gzip,hashlib,json,re,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
rev='95e63628f9d70b8dd6e6fede64334696c6995009';name='ASSESSMENT-TRAVEL-ADMISSION-SOURCE-ASSERTIONS-95E.json';doc=load(name)
ids=['REVIEW-TRAVEL-ITINERARY-ACTOR-ADMISSION-001','REVIEW-CAMP-FATIGUE-ACTOR-ADMISSION-001','REVIEW-APPROVED-TRAVEL-ID-ADMISSION-001','SEGOVB-L12-TRAVEL-ORIGIN-TEXT-DISPATCH','SEGOVB-L12-TRAVEL-CASE-SITE-PRIVATE-RETENTION','Q-L12-90869165a5e467c9e544','Q-L12-d560b788dea590d225d7']
assert doc['source_revision']==rev and doc['lenses']==[3,12] and [r['id'] for r in doc['candidates']]==ids
tree={}
for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines():
    meta,path=line.split('\t',1);tree[path]=meta.split()[2]
contents={}
for row in doc['current_sources']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes())
    assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']==tree[row['path']]
    assert len(raw)==row['bytes'] and len(raw.splitlines())==row['lines'] and row['revision']==rev
    assert all(1<=a<=b<=row['lines'] for a,b in row['ranges']);contents[row['path']]=raw
base='crates/adventuresim-stdb-module/src/strategic/'
mod=contents[base+'mod.rs'].decode();section=mod.split('pub(crate) const STRATEGIC_SOURCE:',1)[1].split(');',1)[0]
includes=re.findall(r'include_str!\("([^\"]+)"\)',section);meta=doc['source_selector_evidence']
assert includes==meta['strategic_source_include_order']
joined=b''.join(contents[base+x] for x in includes);marker=b'pub(crate) fn ensure_bound_mission_authority'
assert joined.count(marker)==1
selected=joined.split(marker,1)[1].split(b'#[reducer]',1)[0]
assert sha(selected)==meta['mission_selector_sha256'] and len(selected)==meta['mission_selector_bytes']
assert b'outcome_entropy: ctx.random()' not in selected
assert b'MissionAuthority::capture(' in selected and b'mission_approach_capability_id(' in selected
inventory=contents[base+'inventory_trade.rs'];selected=inventory.rsplit(b'fn finalize_storefront_trade_impl',1)[1].split(b'#[reducer]\npub fn leave_party',1)[0]
assert sha(selected)==meta['merchant_selector_sha256'] and len(selected)==meta['merchant_selector_bytes']
assert b'let problem_minute = StrategicMinute::new(' not in selected and b'let problem_minute = ctx' in selected
assert b'outcome_entropy: ctx.random()' in contents[base+'mission_roster.rs']
tests=contents[base+'tests/authority_trade_dialogue.rs']
assert b'assert!(binding.contains("outcome_entropy: ctx.random()"))' in tests
assert b'assert!(trade.contains("let problem_minute = StrategicMinute::new("))' in tests
travel=contents[base+'travel_reducers.rs']
def fn(name):return travel.split(b'\npub fn '+name.encode()+b'(',1)[1].split(b'\n}',1)[0]
for method in ('set_party_camp_fatigue_percent','set_party_travel_itinerary'):
    body=fn(method)
    assert b'require_living_character' in body and b'party.leader_id != character_id' in body
    assert b'require_strategic_character_authority' not in body and b'require_strategic_gateway' not in body and b'ctx.sender()' not in body
    assert method.encode() in contents[f'crates/adventuresim-stdb-client/src/{method}_reducer.rs']
assert b'(10..=100).contains(&fatigue_percent)' in fn('set_party_camp_fatigue_percent')
assert b'refresh_party_journey_forecast(ctx, &party_id)' in fn('set_party_travel_itinerary')
assert b'session.character_id_u64()' in contents['crates/strategic-web/src/routes/settlements/camp.rs']
assert b'case_site_id.into_string()' in fn('travel_to_case_site')
assert b'case_site_id.into_string()' in fn('travel_to_case_site_planned')
private=travel.split(b'\nfn travel_to_case_site_impl(',1)[1].split(b'\n}',1)[0]
assert b'case_site_id: String' in private and b'exact_case_site_for_observer' in private and b'CaseSiteId::from(case_site_id)' in private
settlement=travel.split(b'\nfn travel_to_settlement_impl(',1)[1].split(b'\n}',1)[0]
for x in (b'"settlement"',b'"case_site"',b'match origin_kind',b'JourneyEndpoint::CaseSite',b'CaseSiteId::try_new'):
    assert x in settlement
gov=contents[base+'governance.rs'];request=gov.split(b'pub fn request_party_action(',1)[1].split(b'\n}',1)[0]
assert b'payload,' in request and b'allowed.contains(&action_kind.as_str())' in request
approve=gov.split(b'pub fn approve_party_action_request(',1)[1].split(b'\n}',1)[0]
assert b'require_strategic_gateway(ctx)?' in approve and b'serde_json::from_str(&request.payload)' in approve
assert b'action.kind() != request.action_kind' in approve and b'action.execute(ctx' in approve
planned=gov.split(b'pub fn approve_party_action_request_planned(',1)[1].split(b'\n}',1)[0]
assert b'travel_to_case_site_impl(ctx, leader_id, case_site_id, Some(route))' in planned
approved=contents[base+'approved_action.rs']
assert b'case_site_id: String' in approved and b'CaseSiteId::from(case_site_id)' in approved
place=contents['crates/adventuresim-core/src/strategic_place.rs']
assert b'value.is_empty()' in place and b'Self::try_new(value).expect(' in place
transport={r['path']:r for r in json.loads(gzip.decompress((root/'recovered/original-transport-manifest.json.gz').read_bytes()))['entries']};by={}
assert len(doc['recovered_historical_records'])==15
for row in doc['recovered_historical_records']:
    raw=gzip.decompress((root/row['preserved_bytes']).read_bytes());m=transport[row['original_path']]
    assert (row['sha256'],row['git_blob'],row['bytes'])==(m['sha256'],m['git_blob'],m['bytes'])
    assert sha(raw)==row['published_sha256'] and len(raw)==row['published_bytes']
    if row['redacted_local_path_occurrences']:assert b'<LOCAL_PATH_' in raw and 'pinned 059 transport' in row['original_recovery']
    else:assert sha(raw)==row['sha256'] and blob(raw)==row['git_blob']
    by[row['original_path']]=(row,json.loads(raw))
def original_sha(p):return by[p][0]['sha256']
for number,index in ((303,1235),(304,1236),(305,1237)):
    key=f'segovb/READ-L12-{number}.json';r=by[key][1]
    assert r['lens']==12 and [x['original_index'] for x in r['rows']]==[index]
    assert r['accepted_unique_count_change']==0 and r['current_source_reconciled'] is False and r['implementation_gate_complete'] is False
    assert original_sha(r['plan'])==r['plan_sha256']
    for x in r['rows']:
        assert original_sha(x['registration'])==x['registration_sha256'] and x['other_lens_verdict_transfer'] is False
        assert x['fresh_personal_body_reads']==x['assessed_ranges']
    v=by[f'segovb/VERIFY-L12-{number}.json'][1]
    assert v['receipt_sha256']==original_sha(key) and v['state_sha256']==original_sha(f'segovb/STATE-L12-{number}.json')
histname='ASSESSMENT-L01-HISTORICAL-1235-1237.json';history=load(histname)['historical_source_roles'];roles=load('COVERAGE-L01-ROLES.json')['roles']
assert [r['original_index'] for r in history]==[1235,1236,1237]
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for row in history:
        raw=archive.extractfile(row['sha256']).read();assert sha(raw)==row['sha256'] and len(raw.splitlines())==row['lines']
        assert row['new_personal_read_ranges']==[[1,row['lines']]] and row['current_comparison']['byte_identical'] is True
        assert blob(raw)==tree[row['source_id'].split(':',1)[1]]
        role=roles[row['original_index']];assert role['historical_assessment']==histname and role['status']=='full-historical-body-trace-open'
q=load('RECONCILIATION-059.json')['entries']
for i in (243,244,998,999,1003):assert q[i]['current_reconciliation'].startswith('partial ')
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for identity in ids:
    matches=[x for x in ledger if x['id']==identity];assert len(matches)==1 and matches[0]['record']==name
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert doc['repair']=='not started' and doc['validation']=='not performed'
print('verified travel source evidence and static selector bytes; no product ran')
