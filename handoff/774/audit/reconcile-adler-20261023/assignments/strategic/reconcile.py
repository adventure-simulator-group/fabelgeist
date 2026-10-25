import json, pathlib, re, hashlib, collections, zipfile
R=pathlib.Path('target/bespoke-audit-774-merged-20261022/reconcile-adler-20261023')
O=R/'assignments/strategic'; S=R.parent/'source'
a=json.loads((O/'adler-input.json').read_text()); local=json.loads((O/'local-input.json').read_text()); L={x['id']:x for x in local}
# Explicitly reviewed defect/remedy links. A link means connected scope, not identity.
links={
1:'M01',2:'M25 M26',3:'F12 M01',5:'M46',6:'M01',7:'M01',8:'M01',9:'M01',10:'M38',11:'M58',12:'F12',14:'M50',17:'M27 M29',18:'M50',19:'M29',21:'M29',22:'M31',23:'M01',25:'M31',27:'M41',28:'M41',29:'M42',30:'M39',32:'F15',34:'M24',35:'M01',36:'M01',40:'M33',45:'M01',48:'M37',49:'M74',54:'M41 M46',57:'M37',58:'M36 M37',59:'M50',60:'M43',61:'M24',63:'M27 M29',64:'M29',65:'M44',67:'M46',68:'M46',69:'M37',72:'M46',73:'M46',74:'M43',75:'F23 M53',242:'M31',250:'M01',251:'F19 M43',255:'M52',256:'M40',257:'M37',258:'M31',261:'M30',265:'M51',266:'M51',267:'M51',269:'M47',278:'F04 M01',280:'F05 M30',282:'M31',283:'M31',284:'M27 M33 M48 M50',287:'M30',288:'M37',310:'M26',311:'M55',318:'M29',319:'F19 M43',320:'M38',322:'M31',323:'M01',324:'M01',325:'M38',326:'M50',329:'M24',330:'M23 M53',331:'M55',332:'M21',333:'M01',334:'M74',335:'M74',336:'M51',340:'M35',341:'M51',345:'M55',346:'M48',351:'M37',358:'M01',359:'M01',360:'M01',361:'M51 M52',362:'M27',364:'M37',365:'M01',368:'M37',370:'M38',371:'F04',372:'M01',373:'M01',374:'M01',375:'M01',376:'M01',377:'M48',378:'M29',379:'M01',380:'M01',381:'M01',382:'M51',383:'F13 M26',384:'M26',385:'M01',387:'M01',388:'M01',389:'M35',390:'M53',397:'M01',398:'M53',399:'M46',400:'M01',401:'F15',402:'M27 M29',403:'M81 M01',404:'M37',405:'M31',406:'M01',407:'M01',408:'F05',409:'M51',410:'M55',412:'M23 M53',413:'M01',414:'M01',415:'F05 M30',416:'F05',417:'M38',418:'M37',419:'M58',420:'M42',422:'M31',423:'M29',425:'M48',426:'M30',427:'M30',428:'M44',431:'M01',432:'M01',437:'M24'}
# Supporting supplier rows, grouped by the actual owner-fit comparison in each row.
supportgroups=[(range(76,81),'M01 F05'),(range(81,87),'M01 M26 M57'),([87,187],'M01 M56 F05'),([88,89,188],'M51 M57'),([90,91,92,189],''),([93,94,95,190],'M31'),(range(98,102),'M27 M29'),([102], 'M29'),([103,108,109,192,197,198],'M25 M29 M31'),([104,106,110,193,195,199],'M28 M29'),([105,107,111,112,113,114,194,196,200,201,202,203],'M29 M30'),([115,116,204,205],''),([117,118],'M29'),([119,120,221,241],'M30 M58'),([121,222],''),([122,123],'M35 M51'),([124],'M01'),([125,126],'M53'),([127,128,129,130,224,225,226,227],'M31'),([131,132,133,228],'M57'),([134,135],'M26 M30'),(range(136,144),'M35'),([144],'M42'),([145,146,147],'M51'),([148,149,150,151,152],'M48 M50'),([155,230],''),([156,174,231],'F19 M43'),([157],'M40'),([158],'M55'),([159,160,161,162,163,234,235,236,238,239],'M43 M48'),([164],'M38'),([165,240],''),([166,167],''),([168],'M33'),([169],'M58 M30'),([170,171,172],'M42 M58'),([173,175],'M58'),([176],'M43'),([177,178],'M36 M37'),([179,180,181,182],'M01'),([183],'M55 M35'),([184,185,186],'M01 M26 M57'),([191,206,220,223],'M28 M30'),([207,208,209,212,213,216],'M43 F19'),([210,214],'M43'),([211,217,218,219],'M49'),([215],'M43 F19'),([229],'M35'),([232],'F05'),([233],'M44'),([237],'M42 M53')]
for indices,ids in supportgroups:
 for i in indices: links[i]=ids
aliases={14:59,20:62,21:318,22:242,61:15,63:17,64:19,72:67,73:68,245:347,250:9,258:38,263:62,271:291,277:51,283:242,325:320,337:55,349:13,352:244,355:247,373:6,374:8}
# Exact overlaps, supported by identical failed contract and required correction.
same={27,59,319,320,326,331,332,371,383,384,418,419,420,428}
# Specific corroborated current source slices; these do not establish lens coverage.
read_ranges={
'crates/adventuresim-stdb-module/src/time/advancement.rs':[(1,145),(440,582)],
'crates/adventuresim-stdb-module/src/time/activities.rs':[(270,350)],
'crates/adventuresim-stdb-module/src/time/schedule.rs':[(228,244),(312,343)],
'crates/adventuresim-core/src/autoresolve/melee_round/scheduling.rs':[(332,391)],
'crates/adventuresim-core/src/autoresolve/joint_melee.rs':[(209,270)],
'crates/adventuresim-core/src/smithing.rs':[(1,50)],
'crates/adventuresim-world-schema/src/basis_points.rs':[(1,38),(94,123)],
'crates/adventuresim-core/src/item_catalog_validation.rs':[(526,544),(682,708)],
'crates/adventuresim-stdb-module/src/investigation/projections.rs':[(776,820),(913,938)],
'crates/adventuresim-stdb-module/src/food/consumption.rs':[(79,96)],
'crates/adventuresim-core/src/world_event.rs':[(46,63),(240,260)],
'crates/adventuresim-stdb-module/src/world_event.rs':[(941,999)],
'crates/adventuresim-core/src/autoresolve/melee_defense.rs':[(180,195)],
'crates/adventuresim-core/src/autoresolve.rs':[(530,560),(1110,1143)],
'crates/adventuresim-core/src/combat.rs':[(465,518)],
'crates/adventuresim-core/src/autoresolve/model.rs':[(110,128)],
'crates/adventuresim-stdb-module/src/strategic/authority_model.rs':[(35,62)]}
current={387:['crates/adventuresim-core/src/autoresolve.rs','crates/adventuresim-core/src/combat.rs','crates/adventuresim-core/src/autoresolve/model.rs'],0:['crates/adventuresim-world-schema/src/basis_points.rs'],2:['crates/adventuresim-core/src/item_catalog_validation.rs'],3:['crates/adventuresim-core/src/item_catalog_validation.rs'],22:['crates/adventuresim-stdb-module/src/food/consumption.rs','crates/adventuresim-core/src/world_event.rs','crates/adventuresim-stdb-module/src/world_event.rs'],242:['crates/adventuresim-stdb-module/src/food/consumption.rs','crates/adventuresim-core/src/world_event.rs','crates/adventuresim-stdb-module/src/world_event.rs'],251:['crates/adventuresim-stdb-module/src/investigation/projections.rs','crates/adventuresim-stdb-module/src/strategic/authority_model.rs'],262:['crates/adventuresim-stdb-module/src/time/advancement.rs'],265:['crates/adventuresim-stdb-module/src/time/activities.rs','crates/adventuresim-stdb-module/src/time/schedule.rs'],278:['crates/adventuresim-core/src/autoresolve/melee_round/scheduling.rs','crates/adventuresim-core/src/autoresolve/joint_melee.rs'],319:['crates/adventuresim-stdb-module/src/investigation/projections.rs'],371:['crates/adventuresim-core/src/autoresolve/melee_defense.rs'],403:['crates/adventuresim-core/src/smithing.rs']}
clusters={
'S01':{'name':'Canonical leaves and complete admission','remedy':'Choose the existing lowest shared owner; seal all safe construction, mutation, Serde and enabled SATS routes. Keep raw rejected DTOs separate.','dependencies':[],'acceptance':'No admitted owner can bypass its actual finite/range/grammar/relation predicate; zero, signed, full-width and namespace policies preserved.'},
'S02':{'name':'P16 physical combat, occurrence and P17 receipts','remedy':'Reconcile preserved P16 graph with current P15; retain checked physical leaves, admitted result aggregates and one consumed attack occurrence through equipment, scheduling, contact, health and reports.','dependencies':['S01','S03','S04','crosspartition: tactical/geometry/report owners'],'acceptance':'Full-width occurrence identity; completed attack consumed once; applied cut/blunt channels agree with recording; native extraction only at final kernels/SDK ports; single visible fatigue pool and wheel semantics retained.'},
'S03':{'name':'Source catalogue products and diagnostics','remedy':'Put shared vocabulary below build/runtime consumers; admit the actual target representation and a coherent compiled product; retain typed paths/spans and concrete errors to compiler/UI output.','dependencies':['S01','crosspartition: shared Skill/authoring owners'],'acceptance':'Build and runtime accept the same completed schema including optional fields and kind-required capabilities; no f64-to-f32 guarantee assumption; deterministic first errors/source positions preserved.'},
'S04':{'name':'Material, food, inventory, trade and forge conservation','remedy':'Carry existing material/volume/mass/fraction/custody owners in associated records and coherent stock quotations through planning, mutation and persistence. Preserve concrete causes and preflight all arithmetic.','dependencies':['S01','S02','S03'],'acceptance':'Exact mass/volume/rounding and ownership conservation; millionths distinct from BPS and aggregate counts; source identity bound to actual inventory object; no saturation presented as checked payable/required quantity.'},
'S05':{'name':'Generated authority, custody and observer knowledge','remedy':'Retain opaque validated generation product; separate private/public case identities and immutable blueprint from refreshed custody expectation; type action targets, testimony pipeline and observer-specific knowledge states.','dependencies':['S01','S03','S08','S09'],'acceptance':'All view/action/retry paths share validated provenance; no manual fallback on invalid generated authority; canonical evidence joins; actor personal time used where execution does; sharing never grants firsthand visit.'},
'S06':{'name':'Strategic time, training, weather and interval settlement','remedy':'Distinguish absolute clocks, actual settled duration, normalized schedule allocation, study hours and interval applicability; unify complete post-clock settlement across planned/plain paths.','dependencies':['S01','S03','S08'],'acceptance':'Unavailable clock remains distinct from actual epoch0; planned and ordinary paths settle exposure; ambient training uses actual elapsed time; finite stored hours and explicit unbounded ceilings; checked exhaustion and interval metadata.'},
'S07':{'name':'Social, personality, dialogue and inference','remedy':'Retain canonical personality/action/fact owners, payload-bearing receipts and observer-scoped availability; derive prose only at presentation; normalize scores with sufficiently wide arithmetic.','dependencies':['S01','S03','S05','S06'],'acceptance':'Truth axes and payloads remain correlated; signed signals/check scales are not forced into UnitInterval; revisions checked before writes; one source of semantic labels; cause and privacy preserved.'},
'S08':{'name':'World events, places, routes and network planning','remedy':'Retain canonical event/place/disease and geographic versus abstract coordinate alternatives through planners; unify canonical persistence model; own coherent route request/result/span applicability.','dependencies':['S01','S03','crosspartition: world-import/native geometry'],'acceptance':'No early stringification/re-admission; WGS84 bounds only for geographic branch; exact integer distance and zero elapsed supported; route cache validates before hit; no stale terrain relabeled baseline.'},
'S09':{'name':'Lifecycle, organizations, housing and browser authority','remedy':'Use explicit frozen heir, birth preparation, organization membership, unique election and browser provenance products; retain precise identity authority and full-width browser transport.','dependencies':['S01','S03','S06'],'acceptance':'Atomic required preflight before lifecycle side effects; no zero/MAX state sentinels internally; actual kinship source respected; positive payment differs zero balance; verified owner/grant authority distinct from bare character identity.'},
'S10':{'name':'Building accepted geometry and prepared facade','remedy':'Separate raw authoring from immutable accepted graph/product; use checked IDs/extents/counts and rebuild correlated supports/geometry atomically; retain framed placement leaves through worker/cache ports.','dependencies':['S01','crosspartition: geometry/building/native render owners'],'acceptance':'Unique connected IDs; coherent roof/aperture/stair/furniture bounds and supports; checked candidate arithmetic before packed cells; legitimate zero extents and signed geometry preserved; no epsilon/compatibility fallback.'},
'S11':{'name':'Foreign/native resource and tool boundary coordination','remedy':'Reconcile exact whole-function ownership with GPU/rig/native partitions; admit coherent resource/count/layout/provenance and parsing progress; retain original native causes through thread/SQL/CLI output.','dependencies':['S01','crosspartition: GPU/rig/native/tools owners'],'acceptance':'No claim of foreign graph coverage from declaration/import; explicit byte/count/timestep/parse progress boundaries; no error-to-string-to-error reconstruction before terminal output; shared #779/#780/#826 owners coordinated.'}}
# Cluster selection is routing only; never used as defect deduplication.
def cluster(i,d):
 t=json.dumps(d,ensure_ascii=False).lower();p=a[i].get('primary_source','').lower()
 if i in [262,265,266,267,279,281,336,337,361,363,364,366,367,369,394,409,411,429,433,434]:return 'S06'
 if i in [251,288,319,428] or any(k in p for k in ['investigation','quest_generation','case.rs','custody','testimony']):return 'S05'
 if i in [22,242,283,322,351,368,404,418] or any(k in p for k in ['world_event','travel','journey','network','route','world-import']):return 'S08'
 if any(k in p for k in ['building-generator','building-kit','furniture','strategic-map']) or i in [243,244,245,246,247,248,270,271,272,274,275,276,277,291,294,295,334,335,347,352,353,354,355]:return 'S10'
 if i in [16,37,41,252,253,256,331,340,343,389,390,391,398,410,419] or any(k in p for k in ['browser','organization','residence','family','pregnancy','inheritance','settlement_population']):return 'S09'
 if i in [0,57,58,65,66,67,68,70,71,72,73,74,348,349]:return 'S01'
 if i in [2,3,5,12,15,24,28,29,30,33,34,54,61,75,310,316,329,330,412,437] or any(k in p for k in ['catalog','bestiary']):return 'S03'
 if i in [17,19,21,55,63,64,119,120,261,287,318,362,378,401,402,403,423,426,427] or any(k in p for k in ['food','material.rs','inventory','smithing','alcohol','cooking']):return 'S04'
 if i in [18,27,44,59,254,269,284,326,346,377,399,425] or any(k in p for k in ['social','personality','dialogue','deduction']):return 'S07'
 if i in [1,4,6,7,8,9,10,23,35,36,45,250,278,323,324,333,356,358,359,360,365,370,371,372,373,374,375,376,379,380,381,382,383,384,385,386,387,388,392,393,395,396,397,400,406,407,413,414,415,416,417,431,432] or any(k in p for k in ['combat','autoresolve','equipment','limb','skill.rs','capability']):return 'S02'
 return 'S11'
def flatstr(v):
 if isinstance(v,str):return v
 return json.dumps(v,ensure_ascii=False)
def fields(d,patterns):
 return {k:v for k,v in d.items() if any(p in k.lower() for p in patterns)}
def identity(d):return d.get('id',d.get('finding_id','record-without-top-level-id'))
def summarize(d):
 keys=['earliest_failed_contract','earliest_unfinished_producer','actual_bypass','bypass','finding','issue','source_contract','operation','actual_path','scenario','case','representable_counterexample','conceptual_counterexample','counterexample','proof','material_owner_comparison','relationship','title','classification','kind']
 return ' | '.join(flatstr(d[k]) for k in keys if k in d)
def remedy(d,c):
 keys=['connected_remedy','earliest_connected_repair','connected_repair','smallest_connected_repair','exact_repair','repair_acceptance','connected_repair_acceptance','repair','acceptance','acceptance_ref']
 return {k:d[k] for k in keys if k in d} or {'planning_requirement':clusters[c]['remedy'],'limit':'Archive record provides no standalone concrete remedy; consume cited contract before choosing final API.'}
entries=[]
for i,x in enumerate(a):
 d=x['record'].get('supplier_finding',x['record']); c=cluster(i,d); ids=links.get(i,'').split(); support=d.get('kind')=='existing-owner-lens-support'
 disp=['qualified'] if support or i in [54,279,280,281,282,284,286,404,405,406,407,409,410,411,412] else ['same'] if i in same else ['extends'] if ids else ['new-candidate']
 if i in aliases:disp=['alias']
 if i in [310,316]:disp=['qualified']
 if i not in current and not support:disp.append('needs-evidence')
 if support:rationale='Supplier owner-fit support, not a standalone new defect. Its concrete semantic fit/limits below qualify the connected producer graph; PR/P16 roles remain separate. Linked local IDs indicate matching required owner/propagation work, not unique-defect identity.'
 elif i==310:rationale='Withdrawn by RIG-ADMISSION03-F086-RETRACTION-001: every Some override selects Quality formatting before the bounded-label match. Do not count the original panic claim or require its proposed fix.'
 elif i==316:rationale='Authoritative correction of the previous archive claim; preserves raw presentation-input qualification only. No repair obligation follows from withdrawn F086.'
 elif i in aliases:rationale='Same supplied failed contract/remedy as the referenced archive entry; preserve this exact sourcevariant, independent lens support and qualifications rather than count a second defect.'
 elif i in same:rationale='Same failed contract and connected correction as the linked local finding(s), established by producer/retained state/recipient semantics in both supplied records, not title/path similarity.'
 elif ids:rationale='Extends the linked local connected repair with the distinct producer/admission/operation/aggregate/recipient evidence below. This is not a statement that every observation is the same defect; no unique count inferred.'
 else:rationale='No defect/remedy-equivalent assigned local finding established. Retain as candidate with its exact supplied source proof and limits; source comparison at final head and global deduplication remain required.'
 q=fields(d,['qualif','limit','pending','counterexample','validation','scope','dedup','correction','retract','authority','historical','policy','current_runtime','bad_input','actual_dependency','witness','boundary','boundaries','later_lens'])
 raw=json.dumps(d,ensure_ascii=False); variants=sorted(set(re.findall(r'\b(pr-working|physical-p16|physical-p17|rig-quality|native-dependency|base-main|base-pr)(?=:|/)',raw)))
 if not variants:variants=sorted(set(v for v in d.get('snapshots',[]) if isinstance(v,str)))
 evidence=fields(d,['source','location','range','sha','proof','contract','trace','evidence','coverage_ref','use_site','definition'])
 lens=[]
 f=x['archive_file']
 for token,name in [('producer','producer-propagation'),('owner-fit','existing-type-owners'),('supplier-owner','existing-type-owners'),('existing-type','existing-type-owners'),('admission','admission-contracts'),('initial04','invariant-operations'),('initial05','aggregate-states'),('initial06','type-ownership'),('initial07','redundant-authority'),('initial09','enum-shapes')]:
  if token in f and name not in lens:lens.append(name)
 if d.get('lens'):lens.append(d['lens'].removeprefix('audit-'))
 if not lens:lens=['source-specific support; see original record']
 ent={'assignment_index':i,'id':identity(d),'key':x['key'],'sourcevariant':variants or ['not-explicit; inspect original source evidence'],'local_ids':ids,'local_link_role':'same failed contract/remedy' if i in same else 'connected planning dependency/support; not proven same defect','dispositions':disp,'actual_defect_or_support':summarize(d),'rationale':rationale,'lens_support':sorted(set(lens)),'connected_repair_cluster':c,'dependencies':clusters[c]['dependencies'],'remedy':remedy(d,c),'acceptance':d.get('acceptance',d.get('repair_acceptance',d.get('connected_repair_acceptance',clusters[c]['acceptance']))),'evidence':evidence,'qualifications':q,'current_proof':'Direct frozen merged source slices personally read; no runtime/compiler proof.' if i in current else 'Supplied archive/static proof only; no independent current-source clearance.','current_source_evidence':current.get(i,[]),'discovery_status':'record reconciled for planning; whole lens incomplete','repair_status':'not performed or authorized by this task','validation_status':'not run','original_record':x['record']}
 if i in aliases:ent['alias_of_key']=a[aliases[i]]['key']
 entries.append(ent)
local_account=[]
for x in local:
 matches=[e for e in entries if x['id'] in e['local_ids']]
 substantive=[e for e in matches if e['dispositions'][0] not in ['qualified','alias']]
 local_account.append({'id':x['id'],'title':x['title'],'status':'connected archive matches; no clearance inferred' if substantive else 'support-only; local obligation remains independently open' if matches else 'local-only/unmatched in this assigned archive partition; preserve independent obligation','archive_keys':[e['key'] for e in matches],'substantive_archive_keys':[e['key'] for e in substantive],'contract_trace':x.get('contract_trace'),'repair':x.get('repair'),'acceptance':x.get('acceptance'),'lenses':x.get('lenses',[]),'original_local_record':x})
suppnames=['audit/continuation/native/initial05-aggregate-states/qualification-426.json','audit/continuation/rig/initial06-type-ownership/reference-qualification-484.json','audit/continuation/rig/admission03/catalogue-schema-receiver-witness-001.json','audit/continuation/native/initial05-aggregate-states/contract-groups-400.json']
supp=[]
with zipfile.ZipFile('/home/brunosegovia/Downloads/774-audit-for-segovb-20261023.zip') as z:
 for name in suppnames:
  b=z.read(name);supp.append({'archive_path':name,'sha256':hashlib.sha256(b).hexdigest(),'record':json.loads(b)})
(O/'supplements-consumed.json').write_text(json.dumps(supp,indent=2,ensure_ascii=False)+'\n')
source_evidence=[]
for p,ranges in read_ranges.items():
 b=(S/p).read_bytes();lines=b.decode().splitlines();source_evidence.append({'path':p,'sha256':hashlib.sha256(b).hexdigest(),'revision':'13fc9dcc71ab879de364117e5462310673e43210','ranges':ranges,'slices':[{'range':[lo,hi],'lines':[{'line':i,'text':lines[i-1]} for i in range(lo,min(hi,len(lines))+1)]} for lo,hi in ranges]})
(O/'current-source-evidence.json').write_text(json.dumps(source_evidence,indent=2,ensure_ascii=False)+'\n')
res={'scope':'Read-only strategic reconciliation; 439 assigned archive records and 58 assigned local findings. Records are not unique defects.','pins':{'current_frozen_merge':'13fc9dcc71ab879de364117e5462310673e43210','base':'3a50b708381ad00c116445c2c999a49c3943088b','archive_pr':'6533e444fa37552927070cc8d8190907fd3b413e','archive_p16':'039329518e264f19206cf5b0b98d177433f40c17','archive_p17':'90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696','archive_rig':'cbbbca08c89a8768926799371f54cf6057312e5e'},'coverage_limits':['Crosswalk is record reconciliation, not complete fresh discovery for any lens.','Source variants and mandatory overlays in HANDOVER/STATE remain outstanding.','Current evidence lists exact personally consumed slices only; all other detailed evidence is supplied/static.','No repairs, checks, tests, builds, CI, pushes or external messages.','Archive ownership and execution instructions are historical evidence, not present user authorization.','No final clearance or authoritative globally unique defect count.'],'repair_clusters':clusters,'archive_crosswalk':entries,'local_accounting':local_account,'bookkeeping':{'assigned_archive_records':len(a),'crosswalk_records':len(entries),'unique_archive_keys':len({e['key'] for e in entries}),'assigned_local_findings':len(local),'local_accounting_records':len(local_account),'disposition_records':dict(collections.Counter(v for e in entries for v in e['dispositions'])),'local_only_ids':[e['id'] for e in local_account if e['status'].startswith('local-only')]}}
(O/'crosswalk.json').write_text(json.dumps(res,indent=2,ensure_ascii=False)+'\n')
print(json.dumps(res['bookkeeping'],indent=2))
