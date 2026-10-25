"""Persist manually compared native contract and publication plan only."""
import hashlib,json
from datetime import datetime,timezone
from pathlib import Path
C=Path(__file__).resolve().parents[1]
NOW=datetime.now(timezone.utc).isoformat()
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):return {'record':'continuation/'+p.relative_to(C).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def new(p,o):
    assert not p.exists(),str(p)
    p.write_text(json.dumps(o,indent=2)+'\n',encoding='utf-8')
regp=C/'camera/initial07-redundant-authority/source-range-registration.jsonl'
rs=[json.loads(l) for l in regp.read_text(encoding='utf-8').splitlines()]
r=next(x for x in rs if x.get('id')=='CAMERA-INITIAL07-REG040')
p=Path(r['resolved_path'])
assert hashlib.sha256(p.read_bytes()).hexdigest()==r['sha256']
assert len(p.read_text(encoding='utf-8').splitlines())==119
scope=read(C/'cumulative-source-scope-parallel-059.json')
assert not any(x['source_id']==r['source_id'] for x in scope['files'])
viewid='pr-working:crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass/view_culling.rs'
v=next(x for x in scope['files'] if x['source_id']==viewid)
vp=C.parents[1]/'774-snapshots/pr-working'/viewid.split(':',1)[1]
assert hashlib.sha256(vp.read_bytes()).hexdigest()==v['sha256']
new(C/'root/admission03/new-parallel-admission-203.json',{
    'id':'ROOT-ADM03-PARALLEL-203','at':NOW,
    'source_roles':[{'source_id':r['source_id'],'sha256':r['sha256'],'registration_id':r['id'],'actual_compared_ranges':[[1,119]],'source_evidence':'Fresh literal full119 after prospective CameraREG040, independently compared current Root03 admission/error/publication contracts; no Camera verdict transfer.'}],
    'existing_scope_support':[{'source_id':viewid,'sha256':v['sha256'],'actual_compared_ranges':[[1,167]],'source_evidence':'Fresh literal globally scoped full167 independently compared to avoid treating absence of reset in generic native node as proof of missing app reset. No other-lens credit.'}],
    'groups':{
        'native-node':'Optional pipeline/cache/allocator/cull resources and not-yet-compiled pipeline return without submission; empty pages and absent page bindgroups skip. Actual one generic compute pipeline binds global cull data then compute/common page bindings and dispatches each nonzero-capacity page. No reset operation in this generic node. This is insufficient to infer that application per-view culling omits reset.',
        'actual-app-route':'Existing PR view_culling31-39 schedules current-camera cull after Prepass/before MainPass; finish47-64 queues reset and main from same shader/layouts. prepare82-117 keeps active camera membership and uploads each current camera uniform. cull119-167 uses current ViewQuery, requires both compiled pipelines, and for each available nonempty page binds current camera, dispatches reset then select within the same compute pass. Actual application reset order is present, so no missing-reset defect is recorded.',
        'propagation':'Native counts, byte sizes, resources, bindgroups and workgroup coordinates are explicit receiving contracts. Canonical firstparty resource/population/layout and launch owners must remain coherent until real native dispatch/binding extraction. Thirdparty native port status is not a blanket exemption for firstparty arithmetic, retained raw state or schema. Float-based generic dispatch count and firstparty draws-size narrowing are not silently cleared by this admission proof; they remain independent invariant-operation/type/owner lens obligations with actual capacity producer bounds.',
        'limits':'Exactly one new native source full119; existing globally scoped firstparty view167 used as direct support only. No new native backend, collector, allocator, pipeline/plugin/test bodies or wholepackage. All initial lenses remain independent/incomplete, auditthenhandover163, no implementation/project execution.'
    },'status':'exact_registered_source_semantic_admission_compared','source_edits':0,'project_execution':0,'other_lens_credit':False})
plan=read(C/'root/scope-publication-plan-202.json')
plan.update(previous_scope='cumulative-source-scope-parallel-059.json',next_scope='cumulative-source-scope-parallel-060.json',reconciliation_record='root/parallel-scope-reconciliation-204.json',reconciliation_id='ROOT-PARALLEL-SCOPE-RECONCILIATION-204',admission_record='root/admission03/new-parallel-admission-203.json',expected_previous_overlays=243,overlay_backup='owner-range-extensions-through243-before060.jsonl',scope_increment_key='parallel_initial_scope_increment204',ledger_increment_key='parallel_scope204',queue_count_key='060_new_roles')
plan['registries']=[{'path':regp.relative_to(C).as_posix(),'ids':[r['id']],'supplier':'camera','effective_ranges':{r['id']:[[1,119]]},'qualifications':{r['id']:'Only pinned Eidolon cull/node.rs1-119 after prospective REG040 and fresh Root03proof203. Generic node absence of reset is not app reset defect: actual globally scoped view_culling167 orders reset/select per current camera. No broader native graph or project execution. All prior mandatory qualifications includingRoot200 retained.'}}]
plan['source_paths']={r['source_id']:str(p)}
plan['limits']='One new exact native node variant2828; overlays remain243. Prior mandatory qualifications includingRoot200 retained. All13 exhaustive initial audits then handover163.'
plan['queue_instructions']='After07 independently compare284 exact pending new-material roles for01/02; preserve historical040/042 and all mandatory qualifications includingRoot200. No semantic carry; all13 initial audits then handover163.'
plan['progress_updates']={'current03':'0602828+243 published roles accounted WITH_FINDINGS; actual native cull-node proof203 plus existing firstparty view support closes missing-reset inference, not arithmetic/type lenses. No other-lens verdict transfer.','historical01_02':'Historical040/042 retained; Camera after07 bounded284-role queue087.','new_scope_frontier':'Through Camera040/Rig036/Native036/Root020 captured060. All mandatory qualifications includingRoot200 retained. Frozen6533+recorded material; last observed remote6223 requires future reconciliation. Auditthenhandover163.'}
new(C/'root/scope-publication-plan-204.json',plan)
print(json.dumps({'admission':pin(C/'root/admission03/new-parallel-admission-203.json')}))
