"""Record exact semantic animation evaluation and its attack/locomotion projections."""
import bookkeeping as b
from record_l11 import record
sources=[(1758,595,[[1,200],[201,400],[401,591]]),(1759,596,[[1,82]]),(1760,597,[[1,200],[201,327]])]
notes=[
'Full591 AnimationEvaluation::from_skeleton takesimmutableSkeletonState andbuildsalllocalcheckedPoseSamples beforepublishingresult, no retainedstatewriteonthispath. Typed SemanticPose/PoseSampling distinguishes Anchor/Cycle/Timeline/Span/CurveSpan/ContinuationSpan; signedcurvecoordinate notgloballyclampedwhilefiniteprogressclampadmissionisexplicit. BlendWeight additiveowner andClipMirror binary kept. Derivedspeed/airbornephase/heightfinitechecks andnestedAnimationValueError propagate. Posture/quickstep/downedfacing fallback ismutuallyexclusive semanticselection, not legacyparallelgraph. Large signedroll cast toi64 isafterinteger-nearbyguard; representablehugef32rolls integer returnbeforecast, no arbitrarysaturatingparitybugassumed. ActualSkeletonState admission/update/nativeposeconsumer/current stillpending. Sourcefixturesnotrun.',
'Full82 actual attack projection. Continuation branch carriesownedtypedstartcoordinate/tangent andusesContinuationReadiness::FOLLOW_READY atline29, connecting257 measuredanchor torealruntimepose; actualauthoritativecurve/configrole stillneededforstructuralversustuningclassification. NoncontinuationcheckedAttackCurve.coordinate/queued_recovery_coordinate propagateserrors. AttackanimationNone defaultsThrust onlyonactualAttack dispatch; actualActionState construction/deserialize invariants required before silentmissingattackfinding, no malformedservereventobserved. CanonicalAttackAnimation->SemanticPose mapping total; optionaloffhandpreparation explicitpolicy, no droppedinnererror.',
'Full327 locomotion projection. NonnegativeSpeed/cyclephase/BlendWeight carryvaluesuntilcomplete direction/blendkernels, nonzeroaxis/total guards orfilteredno-samples beforedivision, allscaled contributions collectedbeforeextend. Runtime animationconfig thresholds/blends andwalk/runreference speed owner; actualruntime_animation_config binding/server-loadedsnapshot/caller provenance requiredbefore compileddefaultdivergence. Canonicalgaitwrap andhalf/quartercontactanchor offsets structuralmapping versusruntime authoredthreshold distinctionpreserved. Downed speedzero whennotturning isanimationselection, notphysicalfatigue/movement-speedpenalty. Puretestgait_pair undercfgtest numerical/referencehelper notlegacyproductionpath. No tests/baking/runtimeexecuted.'
]
roles=[]
for (i,n,_),note in zip(sources,notes):
 g=b.read(b.OUT/f'REGISTRATION-ADLER-{n:03}.json')
 roles.append(dict(source_indices=[i],ranges=g['ranges'],roles='Independent lens11 full exact semantic evaluation/checkedoperations/canonicalowner/structured-error contract; sourcefixturesonly.',disposition=note))
pending=[
'258 full1758..1760 metadataunioncomplete,next1761. All13 populations/extensions/current/all20/aliases/finaldedup incomplete,gatefalse/accepteduniquechange0.',
'ActualevaluationpureimmutableSkeletonState->checkedlocalrecord, no partialretainedstatewritehere. Skeletonadmission/mutatingupdates/nativeFK/Ragdoll/caller/currentstillpending, notwholeanimationclearance.',
'ActualcontinuationusesFOLLOW_READY constant1759line29; actualcurve/configstructuralvsYAMLrolepending. runtime_animation_config use1760 requiresownbinding/server-loadedprovenance before divergenceclaim. Attackkind/optionalAttackAnimation invariants neededbeforeThrustdefaultmisclassification.',
'Allpriorfindingslimits/eightscopedextensions/nativeoriginal4EOF/future2248/2249/2250/2701/partiallater129/L13/L12L13unchanged. Nextregistration598 original1761 metadataunionbeforebody. No productchanges/execution/newtests/CI/commits/publication/externalmessages;goalactive/incomplete.'
]
record(258,257,sources,roles,[228,229,230,241,246,250,251,252,253,254,255,256,257],pending,1761)
