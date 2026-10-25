"""Ignored stdlib receipt bookkeeping; no project commands or source mutations."""
import hashlib
import json
from pathlib import Path

B=Path(__file__).resolve().parents[2]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def load(rel): return json.loads((B/rel).read_text(encoding='utf-8-sig'))
delta_rel='continuation/rig/expanded-producer-delta-map-049.json'
delta=load(delta_rel)
trace_rel='records/final-acceptance-20261021/wave-01/producer-supplement-after-owner/camera/traces.jsonl'
trace=next(json.loads(s) for s in (B/trace_rel).read_text(encoding='utf-8').splitlines() if json.loads(s).get('trace_id')=='CAM-PPS-CAMERA-BOUNDARIES-048')
eof_rel='continuation/camera/reached-role-reconciliation/eof-role-qualification-012.json'
eof=load(eof_rel)
assigned=[(i,x) for i,x in enumerate(delta['delta_observation_map']) if x['parent_trace']=='CAM-PPS-CAMERA-BOUNDARIES-048']
assert len(assigned)==15
identity={(x['path'],x['sha256']):x for x in delta['delta_source_identity_map']}

# Key: original /additional_actual_source_proofs index. Contract witnesses are
# zero-based /contracts indices from the exact parent trace, not search hits.
roles={
0:{'contracts':[2,4], 'fresh':True,
   'role':'draw_camera_rig279-338 takes checked camera/aim debug observations and extracts scene coordinates, normal and native transform only as immediate Gizmos line/cube arguments. The normal endpoint and soft-occluder tick are complete diagnostic arithmetic, not retained application records. Disabled or absent rig returns without drawing; absent optional hit/aim omits only its mark.',
   'receiver':'Gizmos.line291-319/323-336 and Gizmos.cube321 are actual native diagnostic ports; no downstream application re-admission.',
   'policy':'Keep optional marks, colors, blocked-hit predicate and original0.6/0.25 geometry. Real diagnostic output does not waive earlier camera sweep/capture guarantees.'},
1:{'contracts':[2,4], 'fresh':True,
   'role':'update_camera_ui562-601 retains optional CameraAimObservation through reticle/blocked predicates. Checked blend, distance, speed, velocity, screen error/window are projected directly into the final debug TextSpan format; length of velocity is an immediate displayed magnitude.',
   'receiver':'Actual Visibility assignments570-579 and formatted TextSpan582-599; no semantic parsing or retained raw metric record after formatting.',
   'policy':'No aim hides reticle and blocked marker. Debug disabled/absent uses explicit OFF text; preserve Option hit presence independently of measured numeric zero.'},
2:{'contracts':[7], 'fresh':True,
   'role':'Full position_capture_camera5-74 reads native subject/bone Transform observations. Gameplay retains checked animation offset to direct Transform translation35. Side/front complete framing math, optional armor review transform, non-gameplay orbit fallback and final label formatting reach actual camera/label ports.',
   'receiver':'Native Transform35-55 and Text/Visibility58-71. Head and subject/orbit raw values originate native ECS observations, not an already checked source being peeled.',
   'policy':'Keep missing subject/camera early return, checked offset error diagnostic/return, Gameplay no-override None versus missing armor head observation distinctions, and non-gameplay fallback. Existing source strengthening/accepted input requirements stay with whole review owner.'},
3:{'contracts':[9], 'fresh':True,
   'role':'SceneCameraView7-22 retains Position<Scene> target/position and sum-typed looking-at up versus fixed RigidRotation. looking_at25-34 validates changed forward and cross-product directions while retaining the original typed fields; oriented36-42 retains checked rotation.',
   'receiver':'native_transform52-60 is the actual Bevy Transform translation/look-at/rotation adapter. native_up45-49 supplies its native direction, including complete fixed-rotation Y-axis math.',
   'policy':'Invalid/degenerate looking-at produces CaptureCameraError::InvalidView; fixed orientation remains a distinct variant. Preserve original framework look-at arithmetic and exact input policy, without introducing duplicate point/rotation owners.'},
4:{'contracts':[9], 'fresh':True,
   'role':'Full landscape helpers: vista_valley5-30 and vista_peak47-72 use admitted vista extrema, scene focus and typed height/half extent in complete direction/framing/max/clamp equations; horizon32-45 uses optional peak or admitted eye target; terrain_grazing74-85 derives close detail view; city_oblique87-101 uses typed angle and extent in trig framing.',
   'receiver':'Changed Position<Scene> admitted immediately at each SceneCameraView::looking_at call29/44/71/80-84/94-100; actual retained SceneCameraView and its Transform adapter are traced in placement.rs.',
   'policy':'Missing vista is explicit error for required valley/peak; horizon intentionally permits no vista. Preserve native normalization fallback direction, terrain/vista safe-height comparisons, clamp and max order, signed finite input ranges and ground packet. Broader invalid-derived policy remains later work.'},
5:{'contracts':[5,9], 'fresh':True,
   'role':'coordinates1-105 defines framed finite Position, Displacement and Elevation; Position::from_metres67-73 checks finite components, translated78-80 and displacement_to81-83 admit changed arithmetic outputs, Deserialize85-89 calls checked admission. Displacement::ZERO and constructor96-102 preserve zero as legitimate.',
   'receiver':'finite(value,role) actual helper8-18 and canonical error owner; Position::metres75-77/Displacement::metres103-105 are explicit projections requiring caller-specific evidence supplied by camera/diagnostic/Transform paths.',
   'policy':'The bounded1-105 scope ends inside the Displacement impl (source EOF140). Do not infer later methods/elevation admission/fixtures. Finite points may subtract or normalize to invalid derived values; existing CAM-PPS-F005 remains, rather than treating finite admission as all-operation validity.'},
6:{'contracts':[5], 'fresh':True,
   'role':'Full native geometry adapter: finite8-18 checks each XYZ component and preserves GeometryRole/CoordinateAxis in canonical NonFinite error. direction_error21-28 maps exact native Zero/Infinite/NaN variants into canonical DirectionError without prose conversion.',
   'receiver':'Canonical fabelgeist_geometry::GeometryError::{NonFinite,Direction} retains the specific source/role. Camera collision actual invalid-direction policy is separately traced; this adapter is not evidence that every caller preserves it.',
   'policy':'Preserve distinction among zero, infinite and NaN. CAM-PPS-F005 current sweep treats all native Dir3 failure as clear; source error adapter does not close that finding.'},
7:{'contracts':[3,5], 'fresh':False,
   'role':'Exact native Bevy direction15-47/550-611 witness differentiates InvalidDirectionError Zero/Infinite/NaN and native Dir3 normalization admission. Camera048 directly names those variants and points to actual CameraVolume.sweep159-166 where all failures become measured clear.',
   'receiver':'Actual collision.rs152-191/sweep and canonical direction_error adapter; EOF012 EOF-ROLE-02 freshly supplies immediate typed CameraSweep/CameraHit receivers. Actual finite point source allows changed subtraction/length overflow; no runtime probe claimed.',
   'policy':'Legitimate zero-length clear policy and invalid infinite/NaN observation must remain distinct through whole camera/capture policy. Retain CAM-PPS-F005; no whole1511-line foreign module clearance.'},
8:{'contracts':[3], 'fresh':False,
   'role':'Exact workspace Cargo104-116 selects the actual Avian0.7 f32/3d/Parry backend used by the camera cast graph. This is dependency/feature context for primitive native Scalar/Vector/Rotation signatures, not a value producer or a reason to flatten earlier first-party APIs.',
   'receiver':'Actual Avian single/math and Parry cuboid/cast bodies named in Camera048 current_sources12-22 and contract3; config/hit receiver bodies are explicitly qualified by EOF012 entries01-02.',
   'policy':'No builds or feature resolution executed. Source-pinned selection proof stays bounded to104-116 and exact named native math branches.'},
9:{'contracts':[3], 'fresh':False,
   'role':'Exact Avian Cargo364-381 native dependency selection belongs to the same explicitly traced f32/3d/Parry branch. Native primitive scalar/vector/quaternion representations are justified at actual collider/cast APIs by their implementation graph, not from manifest identity alone.',
   'receiver':'Avian current_sources12-19 math/collider/system_param/shape/ray/Transform and actual Parry shape/math current_sources20-22 in Camera048. EOF012 preserves exact shape/ray native config and hit receiver bodies.',
   'policy':'Bounded feature/dependency witness only; no whole407-line Cargo file or foreign module waiver, no build execution.'},
10:{'contracts':[3], 'fresh':False,
   'role':'Exact Parry Cargo60-87 feature context joins Avian/workspace selected f32/3d backend and the actual SharedShape::cuboid/Cuboid half-extents, native cast math receiver implementations explicitly read in Camera048.',
   'receiver':'Parry shared_shape282-300, cuboid55-90/110-127 and math1-48; Avian cuboid633-810 and cast_shape_predicate523-596 actual native bridge. These implement half-extent/native pose math, not a retained first-party geometric contract.',
   'policy':'Bounded manifest context only; no whole397-line manifest or foreign module clearance. Original dependency identity and read endpoints preserved.'},
11:{'contracts':[8], 'fresh':True,
   'role':'Full occupancies4-66 accepts existing ItemDefinition/EquipmentPlacement plus previously equipped definition/entity/placement tuples. It retains channel/order/location, constructs exact armor-review occupancy IDs, and picks first prior compatible garment by location, channel/order, positive capacity and accepted tags before producing CharacterLocation or ItemAttachment anchor.',
   'receiver':'Vec<EquipmentTopologyOccupancy> feeds whole harness physical/runtime spawn already traced by Camera048 full harness511 and clause8. Raw membership/selection/instance IDs existed before checked handles; canonical rig owner propagation is existing owner strengthening, not an evidenced peeled checked membership value.',
   'policy':'Preserve tuple/first-match order, intentional harness expect behavior, generated IDs, requirement index casts and capacity_index0. Later extent/aggregate/invalid input lenses remain open; native Entity anchor is distinct from stable attachment ID.'},
12:{'contracts':[8], 'fresh':False,
   'role':'Exact full readiness1-285 already read in Camera048 clause8: required parts/morph/skin/texture observations retain Missing/Loading/Failed/Ready, with actual readiness decisions before any JSON rendering. Serializable WeightSummary is a native observed record, not proof of an already checked scalar being stripped.',
   'receiver':'Actual mesh/assets/Skin, readiness and output path jointly traced by full harness511 plus native skinning48/material29/topology66 proof bodies. JSON is a final output only after semantic readiness decisions.',
   'policy':'Preserve required observation versus optional absence and all original fixture inputs/count/order/assertions; owner strengthening of raw readiness record remains distinct from a demonstrated producer regression.'},
13:{'contracts':[8], 'fresh':False,
   'role':'Exact full skinning1-48 is an actual native Mesh four-weight/skin observation recipient in Camera048 clause8. WeightSummary originates raw native observations; the trace explicitly distinguishes first-party readiness use before final JSON formatting.',
   'receiver':'Mesh/Skin native attribute data -> WeightSummary -> readiness decisions -> actual JSON/report. This is not an exemption for retaining raw semantic records between the observation and readiness.',
   'policy':'Keep original four-weight representation, missing/loading/failed/ready distinction and original fixtures. No inferred blanket fraction/positive constraints or already checked scalar erasure.'},
14:{'contracts':[8], 'fresh':False,
   'role':'Exact full material1-29 was read as an actual native assets/texture/material observation helper in Camera048 clause8; joins the required material/readiness graph rather than an unexplained file-hash reference.',
   'receiver':'Actual native assets/material presence -> required texture/readiness decisions and final serialized report; Camera048 full readiness285 and harness511 provide the semantic consumer proof.',
   'policy':'Retain absent/loading/failed/ready states and original fixture behavior. Actual output serialization does not waive earlier readiness state/aggregate strengthening requirements.'},
}
rows=[]
for i,obs in assigned:
    n=int(obs['json_pointer'].split('/')[-1]); r=roles[n]
    ident=identity[(obs['path'],obs['sha256'])]
    proof=trace['additional_actual_source_proofs'][n]
    assert proof['path']==obs['path'] and proof['sha256']==obs['sha256'] and proof['read_ranges']==obs['recorded_read_ranges']
    fresh=None
    if r['fresh']:
        p=Path(ident['locator_metadata']['resolved_path'])
        assert sha(p)==obs['sha256']
        fresh={'resolved_path':str(p),'sha256':sha(p),'read_ranges':obs['recorded_read_ranges'],'actual_EOF':len(p.read_text(encoding='utf-8').splitlines()),'mode':'Fresh exact bounded body read in this delta; no source execution'}
    rows.append({
        'delta_observation_index':i,'expanded_ref':obs['expanded_ref'],'source_id':obs['canonical_source_id_candidate'],
        'path':obs['path'],'sha256':obs['sha256'],'new_identity_among36':ident['new_identity'],
        'source_identity_map_membership':{'new_identity':ident['new_identity'],'delta_observation_indices':ident['delta_observation_indices'],'frozen_source_ids':ident['frozen_source_ids'],'cumulative_source_ids':ident['cumulative_source_ids']},
        'original_recorded_read_ranges':obs['recorded_read_ranges'],'original_material_contract_ranges':obs['material_contract_ranges'],
        'qualified_effective_ranges':obs['recorded_read_ranges'],'nonexistent_endpoint_credit':0,
        'exact_prior_body_witness':{'record':trace_rel,'trace':trace['trace_id'],'source_proof_pointer':obs['json_pointer'],'source_proof':proof,'material_contract_witnesses':[{'pointer':f'/contracts/{c}','text':trace['contracts'][c]} for c in r['contracts']]},
        'fresh_actual_source_read':fresh,'fresh_source_actual_read_count':1 if fresh else 0,
        'actual_material_role':r['role'],'actual_receiver_contract':r['receiver'],'preserved_behavior_and_findings':r['policy'],
        'disposition':'Exact producer role accounted with existing findings; bounded native/context/source role only',
        'remaining_actual_material_receiver':[],'owner_fit':'not performed','whole_file_clearance':'none beyond exact existing read scope; partial scopes remain partial',
    })
assert len(rows)==15 and len({(r['path'],r['sha256']) for r in rows})==15
support=[{k:x[k] for k in ['id','original_ranges_preserved','effective_original_ranges','nonexistent_lines_credit','source_proof','semantic_basis','actual_source_to_receiver_graph','remaining_material_receiver']} for x in eof['per_entry'] if x['id'] in ['EOF-ROLE-01','EOF-ROLE-02']]
artifact={
 'id':'CONT-NATIVE-EXPANDED-PRODUCER-ROLE-DELTA-002',
 'authority':'Exact15 Camera048 broader-schema producer role rows only, per root dispatch. No owner-fit, other lens, source edit, authoritative ledger change, project command, runtime, tests, CI or new acceptance.',
 'inputs':[{'record':p,'sha256':sha(B/p)} for p in [delta_rel,trace_rel,eof_rel]],
 'summary':{'assigned_observations':15,'explicit_dispositions':15,'new_identities_among_global36':sum(r['new_identity_among36'] for r in rows),'existing_identities_with_additional_role_context':sum(not r['new_identity_among36'] for r in rows),'fresh_bounded_actual_source_variants':sum(r['fresh_source_actual_read_count'] for r in rows),'exact_prior_actual_body_witness_sufficient_without_new_source_read':sum(not r['fresh_source_actual_read_count'] for r in rows),'remaining_actual_receivers':0},
 'new_source_membership':[{'source_id':r['source_id'],'path':r['path'],'sha256':r['sha256'],'delta_observation_index':r['delta_observation_index']} for r in rows if r['new_identity_among36']],
 'per_observation':rows,
 'consumed_actual_native_caster_and_hit_receiver_qualification':{'record':eof_rel,'id':eof['id'],'entries':support,'limit':'Supporting actual native bodies and immediate collision/aim receivers only. Original shape572-614 and ray384-415 endpoints preserved with zero credit beyond612/410; config constructor473-491 now explicitly read, not assumed from foreign module. Incidental492-571 gets no unrelated contract credit.'},
 'findings_retained':['CAM-PPS-F005 actual zero/invalid Dir3 conflation remains a whole camera sweep/capture policy finding.','CAM-PROP-F013 existing assert!(...).unwrap fixture seam remains source-only, with no execution or assertion changes.','PP020 geographic/render axis, PP021 raw CaptureRecord FOV/boom/arrays before manifest validation, PP022 missing readback and PP024 fixed-camera retained raw representation remain distinct existing findings.','Rig canonical catalogue/membership/selection/instance owner requirements remain; raw observed WeightSummary/subject/head/orbit is source strengthening rather than unsupported claim of erased prior checked owner.'],
 'limits':['All15 original endpoints/material ranges remain unchanged and explicitly qualified. coordinates1-105 ends inside Displacement impl; no later methods are implied.','Manifest and foreign math/cast/hit evidence qualifies exact selected native signatures and operation bodies. No foreign whole-file or generic native-kernel waiver.','Seven zero-new-read dispositions use named exact existing full/bounded body proofs AND material contract narratives, not identity/hash/range equality alone. Eight fresh application reads make specific bounded material roles explicit.','The existing158 packet remains unchanged and accepted only for its exact producer scope. No owner-fit/all13/source-complete/implementation/check acceptance follows from this packet.'],
 'remaining_actual_material_receivers':[],'next_unread_cell':'None within these15 assigned producer-role observations; hold for root dispatch.'
}
p=B/'continuation/native/expanded-producer-role-delta-002.json'
p.write_text(json.dumps(artifact,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'output':str(p),'sha256':sha(p),'summary':artifact['summary'],'bytes':p.stat().st_size}))
