"""Record exact authored building/city/environment/fungus view tables."""
import bookkeeping as b
from record_l11 import record
sources=[(1726,563,[[1,200],[201,400],[401,600],[601,677]]),(1727,564,[[1,69]]),(1728,565,[[1,76]]),(1729,566,[[1,133]]),(1730,567,[[1,200],[201,323]]),(1731,568,[[1,83]])]
notes=[
'Full677 shop19/workplace63 static descriptors. Camera ordinals0..17/0..61, finite authoredFOV12..88, bounded100bps threshold, singlewarmup. Building comparison uses canonical ReviewLod enum, matched views settledpair; other workbay descriptions are sourceidentity labels, actual authored JSONcamera correspondence stillrequires camera producer, no labels-as-proof or other-profile import coverage. No failurecause available to discard in this pure constant table.',
'Full69 city8 views maps seven cameras0..6, matches248 actual city producer7 ordering facade/street/playable/neighbourhood/edge/aerial/horizon. Valid finiteFOV50..72 and fixed100bps allsource; constructor/native ready/gate bodies connected246/250/251, no actual image success. No extra type needed for every local constant solelybecause numeric.',
'Full76 compound8 views seven ordinals0..6, singlewarmup, every recordedview vista+settledpair; 58deg/100bps within authoredrange. Camera table semantic identity and concretegeometry provider stillpending, no capture validation/runtime or other-lens transfer.',
'Full133 environment15 views closes251 optionalcamera ambiguity for thisprofile. Tree root explicitly TreeFocus, branch BranchFocusWithLeafSuppression, rock RockFocus, debris DebrisPair; actual246 manifest requires these targets, so fallbackground cannot certify these required details. Grazing/suppression explicitly diagnostics, peak/valley require actualVista via251; GrassPresent separate. Narrows252 finding to cold tree residency profile lacking corresponding admission. No hiddengameplay penalties or compatibility fallback.',
'Full323 facade32 static descriptors. Canonical ReviewLod Detail/Facade/Shell, all31 recordedplates settledpair, valid58/68deg and100bps. Camera ordinal0..30 finite table, actual spec-to-authoredcamera correspondence/closurepose source stillpending. Labels fixed-glass-open/shutter/door describe intended evidence, not actual physics assertion. No tests/assets/rendering executed.',
'Full83 fungus7 descriptors uses canonical FungusSpecies and checked authoredmetres0.35..20, finite40/55deg and100bps; all6 recordedviews settledpair, community/distance addvista. Actual species/root camera selection comes throughCapturePose plant_camera/accepts_plant sibling, not inferred from tablelabel. No raw commonname lookup/legacy species alias here. Sourceonly.'
]
roles=[]
for (i,n,_),note in zip(sources,notes):
 g=b.read(b.OUT/f'REGISTRATION-ADLER-{n:03}.json')
 roles.append(dict(source_indices=[i],ranges=g['ranges'],roles='Independent lens11 full exact retained native descriptor/type granularity/semantic identity/target gate source contract.',disposition=note))
ledger=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
f=ledger['discovery_findings_and_dependencies']['ADLER-L11-TREE-CAPTURE-TARGET-ADMISSION']
f['trace'].append('READ253 actual environment1729 table explicitly requires TreeFocus/BranchFocusWithLeafSuppression/RockFocus/DebrisPair. Actual246 manifest detail gate makes ground-camera selection fail those missing targets. This distinguishes ordinary explicit environment target admission from cold traversal252 missingtree check; do not expand finding to all optional camera helpers.')
f.setdefault('connected_receipts',[]).append('../../adler/READ-L11-253.json')
pending=[
'253 full1726..1731 all metadataunion complete, next1732. All13 populations/extensions/current/all20/aliases/finaldedup incomplete; no gate/accepteduniquechange.',
'Actual environment required detail targets explicitly gated, narrows252 coldtree targetadmission finding and rulesout blanketfallbackground falsepass for these sourcepaths. Exact city7 camera ordinal table matches248 producer ordering. Other authoring camera JSON/closurepose/plantcamera selection stillpending.',
'All numeric scalar/label values here fixed boundedauthoring; semantic selectors and canonical sourceowner fit remain globalall20 question, not universal wrappers. No new sourceidentity/extensions from table adjacency, no fixture/runtime acceptance.',
'Eight scopedextensions/nativeoriginal4EOF/future2248/2249/2250/2701/partiallater129/L13 and L12/L13 unchanged. Nextregistration569 original1732 exactmetadataunion beforebody. No productchanges/execution/commits/publication/externalmessages; goalactive/incomplete.'
]
record(253,252,sources,roles,[246,247,248,249,250,251,252],pending,1732,findings={'ADLER-L11-TREE-CAPTURE-TARGET-ADMISSION':f})
