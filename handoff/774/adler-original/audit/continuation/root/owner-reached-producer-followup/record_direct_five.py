import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021')
D=R/'continuation/root/owner-reached-producer-followup'
D.mkdir(parents=True,exist_ok=True)
q=json.loads((R/'continuation/owner-reached-producer-followup-queue-002.json').read_text())
items=[x for x in q['items'] if x['assigned_owner']=='root'];assert len(items)==5
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
comparisons={
 'model/storey_index.rs':{
  'producer':'Actual native usize ordinal -> StoreyIndex::new; actual u16 codec observation -> from_serialized; checked Architectural Elevation/PositiveLength -> original divide/round -> reject nonfinite/belowground/>u16MAX -> new tagged ordinal.',
  'retention':'StoreyIndex private usize survives constructors, from_elevation result, Ground/FirstUpper constants and index-key use. OrdinalError retains RecipeOrdinalRole/usize rejected ordinal/TryFromIntError.',
  'operations_and_boundary':'index() is a scalar only where actual collection indexing occurs; serialized_ordinal performs checked narrowing. Serialize calls that and immediately native u16 serializer, Deserialize immediately wraps native u16; Display formats at terminal presentation. Those exact numeric/codec ports are valid, not blanket helper exemptions.',
  'actual_receiving_gap':'Fresh existing gable_openings205–245: StoreyIndex::new(program.storeys.len()).serialized_ordinal()? at241 is retained in first-party RectangularRoofWindow.storey_level:u16. Actual roof_wall_opening1–76 and373–405 confirms it survives build then WallAssembly.storey_level396, before actual Serde. Keep StoreyIndex through both application records and actual consumers; packed u16 conversion belongs codec. Existing index/room/ground0 distinction preserved.',
  'policy':'Allusize new input remains legal until checked serialization; from_elevation rejects rawcast saturation, so do not silently substitute it for arbitrary raw native cast inputs. Original packed0/1/MAX, oversized and overflow/belowground fixture cases unchanged/no execution.',
  'references':['continuation/native/root-building-owner-fit/coverage.jsonl gable_openings and heating/placement exact semantic proof','ROOT-PPS-WALL-SOURCE-CLOSURE-039','RIG-PPS-029','CONT-ROOT-OWNER-ROLES-002'],
  'status':'direct_role_accounted_with_findings'
 },
 'model/timber.rs':{
  'producer':'Native all-u64 words acquire distinct TimberFrameAssembly/Facade/Line/StoreyFrame/Bay/Member/Joint identity namespaces, including0, via actual transparent constructors/Serde. TimberFrameMember already retains typed structural endpoints Position<Architectural>, PlanDimensions section, source/member/joint/solid IDs and material/role.',
  'retention':'Members and typed endpoints/section are cloned into Gable Candidate and retained there; complete member clone head_member in RectangularRoofWindow remains typed. These are genuine preserved edges, not field-count coverage.',
  'actual_receiving_gap':'Candidate.head.section_metres.metres().x enters RectangularRoofWindow.head_height at238 unchanged and is stored f32 before build/nodes/solids. Actual receiver1–76/94–110/137–152 confirms source headheight retained before actual base+sill+clear+head*.5 equation and final StructuralNode admission. Carry source section component through first-party field/signatures until equation; add coherent canonical PlanDimensions component API rather than get/reconstruct at callers.',
  'legitimate_operations':'Candidate midpoint addition/projection and base midpoint.y + sectionx*.5 - tolerance produce new coordinates, so later admission is legitimate. Native .0 only at terminal audit formatting or new numeric identity namespace operation; all-u64 tagged identity does not imply membership or nonzero.',
  'references':['continuation/native/root-building-owner-fit/coverage.jsonl exact gable/heating source comparisons','ROOT-PPS-WALL-SOURCE-CLOSURE-039','RIG-PPS-029','ROOT-PPS-RESOLVED-GEOMETRY-036'],
  'status':'direct_role_accounted_with_findings'
 },
 'model/layout.rs':{
  'producer':'Actual public/Serde RoofPlaneEquation normal/constant, RoofFace polygon/cutouts/pitch/thickness, RoofEdge endpoints and RoofAbutmentSample point are raw source roles. Roof/child/wall/node/resolved IDs, enums, arrays and Options remain tagged. Shape is not admitted merely by a later checked envelope.',
  'operation':'underside_height_at192–198 is actual scaled plane equation using normal.length and constant: keep arbitrary scaled/nonunit normal plus correspondingly scaled constant, division normal.y and exact FP order. Normalizing only direction changes geometry. Result is newly computed height, not an unchanged checked elevation peeled.',
  'newly_reached_receivers':'Actual navigation found six previously unregistered producer/mutation modules; Root registered exact ranges in scope007 before analysis and dispatched Native supplier. Native freshly read six, identified exact roof_polygon_bounds checked Position then discard before raw RoofFace retention, and is tracing eleven pre-registered direct helper recipients. Definition/operation inspected here is not closure of those unread receivers.',
  'frame_and_shape':'Architectural three-dimensional weather-face polygon and holes/concave enclosure differ Scene, model-local or strict convex PlanPolygon. Existing canonical frame/length/angle leaves may be reused after exact original admission policy; new raw source strengthening is distinguished from actual checked-point discard supplied by Native.',
  'references':['continuation/root/owner-fit-delta/source-role-owner-comparison-002.json','continuation/producer-followup-scope-extensions-020.json','continuation/native/root-roof-producer-followup/pending-recipients-001.json'],
  'status':'direct_definition_operation_accounted; materially_reached_roof_producer_recipient_extension_incomplete_until_Native_packet'
 },
 'audit/resolved_geometry.rs':{
  'producer':'Actual ResolvedSolid centre/size/angles and void/support/bond SpatialBounds are checked source values. Existing resolved_solid_bounds returns raw(Vec3,Vec3) then repeated internal overlap APIs retain those values. Fresh71–514 completes actual receiving audit body beyond old1–70.',
  'paths':'Void bounds min/max at149 derives new size/halfwidth; member start projection and section scaling160 are actual dot/threshold arithmetic. But support interface bounds251–253/258–260/299–301 and void_bounds390 -> resolved_shape_overlaps_bounds477 are unchanged bounds across internal helper calls. Preserve canonical Architectural bounds there, with signed tolerance policy and typed error path to actual SAT/sample equations.',
  'other_material_roles':'Same typed IDs/roles stay through member/bay/wall/roof lookup and membership, and are projected to numeric words only actual final AuditIssue message. max section/radius/new penetration are physical lengths, drainage inlet.y-outlet.y is a new elevation difference; not unitless Ratio. map_or0 absent member is an optional policy, not proof of observed0. Bond contact planes may reverse under tolerance; do not replace with ordered SpatialBounds.',
  'actual_callee_proof':'Personally consumed complete ROOT-PPS-RESOLVED-GEOMETRY-036 current producer trace: complete445 resolved helpers, full solid_overlap SAT, bonding_metrics typed result admission and exact numeric receivers. These are exact same bounds/point/tolerance operations; no old owner verdict or path/hash-only closure. Real local SAT/cross/dot/scalar reduction may extract at actual equation, not at retained application API.',
  'references':['ROOT-PPS-RESOLVED-GEOMETRY-036','CONT-ROOT-OWNER-AUDIT-005','continuation/root/owner-fit-delta/audit-owner-comparison-005.json'],
  'status':'direct_role_accounted_with_findings'
 },
 'audit/shared_helpers.rs':{
  'producer':'Actual roof-face polygon/cutout records provide Architectural XYZ vertices; outer/cutout map projects x/z and collects Vec<Vec2>. Incoming point is same architectural plan query.',
  'receiver':'Full predicate85–134 was scoped across original85–120 plus new121–134. Actual tail supplies point_in_polygon_2d outer inclusion and cutout exclusion, preserving source order, empty/edge/ray arithmetic and arbitrary concave holes. Internal projected polygon/point helper APIs retain semantic frame; canonical planpoint leaf should reach actual polygon equation, without rehull or false convex admission.',
  'policy':'Projection creates a plan coordinate, not unchanged XYZpoint; preserve Architectural frame and successful finite policy separately from current raw public/Serde source. Bool final result is actual containment classification. Unrelated135–171 receives no scope/analysis credit.',
  'references':['ROOT-PPS-RESOLVED-GEOMETRY-036','continuation/root/owner-fit-delta/audit-owner-comparison-005.json','continuation/native/root-roof-producer-followup upcoming exact roof geometry receiver proof'],
  'status':'direct_role_accounted_with_findings'
 }
}
rows=[]
for x in items:
 name=x['source_id'].split('/src/',1)[1]
 value=comparisons[name]
 rows.append({'obligation_id':x['id'],'source_id':x['source_id'],'sha256':x['sha256'],'material_ranges':x['material_ranges'],'trace':value,'source_read':'Fresh exact current bounded source body under this producer lens; actual related source bodies read only already-global ranges; explicit prior producer/callee facts personally consumed where reused.','repair':'Existing whole Root/Rig structural source/model/resolved bounds API claims composed after all13. Preserve all old findings; no temporary primitive forwarding, duplicate owner or new fixtures.','validation':'none'})
out={'id':'ROOT-OWNER-REACHED-PRODUCER-DIRECT-FIVE-001','at':now,'lens':'audit-producer-propagation','traces':rows,'global_status':'incomplete_direct_roof_receiver_extension; no overall producer/owner/family clearance','source_edits':0,'project_commands':0}
p=D/'direct-five-traces-001.json';assert not p.exists();p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
progress={'at':now,'assigned_direct_obligations':5,'direct_contracts_inspected':5,'direct_accounted_with_findings':4,'direct_definition_with_receiver_extension_pending':1,'new_roof_source_supplier':'continuation/native/root-roof-producer-followup','first_unfinished':'Native roof recipient traces and global local-registry source reconciliation, then owner-fit delta','source_edits':0,'project_commands':0,'clearance':'none'}
(D/'progress-001.json').write_text(json.dumps(progress,indent=2)+'\n',encoding='utf-8')
print(json.dumps(progress))
