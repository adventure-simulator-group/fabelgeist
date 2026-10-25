import json,hashlib,datetime,sys
from pathlib import Path
sys.stdout.reconfigure(encoding='utf8')
o=Path('target/774-continuation-20261021/continuation/native/admission03'); j=lambda p:json.loads(p.read_text(encoding='utf-8-sig')); regs=[json.loads(l) for l in (o/'source-range-registration.jsonl').read_text(encoding='utf8').splitlines()]
refs={}
def bind(nums,ref):
 for n in nums:refs[n]=ref
bind([1,2,3],'contract-groups-001.json; contract-groups-002.json#reflection-remote')
bind([4,5],'contract-groups-016.json#scene-support-direct-definitions; support-qualification-053.json')
bind([6],'contract-groups-020.json#strategic-minute-supporting-definition')
bind([7,8],'contract-groups-021.json#bounded-native-direction-admission')
bind([9],'contract-groups-028.json#scene-viewer-runtime-install-and-raw-evidence')
bind([10,11],'contract-groups-030.json#cloud-source-native-lattice-and-gpu-uniforms; continuation/root/admission03/determinism-admission-023.json')
bind([12],'contract-groups-029.json#debris-real-roots-and-completed-camera-target-admission')
bind([13],'contract-groups-038.json#actual-sdk-reducer-view-argument-and-error-admission')
bind(range(14,20),'contract-groups-043.json#p16-basispoints-all-adapter-routes; p16-basispoints-admission-witness-042.json')
bind([20],'contract-groups-045.json#activity-clipped-interval-and-clock-advance-admission')
bind([21],'contract-groups-047.json#native-network-physics-codec-boundaries')
bind(range(22,27),'contract-groups-049.json#p16-time-counterpart-admission-current-role-comparison')
bind(range(27,31),'contract-groups-052.json#server-clock-and-supported-scene-admission')
bind([31,32],'contract-groups-054.json#ai-ranged-and-tactics-current-receiver-admission')
bind([33],'contract-groups-057.json#threat-escalation-raw-profile-and-distinct-scale-admission')
bind(range(34,46),'contract-groups-059.json#p17-capture-selected-observation-admission')
bind(range(46,54),'contract-groups-067.json#narrow-current-bot-leaf-dependencies')
bind([54],'contract-groups-069.json#p16-actual-context-admission')
bind([55,56],'contract-groups-072.json')
bind([57],'contract-groups-075.json#exact-model-dependency-contracts-and-two-alternate-route-gaps')
notes={
 1:'Canonical elapsed finite>=0, zero/default/constants/nativeDuration legitimate; manualcheckedSerde. No provenance/nativeDuration representability promise for everyf32.',
 2:'Canonical linear speed finite declaredsigned/nonnegative policy andmanualSerde; source-role usage only, no nativekernel/allphysics certificate.',
 3:'Canonical UnitInterval finiteclosed0..1/manualcheckedSerde/privatebacking/default0, opaque remote copies existingcanonicalSelf.',
 5:'Only actual SupportElevation73–109 independentadmission role;1–120 requestedcontext retained. SurfaceHit reexport is not definition; laterREG030 actual1–23 suppliesit.',
 6:'StrategicMinute actualallwordu64/private/default0/transparentSerde; CalendarYear/incidentalcontext andtests411+excluded.',
 7:'Exactnative constructors/error/normhelper only. Native checkednew lengthfinite>0, uncheckeddebugprecondition; unrelatedoperators/from_angle/tests excluded despitecontext618–660.',
 8:'ActualDir2new_and_length177–184 completesconstructor,205signaturecontextonly.',
 9:'ActualSettings/default/pluginclone/install exact197–352; rawconfigoverride notautomaticallyrevalidated, RootgraphicsF006support. Othermodules/shadertests excluded.',
 10:'RegistrationprecededRootfullSeed current03 supplier; no Native speculativedupgraph. ActualRoot023 allwordSeed/private/default/Serde/currentRNG comparedtocurrentcloudroles.',
 11:'ActualRoot023 RNGprivateSplitMix/Seedconstructor/inclusive route; registeredranges only supportingcontract, no nativeallalgorithm certificate.',
 12:'PublicPair rawcorrelation allowed; actualnearest receiver filtersmaxdistance; no validatedspatialhistorypromise.',
 13:'Directnative SerDeArgs tuplevisitor/with_read_args only; concreteSATSdecoderpolicy authoritative, no Serdeattributevalidation assumed.',
 19:'Singletraitmethod declaration dependencyqualification only, noimpl/genericSATSgraphcredit.',
 20:'InjuryRecoveryMinutes privateu64 NONE0/infalliblenewallu64/get, noSerdeDefaultReflect or strictpositive policy.',
 21:'Directactual RuleFns codec/in-place/consume route andnativeFrom adapter only; no blanket firstpartybody/foreignwholefile waiver.',
 22:'ExactP16 counterpart independentlycompared: allwordrecoveryquantityincl0/MAX.',
 23:'OpenoperationString FromString/&str legitimatefinalreturnboundary plusstructuredleafvariants, notacceptedvaluevalidator.',
 24:'P16fullallocation364actualread; publicScheduledzero/blank andDailyPlanoverflow supportsRootF005, privateValidatedSchedule gatedintact.',
 25:'ExactACTIVITY_SEGMENT_MINUTES15constantonly.',
 26:'P16StrategicMinute supportingallwordu64+methods/constant; actualPRbody+fulltextdelta/currentrolecompare, incidentalCalendarYear andtestheaders notindependentbodycredit.',
 27:'PrivateTacticalElapsed actualticks64Hz producer finite>=0 forallu64, no rawfloatconstructor/Serde/Reflect.',
 28:'ExactGateDatum/SceneDoorPose finite/frame admission/getters; unrelatedspatialroles excluded.',
 29:'Exactactualrequired_height_at/height/coarseheight delegatesSupportElevation andpreservesMissingHeight, otherqueries excluded.',
 30:'ActualSurfaceHit1–23 publictypedfield/constructor, notsealedterrainmembership; corrects meregroundingreexport.',
 31:'Exact CombatDuration/Instant nativeDuration+AnimationElapsed checkedport,0valid; no arbitraryfloatdecoder/provenance.',
 32:'P16exactclockbody independentlyread/currentrolecompared, sameactualnativeportcontract.',
 33:'Twoactualpositiveauthored threatpowerconstants only; no entireprofile validationinferred.',
 34:'ActualP17Radians finiteunwrappedprivate/default/manualSerde/opaqueReflect,maxselectsadmittedoperand; PlanDirection44+excluded.',
 35:'ActualP17SignedLength finiteSIGNED incl0/manualSerde/min/max/scale readmission; Area declarationcontext only,81+excluded.',
 36:'ActualP17PositiveLength finite>0/private/manualSerde,noDefault; validsubsetPositive→Signed.',
 37:'ActualP17ScenePlanPoint finiteXZ/private/checkedOption/Serde, FromPosition depends on latentlowerrawbackingG007.',
 38:'ActualP17SupportElevation signedfinite/private/manualSerde/checkedtranslation; nopositive/terrainmembershippromise.',
 39:'Dependency-only P17Bevy/geometry namespace edge, not standaloneacceptedstate.',
 40:'Dependency-only optionalReflectfeature/version path, not newvalueadmission.',
 41:'Dependency-only workspaceBevyconstraint; actualresolvedversion042.',
 42:'Dependency-only actualresolvedBevy/reflect/derive0.19.1 supports exactRigreflection001 reuse, no foreignbodyexpansion.',
 43:'Dependency-only localgeometrypathversion, no numericadmissioncoveragefrommanifest.',
 44:'ActualP17Position/Displacement pub(super)rawmetres latentfiniteencapsulationgap sameRootG007; dependentPointFrom inheritsinvalidSelf possibility, noactualbadcaller.',
 45:'ExactfiniteXYZhelpererrororder only, no allgeometryproof.',
 46:'ActualPRReportedPrecision21–40 privatefinitef32, intentionallynegative/>1allowed; noDefaultSerdeReflectFrom.',
 47:'ActualPRRecoveryTime andfactory1–54 only;55–57 context excluded. ZeroAftervalid/CannotRecover explicit,Serializeonly.',
 48:'ActualPRclosedarrival31–37/contact_timing60–80, typedelapsed/nativeauthoredfallback, no unrelatedlungegeometry.',
 49:'ActualP16ReportedPrecision24–41 firstpartialbody; shiftedgettercompletedseparately052. Imports/neighborcontext excluded.',
 50:'ActualP16RecoveryTime1–54 independentlyread;55–57 context excluded.',
 51:'WrongPR-stylelocations reachedincidental P16geometry context only: NO semanticcoverage for051. Originalrequestedranges preserved; actualneededarrival053 registeredbeforebody.',
 52:'ActualP16getter42–43 completes049;44contextonly.',
 53:'ActualP16arrival49–57/contact_timing88–108 independentbody/currentsemanticcomparison; not automaticPRcarry.',
 54:'FullP16presence helper196 actualPR137body pluscompleteP16delta/currentrole comparison; rightopen/equalinstantempty/uniqueclaim/currentobserver rolesonly.',
 55:'P16strategicaction171–714 actualPR108body+completeP16delta/currentrolecompare for19/20directcallees. Original21only1–170 separatelypreserved;715+tests excluded.',
 56:'P16ForageDraw belowexactNonZeroexpect/nativeRNG; actualuppers1000/1001/positiveauthoredwidth. Arbitrary0notcertified.',
 57:'Exactmodelmacros/SiteKind/Weight/rawContext/Candidate/scopedID only; F007publictuple/derivedSerde andF008staticfactorysyntax gaps. Othermodelroles/Rustmacroexpansions after161 excluded.'}
rows=[]
for r in regs:
 n=int(r['id'][-3:]);p=Path(r['resolved_path']);raw=p.read_bytes();sha=hashlib.sha256(raw).hexdigest();eof=len(raw.decode('utf-8-sig').splitlines());assert sha==r['sha256'] and eof==r['actual_EOF'];
 ranges=r['material_ranges'];assert all(1<=a<=b<=eof for a,b in ranges)
 material={5:[[73,109]],7:[[21,48],[58,96],[153,170],[563,594]],8:[[177,184]],19:[[320,320]],46:[[21,40]],47:[[1,54]],49:[[24,41]],50:[[1,54]],51:[],52:[[42,43]]}.get(n,ranges)
 rows.append(dict(id=f'NATIVE-ADMISSION03-QUAL-{n:03}',registration_id=r['id'],original_registration=r,original_requested_context_ranges=ranges,qualified_material_ranges=material,qualification=notes.get(n,'Exact direct namespace/reexport route supporting P16 namedproductSATS macro witness; no general firstpartycodec orforeignpackagecredit.'),current_admission_proof_refs=refs[n].split('; '),frozen_hash_eof_verified=True,semantics_from_hash_or_ranges=False,global_reconciliation='Root authority; raw original registration retained; no globalacceptance from this localqualification',producer_owner_later_lens_credit=False))
qf=o/'source-qualifications-final-082.jsonl';assert not qf.exists();qf.write_text(''.join(json.dumps(r,ensure_ascii=False)+'\n' for r in rows),encoding='utf8')
print(json.dumps(dict(registrations=len(rows),empty_incidental=[r['registration_id'] for r in rows if not r['qualified_material_ranges']],qualification_file=qf.name)))
