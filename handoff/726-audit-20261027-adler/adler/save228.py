"""Record tree raster, asset loading, LOD and material source contracts."""
import copy
import bookkeeping as b
from record_l11 import record

sources=[(1603,436,[[1,150],[151,300],[301,450],[451,600],[601,750],[751,900],[901,976]]),
 (1604,437,[[1,150],[151,300],[301,444]]),
 (1605,438,[[1,150],[151,300],[301,402]]),
 (1606,439,[[1,150],[151,300],[301,450],[451,600],[601,629]])]
notes=[
 'Full976 raster production and all fixtures read. Actual render_tree_card filters branches bycard.includes_branch and leaves bycard.includes_leaf before owned nativebake meshes; confirms227 membership gap reaches actual softwareprojection. Actual camberedleaf mesh consumes retainedorientation/torsion, then project_to_tile uses positions/normals, extending227 fingerprint relevance. Native TreeRasterTarget/Sample/AtlasRegion and TreeSourceMaterial enum preserve separate buffer/sample/material roles; fixedtile/currentcard invariant required before invalidpixelshape claim. Depthtest earlier and writer same <=tie preserve exact tiepolicy, no newdifferentdepth semantic branch. baked_leaf_color deliberately uses semanticshadeXY and ambientW, ignores selfshadowselectorZ as pigment; oldlimebake comment refers replacedbehavior, no activelegacyfallback. Strideowner sorts retainedshootID/ordinal and samplesper-shoot rather than globalstep_by; fixedstride>0 bystyle, no emptyshootdraw on actualloop. All coverage96/192grid/atlasalpha/oak/beech/forcedfixture source-only; projectedproxy coverage approximate, not actualGPU renderedacceptance or group-membership proof. Focusedfixture rawobstacleSeed&3 needs actualoak_variant_for_site source1610 before agreementclaim; previewvariant stream417 distinctcaller. No bake/test/scene generation.',
 'Full444 wire asset/native runtime boundary. Gzip/read and postcarddecoder errors retained in owned IO Error source, not formattedString; PreparedTreeImpostorUsage enum retained. Serialized privatebakes keep speciesu8/lodu8/version/arrays/indices/cluster/provenance asdecoded; actualresolve filters usage/species/seed/lod butnot expectedsourcehash. Versionchecked only runtime assert335, actual semantic/native shape admission candidate below. matches_art_demo expectedsourcehash recomputation and from_runtime generator cfg(test), so do notpresentthem as release-runtime validator. Requestedfixed artdemo asset and optional handles deliberate ownership, no genericmalicious-file/securityfinding or shippedassetfailure. Runtimeimage/mesh constructor contracts stillrequire native source; optional precomputed versus softwarebake use actualpresentationcaller upcoming. Wiredecode shape doesnotprove nativearrays cardinality/nonfinite/atlas compatibility; no unsupportedlegacy migration introduced.',
 'Full402 union includes former1..180prefix and sourceonlyticks/isolationfixtures. Actual existing TreeLod wrapper holds pubcrateu8 unchecked and overrideOptionu8 flows to direct branchselection/raw helpercalls; boundedsemanticowner candidate below, no duplicatePlantLod. Exact leafrepresentation enum, dedicated Playable markers distinguishwild/review/nativefamilies; fivebenchmarkhidebools independentaxes, not automatically invalidaggregate. ProjectedScale nativecameraFOV/defaultreference usesclamp, actualCLIadmission/nonfinite/camera lifecycle pending. Single cameraquery returns ifabsent/multiple, no arbitraryfirstcamera. Writesonlyifchanged preserve componentticks. ForcedLOD makes bothtrunkmarkers abrupt visiblefor<4; preservecapturequalification that forcedview/benchmark maydiffer normaldistance selection, actualreceiver/capture policyneeded beforeacceptedbug. No CPU/GPUtestsor App.update executed.',
 'Full629 includes former1..130roles but materiallyaddsactualbark/leaf/impostoruniform construction andfixtures. SceneElevationRange canonical owner extractedonlynativeheightuniform, speciesrawconstants/Vec4 are finalnativecalibration, not everyprivatefloatneedsa wrapper. Aggregatebark distinctbespokematerial/type/layout intentionallyremovesexpensiverelief pipeline, sharedExtendedMaterial alias nativeAPI not compatibility. Bark COLOR rootheightgeometry/leafRGBsemanticchannels confirmed; softwarebake singlecolorproxy deliberatelydifferentfrom texturedlive material, no requirementinventedidenticalprojection. Currentconstructor defaults celestial lighting then shared221 updateronlychanges can leave newcohort question, nativeevents/actualscheduling upcoming. All shaderstringcontains/performance-symbol/derivative/soil/optics/transmissionfixtures read-only; shaderbodies/compilation/nativeGPU notcreditfrom strings. Canonicalmaterialcrate reexports need registerednativeowner, not coveredbyimport. No texturesgenerated/testsrun.',
]
roles=[]
for (i,n,_),note in zip(sources,notes):
 r=b.read(b.OUT/f'REGISTRATION-ADLER-{n:03}.json')
 roles.append(dict(source_indices=[i],ranges=r['ranges'],roles='Full exact original/owner/producer/reached/native source and every included fixture; independent source-only lens11.',disposition=note))
old=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')['discovery_findings_and_dependencies']
selection=copy.deepcopy(old['ADLER-L11-TREE-CROWN-SELECTION-OWNERSHIP'])
selection.setdefault('connected_receipts',[]).append('../../adler/READ-L11-228.json')
selection['trace'].append('Actual raster1603 render_tree_card55..60 filtersbrancheswithsamepredicate and stratified_tree_bake_leaves102..106 filtersleaves before nativecamberedgeometry. Wholecrownapproximateoccupancyfixtures include beech butdo notassert owningprimaryset membership; sourceonlynoresults.')
provenance=copy.deepcopy(old['ADLER-L11-TREE-BAKE-PROVENANCE-GEOMETRY-KEY'])
provenance.setdefault('connected_receipts',[]).append('../../adler/READ-L11-228.json')
provenance['trace'].extend([
 'Actual raster1603 builds sourcecamberedmesh80 and projectsactualvertexpositions180; omitted leaforientation/torsion are actualbakegeometrydependencies, not deadretainedfields.',
 'Preparedasset1604 release resolve61..76 selectsusage/species/seed/lod; matches_art_demo98..144 recomputesexpectedhash onlycfg(test). RuntimecopiesstoredhashintoProvenance. This establishes diagnostic/fixturecorrespondence use, not generalcachekey or runtimeexpectedhashvalidation.',
])
asset=dict(
 classification='Frozen fallible preparedasset decode publishes unchecked bake records; declared version rejection deferred to runtimeassert, native shape/trust/current pending',
 source_receipt='../../adler/READ-L11-228.json',
 source_key='792c336a1d9612c83b4ae0f385c94a35175009ae202f71d4e3231d10179f899a:52-76,202-223,265-282,334-374,377-444',
 trace=['from_compressed_bytes succeeds after gzip/postcard structuraldecode and storesrawrecords; no version/semanticadmission in fallibleloader.','resolve returns Option<TreeLodBake> and runtimeassert_eq onbake_version before nativearrays/Imageconstruction. A decodedmatchingrecord with differentversion canloadOk thenassertonresolve; source controlflowcounterexample, not an executedasset/test.','Source geometry/normal/UV/index/atlas/radius/version records have separate nativevaliditypremises. No arbitrarymaliciousasset/memory/securityclaim; fixedtrustedcontent/canonicalproducer and nativeImage/Mesh contracts mustbereconciled.','I/O and postcard causes alreadyretained; preserve them whenadding actual semanticpreparationerror, do notreplacewithString.'],
 first_failed_contract='The fallible prepared-asset admission boundary should establish runtime bake invariants, including supportedversion, before publishing an object whose resolution promises a usable bake.',
 canonical_owner='Admitted prepared tree-impostor record/asset, with owned decode and semantic/native context before conversion.',
 repair_dependencies=['Actual fixedasset request/resolve/renderingcaller and trust/canonicalgeneratorpolicy','Exact nativeImage/Mesh/index/layout invariants and serializedschema','All20/aliases/current/sharedassetadmission dedup; no compatibilityversionmigration'],
 acceptance=['Unsupportedassetversion rejected through fallibleloading/preparationwith inspectablecontext before runtimeconversionassert','Only actuallyrequired native/card/atlas/cluster invariants admitted, supportedauthoredasset preservesbehavior','No legacydecoder, migration/fallbackschema or genericunneeded untrustedinputpolicy'],
 repair_status='not started; exhaustive audit ledger incomplete',
 validation_status='Exact frozen loader/resolve/runtime source controlflow; no forged/decode/nativeinput execution, actualshippedartifact validity/current unverified',
 accepted_unique_count_change=0)
lod=dict(
 classification='Frozen existing TreeLod wrapper does not admit boundedsemanticstate and rawu8 crosses override/bake/visibility ports; structuraltypefit candidate, caller/current pending',
 source_receipt='../../adler/READ-L11-228.json',
 source_key='9e94d087fccffc1d1ed76815d33256a0d9c53b941cf11929db12be7d6419b4bd:6-7,64-69,119-170,190-205; 918bd2a4ed873968c420e4ad7c3070043b2073a9bbdd30e27d1dbbd6c1a4d2a1:188-212,455-595,766-804,879-887; 792c336a1d9612c83b4ae0f385c94a35175009ae202f71d4e3231d10179f899a:61-75,265-282',
 trace=['TreeLod(pubcrateu8) isexistingsemanticownerbutconstructiblewithanyword, override retains Optionu8 and helpers rawu8 assert/unreachable boundedrange.','Representation0..4 and aggregatebake1..4 distinct admittedsets; currentnative1..4 loops mayjustify actualcalls butdonotgive typeits stated invariant.','Native uniform/payload and wire encoding legitimately integer; candidate concerns retainedstate/portsbefore thoseleafextractions, not universalbytewrapper. No actualinvalidCLIvalue/treeLODpanicobserved.'],
 first_failed_contract='The existing canonical tree-representation owner should establish valid semantic levels and preserve that admission through selection/bake/asset ports.',
 canonical_owner='Existing TreeLod boundedrepresentation owner, with explicit aggregatebake subset where required; reuse before paralleltype.',
 repair_dependencies=['Actualpresentation/storage/CLIoverride/readiness sources and releasecodec','Canonicalsharedrepresentation/enumshape/all20/alias/current reconciliation','FinalnativeGPUuniform/wireencoding boundaries'],
 acceptance=['Unsupportedlevels rejected before retainedstate/nativebakevisibilityunreachable paths','Allsupported representation0..4 and aggregate1..4 transitions/override/codec retain intendedbehavior','No rawlegacyLODforwarding/parallelowner or duplicateunrelatedPlantLodreuse'],
 repair_status='not started; exhaustive audit ledger incomplete',
 validation_status='Exact structural existingtypefit/rawport chain, actualcalleradmission/current notcomplete; no invalidinput/runtime tests',
 accepted_unique_count_change=0)
pending=[
 '228 completes4 original1603..1606 full976/444/402/629 union; next1607presentation1392. Previous goalturn progress verified227, currentauthoritativesource/ledger advancement; no no-progressblocker.',
 'All13 populations/extensions/current/all20/aliases/connected acceptance incomplete; gatefalse/accepteduniquechange0; earlier count/provenance/current/native/extension/preservation qualifications retained.',
 'Crownselection227 reachesactualrasterinputs. Fingerprint227 omittedorientationreachesactualmeshprojection; preparedexpectedhashmatcher onlycfg(test), releaseconsumer selectsusage/species/seed/lod. No generalcachekeyreuseor actualrenderartifactclaimed.',
 'Preparedassetversion/semanticadmission and existingTreeLodtypefit candidates recordedsource-only. Actualfixedartifactcontent/trust/current/caller/nativebounds stillpending; no malformedinput/decompression/native/runtimeprobe.',
 'Focusedfixture1603 rawobstaclemask&3 versus production oak_variant_for_site needsactual1610body; otherpreviewcaller417namedstreamnot automaticallysamepolicy. Do notassume agreementorfailure untilnativeowner.',
 'ForcedLODbothtrunktiers get abruptvisibilityfor<4, differs normaldistanceranges; actualcapture/benchmarkcontract pendingbeforefinding. Sharedlightingnewassetcohort221 remainspendingactualschedule.',
 'Nextregister440 presentation1607full1392 thenresidency1608full194/source1609full245/specimen1610full28/ownership1611full59.1612REG373READ218firstorderedreusepending;2248/2249/2250samequeue. Nextregistration440.',
 'No product changes/builds/tests/lints/scanners/generators/runtime/nativeprobes/CI/commits/publication/messages; allnewworkignoredsource/evidencebookkeeping.',
]
record(228,227,sources,roles,[206,219,220,221,222,223,224,225,226,227],pending,1607,
 findings={'ADLER-L11-TREE-CROWN-SELECTION-OWNERSHIP':selection,
           'ADLER-L11-TREE-BAKE-PROVENANCE-GEOMETRY-KEY':provenance,
           'ADLER-L11-PREPARED-TREE-ASSET-ADMISSION':asset,
           'ADLER-L11-TREE-LOD-ADMISSION':lod})
