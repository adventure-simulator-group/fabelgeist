import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021')
D=R/'continuation/root/camera-supplier-owner-fit'
N=R/'continuation/native/camera-root169-supplier-owner-fit'
Q=json.loads((R/'continuation/camera/owner-fit-delta/optional-root-supplier-queue-010.json').read_text())['sources']
root=json.loads((D/'coverage-root21.json').read_text())
native=[json.loads(l) for l in (N/'coverage.jsonl').read_text(encoding='utf-8').splitlines()]
progress=json.loads((N/'progress.json').read_text())
assert len(root)==21 and len(native)==169 and len(Q)==190
for fn,sha in progress['payload_sha256'].items():
 assert hashlib.sha256((N/fn).read_bytes()).hexdigest()==sha,fn
qi={x['index']:x for x in Q}
ri={x['index']:x for x in root}
ni={x['original_camera_index']:x for x in native}
assert len(ri)==21 and len(ni)==169 and not (set(ri)&set(ni))
assert set(ri)|set(ni)==set(qi)
def obskey(x):
 return (x.get('record'),x.get('receipt'),x.get('json_pointer',''))
rows=[]
obs_count=0
for i,q in qi.items():
 if i in ri:
  x=ri[i]
  witness=next(p for p in json.loads((D/'contract-witnesses-root21.json').read_text())['proofs'] if p['index']==i)
  observations=[v['observation'] for v in witness['producer_observation_dispositions']]
  ref='continuation/root/camera-supplier-owner-fit/contract-witnesses-root21.json#'+witness['id']
  comparison=witness['comparison']
 else:
  x=ni[i]
  assert not x['remaining_actual_unread_selected_roles']
  assert not x['new_source_or_range_requests']
  observations=[v['original_observation'] for v in x['per_current_producer_observation']]
  ref='continuation/native/camera-root169-supplier-owner-fit/coverage.jsonl#'+x['id']
  comparison=x['source_role_comparison']
 assert x['source_id']==q['source']['source_id'] and x['sha256']==q['source']['sha256']
 expected={obskey(v) for v in q['producer_role_observations']}
 actual={obskey(v) for v in observations}
 assert expected==actual,(i,expected-actual,actual-expected)
 obs_count+=len(expected)
 rows.append({'original_camera_index':i,'source_id':x['source_id'],'sha256':x['sha256'],'owner_comparison':comparison,'proof':ref,'exact_producer_observations':observations,'status':'accounted_with_findings','original_audit_and_repair_authority':'Camera738 unchanged; Root supplied only source evidence','later_lens_repair_validation_credit':'none'})
assert obs_count==366
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
def pin(p):return {'record':str(p.relative_to(R)).replace('\\','/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
out={'id':'CONT-ROOT-CAMERA190-OWNER-SUPPLIER-COMPLETION-015','at':now,'lens':'audit-existing-type-owners','status':'190 exact source roles/366 original producer pointers exhausted_with_findings; awaiting Camera authority reconciliation','counts':{'root_fresh_roles':16,'root_same_contract_carries':5,'native_semantic_supplier_roles':169,'total':190,'original_producer_pointers':366},
 'semantic_reconciliation':'Root consumed all14 Native comparison groups, all16 source-specific exception groups, all60 detailed owner comparisons and10 literal current observation pointers (compact reread after metadata output truncation), plus Root21 fresh/carry role proof. Each190 source retains actual current producer/retained/recipient comparison, original source/scope qualifications and findings; no file/hash/oldverdict-only clearance.',
 'mandatory_qualifications':['reference-errata-root21.json: CONT-ROOT-OWNER-BOUNDARIES-007 plural exact reference.',
 'schema-boundary-qualification-root21.json: first-party fraction table schema retains ConsumableFractionMicros to actual codec; raw native field is not blanket first-party record exemption.',
 'Actual current Rig dialogue supplier corrects historical catalogue revision blake3 narrative to SHA-256/domain v2; no source behavior change or representation-only digest reuse.',
 'P16 stale snapshot cannot erase PR AttachmentCapacity enum/P15 chronology/reservation/sequence APIs; preserve coherent whole-function union.',
 'Signed birth/pre-epoch, privacy-redacted0, measured0 and unavailable required clock differ; no fabricated success.',
 'Canonical case prefix160 vs broader investigationASCII256; imported package digest vs case-route binding digest; positive CoinAmount vs zero-valid balances; open authored profession/name/reference fields remain accepted.',
 'Correct actual dialogue facts are organization::StartingProfession10 and case::ContractStatus, not populationProfession5/payloadContractState.',
 'Root family helper material tail ends114; next declaration115–126 excluded. P16 container tail material ends1088. Foreign EOF and fixture scope limits remain exact.'],
 'source_dispositions':rows,'no_source_range_extension':True,'new_human_decisions':[],'source_edits':0,'validation':'none; no implementation/family/lens clearance',
 'pins':[pin(N/'progress.json'),pin(N/'compact-semantic-index.json'),pin(N/'semantic-owner-comparisons.json'),pin(N/'literal-producer-observation-witnesses.json'),pin(D/'coverage-root21.json'),pin(D/'contract-witnesses-root21.json'),pin(D/'reference-errata-root21.json'),pin(D/'schema-boundary-qualification-root21.json')]+[pin(N/fn) for fn in progress['payload_sha256'] if fn not in ['compact-semantic-index.json','semantic-owner-comparisons.json','literal-producer-observation-witnesses.json']]}
p=D/'completion-root190-015.json'
assert not p.exists()
p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'roles':190,'exact_producer_pointers':366,'completion_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'Camera_authority_reconciliation':'pending'}))
