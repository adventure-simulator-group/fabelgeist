"""Persist manually compared new roles and EOF qualification; metadata only."""
import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1];NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def new(p,o):
    assert not p.exists(),str(p)
    p.write_text(json.dumps(o,indent=2)+'\n',encoding='utf-8')
qp=C/'native/initial05-aggregate-states/source-qualifications.jsonl';qs=[json.loads(l) for l in qp.read_text(encoding='utf-8').splitlines()];q=next(r for r in qs if r.get('id')=='NATIVE-INITIAL05-QUAL058')
source=C.parents[1]/'774-snapshots/p16/crates/adventuresim-core/src/reputation.rs'
assert len(source.read_text(encoding='utf-8').splitlines())==331
assert hashlib.sha256(source.read_bytes()).hexdigest()==q['sha256']
qualification=C/'root/parallel-scope-qualification-188.json'
new(qualification,{'id':'ROOT-PARALLEL-SCOPE-QUALIFICATION-188','at':NOW,'source_id':q['source_id'],'source_sha256':q['sha256'],'actual_EOF':331,'effective_scope_limit':[1,331],'excluded_nonexistent':[332,338],'source_qualification':dict(pin(qp),id=q['id'],row_1based=qs.index(q)+1),'qualification':'Inherited owner310-338 ends at actual331. Nonexistent332-338 give no semantic coverage or material obligation in any lens. Native companion displays outside registered unions explicitly uncredited in QUAL058; REG035/036 registered before fresh bounded reads, not retrospective coverage. No whole companion file claim.'})
queuep=C/'bounded01-02-newmaterial-queue-087.json';queue=read(queuep);queue['mandatory_scope_qualifications'].append(pin(qualification));queuep.write_text(json.dumps(queue,indent=2)+'\n',encoding='utf-8')
regp=C/'native/initial05-aggregate-states/source-range-registration.jsonl';regs=[json.loads(l) for l in regp.read_text(encoding='utf-8').splitlines()]
selected=[next(r for r in regs if r.get('id',r.get('registration_id'))==ident) for ident in ['NATIVE-INITIAL05-REG035','NATIVE-INITIAL05-REG036']]
paths={};roles=[]
for r in selected:
    path=C.parents[1]/'774-snapshots/pr-working'/r['source_id'].split(':',1)[1]
    assert hashlib.sha256(path.read_bytes()).hexdigest()==r['sha256']
    assert len(path.read_text(encoding='utf-8').splitlines())==r['actual_EOF']
    paths[r['source_id']]=str(path)
    roles.append({'source_id':r['source_id'],'sha256':r['sha256'],'registration_id':r['id'],'actual_compared_ranges':r['material_ranges'],'source_evidence':'Fresh literal registered terminal role after REG035/036; independently read existing function prefix as context. No whole file/native runtime or other-lens credit.'})
new(C/'root/admission03/new-parallel-admission-189.json',{'id':'ROOT-ADM03-PARALLEL-189','at':NOW,'source_roles':roles,'groups':{'pain-terminal':'Actual PR pain327-334 maxes total damage/will at0, returns0 for damage0, otherwise damage/(damage+0.5*will)*exp(-0.2*will). Newly registered331-334 complete function rather than inventing all remaining527-line morale scope. No checked finite result admission inferred from max/return type; canonical damage versus total incapacitation meaning and finite computed sums remain connected earlier physical/admission repair obligations.','local-offense-terminal':'Actual PR unsettled_local_offenses172-188 queries exact character index, filters matching settlement AND not settled, collects then sorts by occurred minute and ID before return. Sorting yields deterministic tie ordering; not stronger uniqueness or immutable DB snapshot proof. Registered180-188 completes exact function; no arrest-charge190+ or whole440-file clearance. Preserve exact offense IDs/strategic minutes/settlement identity and native index/filter ports, reuse existing owners across internal APIs.','qualification':'Mandatory188 excludes nonexistent P16Core reputation332-338. Earlier full companion displays have no outside-union coverage. Prospective new terminal roles now freshly read, not retrocredited.'},'mandatory_qualification188':pin(qualification),'status':'exact_registered_source_semantic_admission_compared','source_edits':0,'project_execution':0,'other_lens_credit':False})
plan=read(C/'root/scope-publication-plan-186.json')
plan.update(previous_scope='cumulative-source-scope-parallel-054.json',next_scope='cumulative-source-scope-parallel-055.json',reconciliation_record='root/parallel-scope-reconciliation-190.json',reconciliation_id='ROOT-PARALLEL-SCOPE-RECONCILIATION-190',admission_record='root/admission03/new-parallel-admission-189.json',expected_previous_overlays=238,overlay_backup='owner-range-extensions-through238-before055.jsonl',scope_increment_key='parallel_initial_scope_increment190',ledger_increment_key='parallel_scope190',queue_count_key='055_new_roles')
quals={r['id']:'Only prospective registered terminal role completes already scoped function. Fresh Root03 proof189, mandatory188 EOF331 and prior qualifications retained. No whole companion source/runtime/other-lens credit; preceding out-of-union display explicitly uncredited.' for r in selected}
plan['registries']=[{'path':regp.relative_to(C).as_posix(),'ids':[r['id'] for r in selected],'supplier':'native','effective_ranges':{r['id']:r['material_ranges'] for r in selected},'qualifications':quals}]
plan['source_paths']=paths
plan['limits']='Variants unchanged2825; two mandatory overlays239-240 complete PR pain331-334 and local-offense180-188 tails. Mandatory188 excludes nonexistent P16reputation332-338. All prior qualifications retained; exhaustive all13 initial audits then handover163.'
plan['queue_instructions']='After07 independently compare278 exact pending new-material roles for01/02; preserve historical040/042 and all mandatory qualifications including188 reputationEOF331. No semantic carry; all13 initial audits then handover163.'
plan['progress_updates']={'current03':'0552825+240 published roles accounted WITH_FINDINGS; actual new terminal role proof189 and EOF qualification188. No other-lens verdict transfer.','historical01_02':'Historical040/042 retained; Camera after07 bounded278-role queue087.','new_scope_frontier':'Through Camera038/Rig035/Native036/Root017 captured055. Mandatory188 P16Core reputationEOF331, no332-338 credit. Frozen6533+recorded material; remote6223 future reconciliation. Auditthenhandover163.'}
new(C/'root/scope-publication-plan-190.json',plan)
print(json.dumps({'qualification':pin(qualification),'admission':pin(C/'root/admission03/new-parallel-admission-189.json')}))
