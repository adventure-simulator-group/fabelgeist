import json,pathlib,re,hashlib,zipfile,collections
ROOT=pathlib.Path('target/bespoke-audit-774-merged-20261022/reconcile-adler-20261023')
OUT=ROOT/'assignments/engine'; SRC=ROOT.parent/'source'; AUD=ROOT/'audit'
a=json.loads((OUT/'adler-input.json').read_text()); local=json.loads((OUT/'local-input.json').read_text()); loc={x['id']:x for x in local}
# Manual semantic decisions: exact defect/remedy matches only; shared function
# scheduling is separately represented by repair_cluster and local_related_ids.
match={3:['M15'],19:['M15'],23:['F06'],36:['M16'],38:['M16'],39:['M16'],42:['M16'],70:['M17'],71:['M19'],72:['M19'],81:['M18'],85:['M19'],96:['F01'],99:['M02'],100:['M02'],122:['M61','M67'],123:['M61'],139:['M62','M63'],144:['M61'],145:['M60','M76'],146:['M64','M65'],150:['M70'],153:['M71'],154:['M66'],159:['M73'],173:['F40'],183:['M61'],190:['M12'],191:['M12'],192:['M68'],194:['M73'],195:['F40'],196:['F32'],204:['M64'],212:['M02'],213:['M02'],214:['M02'],217:['M62'],219:['M63'],221:['M63'],236:['M64'],253:['M60'],256:['F02']}
related={4:['M16'],7:['M17'],8:['M17'],10:['M07'],11:['M18'],12:['M17','M20'],13:['M19'],14:['M16'],17:['M16'],18:['M16'],20:['M16'],21:['M20','M22'],22:['M16'],24:['M16'],25:['M16'],26:['M16'],27:['M16'],28:['M16'],29:['M16'],30:['M20','M22'],31:['M17'],33:['M16'],34:['M17'],35:['M07','M10'],37:['M19'],43:['M16'],44:['M16'],67:['M16','M19'],69:['M10'],73:['M16'],74:['M16'],87:['M16'],90:['M14'],91:['M14'],92:['M15','M18'],93:['M15'],94:['F07','M11'],95:['M09'],97:['M17'],98:['M03'],101:['F02'],102:['M08','F08'],103:['M10'],104:['M07'],105:['M02'],106:['M08'],107:['M02'],108:['M80'],112:['M04'],113:['M04'],114:['M08'],115:['M02'],116:['M61'],117:['M02'],118:['M02'],119:['M02'],120:['M02'],121:['M02'],124:['M61'],125:['M64'],126:['M64'],127:['M64'],128:['M72','M76'],129:['M70'],130:['M72'],131:['M70'],132:['F07'],133:['M08'],134:['M09'],135:['M09'],136:['M64'],137:['M64'],138:['M64'],140:['F07'],141:['M02','M09'],142:['M12'],143:['M12'],147:['M66'],148:['M64'],149:['M70'],151:['M69','M70'],152:['M61'],155:['M61'],156:['M61'],157:['M61'],158:['M61'],160:['M61'],161:['M61'],162:['M61'],163:['M61'],164:['M61'],165:['M79'],166:['M66'],167:['M70'],168:['M64'],169:['M64'],170:['M72','M76'],171:['M66'],172:['M76'],174:['F01'],175:['M59'],176:['M12'],177:['M76'],178:['M64'],179:['M13'],180:['M61'],181:['F02'],182:['F07'],184:['M80'],185:['F07'],186:['M08'],187:['M08'],188:['F01'],189:['F01'],193:['M76'],197:['M61'],198:['M64'],199:['M64'],200:['M76'],201:['M70'],202:['M82'],203:['F38','F41'],206:['M82'],207:['F07'],208:['M02'],209:['F01'],210:['M02'],211:['M61'],215:['M08'],216:['M77'],218:['M63'],220:['M63'],222:['M64'],223:['M66'],224:['M64'],225:['M64'],226:['M64'],227:['M64'],228:['M64'],229:['M82','F38'],230:['M82'],231:['M20'],233:['M60'],234:['M09'],235:['M64'],237:['M76'],238:['M68'],239:['F10','M16'],240:['M80'],241:['M70'],242:['M68','M70'],243:['M64'],244:['M64'],245:['M63'],246:['M65'],247:['M64'],248:['M64'],249:['M64'],250:['M70'],251:['M76'],252:['M68','M76'],254:['M64'],255:['M08'],257:['M64'],258:['M76'],259:['M73']}
# Explicit equivalence/support identities from actual records and corrections.
canon={13:37,14:25,18:26,19:3,22:33,24:17,39:38,43:25,44:29,48:46,50:45,65:25,66:66,74:38,76:55,77:56,139:217,142:190,153:153,170:128,173:195,177:193,192:192,193:251,198:169,200:172,201:131,205:230,218:220,219:221,222:249,232:88}
alias={13,14,19,22,24,43,48,65,76,77,139,177,193,198,200,201,205,218,219,222,232}
# Every cluster below is a connected scheduling unit, not an asserted unique defect.
clusters={}
def group(key,indices,owner,dep,accept):
 clusters[key]={'archive_indices':indices,'canonical_owner':owner,'dependencies':dep,'acceptance':accept}
 for i in indices: idx_cluster[i]=key
idx_cluster={}
group('creator-realization-and-cache',[0,2,3,7,8,9,10,12,19,70,81,92,93], 'Creator body/fitting/model generation; canonical Rig/MHR source owners', ['rig-skin-and-mhr','shared-gpu-resource-contracts','creator-checked-geometry'], 'Immutable producer/configuration/topology identity survives body/morph/device/cache recipients; count equality never substitutes for source identity. Retain source-specific UI/API and label qualifications.')
group('creator-presets-and-abi',[1,4,5,6,11], 'Canonical armor material/course/layout/frame-rule and MHR expression owners', ['shared-gpu-resource-contracts'], 'One authoritative preset/count/layout/encoding declaration feeds both host and actual native receivers; current agreed values and packing order preserved.')
group('creator-checked-geometry',[17,18,20,21,24,26,27,28,29,30,31,32,34,35,44,87], 'Creator cloth/shell/loadout/settled-drape admission and shared GeneratedArmor owner', ['rig-skin-and-mhr','shared-gpu-resource-contracts'], 'All exposed constructors, decode/mutation and final completion routes establish their exact topology/attribute/finite/result relation before indexing/publication; preserve empty/raw builder and warning-only policies.')
group('creator-inventory-identity',[23,69], 'Creator InventoryItemId allocation and generic Core EquipmentGraph key owner', ['equipment-topology-and-caches'], 'Checked full-u64 exhaustion is atomic; generic graph retains caller identity without conflating Creator IDs with durable DB IDs.')
group('armor-layout-and-design',[13,36,37,38,39,40,41,42,67,71,72,73,74,82,85], 'Armor GpuBody/PartRecipe/chart/shell/grid/staging owners; Creator fitting facade', ['shared-gpu-resource-contracts','rig-skin-and-mhr','creator-checked-geometry'], 'Admitted design/layout/native address capacity and resource association survive complete functions; gauge overflow rejected structurally; byte scratch minimum-four stays distinct from metal word scratch.')
group('shared-gpu-resource-contracts',[45,46,47,48,49,50], '#780/#826 coordinated connected GPU owners, exact whole-function split pending', ['canonical #779 71f9cab8 WorkgroupGrid/byte/offset contract','reported #826 1f7 successor and #780 6f6d77878 lineage must be source-compared'], 'One sealed context/schema/layout/allocation contract reaches parser, fast/generic/ComputePass packing, cache, submit/readback and recording; native causes retained and callbacks only on successful submission. Generic helper Unsigned gap is not KernelBatch fallback.')
group('hair-executable-shapes',[15,16,22,33,51,52,53,54,55,56,57,58,59,68,75,76,77], 'Hair canonical identities/curves/parting/density/surface/grid and executable native shape owners', ['shared-gpu-resource-contracts','creator-realization-and-cache'], 'Admit complete executable populations/products/packed lanes/curve and surface correlation before native use; preserve all-word seeds, intentional zero/no-mask/empty-storage and infinite-search meanings; no invented universal caps.')
group('rig-skin-and-mhr',[14,25,43,60,61,62,63,64,65,66,79,80,84,174], 'Canonical fabelgeist-rig, fabelgeist-mhr and compatible geometry owners', ['source-version-and-owner-reconciliation'], 'Validate full weight/ordinal/row/topology/parent/model/corrective/FBX transform relationships at true admission; retain canonical leaves and exact parser policies through native recipients.')
group('geometry-shared-leaves',[83,178,222], 'Lowest shared fabelgeist-geometry/math quantity owners', ['source-version-and-owner-reconciliation'], 'Reuse exact compatible leaves without changing frame, signed-zero, full range or arithmetic order; do not merge support provenance wrappers by name alone.')
group('source-and-fixture-obligations',[86,88,232,239,257], 'Existing whole source/fixture owners and coordinator', ['rig-skin-and-mhr','armor-layout-and-design'], 'Withdrawn container candidate remains withdrawn; exact old compile defects get merged-source disposition only; remaining fixture ordinal/LOD/correspondence obligations need final source and existing hosted acceptance.')
group('p16-physical-weapon',[89,140,173,195,202,203,205,206,207,229,230,231,234], 'P16 camera physical catalogue/recipe/evaluation owners with root Holder/derived and native observation owners', ['geometry-shared-leaves','source-version-and-owner-reconciliation','equipment-topology-and-caches'], 'Retain precision-preserving f64 physical construction and resolved component identity through evaluated metadata/holder/icon/contact/equipment and real DB projection; preserve zero/unbounded precision and typed native causes; do not replay stale branch or import human-only mass bounds.')
group('equipment-topology-and-caches',[90,91,94,103,104,192], 'Tactical equipment/inventory ownership/slot/preview/runtime presentation owners', ['rig-skin-and-mhr','p16-physical-weapon'], 'Canonical owned-held topology/address traversal and exact generation feed all cache/HUD/ack/spawn recipients; preserve P15 revisions, reservations and chronology; static drop-hook ordering remains qualified.')
group('animation-config-and-ik',[96,97,98,105,106,107,112,113,114,115,116,186,187,188,189,208,209,210,215], 'Tactical-core animation lifecycle/configuration and client pose/IK owners', ['geometry-shared-leaves','rig-skin-and-mhr','p16-physical-weapon'], 'One admitted config/lifecycle/input/result reaches actual pose publication; preserve periodic unwrapped phases, valid raw input policy, solved-output arithmetic and partial-primary-hand qualifications.')
group('p17-capture-and-diagnostics',[95,99,100,117,118,119,120,121,135,141,212,213,214], 'P17 native report/capture owners, camera motor/timing owner and coordinator', ['animation-config-and-ik','p16-physical-weapon','geometry-shared-leaves'], 'Typed quantities, exact occurrence/stream/frame/time identities and applicable/missing states survive capture/metric/report; configured sole offset and sampling time agree; preserve independent oracle thresholds and genuine inapplicable jitter.')
group('camera-and-view-state',[78,101,108,181,184,216,240,256], 'Camera/controller/view owning modules and compatible shared geometry', ['geometry-shared-leaves','animation-config-and-ik'], 'Explicit view state/derived failure/orientation convergence retains frame and mode meaning to Transform, with zero motion distinct from invalid direction and API-only mutation qualified.')
group('input-and-timing',[102,133,134], 'Canonical typed ScriptCommand/input and authoritative action timeline owners', ['animation-config-and-ik','p16-physical-weapon'], 'Diagnostic labels have no gameplay authority; complete action interval capacity and precision admission precede publication without imposing a probability range on signed trusted precision.')
group('graphics-audio-config',[122,123,124,144,152,155,156,157,158,179,180,183,196,197,233,253], 'Client graphics/audio/configuration and owning native shape/config-error modules', ['geometry-shared-leaves','shared-gpu-resource-contracts'], 'Keep raw config separate from checked complete executable settings and structured causes; prove fade widths/native products and source geometry prerequisites, preserve exact ordinary rendering/audio arithmetic.')
group('city-support-and-ground',[125,126,127,129,130,131,148,149,168,169,178,198,201,222,224,225,226,227,235,243,244,247,248,249,250,254,257], 'Canonical city grounding/support/region/profile owners and affected native recipients', ['geometry-shared-leaves','scene-admitted-products','terrain-source-and-route'], 'Immutable source/support geometry and explicit absence/capacity states survive union/grade/query/diagnostic consumption. F022 intrinsic extreme counterexample is not normal admitted passage proof; prove source bounds before promoting.')
group('scene-admitted-products',[128,145,146,150,151,153,154,159,166,167,170,171,194,223,236,241,242,252,259], 'Canonical generated/prepared scene, worker product, InteriorField and identity owners', ['city-support-and-ground','geometry-shared-leaves','graphics-audio-config'], 'Decode and retain coherent program/plan/collision/mesh/grid/circulation identities and named records before install or semantic consumers; native codec representation alone is not admission.')
group('tree-native-admission',[160,161,162,163,164,165,166,237,238], 'Canonical woody geometry/growth/impostor atlas/provenance owners', ['graphics-audio-config','scene-admitted-products'], 'Finite executable branch/leaf shapes, bounded mask/LOD/atlas and full-word Seed provenance reach actual native output; empty/sparse geometry gets explicit existing policy.')
group('terrain-source-and-route',[136,137,138,177,193,204,228,245,246,251,258,172,199,200], 'Terrain package/SourcePackageDigest/route/landform and dispatcher source owners', ['geometry-shared-leaves','scene-admitted-products'], 'Retain exact source/grid/geographic identity through partial-cell membership, normalized mixtures, wide route addressing, quantized recipe/collar correlation and actual consumers; preserve valid sparse partial terrain chunks.')
group('combat-condition-state',[132,182,185], 'Transient TacticalCombatState/condition source-category/trend owners', ['p16-physical-weapon','input-and-timing'], 'Coherent current impairment observation and signed rates remain distinct; one general fatigue feeds incapacitation once; preserve wheel cutoff and presentation-only projected extensions.')
group('sky-and-cloud-contracts',[139,217,218,219,220,221], 'Core CloudForm/celestial source owners plus explicit client render-frame adapter', ['geometry-shared-leaves','graphics-audio-config','p17-capture-and-diagnostics'], 'Retain CloudForm and geographic quantities; preserve east/up/negative-north native celestial mapping and distinguish absent observations. Cloud wind from/to heading meaning remains unresolved; no blind axis flip.')
group('strategic-observer-and-boundary',[109,110,111,175,176,143], 'Strategic observer frontier/canonical names/placement owners and root targeting owners', ['source-version-and-owner-reconciliation','p16-physical-weapon'], 'Keep durable and transient identity/authority separate and honor actual observer-time frontiers; these support records cross core/strategic partitions and confer no broader source verdict.')
group('semantic-source-identities',[142,147,190,191,207,211,238,255], 'Existing source-specific door, fitting, skill, ragdoll and deterministic identity owners', ['geometry-shared-leaves','equipment-topology-and-caches'], 'Retain the existing exact semantic source identity/quantity to actual numerical or hash ports; no equal-width conflation or wrapper-only completion.')
# All remaining records must still be accounted individually.
assert len(a)==260
for i in range(len(a)):
 if i not in idx_cluster:raise SystemExit('unclustered '+str(i))
# Resolve deliberate connected cluster overlaps by last definition only; this does
# not change defect identity and cluster membership references remain explicit.
variant_pins={'pr-working':'6533e444fa37552927070cc8d8190907fd3b413e plus four preserved integration/quality edits; exact hashes authoritative','physical-p16':'039329518e264f19206cf5b0b98d177433f40c17 preserved lineage; execution ledger also records a1203909 and159 dirty paths, use exact hashes/ranges','report-p17':'90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696','rig-quality':'cbbbca08c89a8768926799371f54cf6057312e5e','main':'3a50b708381ad00c116445c2c999a49c3943088b'}
# Parse original byte evidence without ever executing package scripts.
zip_path='/home/brunosegovia/Downloads/774-audit-for-segovb-20261023.zip'
ids={str(x['record'].get('id',x['record'].get('finding_id',''))) for x in a}
qual=[]; claims={}; more=[]
with zipfile.ZipFile(zip_path) as z:
 for name in z.namelist():
  if ('qualification' in name or name.endswith('finding-supplements.jsonl')) and name.endswith(('.json','.jsonl')):
   try:
    raw=z.read(name).decode(); vals=[json.loads(t) for t in raw.splitlines() if t.strip()] if name.endswith('.jsonl') else [json.loads(raw)]
   except Exception:continue
   for n,v in enumerate(vals):
    txt=json.dumps(v)
    hits=sorted(x for x in ids if x and re.search(r'(?<![A-Za-z0-9])'+re.escape(x)+r'(?![A-Za-z0-9-])',txt))
    if hits:qual.append({'ref':name+('#'+str(n) if name.endswith('.jsonl') else ''),'matched_ids':hits,'record':v})
 for who in ['root','camera_targeting','rig_garments','native_reports']:
  name='audit/records/parallel-ownership/'+who+'-claims.json'; obj=json.loads(z.read(name)); claims[who]={'ref':name,'owner':obj.get('owner'),'claims':obj.get('claims',[]),'transfers_out':obj.get('transfers_out',[]),'limits':'Historic whole-function declarations, must apply later verified human GPU takeover and settle exact connected780/826 split; no unrelated automatic transfer.'}
 for name in ['audit/continuation/rig/initial06-type-ownership/reference-qualification-484.json','audit/continuation/native/initial05-aggregate-states/qualification-426.json','audit/continuation/rig/initial09-enum-shapes/resume-frontier-021.json']:
  more.append({'ref':name,'record':json.loads(z.read(name))})
(OUT/'qualification-evidence.json').write_text(json.dumps({'matched_qualifications':qual,'mandatory_exact_records':more},indent=2)+'\n')
# Source refs can be nested in dictionaries/lists or embedded as strings. Preserve
# original records verbatim; computed hash matches establish byte mapping only.
def walk(o):
 if isinstance(o,dict):
  yield o
  for v in o.values():yield from walk(v)
 elif isinstance(o,list):
  for v in o:yield from walk(v)
def paths(o):
 s=json.dumps(o);out=set()
 for p in re.findall(r'(?:crates|assets|scripts)/[A-Za-z0-9_.\-/]+\.(?:rs|wgsl|js|toml|py)',s):out.add(p)
 return sorted(out)
def variants(o):
 out=set(re.findall(r'\b(pr-working|physical-p16|report-p17|rig-quality|main):',json.dumps(o)))
 if isinstance(o,dict):
  for v in o.get('snapshots',[]):
   if v in variant_pins:out.add(v)
 return sorted(out) or ['record-path-only; exact historical variant unresolved']
hashes={}
def hash_for(p):
 if p not in hashes: hashes[p]=hashlib.sha256((SRC/p).read_bytes()).hexdigest() if (SRC/p).is_file() else None
 return hashes[p]
def observations(o):
 out=[]
 for d in walk(o):
  h=d.get('sha256',d.get('source_hash',d.get('source_hash256')))
  if not isinstance(h,str) or len(h)!=64:continue
  pp=paths({k:v for k,v in d.items() if k in {'source_id','path','source','file','resolved_path','location'}})
  if len(pp)!=1:continue
  p=pp[0];mh=hash_for(p);out.append({'path':p,'recorded_sha256':h,'merged774_sha256':mh,'comparison':'same-bytes' if mh==h else 'different-bytes' if mh else 'missing-at-merged774','credit':'hash identity mapping only; no independent lens/body credit'})
 return out
manual_notes={
3:'Exact M15 immutable realization/cache/source association. Native support312 is same graph; actual normal new/reset paths remain positive.',
13:'P16 counterpart support for Rig03F049; not a new gauge defect. Preserve exact P16 hash, not blanket PR equivalence.',
14:'Whole weight-row proof supplements Rig03F010. Valid ordinals do not establish finite normalized weights.',
18:'ClothingShell refit is an additional admission route; do not equate its late validation with ClothSkin::new or claim current generated shell is malformed.',
20:'Adds one-unique-point hull witness to Rig03F012 empty-support gate; no GPU hang or ordinary MHR body occurrence established.',
22:'QUAL095 explicitly identifies exact Rig03F018 packed20-bit triangle graph; no extra defect.',
23:'Exact F06. Merged774 inventory.rs225-239 independently still contains id.0+1 before MAX; retained preexisting issue, no runtime executed.',
39:'Archive calls this an extension of chart admission; keep ring-pair requirements and Default empty builder separate within complete native topology plan.',
48:'REFERENCE021 narrows Unsigned omission to generic create_bind_groups. Actual KernelBatch fallback ComputePass::record does pack Unsigned99-105. Width/type gap remains across routes.',
67:'REFERENCE484: F004 is duplicate prepared PartRecipe+packed-design product only; it is not GeneratedArmor whole-mesh correlation/atomicity proof.',
69:'Shared generic graph should retain Creator InventoryItemId without conflating durable DB identities. This is not the inventory exhaustion defect.',
70:'ForearmMorphSample location is misplaced for full shared body realization. Joins M17 named associated samples, but moving owner alone does not prove morph count/order/normal correspondence.',
80:'Actual numeric Euler-axis kernel may justify raw indexing; classify named semantic sequence crossing APIs, do not mechanically rewrite tensor axes.',
86:'Withdrawn candidate: EquipmentItemSelection private primitive backing is intentional within admitted container; preserves duplicate catalogue-ID selection. No human waiver and no clearance for other catalogue paths.',
87:'Independent merged774 dressing.rs97 now uses usize::from(*joint), so archive invalid primitive-cast compile defect is absent. Remaining RigJointTable propagation suggestion is distinct and not proven repaired by that conversion.',
88:'Independent merged774 camera/collision/tests.rs108-113 ends standard assert! with );, without archive trailing .unwrap(). Exact historic compiler defect is absent in frozen merged774. No compilation executed; do not carry compiler-defect obligation as current.',
94:'Drop cache defect is separate from local F07 stack mass. Preserve conditional deferred-command/hook ordering and actual remaining-item precondition; no executed stale shield/block assertion.',
95:'Duplicate per-combatant counters is a separate remedy from M09 string-category typing; schedule same report functions once but do not collapse these defect identities.',
96:'Configured grid cache identity extends F01 connected bank operations, but is distinct from F01 swallowed rejection. Both dependencies must pass; no normal repeated-install producer established.',
97:'QUAL204 retains only conditional partial-primary target API witness. No normal target producer established. Orientation observationCON336 is not this finding or toe F064.',
98:'Actual F064 toe equation duplication is its own policy graph. QUAL205 forbids global dedup under unrelated roster/tolerance/digest labels.',
100:'QUAL206/CON348 closes pending fixed64Hz source comparison. Default64 agreement does not remove custom32Hz temporal inconsistency; trajectory normalized progress remains separate.',
112:'QUAL010/011 actual ordinary phase producers wrap; F008 public/unwrapped API counterexample remains, no normal phase2 bug asserted.',
119:'QUAL014 all-ragdoll empty jitter may be intentionally inapplicable; cannot force required observable windows in such runs. Completeness policy must express actual eligibility.',
124:'QUAL022 normal bounded sampling grid and spacing bound adjacent-cell arithmetic; native key capacity API witness is not ordinary scene overflow.',
125:'QUAL024 actual accepted passage source uses coordinates0..completed length; negativeMAX-to-positiveMAX intrinsic witness cannot establish normal accepted passage failure. Need full accepted limits/product witness before stronger claim.',
130:'QUAL028 physical fixed-cell relationship belongs SceneBoundary; arbitrary unrelated valid support mesh is not itself invalid. Admit the actual descriptor→geometry relationship.',
132:'QUAL030 keeps intentional raw partial observation fixtures; QUAL031 normal tick recomputes total; QUAL032 proves normal consequence writer later changes impairment without refresh. Plan coherent observation or bounded phase barrier, not new fatigue pool.',
133:'QUAL034 MAX coordinate overflow algebra independent of64Hz. Do not fix clock capacity by a rate-specific cap.',
136:'QUAL057 valid sparse partial1x1 chunks remain permitted; prove requested local x/y membership, do not force full tile dimensions.',
153:'Retain malformed aperture update ignored semantics; UnitInterval admission must not turn ignored bad input into a new accepted zero measurement.',
173:'Icon implicit/label identity extends the canonical resolved-component repair represented by F40 and ROOT-ADM-F002; no executed MissingFocus claim.',
175:'Observer-frontier disclosure candidate crosses strategic partition. Self/grants and explicit current-fact controls differ; no live player state inspected.',
178:'SupportElevation does not certify support provenance by constructor. Lowest shared Elevation<Scene> proposal retains GateDatum/profile roles and frames.',
182:'Rates have signed fraction/second units; contribution levels and projected increases remain separate. This is not evidence of a second fatigue pool.',
195:'Independently read merged774 holders.rs1-261, catalog.rs100-134, validation.rs37-76, recipe/mod.rs217-222 and placement.rs109-116. Constructor still uses original component.id.expect in both branches after resolved clone construction. This extends F40 shared canonical identity repair. Coordinator supplied minimal no-id war_hammer grip witness; full validation reachability is reported coordinator evidence, not executed here.',
218:'Later duplicate Native ownerF034 record220 qualifies concrete sky frame; use generic framed successful observations and explicit unavailable state. Never import earlier exact Scene claim.',
219:'Later ownerF035 record221 supersedes exact Scene/wind inference. Celestial north reflection proved; wind heading from/to meaning remains an explicit policy/evidence gap.',
220:'Canonical later correction for duplicate Native ownerF034 record218. Absence versus measured black/zero remains a real distinct contract beyond local M63 raw-unit retention.',
221:'Canonical later correction for duplicate Native ownerF035 record219; do not alter wind vectors by inferred geographical convention.',
232:'Exact duplicate historic assert/unit compiler defect A088; merged774 source proves trailingunwrap absent. No test compilation/clearance inferred.',
239:'Independently merged774 anime_tests.rs173-214 still reads native u32 joint and indexes RigJointTable with usize. Preserve existing assertions and typed-ordinal/table contract; source-only compile mismatch, no build run.',
256:'Exact local F02 derived camera direction error collapsed into clear sweep; zero direction and nonfinite subtraction/norm outcomes must remain distinct.'}
# At least one human-authored defect key remains per semantic remedy; aliases never
# create a global count. New candidates still need current-source proof.
rows=[]
for i,x in enumerate(a):
 r=x['record'];rid=r.get('id',r.get('finding_id',x['key']));cv=canon.get(i,i);v=variants(r)
 disp='alias' if i in alias else 'same' if i in match else 'new-candidate'
 if i in match and i not in {3,19,23,81,153,183,190,191,192,194,196,204,217,236,253,256}:disp='extends'
 if i in {39,50,74,18,20,66,70}:disp='extends'
 if i in {80,86,87,88,125,175,178,180,181,184,218,219,232}:disp='qualified'
 if i in {109,110,111,143,144,145,146,147,148,149,150,151,152}:disp='extends'
 cluster=idx_cluster[i];cl=clusters[cluster]
 qual_matches=[q['ref'] for q in qual if rid in q['matched_ids']]
 meaningful_keys=['connected_repair','connected_remedy','connected_remedy_proposal','smallest_connected_repair','exact_repair','repair','proposed_connected_repair','connected_repair_acceptance','remedy']
 repairs={k:r[k] for k in meaningful_keys if k in r}
 ac={k:r[k] for k in ['acceptance','acceptance_criterion','connected_repair_acceptance','repair_acceptance'] if k in r}
 deps={k:r[k] for k in ['dependencies','dependency','repair_and_acceptance_dependencies','shared_dependencies','family_requirements','shared_functions_owners','ownership','claim_authority','repair_owner','owner','limits','coverage_limit','initial_coverage_limit_history'] if k in r}
 row={'assignment_index':i,'id':rid,'key':x['key'],'archive_file':x['archive_file'],'record_index':x['record_index'],'source_variant':{'labels':v,'pinned_context':{s:variant_pins.get(s,'unresolved exact variant; retain original references') for s in v},'primary_source':x['primary_source'],'merged774_mapping':observations(r)},'local_ids':match.get(i,[]),'local_related_ids':related.get(i,[]),'disposition':disp,'canonical_defect_key':a[cv]['key'],'repair_cluster':cluster,'evidence_refs':[x['key']],'qualification_refs':qual_matches+['HANDOVER.md','STATE.json'],'reconciliation':manual_notes.get(i,'Reported source review retained with its exact source variants, routes, limits, native policy and recipient evidence. No current-source repair or clearance inferred; this is an actionable reconciliation candidate, not executed validation.'),'reported_record':r,'repair':repairs,'dependencies':{'source_specific':deps,'connected_cluster':cl['dependencies'],'ownership':cl['canonical_owner']},'acceptance':{'source_specific':ac,'connected_cluster':cl['acceptance'],'current_state':'unimplemented/unvalidated in this read-only reconciliation; inspect changed native/source recipients before concluding any source-specific obligation is satisfied'},'crossdependencies':[]}
 if any(('stdb' in p or 'strategic-web' in p) for p in paths(r)):row['crossdependencies'].append('core/strategic partition: observer authority, durable schema/cause/identity and generated-client owner; tactical state remains transient')
 if any(p.startswith('crates/fabelgeist-geometry/') or p.startswith('crates/fabelgeist-math/') for p in paths(r)):row['crossdependencies'].append('shared geometry/math owner reconciliation in other partition; no competing equivalent types')
 if cluster in {'armor-layout-and-design','shared-gpu-resource-contracts','hair-executable-shapes','creator-realization-and-cache','rig-skin-and-mhr'}:row['crossdependencies'].append('connected GPU780/826 coordinated ownership authorized per coordinator user-message verification; exact whole-function split/final signatures and combined-source proof still pending')
 if cluster in {'p16-physical-weapon','p17-capture-and-diagnostics','animation-config-and-ik'}:row['crossdependencies'].append('P16/P17 complete motor/optional tick, canonical physical leaves, actual DB/report recipients and current P15 policy; no whole stale branch replay')
 rows.append(row)
account=[]
for l in local:
 ma=[r['key'] for r in rows if l['id'] in r['local_ids']];ra=[r['key'] for r in rows if l['id'] in r['local_related_ids']]
 account.append({'id':l['id'],'title':l.get('title'),'status':'matched-or-extended' if ma else 'unique-unmatched-within-engine-assignment','matches':ma,'related_only_not_deduplicated':ra,'evidence':l,'qualification':'No match among these260 archive records does not prove absence from other partitions or unread/unstarted lenses. Retain local finding and original acceptance.'})
result={'schema':'bespoke-reconciliation-engine-v1','scope':'260 assigned archive records and66 local findings; no final unique global count','snapshot':{'local_merged774':'13fc9dcc71ab879de364117e5462310673e43210','local_base':'3a50b708381ad00c116445c2c999a49c3943088b','archive_source_pins':variant_pins,'source_root':str(SRC.resolve()),'reported_successors':'#8261f7ecbe13; later#780 combined6f6d77878 includes8261f7 per coordinator report; not independently implementation-audited here'},'status':{'discovery':'incomplete across13 lenses; this partition reconciles preserved records only','repair':'not authorized in current task; no source edits','validation':'no builds/tests/checks/CI/UI/native execution; byte/hash/source inspection and ignored bookkeeping only'},'ownership':{'human_authority':'Coordinator verified actual human messages authorizing coordinated780/826 takeover of connected774GPU functions. Exact whole-function split remains to agree; no permanent missing-implementer blocker.','historical_claims':'claims-evidence.json retains exact historic per-agent declarations; do not override with generated ownership-map or archive prose. No automatic transfer of unrelated P16/P17/terrain/strategic functions.'},'rows':rows,'local_accounting':account,'repair_clusters':clusters,'qualification_evidence':'qualification-evidence.json','counts_for_accounting_only':{'archive_records':len(rows),'unique_archive_keys':len({r['key'] for r in rows}),'local_records':len(account),'local_with_exact_or_extension_match':sum(bool(x['matches']) for x in account),'dispositions':dict(collections.Counter(r['disposition'] for r in rows))},'qualification_rule':'Every full reported_record is retained. Qualifier references/supplements and later corrections supersede only their exact claims, never whole-lens coverage. Hash match alone is no source review; same file/title or type conversion count is no deduplication/clearance.'}
(OUT/'crosswalk.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
# Only exact source-matching claims, not generated ownership summaries.
allpaths=set(p for x in a for p in paths(x['record']))
selected={}
for who,c in claims.items():
 selected[who]={**c,'claims':[x for x in c['claims'] if set(paths(x)) & allpaths]}
(OUT/'claims-evidence.json').write_text(json.dumps(selected,indent=2,ensure_ascii=False)+'\n')
# Structural accounting only, not a project check or test.
assert len(rows)==len(a) and len({r['key'] for r in rows})==len(a)
assert {r['key'] for r in rows}=={r['key'] for r in a}
assert {r['id'] for r in account}==set(loc)
print(json.dumps(result['counts_for_accounting_only'],indent=2));print('qualification records',len(qual),'files written to',OUT)
