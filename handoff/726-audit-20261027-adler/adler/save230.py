"""Record exact retained/source/variant/owner contracts and first ordered error reuse."""
import copy
import bookkeeping as b
from record_l11 import record

sources=[(1608,441,[[1,194]]),(1609,442,[[1,150],[151,245]]),
 (1610,443,[[1,28]]),(1611,444,[[1,59]]),(1612,373,[])]
notes=[
 'Full194 retainedscene/terrainbinding owner. activate computes/admitssourceTerrainContactHeightmap before movingactivecache on newsceneerror; matchingretainedscene restored byexactdigest, existingactive remains onadmissionfailure, narrowingpartialswapclaim. Boundactive+2previous viaDeque explicit. Restorevariant clonesimmutableCPU/nativehandles, rebindsbark to newterrain, not preserveoldtacticalmutable state/legacycompatibility. Specieskey saltedupstream229; no genericcrossspeciesreusebyname. Diagnosticsdedupvariants viaSeed andatlases viaMaterialAssetId fitsactualclonedcachedcard/material ownership; newpolicy onlyifactualaliasingnativeproducer discovered. Environmentdigest/Terrain relation actualgenerator/nativeproof stillpending. NativeimageCPUextraction differences from216ImageSDK remainforTerrainContactowner; no runtimegeneration/tests.',
 'Full245 actualfactory independentlyread. Immutable typedTreePresentationSpecies maps exactlyEnglishOak/CommonBeech and canonicalbake_style. CommonBeech playableandvistafactories callCOMMON_BEECH_PARAMETERS skeleton/leafproducer, closes226/227/229 actualsourcebeechpipeline gap. CommonBeech preset12leaves and rootdepth0 exact225 witness; sourcefactsconnect no generatedimageobserved. Oaksite/growthkey excludescurrentweather intentionally; hilly/wetland/cultivation/canopy/location/elevation plusvariantseed canonicalgrowth, actualgnarlingstress addsclamped0..1controls fromadmittedbpssite; supportsfiniteauthoredgrowthpremise. Species usescurrentgroundmoisture community probability, cacheincludesactualspecies salt; no automaticstalegrowthbugfromweatherexclusion. Vista nativeVec2LOClocalcoords is deliberateLod-localcommunityprojection, not fabricatedScenePlanPoint validity. No storedserver/tacticalauthority change. sitekey u64fromSeed couldfitglobalidentityowner, sharedall20dedupbeforeextra wrapper. Actualparams for oakfixedrecipepolicy owner sourcewitnessed, not generalcustomeditableallfieldscontract.',
 'Full28 actualoak_variant_for_site namedStreamId(specimen.variant-selection).rng(obstacle_seed).index(showcase.len), returningactualslotpluscanonicalnamedSeed. site-rootexposure uses samecanonicalchoice and growthowner; notpreview417purposewhichintentionallyselectsinspectionvariant. Focusedrasterfixture436line840 usesrawSeed&3 instead; oraclefamilybelow. No deterministicindex/seed calculationor assertionrun; actual0..3 drawbounds protects sourceOAK_GNARLING_SHOWCASE[index]. oak_variant_seed usize is authoredslotcontext internally, broadercanonicalslotwrapper/all20policy pending before isolated redundanttype.',
 'Full59 PresentationOwner enumScene/RegionalMap and fixedgenericowner slots retainidentity; no unboundedHashMapowner oroptionalparallelstrings. get/get_mut totalmatch, render_layers nativeport separatelyScene default/RegionalMap3. Fixedlayerindexnativecapabilityconstant not per-valuewrapper mandate. Rootmarker wasm/test scoped intentionally platformintegration, notcompatibility alias. Generationtickets218 actuallyownerqualified; no sourcecredittransferred tootherlenses.',
 'First ordered1612 adopts exact independent lens11 full115 REG373/READ218 connectedproof, no freshreadclaim. Canonical sharedPreparationError includes ProductKind, ticket/binding mismatches andownedJSON/CBOR/geometry/building/interior/VistaMesh; GraphicsConfiguration owns outermessageString after early217causeerasure. Retiredgeneration/error original1537 historical111 neverusedasliveproof. No runtime/newgeneratorcurrentcomparison orotherlenscredit; sharedtypederror not automatically fixes graphicsparserownership.',
]
roles=[]
for (i,n,_),note in zip(sources,notes):
 reg=b.read(b.OUT/f'REGISTRATION-ADLER-{n:03}.json')
 role=dict(source_indices=[i],ranges=reg['ranges'],roles='Full exact original/owner/producer/reached source; independent source-only lens11.',disposition=note)
 if i==1612:
  prior=b.OUT/'READ-L11-218.json';proof=next(x for x in b.read(prior)['connected_reinspection'] if x.get('original_index')==i)
  role['roles']='First ordered credit by exact same-lens full connected proof adoption, no freshbodyclaim.'
  role['reuse_proof']=dict(receipt=str(prior),receipt_sha256=b.sha(prior),connected_registration=proof['registration'],registration_sha256=proof['registration_sha256'],source_sha256=proof['source_sha256'],read_ranges=proof['fresh_reads'],origin='READ218 future1612 full115 zeroorderedcredit then; firstoriginalfrontier now, retired1537 historicalproof remains separate.')
 roles.append(role)
old=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')['discovery_findings_and_dependencies']
updates={}
for key,note in {
 'ADLER-L11-MID-TRUNK-ROLE-ADMISSION':'Actualsource1609 playable_tree_source191..200 invokesCommonBeech presetnativegenerator, source1609 bake_styleCommonBeech returnsBeechstyle. With229actualsharedmidhelper593 and225rootconstructor thisclosesfrozenfactory/callerdependency; native/render/currentacceptance stillpending.',
 'ADLER-L11-TREE-CROWN-SELECTION-OWNERSHIP':'Actualsource1609 mapsCommonBeech toCOMMON_BEECH_PARAMETERS skeleton/leaves andBEECH_TREE_BAKE_STYLE. Together229actualbakecall and228actualraster thisclosesfrozenbeechfactory/selectionconsumer chain; no runtime/GPU/currentoutcomeclaimed.',
 'ADLER-L11-TREE-CARD-TRIANGLE-DIAGNOSTIC-AUTHORITY':'Actualcanonicalbeechfactory1609 preset12leaves andoakfixed16 corroboratebalancedordinalquarterselection. gitgrep locator identifies actualTreeLeafTriangleCount reader tactical_scene_viewer.rs1675; readerbodyUNREAD, no diagnosticreporteffectcreditfromlocator.',
}.items():
 f=copy.deepcopy(old[key]);f.setdefault('connected_receipts',[]).append('../../adler/READ-L11-230.json');f['trace'].append(note);updates[key]=f
oracle=dict(
 classification='Frozen focusedoak acceptancefixture reconstructs variant from rawbits while productionusesnamedstream; oracleauthority mismatch, exactvalue/runtime/current pending',
 source_receipt='../../adler/READ-L11-230.json',
 source_key='8a8c48a47ab97a26549f1e28a554bfca3fb50debb1f90d1217af97bc63e5b184:10-14,25-28; ee16efd8a79ab3116e35ecd6f0e7ac5eb6a5e48c573e33e84eba57b4c5634913:819-846,880-908; eb8c1a54c7a158a3a3858b2eb9ba2b4f5c62ce6fb1fde444bc6b2c2207b824be:863-891; ad0f8971f300e87ee98b0cd1d44273bb361c576566444a2b45216f9ddb65dff4:5-46; c2ec90f6b96a13e2f97e9b20426954990d9df8a0331b08bb5709d73e0d09cec6:34-61',
 trace=['Focused sparsefixture claimsproductionpoint/variantseed, butvariant_index=(obstacle_seed.to_u64()&3) at1603line840.','Actualproductionpresenter871 calls oak_variant_for_site; canonical1610 uses separatelynamedStreamId andfixedwidthrng.index(showcase.len). Seed/StreamId220 confirms BLAKE3-derivedstream+SplitMix/RandUniform aredistinctcontractfromrawparentmask.','Fixturepassesitsrawslotinto oak_variant_seed, thenhardasserts slot1andfixedFOCUSED_TREE_RECIPE_SEED beforebuildinggeometry. No exactdraw/seedvalueverified, so neither an observedassertfailure nor a provenparticularwrongframedtree claimed.','Preview417namesvisual.tree.preview-variant independently; do notconflatepreviewchoicewith productioncanonicalsitechoice. Related TwigTopologyOracle220 has samesamplingmigrationclass butdifferentactualgeometryowner; sharedfamily/all20dedup stillpending.'],
 first_failed_contract='A fixture claiming to reproduce the production site specimen must derive it from the canonical site selection contract and independently verify the claimed fixture identity.',
 canonical_owner='Existing oak_variant_for_site canonicalsite/slot/variantseed choice, preserving authoredfixture correspondence and deterministic marker contract.',
 repair_dependencies=['Exactfocusedscene/manifests/capturetarget/currentproducer and fixtureexpectedreplaymarkers','Canonical Seed/StreamId andvariant-slotowner; relatedTwigTopologyOracle220 all20/aliasdedup','Keepauthoritativefixture input/camera/source geometry acceptance ratherthanselectingeasierfixture'],
 acceptance=['Focusedsource/capture fixture usesactualcanonicalproduction site choice and assertedfixtureidentity','Anyintentionallychanged seedmarkers/input/camera explicitlyreconciled; no raw-bit compatibilityselector','Independent projectedcoverage/nativeacceptance remains meaningful aftercorrectspecimen selection'],
 repair_status='not started; exhaustive audit ledger incomplete',
 validation_status='Exactfrozenactualsite/presenter/focusedfixture/SeedRngsource; no calculatedseed/RNGprobe/assertion/native/currenttest outcome',
 accepted_unique_count_change=0)
updates['ADLER-L11-FOCUSED-TREE-VARIANT-ORACLE-AUTHORITY']=oracle
pending=[
 '230 completesoriginal1608..1612 fourfullfreshsources plus firstordered115errorownerreuse218. Next1613.228..230thisgoalturnauthoritativeprogress, no wait/blocker. No broadercompletionclaim.',
 'All13 populations/extensions/current/all20/aliases/connected acceptance remainincomplete, gatefalse/accepteduniquechange0; preserveall earlierprovenance/deletion/native/current/stash/publicationqualifications.',
 'Beech sourcefactory/bakestyle confirmsactualfrozenchain forrootmidrole226/crownmembership227. Theseareconnectedsourcefindings, not tests/render/currentevidence. Triangle countactualreader onlylocatortactical_scene_viewer1675, bodypending.',
 'Focusedfixture rawparentmask vs actualcanonicalnamedstream sourceauthority mismatch, not exactassertionfailure. RelatedTwigOracle220 sameglobalmigrationclass maydedup afterall20; no newuniqueacceptedcount.',
 'Retainedterrainactivate buildsnewadmittedcontactbeforemovingactivecache; fieldatomicfailurepremise narrows. Digestvalidity and actualTerrainContactHeightmap/Image/native/retainedcache source dependencies stillpending.',
 'Source nativevariantindex boundedactualdraw0..3, growthfixedpresets/clampedsitecontrols witnessed. Do notinventarbitraryeditor/maliciousasset inputs or demand allprivatefloats bespoke wrappers.',
 '1612firstcredited230, no longerfuturequeue.2248REG398/2249REG396/2250REG397 READ220 awaitfirstorderedadoption; later5/6/7/11/127/128/129connectedproofsnotorderedlaterpopulationcredit. Rootprofile129tail54..117unread.',
 'Next1613metadata/prospectivehash/fullunionbeforebody; nextregistration445. No sourcebody beyond444read orregistrationbeyond444made. Treepresentation/diagnostics importednativeowner stillnotbodycreditedbyname.',
 'No product changes/builds/tests/lints/scanners/generators/runtime/nativeprobes/CI/commits/publication/messages; ignoredsource/evidencebookkeeping only. Fullgoalremains active/incomplete.',
]
record(230,229,sources,roles,[205,206,217,218,219,220,221,222,223,224,225,226,227,228,229],pending,1613,findings=updates)
