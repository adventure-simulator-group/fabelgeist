"""Reconciliation-only JSON bookkeeping. Does not run project/archive scripts."""
import json,re,pathlib,collections
R=pathlib.Path(__file__).resolve().parents[2]
OUT=pathlib.Path(__file__).resolve().parent
assigned=json.load(open(OUT/'adler-input.json'))
allrows=json.load(open(R/'adler-finding-records.json'))
quals=json.load(open(OUT/'qualification-evidence.json'))
locals_=json.load(open(R/'local-findings-snapshot.json'))
def ident(r):return r.get('id') or r.get('finding_id') or r.get('supplier_finding',{}).get('id')
def key(x):return x.get('key') or x['archive_file']+'#'+str(x['record_index'])
known=collections.defaultdict(list)
for x in allrows:
 if ident(x['record']):known[ident(x['record'])].append(key(x))
# Exact full identifiers only: short Fxxx references are never expanded by guess.
pat=re.compile(r'(?<![A-Za-z0-9-])(?:'+ '|'.join(re.escape(i) for i in sorted(known,key=len,reverse=True))+r')(?![A-Za-z0-9-])')
relfields={'qualifies','qualifies_prior','parent_finding','prior_finding_refs','existing_finding_ref','existing_finding','existing_finding_refs','existing_refs','producer_finding','canonical_finding','canonical','dedup','deduplicates','dedup_original','dedup_graph','cross_lens_dedup','deduplication','supports','finding_relationship','prior','related','prior_exact_findings_or_claims'}
def refs_from(obj, fields):
 return {k:v for k,v in obj.items() if k in fields or any(w in k for w in ['contract','trace','evidence','proof','source_ref','packet_ref','receipt'])}
def strings(o):
 if isinstance(o,str):return [o]
 if isinstance(o,dict):return sum([strings(v) for v in o.values()],[])
 if isinstance(o,list):return sum([strings(v) for v in o],[])
 return []
def ids_in(o):return sorted(set(sum([pat.findall(s) for s in strings(o)],[])))
qualifier_indices={0,12,14,25,26,27,28,29,30,31,32,33,34,36,37,50,51,53,54,55,56,57,58,59,60,62,63,76,77,78,79,80,81,82,83,84,85,86,87,88,97,98,103,104,139,140,141,142,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,190,213,214,217,220,226,236,237,275,277,278,281,282,284,290,291,294,295,297,298}
qualifier_indices.update(range(64,76));qualifier_indices.update(range(238,266))
alias_indices={13};alias_indices.update(range(38,50))
needs_indices={10,106,302}
# Literal manual interpretations added only after reading the actual saved records.
notes={
0:'Supersedes the original pending SATS wording for CAMERA-ADM03-F001 only: saved decoder route is sats-basispoints-route-001. Does not certify every SATS type or runtime feature.',
1:'Raw physical public records are an admission/migration inventory. Do not infer a physical gameplay defect without the missing downstream required predicate.',
2:'Finite f64 -> stored f32 overflow. PR dimensions/grip raw; P16 checked Cuboid/SignedLength already reject those two targets. Anchor/tangent remain affected; do not erase this source distinction.',
3:'Positive active skill subtotal may round to zero in stored f32. Preserve approximate sum tolerance, inactive zero/default and both-mode policy; different target role from the local tangent witness.',
4:'Fallible ContactPrecision admission must preserve finite>=0 unbounded policy (zero and4 valid). Public raw design/equipment paths are the evidence; no executed ordinary defect.',
10:'CONT-CAMERA-PP-F001-QUALIFICATION supersedes the original producer-loss classification: current native CLI/browser inputs are rawu64; exact CharacterId fit is strengthening/granularity work, not proven checked-value erosion.',
12:'Limits CONT-CAMERA-PP-F001 to existing-owner fit/strengthening. Shared backing representation alone cannot justify mandatory producer repair.',
13:'Exact nested supplier record also exists as NATIVE-CONT-WORLD-OWNERSHIP-001 in native/findings.json#1; this receipt creates no new defect or transfer.',
14:'Exact shared roof descriptor update cause with Native05F004/Root04F004. Bearing consumer is many-to-many same-owner/node overlap; no inevitable audit failure or one-to-one-ID requirement.',
17:'Actual hybrid selected-weapon mobility receiver disproves the blanket no-mode-drift inference from CameraCONTRACT023; preserve separate legitimately equipped melee/ranged alternatives.',
21:'Source-specific support of ROOT-ADM-F002 original/resolved identity graph, not another globally unique defect.',
25:'Actual source-owner fixed point support only. New source definitions still require own producer/owner roles; no blanket cycle or other-lens credit.',
34:'Corrects prior id:String prose to actual Coreid:u64. Synthetic and durable identity authorities are not interchangeable.',
36:'Preserve f64 authored geometry, signed/zero physical outputs and DB quantization. Current family links are navigation, not automatic whole graph dedup.',
37:'Digest is SHA256 and dialogue buildEOF580 per RIG-CAMERA-ERRATUM001; private relationship wrappers have narrower authority than generic strings.',
50:'Preserves WorldF001 and root21 first-party fraction schema qualification. No claim transfer or blanket external-format waiver.',
51:'Supplier evidence requires exact shared repair-target reconciliation; no source/other-lens credit.',
52:'ThreatId can fit successful strictly validated compiled monster identity, not every raw direct-compile/source key. Preserve lexicalString iteration versus ThreatId length-firstOrd.',
55:'P16 coordinateEOF140/orientation142; familyEOF114. Excluded docs/header/context grant no body credit.',
59:'New definitions provide exact-role owner evidence only. Pending producer follow-up is not waived.',
60:'Actual native gain/release owner closure resolved the earlier unknown helper; Never versus positiveAfter and zero-valid rise/limp remain distinct.',
62:'Dependent UnitBasisPoints transparent decoder defect joins CAMERA-ADM03-F001; repairing inner canonical decoder is enough, no redundant wrapper validator.',
63:'Exact rawTerrainFeature/TravelGeometryPoint path strengthens ROOT-ADM-F007; no new global defect.',
77:'Exact support of Camera07F048 first-rig/row-count cache. Same-count replacement/reorder is API-only; ordinary full rebuild changes rig.',
80:'Existing SignedLength.scaled_by factor can retain broad all-f32Ratio; multiply/result admission and failure order remain unchanged.',
86:'Arbitrary scaled plane normal+constant and loose concave/cutout polygons are not equivalent to unitDirection or strict convexPlanPolygon. Missing aggregate policy stays explicit.',
87:'Measured contact min/max can reverse under tolerated gap. Ordered spatial bounds would reject valid measured state.',
88:'Freeform editor floor/side0 label does not establish generated occupied RoomIndex membership; None/Some0 distinctions remain.',
89:'Raw roof owner-strengthening inventory differs from previously checked point loss. Preserve scaled planes, concavity/holes, signed winding, zero/None and original error policy.',
94:'Forge raw JSON f32 overflow depends on reviewed non-float_roundtrip Serde branch. Actual unified target feature resolution is unproved; keep conditional.',
97:'Count native executable-capacity evidence extends Rig03F073; no new finding. Keep generic allusize/zero count distinct from checked GPU executable capacity.',
98:'Worker product semantic/correlation gate supplements native row admissionF028 and raw workerDTO notes. Ticket matching alone is not semantic admission.',
103:'Actual catalogue revision is SHA256/domain dialogue-catalog.v2, not another digest owner. BuildEOF580, no581..592 credit.',
104:'Navigation correction only: expanded-producer-role-delta-reference-errata-052 and world5-final-packet.json#NATIVE-CONT-WORLD-AUTHORITY-FINAL-029 are actual references.',
105:'Move generic checked geometry once to neutral owner, retain game frame markers and cross-frame authority in owning applications. Canonical backing visibility must be repaired coherently.',
106:'NonnegativeLength neutral placement remains suggestion pending full actual consumer map; same numeric invariant alone is not independent duplicate-definition proof.',
120:'Compose existing CollisionCuboid but preserve the separate shape-envelope relation Native05F005; composition alone does not seal it.',
123:'F025 in this record is roster/policy navigation. No automatic union of distinct personality/material/book repair graphs (Camera07QUAL205).',
126:'CharacterLod4..6 belongs with actual MHR loaders; armor runtime/bake detail and terrainVista are different roles. Exact localM18 overlap.',
127:'Canonical influence reduction and preview differ in duplicate/zero policy. Resolve Native05F010/Rig03F010 admission contract first; do not blindly copy NaN behavior.',
128:'CanonicalSkill identity and book-eligible subset/ceilings can be shared. EverySkill is not automatically book-eligible; terrain needs separate semantics. F025/F011 are broad navigation, not global dedup.',
129:'Three strictly positive tuning lengths differ from signed/zero battle measures. ContactAtTime is spatial metres (QUAL052), not clock.',
134:'Existing ValidatedQuestGenerationAuthority is discarded into swapped manifest/context Strings; exact localF19 defect. Rig06QUAL069 corrects actual source IDs to investigation/projections.rs and strategic/authority_model.rs.',
135:'Modeled non-ranged combat fact must own insight eligibility; matching displayed English sentence is current accidental authority. Exact localM41 overlap.',
136:'Source-level state-shape migration, not claim of an observed malformed persisted row. Preserve actual parsed_state gates until typed product reaches callers.',
137:'Required target belongs in target-requiring TemporalScope variants. Actual NPC/clock permissions remain separate.',
138:'Paid/InsufficientFunds payload distinction must preserve planned unpaid amount and NoDowry father. Current producer is coherent; no runtime defect claimed.',
157:'No available same-authority typed Mesh/BVH/review cause established. This is missing-owning-reason design work, distinct from actual SDK cause erasure.',
158:'Boundary wall slots are not RoomIndex, StoreyIndex, rig ordinals or durableWallAssemblyId; exact missing owner remains explicit.',
159:'Raw puzzle authority bypass and validator omissions remain candidate obligations. Normal replay/generator protections are not withdrawn; Rune nonrepeat omission joins Native05F045.',
160:'Final folded-grid counterexample is20971201, not16777217 alone. External capacity/device/shader feasibility and first-party use unproved; supplier obligation is not unconditional third-party edit instruction.',
168:'Named checked generated authority discarded/swapped; same localF19/Rig06F050 graph, not new defect.',
172:'Bounded emitted probability fit differs raw broad/NaN ingress; do not silently strengthen every primitive input.',
183:'Typed bounded emitted wetness/exposure leaves do not narrow fulli32 strain, rawu16 wetness or native frostbite duration semantics.',
185:'UnknownMotion can retain admitted MotionId; InvalidMotionStem must retain rejected rawString. Exact localM05 overlap on UnknownMotion only.',
188:'Actor-root fallback coordinate frame must stay distinct from worldScene; native quaternion/vector data are not blanket Scene positions.',
189:'Jitter diagnostics intentionally retain invalid/nonfinite evidence; finite geometry or UnitInterval would erase observations. Angular acceleration and jerk require separate unit evidence.',
190:'Ownership metadata update only; no independently stated defect. Does not transfer or establish repair acceptance.',
192:'Exact localM06 cache cause erasure, preserving distinct Pending/Failed and original typed SDK/rig causes; no automaticSend/SyncArc choice.',
198:'Exact localM14 DesignHash retention and fixed cache spec overlap; type alone does not authenticate payload or prove every field distinguishes admitted artifacts.',
204:'Typed rig ordinal-asusize cast is a specific source defect, not a new table namespace. Joins RIG-CODE-003.',
210:'Exact localM74 checked timber/cuboid producer and retained facade bridge; zero extents remain valid.',
211:'GrantedCharacterId is authority refinement, scalarCharacterId inner leaf is insufficient. Character JS precision exact localF31; grant erasure overlaps localM55.',
212:'Inventory JS fullu64 precision is distinct authority from character localF31. Shared precision failure class does not merge identities.',
228:'RIG-PP-004 was withdrawn: private EquipmentItemSelection membership implementation is intentional. This actual retained-key gap survives; no revival ofPP004.',
229:'Local inventory graph identity is not proof of DB identity provenance; retained keys can share canonical local owner without imposing account/durable authority.',
230:'RIG-PP-004 intentional membership implementation remains withdrawn. Actual acquire->CatalogArticle retained item/placement text graph still requires repair.',
231:'Historical static type mismatch at frozen source. Recheck actual current device_fit/Skin API before scheduling code fix; no compiler execution in reconciliation.',
233:'Historical canonicalCatalogItem writer mismatch at frozen source. Source must be revalidated on chosen integration head, no current build failure claim.',
234:'Exact localM18 loaded CharacterLod strip/re-admit graph; do not borrow armor/bakeLOD contract.',
235:'Exact localM07 admitted catalogue selection discarded into presentation Strings; preserve LastFit recovery/held/unworn policies.',
253:'dedup_correction adds original NativePP015 normalized attack-height graph; raw normalized source is owner strengthening, not always previous checked-leaf loss.',
264:'Automatic frame migration mismatch, no gameplay defect inferred. Do not conflate actor-local and worldScene.',
266:'Explicit new enum escape at exact retained source seam; no prior exact dedup asserted.',
267:'Explicit new checked identity escape, preserve identity authority and real output boundaries.',
268:'Typed application input encoded then reconstructed before native boundary; an actual serialization API call does not itself end first-party semantic responsibility.',
269:'Native serialization cause erased inside typed configuration error, exact source seam distinct from broad cause category.',
270:'Missing actor-local observation/frame contract differs worldSceneVelocity CAM-PROP-F015; broad geometry links are related constraints, not alias.',
271:'Absent actor identity substituted with valid ordinal0 is not the same defect as missing clock CAM-PROP-F023.',
272:'Capability telemetry source differs commitment/effective-response CAM-PPS-F002. Same enum-propagation class is not exact dedup.',
273:'Biological BodyMassKg checked-leaf loss differs generic total/equipment mass owner design; no global narrowed mass.',
275:'Corrects original D1 native-kernel exemption for internal composite grab APIs. Genuine final numerical/native operations remain proper ports.',
277:'Corrects old retained-equipment cause exemption and adds actual byte/cause producer support; existing lower owners are retained before true native output.',
278:'Supersedes D1 contracts018 numerical-kernel rationale and017 Lookguides sentence for internal signatures/state only; actual native body evidence stays valid.',
281:'Only actual typed application causes are retained; native remote rejectionString, fail-closed token classification and final rendered feedback remain distinct.',
283:'Available typed first-party cause erasure is separate from message-only owning rejection inventory localM20; preserve actual source chains.',
284:'Inventory fullu64 precision graph aliases RIG-OWNER-F026, not character DOM localF31.',
286:'CanonicalRoomIndex retained gap differs raw physical source owner/admission strengthening; keep actual source identity and None/Some0.',
287:'Checked PlanDimensions component stripped into roof-window headheight; newly computed min/max/rotation still require own admission.',
290:'New concrete observation/identity/cause sites support existing canonical groups; no claim all optionals/errors share a remedy.',
291:'Already-admitted route point loss differs broad raw abstract weather lattice. Do not globally impose WGS84 or positive elevation.',
292:'Follow-up sampler expects nonoverflow while authored independentu32 weights admit summedu64 overflow. Preserve existing SamplingError path and legitimate emptyNone, RNG/order/weights; no new tests.',
293:'HTTP body Err->empty and quest query defaults add distinct source seams to existing cause/required-observation groups. Keep fail-closed readiness and privacy.',
294:'Exact localM82 pose/physical retention overlap. Broad weapon cause/physical groups are dependency links, not one unique defect by assertion.',
296:'Finite-or-unbounded curve policy needs independent enum/admission/operations evidence before implementation. No automatic numeric strengthening.',
297:'Extends exact weapon shape leaf propagation; old broad physical group is navigation until shared actual repair shown.',
299:'No-edge/selection sentinel and mixed-unit scalar policy are later-lens candidates; no accepted final source verdict.',
300:'Exact localM24 semantic diagnostic path/cause retention. Preserve native source coordinates/first-error/rendering.',
301:'Canonical equipment anatomy/channel/occupancy parse result discarded then rechecked. Raw JSON invalid diagnostics must remain representable.',
302:'Targetu32 range versus roundedf32 limit/saturating cast has not been established in this saved candidate; require exact operation/admission evidence and current recipients before treating it as a defect.'}
# Concrete local overlap labels; these are reconciled obligations, not snapshot clearance.
localmap={2:[('F12','same_tangent_target_representation_defect')],3:[('F12','additional_skill_distribution_role_same_acceptance_family')],45:[('F02','supplier_capture_contains_same_failed_sweep_contract')],71:[('F02','supplier_capture_contains_same_failed_sweep_contract')],94:[('M80','same_forge_view_state_additional_feature_conditional_admission')],98:[('M66','same_worker_product_admission_native_columns_subset'),('M70','connected_semantic_interior_product_gate_distinct_predicate')],109:[('M27','overlapping_alcohol_product_and_composition_contract')],112:[('M28','same_material_quantity_retention')],114:[('M81','same_forge_stock_retention')],115:[('M26','same_damage_bins_retained_product_subset')],126:[('M18','same_MHR_LOD_owner_and_retention')],128:[('M23','same_book_skill_identity_and_policy')],129:[('M01','same_physical_quantity_graph_positive_tuning_subset')],134:[('F19','same_swapped_generation_manifest_defect')],135:[('M41','same_display_lore_mechanic_authority')],149:[('M14','same_weapon_designhash_retention_subset')],154:[('M12','same_door_window_checked_leaf_retention')],161:[('M26','same_DamageBins_product_role_in_fixture_helper')],164:[('M56','same_canonical_own_religion_retention_subset')],168:[('F19','same_swapped_generation_manifest_defect')],185:[('M05','same_UnknownMotion_error_payload_subset')],192:[('M06','same_retained_runtime_equipment_cause')],198:[('M14','same_designhash_cache_retention')],210:[('M74','same_timber_component_and_placement_retention')],211:[('F31','same_character_JS_precision_defect'),('M55','same_grant_refinement_erasure')],213:[('M01','connected_precision_physical_graph_distinct_leaf')],220:[('M60','same_graphics_configuration_cause_subset'),('M61','same_graphics_configuration_retention_subset')],221:[('M66','connected_worker_product_native_row_owner')],224:[('M73','same_interior_lighting_origin_frame_geometry_subset')],225:[('M64','same_vista_extent_graph_subset')],232:[('M17','same_skeletal_morph_identity_retention')],234:[('M18','same_MHR_LOD_owner_and_retention')],235:[('M07','same_runtime_equipment_selection_retention')],257:[('F02','same_sweep_failure_contract_with_related_camera_state')],266:[('M62','same_canonical_CloudForm_retention')],267:[('M31','same_canonical_disease_event_identity_retention')],272:[('M09','related_typed_combat_categories_distinct_capability_source')],275:[('M12','same_internal_door_window_grab_API')],280:[('M36','same_checked_route_geographic_input_retention_subset')],286:[('M72','connected_room_identity_retention_distinct_admission')],294:[('M82','same_weapon_placement_pose_retention')],297:[('M82','same_weapon_point_graph_additional_shape_sites')],300:[('M24','same_catalogue_diagnostic_path_cause_retention')]}
manualparents={10:['CONT-CAMERA-PP-F001-QUALIFICATION'],13:['NATIVE-CONT-WORLD-OWNERSHIP-001'],14:['NATIVE-INITIAL05-F004','ROOT-INITIAL04-F004'],17:['CAMERA-INITIAL07-F009'],21:['ROOT-ADM-F002'],34:['CAM-OWN-BATTLE-ADMISSION-039'],62:['CAMERA-ADM03-F001'],63:['ROOT-ADM-F007'],123:['CAMERA-INITIAL07-F011'],128:['CAMERA-INITIAL07-F011'],134:['CAM-OWNER-F011'],168:['RIG-INITIAL06-F050'],211:['RIG-OWNER-F025'],212:['RIG-PP-006'],228:['RIG-PP-004'],230:['RIG-PP-004'],284:['RIG-OWNER-F026'],292:['RIG-PPS-F020']}
for _i in [149,257,266,267,213,221]:localmap.pop(_i,None)
qual_by_id=collections.defaultdict(list)
for _q in quals:
 for _id in ids_in(_q['record']):qual_by_id[_id].append(_q)
result=[]
for i,x in enumerate(assigned):
 t=x['record'];id_=ident(t);d='alias' if i in alias_indices else 'needs-evidence' if i in needs_indices else 'qualifier' if i in qualifier_indices else 'newcandidate'
 rf={k:v for k,v in t.items() if k in relfields}; nested=t.get('supplier_finding')
 if isinstance(nested,dict):rf['supplier_finding']={k:v for k,v in nested.items() if k in relfields}
 parentids=sorted(set(ids_in(rf)+manualparents.get(i,[]))-{id_})
 if i==13 or i in range(38,50):parentids=[nested['id']]
 qlinks=[]
 for q in qual_by_id.get(id_,[]):
  # Exact full ID occurrence anywhere; link is evidence association, never inferred clearance.
  if id_:qlinks.append({'archive_file':q['path'],'record_index':q['index'],'id':ident(q['record']),'role':'qualification_or_supplement_to_consume_before_source_verdict'})
 short=[]
 for s in strings(rf):
  stripped=pat.sub('',s)
  short+=re.findall(r'(?<![A-Za-z0-9])(?:F\d{3}|RootF\d{3}|Native\d+|CAM-PROP physical contact precision|cameraF\d{3})(?![A-Za-z0-9])',stripped)
 ev=refs_from(t,relfields)
 for name in ['source','source_roles','locations','source_locations','source_backed_targets','source_sites','earliest_sources','actual_source_roles','earliest_producers','use_locations','use_sites','uses']:
  if name in t:ev[name]=t[name]
 correction={k:v for k,v in t.items() if any(s in k.lower() for s in ['qualif','limit','correct','retract','preserv','dedup','classification','kind','invariant','contract','gap','finding','acceptance']) and k not in ['id','finding_id']}
 entry={'key':key(x),'id':id_,'disposition':d,'interpretation':notes.get(i,('Exact saved supplier receipt duplicates the separately identified supplier record; retain parent evidence and count once.' if d=='alias' else 'Saved record supplies source-specific support or an explicit qualification for the linked obligation; preserve its original bounds and do not count an additional defect.' if d=='qualifier' else 'Saved source/counterexample/repair assertion is a candidate obligation pending source recovery and semantic revalidation at the selected execution snapshot; no globally unique or current-head defect claim.')),'parent_links':[{'id':q,'finding_record_keys':[k for k in known.get(q,[]) if k!=key(x)],'status':'exact_identifier_resolved' if q in known else 'identifier_not_present_in_1002_finding_records','inference_limit':'Record assertion/navigation only; no merge by shared prefix or broad family.'} for q in parentids],'asserted_relationships':rf,'ambiguous_short_references':sorted(set(short)),'evidence_refs':ev,'qualification_refs':qlinks,'correction_or_limits_from_record':correction,'local_links':[{'id':a,'relation':b,'snapshot_limit':'Local merged13fc9dcc evidence; frozen6533/preserved evidence must be compared before common retirement.'} for a,b in localmap.get(i,[])],'discovery_status':'package_reconciled_only','repair_status':'not_authorized','validation_status':'no_project_checks_or_source_clearance'}
 result.append(entry)
assert len(result)==303 and len({x['key'] for x in result})==303
assert {x['key'] for x in result}=={key(x) for x in assigned}
for x in result:
 for y in x['local_links']:assert any(z['id']==y['id'] for z in locals_)
(OUT/'crosswalk.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
summary={'assigned_records':len(result),'keys_covered_once':len({x['key'] for x in result}),'dispositions':dict(collections.Counter(x['disposition'] for x in result)),'records_with_parent_links':sum(bool(x['parent_links']) for x in result),'records_with_qualification_links':sum(bool(x['qualification_refs']) for x in result),'records_with_local_links':sum(bool(x['local_links']) for x in result),'limits':'Disposition counts are record accounting only, not unique defects or source/lens coverage. Unresolved precise IDs and shorthand remain explicit.'}
(OUT/'crosswalk-summary.json').write_text(json.dumps(summary,indent=2)+'\n'); print(json.dumps(summary,indent=2))
