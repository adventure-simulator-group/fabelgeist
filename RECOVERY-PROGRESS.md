# #726 recovery progress

## Quantitative summary

- **Publication decision:** the user accepted the partial recovery as the
  handoff and authorized publication. The final authoritative ledger, queue
  and checkpoint remain missing; the recovery goal is blocked.

- **Public-progress receipt check:** **7,196** preserved public progress files
  were checked against the **3** root 719 prior receipt hashes; none matched.
  This check does not establish absence from other revisions or sources.

- **Root 719 prior receipts:** **3** recorded original receipt hashes had no
  match among **115,534** distinct transcript JSON values under the tested
  serializations or the scanned output strings. Truncated output and other
  encodings remain possible sources; the receipts are still missing.

- **Continuation guidance:** **1** final-plan uncertainty is now called out
  directly in `AUDIT-RESUME.md`; the full verifier passed at `3d73914`.
  Publication still requires a separate review and user authorization.

- **Archived predecessor search:** **36,487** manifest entries and **6,405**
  archive members supplied **30,461** distinct candidate hashes; none
  reproduced the final plan under the conditional template. This does not
  search other archives, other encodings or alternative templates.

- **Computed predecessor search:** **1,277,240** distinct hashes from
  transcript output blocks, strings and tested JSON serializations did not
  reproduce the final plan under the conditional candidate template. This
  does not test truncated or absent output, other encodings, or other plans.

- **Final plan predecessor search:** SHA-256 values of **3,104** tracked
  package files at commit `95edd52` supplied **2,808** distinct candidate
  state hashes; none reproduced the recorded plan hash with the existing
  conditional plan template. This does not search untracked files, archive
  members, other serializations, or alternative plan templates.

- **Latest portable checkout:** **1** fresh same-machine clone of commit
  `3f0cc92` passed the full evidence verifier using the documented bundle
  branch and line-ending settings. This is not an independent backup.

- **Latest 14 batches (roots 710–723):** **6** qualifications and **6**
  packets match their original hashes; **8** of each remain missing. No adoption
  receipt in this span is yet recovered. The previous **19** adoption receipts
  and states, roots 691–709, are exact. These counts do not measure findings or
  overall audit completion.

- **Original source registry:** **2,846 of 2,853** identities now have exact
  source bytes in a self-contained archive, representing **2,462** distinct
  objects. The other **7** rows describe deleted files; all **7** recorded
  base versions are verified and preserved as **6** distinct objects. These
  are source-byte counts, not audit completion.
- **Selected revision source:** **2,193** registry paths map to **1,824**
  verified Git blobs at the final selected revision; **660** registry rows
  have no blob at their path. Git commit and tree proofs verify both claims;
  a same-machine restore passed and refused an existing destination. These
  bytes do not establish audit coverage.
- **Missing dependency crosscheck:** neither source archive contains any of
  the **484** still-unmatched hashes in the earlier limited inventory. The
  search remains open to other evidence; these are not all missing records.
- **Printed-value search:** **10,352** distinct preserved outputs yielded
  **13,319** JSON values starting on a line; none matched the **484** hashes
  under the tested serializations. Other encodings and evidence remain open.
- **Private transcript search:** **65,972** distinct tool-output text blocks
  yielded **79,131** unique JSON objects and arrays, including nested values.
  None matched the same **484** hashes under the tested serializations. The
  scan does not cover arbitrary formatting, truncated output or other sources.
- **Git evidence search:** **31,726** text-blob checks across three pinned
  trees, with overlap, found no match for the **484** dependency hashes.
  Other file types, larger blobs and unexamined revisions remain open.
- **Draft change preservation:** **8** draft trees, **528** changed path
  entries and **602** distinct before-and-after blobs are portable without
  the base repository. Full trees verify change completeness. Unchanged files
  and commit history remain outside this archive; validation is unchanged. A
  fresh local bundle clone at `45cfd8f` passed the full verifier.
- **Final frontier entry:** **1** full queue row was printed at queue 4151.
  The next **5** derived writers target that row and report successful runs;
  five appended views are conditional on the recorded sequence being complete.
  Literal preparation fields from all **5** commands are now preserved and
  checked against their command bodies. The first update links an earlier
  **445-line** historical receipt to **382** verified displayed tail lines.
  The tail registration template is reconstructed with an unknown timestamp;
  exact final values and original registration bytes remain unavailable.
- **Late frontier source:** **7** source blobs now support **12** recorded
  observations and **755** displayed lines without a separate repository.
  Multi-file output headings are checked before comparing line numbers. A
  fresh same-machine bundle clone at `5fc6403` passed the full verifier.
- **Handoff instructions:** the central prompt and status now include the
  latest source and draft evidence and the user’s publication hold. A fresh
  same-machine bundle clone of `2b86b8b` passed the full verifier.
- **Lens 13 root 712:** **11** original transport files now preserve two
  delegated partial-read unions and their registrations and verifications.
  The later adoption receipts and root 712 qualification remain missing.
- **Lens 13 root 720:** **14** original transport files now preserve three
  delegated source records and associated qualification context. Four prior
  row excerpts were complete; the truncated fifth row was recovered from a
  later printed queue context. Full adoption receipts and packets remain
  missing. A fresh local bundle clone at `6405ffe` verified the earlier
  support files; the later row was added after that test.
- **Lens 13 root 715:** **5** earlier source rows and their printed packet,
  plan and writer references are preserved from a potentially truncated
  output. The full prior receipts and root qualification remain missing.
- **Lens 13 root 714:** **2** earlier source row excerpts and their printed
  packet, plan and writer references are preserved; full prior receipts and
  the later qualification remain unavailable.
- **Lens 13 root 719 proposal:** **18** literal source assessments and **1**
  proposed open question are preserved from its preparation command. Packet
  and plan hashes are recorded; their original bytes and adoption remain
  missing. No runtime failure or adoption is inferred.
- **Roots 714 and 720 proposals:** **25** literal source assessments, **1**
  literal open question and both packet and plan hashes are preserved from
  their recorded preparation commands. Full original records remain missing.
- **Roots 710, 711, 712 and 721 proposals:** **30** literal assessments,
  **1** open question and **4** printed packet and plan references are
  preserved. Root 710’s original plan is exact; the other plans and all four
  packets and adoptions remain missing.
- **Root 715 preparation:** **2** fresh assessment literals and **5**
  conditional reused assessments are reconstructed from its successful
  command and earlier rows. Missing qualification bytes prevent confirming
  the runtime inputs; packet and plan hashes are recorded, not their bytes.
- **Root 719 prior rows:** **3** complete printed rows match original
  delegated records. Three further rows in that output were truncated; their
  full prior receipts and the root qualification remain unavailable.
- **Late plan references:** all **14** roots have printed packet and plan
  hashes. **6** packets and **1** plan match recovered original bytes;
  **8** packets and **13** plans remain missing. This is not adoption proof.
- **Late post-apply references:** **14** historical self-reports contain
  **56** distinct ledger, selector, queue and selected-lens hashes. Their
  full state bytes were not found in the checked recovery files, and these
  earlier reports do not establish the final checkpoint.
- **Late verification references:** **14** historical self-verification
  summaries preserve their file hashes and reported scope counts. None of the
  original verification bodies was found; these are not independent reviews.
- **Late contract revisits:** **14** additional historical self-reports
  preserve their post-apply state and verification references. Each reported
  selected-lens pointer matches the preceding batch report; this does not
  restore the underlying state or exclude intervening changes.
- **New state-hash search:** **141** recorded plan, state, verification and
  final-checkpoint hashes were checked against **115,534** distinct nested
  JSON values and additional output strings in the private snapshot. No match was
  found; truncated output and evidence outside the snapshot remain untested.
- **Final Lens 13 plan candidate:** **4** predecessor references match the
  recorded queue 766 revisit report. The preceding state hash is still unknown;
  the original plan and intervening state remain unverified.
- **Recovery package:** a fresh same-machine clone of commit `cb08c68` from
  the verified local bundle passed the full verifier, including all deleted
  base versions. A fresh local clone of `6ea6fa5` also passed, including all
  selected-revision path proofs. An independent-machine restore is untested.
- **Base and public records:** 36,487 original files restored and 7,196 public
  progress files verified against their manifests.
- **Selected state:** snapshot 440 is exact; 10 of its 13 selected state files
  match their recorded hashes. The remaining 3 states and final checkpoint 3821,
  ledger and queue bytes remain missing. Five intermediate Lens 2 states are
  recovered, including state 027 and its linked receipt and policy. A ten-row
  Lens 13 predecessor discovery, registration, qualification, helper and source
  packet match their printed original hashes; an earlier 30-row discovery and
  its helper and 13-row registration also match. Two earlier qualification
  helper versions, including root 691’s, also match their original hashes.
  Root 691’s earlier discovery, registration, qualification, packet, plan and
  adoption receipt and successor state also match original hashes; the next
  plan is exact as well, and its qualification helper and **8** source
  objects are preserved. Root 576’s reuse qualification and fresh
  registration also match their original hashes. Root 693’s helper,
  registration, qualification and packet are exact.
  Root 371’s registration, packet, plan, adoption receipt and successor state
  also match their original hashes. Root 563’s clock reuse qualification
  and its **4** cited prior files also match their recorded hashes; its
  earlier three-source fixture registration and a separate three-source
  logging registration also match their original hashes.
- **Evidence:** 212 historical JSON versions match recorded hashes; all 67
  queue first-contract values in one Lens 2 comparison were directly observed;
  67 partial queue rows and 6 primary observations have writer-derived
  dispositions; 7 frontier registration structures are reconstructed. Five
  additional historical reuse rows, four detail sets and three exact prior
  receipt objects are preserved. All 8 root 691 source objects are present.
  A limited dependency inventory has 484 cited hashes still unmatched. These
  overlapping numbers are not an audit completion percentage.
- **Recent Lens 13 chain:** 19 consecutive adoption receipts and successor
  states, roots 691–709, match recorded or independently linked hashes. Root
  692’s original registration and qualification remain missing; the other
  18 registrations and qualifications in that span are exact. Root 696 has
  **1,672** displayed scoped lines; root 697 has **834**. Both have
  hash-linked successor plans. Root 698 has **705** displayed source lines,
  including **73** fixture lines, and a hash-linked successor plan. Root 699
  has **1,007** displayed lines and a hash-linked successor plan. Root 700
  has **5** exact preparation artifacts and **5,111** displayed lines. Root 701
  has **4** exact preparation artifacts and **35** fresh displayed lines; its
  prior adoption receipt remains missing; its qualification, packet and
  linked adoption are exact. Root 702 has **3** exact preparation artifacts
  and **365** registered delegated lines; its packet and adoption are exact.
  Root 703 has **4** exact preparation artifacts, an exact qualification and
  packet and adoption, and **36** delegated lines. Root 704 has **3** exact
  preparation artifacts and **245** fresh displayed lines; its qualification
  packet and linked adoption also match. Root 705 has **4** exact preparation
  artifacts, an exact qualification, packet and adoption, and **113** fresh
  displayed lines. Root 706 has **3** exact preparation artifacts and **168**
  delegated lines; its qualification, packet and linked adoption match. Root
  707 has **3** exact preparation artifacts, **4** frozen source objects and
  **30** fresh displayed lines; its qualification, packet, adoption and
  successor plan match. Root 708 has **4** exact preparation artifacts, an
  exact packet and adoption, **7** source objects and **913** displayed lines.
  Root 709 has **4** exact preparation artifacts, an exact packet, **4**
  source objects, **340** fresh displayed lines and **132** reused lines; its
  adoption and next plan are exact. Root 710 has **3** exact preparation
  artifacts, **5** source objects, **769** fresh displayed lines and **3**
  prior row extracts. Root 711 has **3** exact preparation artifacts, **6**
  source objects, **335** displayed lines and **2** prior row extracts. Root
  712 has **3** exact preparation artifacts, **6** source objects and **200**
  displayed lines. Root 713 has **3** exact preparation artifacts, an exact
  qualification and packet, **2** earlier native verifications, **8** source
  objects and **200** displayed lines. Root 714 has **3** exact
  preparation artifacts, **7** source objects and **498** displayed lines. Root
  715 has **3** exact preparation artifacts, **7** source objects and **111**
  displayed lines. Root 716 has **3** exact preparation artifacts, **14**
  source objects, **821** displayed lines, an exact qualification and packet,
  **22** prior support files. Root 717 has a checksum-matched
  native archive with **7** verified members, **2** earlier registrations,
  an earlier connected packet and adoption, an exact qualification and packet,
  and **4** preparation/provenance artifacts, **14** source objects and **538** displayed
  lines. Root 718 has **3** exact preparation artifacts, **16** source objects
  and **437** displayed lines; its qualification, packet and **27** prior
  support files match original hashes. Its pinned serde_json archive and
  **1** cited source member also match. Root 719 has **3** exact preparation
  artifacts,
  **18** source objects, **1,282** displayed lines, **18** prior support files
  and **8** native sources verified against the pinned Git commit. Root 720 has **3**
  exact preparation artifacts, **18** source identities and **713** displayed
  unique fresh lines. Root 721 has **3** exact preparation artifacts, **13**
  source objects, **425** fresh displayed lines and **12** prior support files;
  its discovery spans the final **30** registry entries. Its **1** cited native
  source also matches the pinned Git commit. Root 722 has **3** exact preparation
  artifacts, an exact qualification and packet, **10** source objects and
  **573** displayed lines. Root 723 has
  **3** exact preparation artifacts, an exact qualification and packet, **7**
  source objects and **409** fresh displayed lines.
- **Additional native objects:** **8** original registry source objects, indices
  31–38, match the preserved native Git revision; **2** more, indices 2589
  and 2590, match the pinned serde_json archive. Their recovery does not
  establish historical audit credit or current activation.
- **Drafts and sources:** 8 draft commits are preserved in an incremental bundle;
  786 Git source identities across seven manifests represent 691 distinct blobs.
- **Restoration:** a same-machine branch-specific bundle clone of `39e44f7`
  passed the verifier with `core.autocrlf=false`; **8** draft refs were restored
  and existing-destination refusal was checked again at `22ce462`. Seven source
  restorers recovered 691 distinct blobs locally.
  Independent-machine restoration remains untested.

## Detailed recovery inventory

- Root 692’s decoder source also matches the `serde_json` archive pinned by
  the frozen lockfile. This verifies the historical bytes, not current package
  activation or feature selection. Its registration remains missing.

- At `22ce462`, the draft bundle restored all **8** pinned refs in a fresh
  same-machine repository. A second run refused the existing destination and
  left its refs unchanged. The restored repository uses the source object store
  as an alternate; this is not an independent-machine restoration.

- A fresh same-machine bundle checkout of `39e44f7` passed the full verifier.
  A first checkout of `c73ff7c` exposed two source manifest line-ending
  mismatches; `39e44f7` preserves their original hashed bytes in Git. The
  checkout does not substitute for an independent-machine test.

- Root 716’s qualification, packet and **22** delegated support files match
  their original hash links. **4** earlier adoption receipts and root 716’s
  adoption remain missing.

- Root 713’s qualification, packet and **2** prior native verification records
  match their hash links. Root 692’s original registration and root 713’s
  adoption remain missing.

- Root 563’s logging registration matches its recorded hash, and all **3**
  referenced frozen source objects match. The registration alone does not prove
  adoption or current behavior; its packet and receipt remain missing.

- Root 723’s discovery, helper and registration match original hashes. The
  final seven source identities cover **409** fresh displayed and **47**
  earlier delegated lines. Programme role 2846 combines fresh line 216 with
  inherited 217–243; the prior line 244 is outside the new role. Qualification
  and packet match original hashes, as do **4** prior supporting files. Root
  042 adoption, root 723 plan and adoption remain unrecovered; final audit
  completion is unproven.

- Root 722’s narrowed discovery, corrected helper and registration match
  original hashes. Ten source objects cover **573** fresh displayed and **285**
  earlier delegated lines. Fixture source was not executed; a historical
  omitted original field remains absent. Qualification, packet and **5** prior
  supporting files match original hashes. Root 109 adoption and root 722 plan
  and adoption remain unrecovered.

- Root 710 depends on prior roots 641 and 644. Their two self-verification
  summaries survive, including hashes of the full reports, but those report
  bodies and the native verification and state references needed to reconstruct
  root 710’s qualification are not yet recovered. The summaries report passed
  source checks; they do not prove the later root 710 adoption.

- Root 721’s discovery, helper and registration match original hashes. Its
  discovery covers the final **30** registry entries; the registration covers
  the first **13**, with **425** fresh displayed and **1,965** earlier-read
  lines. The other **17** discovery rows provide navigation only. Qualification,
  packet and adoption remain unrecovered. **12** delegated support files and
  one printed root 563 source row are preserved; the root 563 receipt is missing.
  Native source 2827 matches the frozen Git commit stored with root 719.

- Root 720’s discovery, helper and registration match original hashes. Its
  **18** source identities cover **713** unique fresh displayed, **324**
  historical reused and **221** same-batch shared lines. Identical body bytes
  retain distinct source roles. Fourteen original delegated support files are
  preserved. Five prior row excerpts are complete; the initially truncated
  original 2805 row matches a later printed context. Later adoption receipts,
  qualification, packet and adoption remain unrecovered.

- Root 719’s discovery, corrected helper, registration and frozen lock match
  original hashes. Eighteen source objects cover **1,282** fresh displayed and
  **540** earlier-read lines. Native bevy_eidolon is pinned in the frozen lock;
  current activation remains unresolved. A local Git bundle preserves the
  pinned commit; all **8** cited native sources match its bytes. Registry byte
  fields stay absent.
  **18** delegated support files match, including four original records for
  an earlier connected source read; a prior verification is preserved as a
  printed extract. Qualification, packet and adoption remain unrecovered.

- Root 718’s discovery, corrected helper and registration match original
  hashes. Sixteen source objects cover **437** fresh displayed and **812**
  earlier delegated lines. Missing registry byte fields remain absent; source
  sizes are separately derived. Pinned serde_json source does not prove current
  feature activation. Its lock-pinned archive and source member match exactly.
  The qualification, packet and **27** earlier support files match recorded
  hashes; **5** earlier adoption receipts and the root
  718 adoption remain unrecovered.

- Root 717’s discovery, helper, registration and prior native provenance match
  original hashes. Fourteen frozen source objects cover **538** fresh displayed
  and **581** prior native lines. The original rand archive matches its recorded
  checksum and size; **7** cited archive members match the recovered objects.
  Two earlier registrations, the native verification, connected packet and
  connected adoption also match their original hashes. Their older stream
  packet and adoption remain missing. Current activation is unproven. Root 717
  qualification and packet match their hashes; its adoption remains missing.

- Root 716’s discovery, corrected helper and registration match original
  hashes. Fourteen source objects cover **821** fresh displayed and **1,294**
  delegated prior lines. The helper retains original Windows line endings;
  prior partial history stays attributed to the original readers. Qualification
  and packet match their original hashes; adoption remains unrecovered.

- Root 715’s discovery, helper and registration match original hashes. Seven
  source objects cover **111** fresh displayed and **960** earlier-read lines.
  The helper retains its original Windows line endings; original 2742 is a
  pinned dependency source, with native activation and current behavior open.
  Five prior source row extracts and their printed references are preserved;
  full prior receipts, qualification, packet and adoption remain unrecovered.

- Root 714’s discovery, helper and registration match original hashes. Seven
  source objects cover **498** fresh displayed and **544** earlier-read lines.
  Two prior source row extracts and their printed references are preserved.
  Historical SATS test source is not executed. Full prior receipts,
  qualification, packet and adoption remain unrecovered.

- Root 713’s discovery, helper and registration match original hashes. Eight
  source objects cover **200** fresh displayed and **369** earlier-read lines.
  Original 2728’s P16 design differs from its frozen PR counterpart. Generated
  bindings are source evidence only. Qualification and packet match their
  original hashes; adoption remains unrecovered.

- Root 712’s discovery, corrected helper and registration match original
  hashes. Six sources cover **200** fresh displayed and **1,607** earlier-read
  lines; three earlier root-personal rows are transcript extracts. Two
  delegated partial unions and **11** associated original transport files are
  preserved with their historical attribution. The later root adoption
  receipts, qualification, packet and adoption remain unrecovered.

- Root 711’s discovery, helper and registration match original hashes. Six
  source objects cover **335** fresh displayed and **56** earlier-read lines;
  **2** prior rows are transcript extracts. The 50 fixture-writer lines are
  source inspection only, not execution or seeding. Its qualification,
  packet, plan and adoption remain unrecovered. Root 710’s preceding chain
  remains incomplete, so this evidence does not close that gap.

- Root 708’s discovery, helper, registration, qualification, packet and plan
  match original hashes. The seven archived sources cover **913** fresh lines.
  One corrupted character in the display of original 2696 line 133 is restored
  from an escaped historical display and the hash-verified source. The
  adoption, successor state and next plan also match. No current behavior is
  implied.
- Root 709’s discovery, helper, registration, qualification, packet and plan
  match original hashes. Its four sources cover **340** fresh and **132**
  prior-read lines. The old root 434 native verification is exact; its
  adoption, packet and state remain unavailable. Root 709’s adoption, state
  and successor plan match.
- Root 710’s discovery, helper and registration match original hashes. Its
  five sources cover **769** fresh displayed and **271** prior-read lines.
  One partial role combines six earlier lines with fresh line 44. Three prior
  source rows are preserved as transcript extracts. The qualification, packet
  and adoption remain unrecovered.

- Root 707’s discovery, helper and registration match their historical hashes.
  Its four sources contain **300** material lines: **30** fresh lines verified
  against the historical display and **270** earlier-read lines. The earlier
  root 005 read completed the authority file after root 004; the new role
  claims only lines 24–43. Its qualification, packet, adoption, state and
  successor plan match; **11** prior supporting files are also preserved.
  Earlier adoption and root 319 packet/state bytes remain unavailable. These
  archived sources do not establish current behavior.

- Root 706’s exact discovery, corrected helper and registration link **2**
  archived sources and **168** delegated lines, with **0** fresh lines. Its
  discovery preserves **5** prior hits, **2** future proofs and **3** earlier
  applications. Qualification and packet match; **4** shared prior supporting
  files are preserved. The next exact plan verifies its receipt and state.
  Earlier root 053 and application receipts remain missing.

- Root 705’s **2** exact discovery records, helper and mixed registration link
  **2** archived report-P17 sources and **113** fresh displayed lines. Two
  manifest lines retain earlier delegated attribution; the root 701 alias
  application is exact, while the older root 023 adoption remains missing.
  Qualification, packet, receipt and state match; the next exact plan verifies
  the latter two.

- Root 704’s exact discovery, corrected helper and mixed registration link **6**
  archived report-P17 sources and **245** fresh displayed lines. Another **150**
  registered lines retain earlier delegated and root attribution. Its
  qualification and packet match; **4** delegated support files are preserved.
  The next exact plan verifies its receipt and state; earlier qualifications
  and adoption files remain unavailable.

- Root 703’s **2** exact discovery records, helper and registration link **2**
  frozen sources and **36** delegated lines, with **0** fresh lines. Its narrow
  discovery preserves **4** prior hits, **2** future proofs and the original
  2056 application. Its qualification and packet match; **7** prior support
  files are preserved. The next exact plan verifies its receipt and state.
  Root 212 and the old 2056 application receipt remain missing.

- Root 702’s exact discovery, patched helper and registration link **2**
  archived source objects and **365** delegated material lines, with **0**
  fresh lines. Its qualification, packet, receipt and state match, and **8**
  prior supporting files are preserved; the next exact plan verifies the
  receipt and state. The two earlier adoption receipts remain missing.

- Root 701’s **2** exact discovery records and helper preserve **7** prior-hash
  hits, **2** future proofs and **1** prior application. The narrowed discovery
  keeps only the original 2671 hit. Its mixed registration, **4** source
  objects, **35** fresh displayed lines and **4** supporting prior files are
  exact. Its qualification, packet, receipt and state also match; the next plan
  verifies the latter two. The older prior adoption receipt remains missing.

- Root 700’s **2** exact discovery records, helper, registration and qualification
  link **7** frozen sources and **6** historical displays covering **5,111**
  lines: **668** production and **4,443** fixture source lines, with no test
  execution. Original 2671 had a prior-receipt hit and is excluded; the P16
  manifest is historical. Its packet, receipt and state are exact; the next
  plan verifies the latter two and preserves the separate delegated reuse.

- Root 699’s exact discovery, helper, registration, qualification, packet, plan,
  receipt and successor state link **8** frozen source objects and **4**
  displays covering **1,007** lines: **840** production and **167** fixture
  source lines, with no test execution. A corrupted glyph was redisplayed as
  escaped Unicode. The next exact plan verifies the receipt and state.

- Root 698’s exact discovery, helper, registration, qualification, packet, plan,
  receipt and successor state link **8** frozen source objects and **3**
  historical displays covering **705** lines: **632** production and **73**
  fixture source lines, with no test execution. The next exact plan verifies
  the receipt and state.

- Root 697’s exact discovery, helper, registration, qualification, packet, plan,
  receipt and successor state link **8** frozen source objects and **3**
  historical displays covering all **834** scoped lines. The next exact plan
  verifies the receipt and state.

- Root 696’s exact discovery, helper, registration, qualification, packet, plan,
  receipt and successor state link **8** frozen source objects and **8**
  historical displays covering all **1,672** scoped lines. The next exact plan
  verifies the receipt and state.

- Root 695’s exact discovery, plan, helper, registration, qualification, packet,
  receipt
  and successor state link **8** source objects and **7** historical displays
  covering all **1,546** scoped lines. The next exact plan verifies the receipt
  and state.

- Root 694’s exact discovery, helper, registration, qualification, packet, plan,
  receipt and successor state link **8** source objects and **3** historical
  displays covering all **666** scoped lines. The next exact plan verifies
  the receipt and state; sampling tail 44–71 remains outside the scope.

- Root 693’s exact discovery, helper, registration, qualification, packet,
  plan, receipt
  and successor state link
  **8** source objects and **5** historical displays covering all **1,030**
  scoped lines. The excluded incidental helmet line is preserved as historical
  metadata, and the exact following plan verifies the successor state.

- Root 576’s exact reuse qualification and fresh registration link **4**
  prior files and **6** source objects. **9** historical displays cover all
  **887** fresh lines; **456** inherited lines retain their prior attribution.
  The historical 2104 range overrun remains explicitly qualified. Its packet,
  plan and receipt are still missing.

- Root 692’s exact packet, plan, adoption receipt, state and qualification
  helper are recovered with **8**
  source objects. One untruncated display matches **360** material lines
  across **2** roles. The packet writer preserves **8** assessments and **6**
  remaining obligations. Displays for the other six roles have not been located; the original registration,
  qualification remains unavailable. The exact next plan verifies the state.

- Root 563’s earlier fixture registration matches its original hash. **9**
  displays cover **871** lines across **3** sources; the **163** fixture lines
  overlap the later root 563 role. Its packet writer preserves **6**
  observations across **4** source roles, but the packet is missing.

- Root 563’s clock-alias qualification matches its original hash and links
  **4** exact prior files to a **29**-line inherited source. It carries no new
  source credit; root 563’s packet, plan and adoption receipt remain missing.

- Root 371’s exact registration, packet, plan, adoption receipt, state and
  native verification are recovered, along with **9** source objects and
  **9** historical displays covering **2,178** lines. The packet writer
  preserves **24** observations. Its **143**-line source and **2** observations for one
  role overlap root 691’s later reuse. Historical displays confer no new
  credit.

- A historical root 691 display matches **264** source lines across **3**
  sources. It also preserves **5** prior-reuse references, **1** root
  observation set and **3** delegated inventory and contract sets. Its
  earlier discovery output identifies **10** prior-hit rows across **7**
  receipts. Three earlier source metadata rows are preserved; two match
  complete base receipts; the third matches the recovered root 371 receipt.
  A third exact base receipt covers two further reused roles. The discovery,
  registration, qualification, packet, plan and receipt match original hashes
  after recovering an earlier ordered application. Two source hashes link the display to **114**
  later role-lines in the root assessments counted below. The successor state
  is verified by the exact next plan. Its qualification helper matches
  the original hash, all **8** registered source objects are hash-verified,
  and its writer is a candidate without a known original hash. No new credit
  or current reconciliation follows.

- **1** earlier root-read registration now matches its printed original hash;
  its **3** source objects and **7** displays covering **683** historical lines
  are preserved. The **89** lines later reused for role 2824 overlap the root
  assessments counted below. A printed preflight cites the root 563 receipt
  hash, but the receipt itself remains missing.

- **2** historical displays match all **425** fresh lines across **3** earlier
  Lens 13 roles. Together with **1,762** delegated material lines linked to
  original evidence and **203** root-assessed lines, the **2,390** registered
  material role-lines are described. One root-adoption receipt and the full
  qualification remain missing; these are historical reads, not new credit.

- **15** original inherited evidence objects substantiate **7** delegated
  Lens 13 roles covering **1,762** historical material lines. All **3** root
  assessments cover their **203** requested lines; two full row projections
  and one truncated row are preserved. Root-adoption receipts and the later
  qualification remain missing, and no new source credit is granted.

- An earlier Lens 13 **registration** matches its printed hash, with **13**
  source objects and **26** nested range fields preserved. It records **425**
  fresh and **1,965** previously read material lines as historical scope; its
  qualification and adoption plan remain missing, and no new credit follows.

- An earlier Lens 13 discovery and qualification helper match their printed
  original hashes. The discovery preserves **30** registry rows, **14** prior
  receipt hits and **3** future-reuse references; its 10-row overlap with the
  later discovery matches exactly. This metadata does not add body credit.

- **1** earlier Lens 13 plan candidate links the recovered packet and
  qualification to **4** preceding printed authority references. Its state
  721 hash is unknown; none of **16,973** SHA-shaped transcript tokens matched
  the printed plan hash under this candidate. Unobserved changes or unprinted
  hashes remain possible; the original plan and state are not recovered.

- The exact Lens 13 predecessor **source packet** matches its printed hash.
  Two preserved displays match **573** historically fresh source lines across
  **8** roles; **285** lines across **2** other roles retain inherited
  attribution. The packet is historical evidence, with no new audit credit.

- The exact preceding Lens 13 **registration** and its **qualification
  helper** match printed original hashes. The registration links **10**
  original source roles, **20** nested range fields, **573** historically
  registered fresh lines and **285** prior material lines. Its exact
  qualification and **16** source/inherited objects are also preserved.
  Predecessor state remains missing; no new read credit is granted.

- **1** Lens 13 predecessor discovery record, covering **10** original
  registry entries, now matches its printed original SHA-256 after a bounded
  timestamp search. Its metadata records **2** prior receipt hits and **1**
  future-reuse proof; it grants no source-body credit. A scan of **7,196**
  public progress files found no state 722 filename or snapshot 414 hash.

- **1** partial Lens 13 plan candidate combines the exact recovered packet and
  qualification with **4** earlier printed authority references. The state 722
  predecessor hash is unknown. None of **16,973** SHA-shaped transcript tokens
  reproduced the printed plan hash under this candidate; intervening changes
  and other candidate values remain possible. Original plan and state bytes
  remain missing, and the private-transcript search is not reproducible from
  this package alone.

- **7** source restorers succeeded in fresh directories against the local Git
  repository; their **812** overlapping manifest rows produced **691** distinct
  hash-verified blobs. Each refused an existing destination on a second run.
  Remote object availability and independent-machine restoration remain untested.

- A hash scan of **18,084** private extracted files found **0** matches for
  the **484** hashes then unmatched in the limited inventory. Seven pinned Git
  manifests and hash fields from **2,727** preserved helper/literal candidates
  also produced no matches. The raw transcript, index and other stores were outside
  this scan; this is not proof the missing bytes cannot be recovered.

- **4** cited hashes from the earlier dependency inventory match files already
  preserved elsewhere in the recovery package. The limited inventory now has
  **44** later matches and **484** unmatched hashes; matches overlap other
  collections and establish historical bytes, not current authority.

- **7** recorded process intervals and private transcript-event hashes are
  preserved for frontier registration writes. They are harness timing, not the
  original registration timestamps; clock agreement is unverified. A scan of
  command-output fields in the available snapshot found no printed occurrence
  of the seven registration filenames and therefore no original hashes there.
  The raw snapshot is private and its completeness is unverified.

- A same-machine clone of bundle `f1ee871` passed the verifier with a clean
  worktree. This includes the latest frontier registration templates and does
  not establish independent-machine restoration.

- **7** latest-frontier registration structures covering **12** source rows
  are reconstructed as templates from recorded writers and pinned-source
  traces. Their original timestamps, hashes and bytes are unavailable; the
  templates do not establish authority, additional audit credit or validation.

- **6** primary candidate observations preserve their printed first-contract
  and owner fields, plus writer-derived dispositions. The historical command
  failed after printing them. **12** recorded intervening commands are
  read-only, but unrecorded activity or ledger changes are not excluded, so
  these do not prove the original comparison fields.

- **67** queue comparison rows combine directly printed candidate identities
  and first-contract fields with dispositions derived from the recorded writer.
  **7** row lenses were directly observed; the other lens values and full
  original candidate objects remain unresolved. This is a partial projection,
  not the missing original comparison or a new audit finding.

- A fresh same-machine clone of bundle `0ac5faf` passed the verifier and had
  a clean worktree. A clone without `--branch` did not check out files because
  the bundle has no default `HEAD`; the documented branch-specific command
  succeeded. Neither test establishes independent-machine recovery.

- **11** historical reuse roles link to **10** original registrations matching
  printed hashes. **3** same-batch shared roles cover **223** lines already
  present in captured parent displays. **18** original state, receipt and
  verification objects link all **766** prior role-lines to earlier reads.
  Their receipts also link **72** references to **65** hash-verified direct
  dependency objects. The later full qualification remains missing; no new
  credit is inferred.

- **66** final Lens 12 registry scopes and all **62** distinct referenced
  source objects are preserved. **66** captured display fragments cover
  **5,795** unique historically fresh role-lines across **52** roles, with
  **975** repeat occurrences. The **6,784** total material role-lines also
  include prior and shared evidence; no new credit or closure is inferred.

- **7** original Lens 12 final-batch registrations now match hashes printed in
  the partial state 960 prefix, with **7** corresponding source objects verified.
  Four recorded source displays match **611** historical lines, and their
  batch qualification helper matches its printed hash. **4** printed batch
  source rows link those displays to the exact individual registrations. These
  recoveries confer no new credit; full state 960 and the selected Lens 12
  state remain missing.

- **1** original publication kernel matches its recorded hash; its successor
  is reconstructed by the recorded deterministic replacement. The source
  asserts preservation of unrelated queue entries, and apply/verify output
  reported success for the entry 2342 update. This indirectly supports the
  separately observed entry 3720 predicate, but missing queue and wrapper
  bytes prevent a definitive proof of historical queue equality. The original
  wrapper also matches a printed hash; its later patch and successor import
  are reconstructed as candidates without an independent successor hash.

- A same-machine clone of `0dd01f7` passed the full verifier, including the
  exact state 026 and its linked original SDK registration checks.

- **36** comparison candidates are linked to separately printed owner
  observations, including **34** first-contract values. Those observations
  are not proven to be the exact fields in historical queue 828.

- **1** original five-record comparison matches its separately printed hash.
  Its referenced candidate index and historical queue hashes are recorded,
  though their original bytes remain missing. The later 73-record comparison
  and final Lens 2 state remain unavailable.

- A same-machine clone of `d1a6342` passed the full verifier, including the
  latest comparison, state-candidate and nested source-evidence checks.

- **15** scoped queue comparison dispositions and reasons are extracted from
  the recorded comparison write and linked to its printed candidate list.
  The original comparison document remains unavailable; no alias is inferred.

- **4** final Lens 2 state 028 templates are preserved as unverified candidates.
  Its predecessor is exact, but the original comparison bytes and hash remain
  missing, preventing exact state validation.

- **4** state 027 templates reproduce the exact state, receipt and policy at the
  matched timestamp. The recovered policy predicate matches the direct queue
  828 projection; original queue files remain unavailable.

- **3** state 026 reconstruction templates reproduce the exact state, receipt
  and policy hashes at the matched timestamp. The prior incorrect-target
  search is superseded by this successful independent hash match.

- **6** source files for the nested Lens 13 questions match the original
  registry hashes. **713** historically displayed lines match four captured
  fragments; **2,932** lines were recorded as prior same-lens reuse. **8**
  linked original receipts, verifications and registrations substantiate
  those **3** reuse mappings. Later registration and packet bytes remain
  missing, with no new credit inferred.

- **2** complete printed nested Lens 13 question projections and their proposal
  references are preserved within historical queue entry 1880, whose declared
  parent lens is 12. Proposal bytes and final queue state remain unavailable;
  no foreign-lens verdict or credit is transferred. Printed references also
  identify **2** qualifications and **2** writers; their original bytes remain
  unavailable, with **4** static write expressions preserved. **8** printed
  source-registration, qualification, packet and plan references are linked;
  their original files and current adoption authority remain unverified.

- A same-machine clone of `ee79e5a` passed the full verifier, including the
  newly recovered journey, native and contract source-display fragments.

- **850** historically displayed native-admission source lines match pinned
  objects: **511** SDK lines across **4** fragments and **339** Git-source
  lines across **5** fragments. This adds no audit credit; the original
  native-admission receipt remains missing.

- **8** printed field projections from historical closure queue 828 preserve
  additional identity, scope and remaining-evidence details. They are partial
  entries from that queue, not the final queue or evidence of closure.

- **6** printed primary first contracts and owners and **45** queue candidate
  entries are linked to the comparison inventory. **44** queue first-contract
  values were printed in full; one was suppressed by that historical display.
  Another **22** queue 828 first-contract projections were printed in full.
  A separate, untruncated queue 828 read printed the suppressed value and
  matches the queue 815 candidate, bringing direct observations to **67 of
  67**. This establishes that field only, not complete queue-entry equality.

- **73** printed canonical-comparison candidates, **6** reported inherited
  comparisons and the original **173**-finding master ledger are preserved.
  The recorded output reports **5** M40 locations covered; the comparison
  document and its original hash remain missing, and no alias or new credit is
  inferred. The preserved master matches the hash printed immediately before
  the comparison; the recorded comparison asserted its master matched its
  then-current ledger.

- The central index and source locators include the five exact Lens 2
  intermediate states and the latest registration evidence. The limited
  dependency-inventory remainder is synchronized at **484**.

- **2** printed Lens 2 contract-support registration references and **5**
  source objects are preserved, covering **928** uncredited historical lines.
  Original registration file bytes remain missing; no whole-case closure or
  runtime witness follows from these source records. **8** recorded display
  fragments match all **928** distinct lines, with **3** repeat occurrences.

- **3** original SDK registration files, including a rejected range, match
  hashes linked through the exact state 026 receipt. Their **2** archives
  match the pinned lockfile, and **3** source
  objects match archive members. **2** more printed Git registration references
  and **5** Git source objects are preserved; all reads were uncredited.

- The `817fa92` bundle clone passes the verifier with the documented Git
  `core.autocrlf=false` setting. A trial clone with `core.autocrlf=true`
  failed verification on a converted queue fragment. Follow the documented
  clone command; do not rely on the destination machine's Git defaults.

- **3** printed Lens 2 journey-support registration references and source
  summaries are preserved, with **2** source objects matching their printed
  hashes. The registrations cover **270** uncredited lines; their original
  registration file bytes remain missing. State 024 and its referenced read
  match their recorded hashes. All **401** lines in that read match the
  preserved historical displays and pinned source objects.

- **5** Lens 2 intermediate states (023–027) and their linked receipts or
  policies match recorded hashes. The final selected state 028 remains missing.
  The journey policy remains a candidate without an independent original hash.

- **1** fresh same-machine clone of `c358f3e` passed verification. An
  intentionally corrupted binary object was rejected, then verified after
  restoration; this tests that object path, not every corruption scenario.

- **3** Lens 12 follow-on qualification, policy and view expressions are
  preserved with literal fields and unresolved dependencies distinguished.
  Their original output files and state bytes remain missing.

- **13** original Lens 12 SATS evidence objects and **4** source rows are
  linked; inherited and fresh ranges together cover **2,523** historical
  lines without new credit. The later qualification remains missing.

- **60** printed lines from the Lens 12 pre-metadata baseline preserve its
  predecessor, receipt and qualification hashes; full baseline bytes remain
  missing.

- **8** Lens 12 metadata state path-and-hash references are preserved from
  recorded output; their referenced state bytes remain unrecovered.

- **45** earlier source objects are preserved for the modified paths in the
  oldest transition; its remaining **109** paths were additions.

- **3** linked current-source transition files are preserved, covering
  **154**, **59** and **31** historical changed paths respectively. The first
  transition and its **148** distinct source objects overlap the original
  handoff; later per-lens queue state remains unverified.

- **1** later transition matches its recorded hash. Its initial **31** paths
  and **403** pending cells are preserved; later cell status is separate.

- **31** later transition paths and **53** distinct source objects match the
  recorded diff; the original transition now matches its recorded hash.

- **2** late current-delta closure counts, **767** and **403**, appear in
  checkpoint 3821 writer assertions; all **3** recorded modes reported pass.
  This is indirect evidence because the final queue bytes remain missing.

- **8** historical current-cell checkpoint reports and **25** named cells at
  checkpoint 509 are traced to the recovered transition. The final queue and
  its later cell statuses remain unavailable.

- **1** historical current-source transition is recovered to its recorded hash;
  the final recorded writer asserts **767** and **403** closed cells in two
  deltas, but their individual final queue states are not independently verified.

- **59** recorded current-transition paths match the pinned Git diff; **96**
  distinct before/after source objects are preserved. The original transition
  file now matches its hash; final per-lens closure state remains missing.

- **409** recorded final Lens 13 source line occurrences in **2** output
  chunks match frozen source and packet ranges; historical attribution remains.

- **1** final Lens 13 qualification and **1** source packet match their
  recorded hashes and link to the recovered registration and prior receipts.
  Their cited ledger, selector and selected-state bytes remain missing.

- **1** final Lens 13 source registration matches its recorded hash, with
  **7** source objects and **14** nested range fields preserved.

- **1** Lens 13 qualification helper matches its recorded original SHA-256;
  it remains inert and its output qualification remains missing.

- **7** original Lens 13 inherited evidence objects are linked to the final
  packet discovery: two source objects, receipt, registration and supporting
  verification. They overlap the restored original handoff.

- **2** Lens 13 successor helper texts are reconstructed from recorded
  literal transformations and checked statically; required input bytes remain
  missing and the historical helpers were not executed.

- **1,017** recorded building-audit source line occurrences across **8**
  sources are preserved; **8** displayed lines contain replacement characters.
  Their original registration and packet remain missing.

- **1** fresh same-machine clone of commit `59c62c4` passed the verifier;
  **8** draft refs were restored and both restorers refused existing paths.
  Independent-machine restoration remains untested.

- **7** final Lens 13 source-row observations and **7** reported construction
  and adoption outputs are traced; selected state bytes remain missing.

- **1** Lens 12 selected-state construction is traced to its preserved helper
  and **3** reported passing modes; predecessor and target bytes remain missing.

- **6** recorded Lens 2 transitions and reported verifications are traced
  through state 028; dynamic inputs and exact final state bytes remain missing.

- **625** recorded building-audit source line occurrences across **2** chunks
  match their pinned source and packet ranges; no new credit is asserted.

- **2,374** recorded profile source line occurrences across **8** chunks are
  preserved: **2,373** match pinned source, and **1** contains a decoding
  replacement character. Ranges may overlap; no new credit is asserted.

- **3** capture-profile registrations and **3** read packets match recorded
  hashes; their **2,374** assessed line occurrences carry no new credit.

- **1** corrected building read packet matches its printed hash; its **11**
  displayed lines match pinned source and carry no new audit credit.

- **2** more building issue registrations match recorded hashes: an invalid
  range and its corrected successor, explicitly preserving non-adoption.

- **2** additional source registrations and **1** read packet match recorded
  hashes; the packet records **625** line occurrences with no new credit.

- **2** additional repair proposals preserved as recorded literal fields and
  expressions; their exact original JSON and dependency references remain missing.

- **13 of 13** historical snapshot 440 selections and its exact original bytes
  are recovered; **10** referenced state files match their cited hashes.
- **1** Lens 2 qualification reconstructed to its recorded hash from six
  original receipts, covering 21 historical source rows without new credit.
- **19** recorded transformations trace the final checkpoint writer from a
  printed predecessor; 19 recorded checkpoint runs reported success.
- **1** historical view-mapping repair contract recovered to a printed hash
  and linked to the recovered animation certification contract.
- **2** capture-owner records recovered to printed hashes: a repair contract
  and a comparison preserving **5** distinct case facets.
- **2,531** recorded capture and fingerprint source line occurrences across
  **9** output chunks match pinned bytes; ranges may overlap.
- **4,674** recorded building source line occurrences across **17** output
  chunks match pinned bytes; overlapping ranges do not count as unique credit.
- **3** building read packets recovered to their printed hashes and linked
  to their proposal and registrations; they record zero fresh credit.
- **3** building source registrations and **20** distinct pinned source
  objects recovered; their recorded scopes grant no new audit credit.
- **2** fingerprint registrations and **2** read packets recovered, linked
  to **6** pinned source objects; three overlap other manifests.
- **1** capture fingerprint proposal recovered to its printed hash; it
  records **8** repair requirements and **6** later acceptance items.
- **1** building capture identity proposal recovered to its printed hash;
  it records **12** requirements and **7** later acceptance items.
- **1** capture-callers source registration and **1** read packet recovered,
  with **4** pinned source objects and no new audit credit.
- **1** later capture-callers repair proposal recovered to its printed hash
  and linked to its previously recovered predecessor.
- **3** related capture-family repair contracts recovered to printed hashes;
  animation, plant diagnostics and remaining callers remain unimplemented.
- **2** historical capture repair contract versions recovered to printed
  hashes; the successor adds native cause and buffered-completion requirements.
- **1** prospective asset-event source plan recovered to its cited hash;
  its existing source bytes are archive-verified and carry no new credit.
- **1** historical Cargo.lock delta recovered and independently reproduced
  from two pinned Git revisions; it adds no audit credit.
- **9** more historical Lens 13 states match printed hashes, with state,
  predecessor and receipt reference fragments preserved.
- **6** additional historical Lens 13 states recovered to cited hashes;
  a seventh matched a state already preserved.
- **2** historical Lens 13 lineage files recovered: state 388 and its plan;
  the predecessor and receipt are identified by hash but remain missing.
- **2** qualification-linked registrations recovered to their cited hashes,
  with **7** preserved source objects; **6** overlap prior source manifests.
- **4** additional printed audit records match cited hashes, including one
  unimplemented AI-precision repair plan and its qualification.
- **9** additional historical program sources match cited hashes; they are
  preserved as inert evidence and have not been executed.
- **8** pinned draft commits are preserved in a verified incremental Git
  bundle; restoration requires the recorded base repository object store.
- **786** deduplicated Git source identities, representing **691** distinct
  blobs, are listed across seven verified restoration manifests.
- **83** distinct dependency or external source blobs are preserved directly.

- **36,487** original handoff files restored and hash-verified.
- **7,196** public progress files checked against the pinned manifest.
- **81** additional dependency files matched to historical cited hashes. A
  limited inventory originally did not locate **528** distinct cited hashes;
  **44** have since been matched and **484** remain unmatched in that inventory.
  This does not prove the remaining bytes cannot be recovered.
- **1** historical helper candidate matches a printed filename/hash line;
  it overlaps other preserved candidates and remains inert.
- **19** JSON bodies match filename/hash lines printed by historical
  commands; original paths and current authority remain unverified.
- **6** linked text objects match citations in recovered records, including a
  late-audit note and one unreviewed historical helper.
- **10** linked JSON bodies match references in recovered parents, including
  checkpoint 476 and a checkpoint 3778 registration.
- **130** further JSON bodies match separately printed references after
  excluding known original, progress and recursive objects; categories overlap.
- **7** native source objects match complete row fragments in recorded
  outputs and four checksum-verified archives; surrounding outputs may be partial.
- **15** additional Git source identities match alternate-field output
  rows, including binary assets; they may overlap other collections.
- **248** further Git source identities (**231** blobs) match complete
  output rows; **2** external source files were preserved from a local cache.
  These identities may overlap other source collections.
- **3** rawzip source objects match printed registration rows and a
  checksum-verified local archive; the registration file remains missing.
- **53** native source objects from **27** checksum-verified archives
  match citations in normalized printed audit documents.
- **6** native tangent source objects match their recovered registration
  and a checksum-verified archive.
- **10** additional native source objects from **8** checksum-verified
  archives are preserved against recovered registration citations.
- **2** native Bevy source objects are preserved and match their cited
  hashes and the pinned crate archive checksum.
- **43** supplemental source identities are verified and restorable from
  pinned Git; they may overlap source identities in other collections.
- **1** complete printed Python qualification is preserved as normalized
  structured data; its original file bytes and authority remain unverified.
- **8** additional registrations match recorded output citations,
  including errantry, injury, inventory and AI source registrations.
- **1** Lens 7 chunk registration matches a citation in a sliced historical
  queue output; the surrounding entry and final queue are not restored.
- **1** Lens 3 chunk registration matches a hash in a normalized printed
  source packet; the packet’s original bytes are not established.
- **15** additional historical registrations match citations in recovered
  parents, covering material, rendering, scene readiness, terrain and Lens 11.
- **3** checkpoint 3802 source observations, totaling **475** printed
  lines across **2** source files, match their registrations and Git bytes.
- **1** terminal-interpolation registration was reconstructed to the hash
  cited by a previously recovered checkpoint 3802 qualification.
- **3** original-handoff dependencies are linked and hash-verified against
  the latest fully printed pre-adoption queue entry; they overlap the 36,487.
- **1** additional historical raw-record candidate is preserved from Git;
  its citation has no hash, so its identity at the time remains unverified.
- **1 of 3** dependencies cited by the historical Lens 4 receipt matches
  its original hash; two same-path candidates have different hashes.
- **1** historical Lens 4 receipt was restored byte-for-byte from a
  Git object and matches the hash asserted by its recorded copy command.
- **12** connected late source observations, comprising **755** printed
  lines across **7** pinned files, match historical Git source. The final
  frontier accounts for 4 observations and 387 lines; counts can overlap.
- **5** late checkpoint preparations and their reported execution results
  are linked to preserved notes; original authority files remain missing.
- **43** audit records reconstructed to printed hashes, including a
  lens 2 proposal with **59** source rows, **37** scoped contract records
  and **16** pending records, plus its historical successor state.
- **17** registrations reconstructed to hashes printed by their historical
  commands, beyond the inventory of references in the 212 files.
- **4** further dependencies recovered through citations in other recovered
  dependencies; these are earlier snapshots, not final state.
- **212** historical JSON file versions reconstructed to recorded path and
  SHA-256 references; their latest-state authority and dependencies remain open.
- **6,233** source/registration-related records published; **338** more
  redacted records are prepared locally. The user subsequently approved their
  export and requested further publication wait until recovery is complete.
  The **57** late records overlap.
- **5,623** audit write command bodies preserved without replay; **265**
  have paths redacted. **359** late command/output records, including **17**
  failures, are also preserved; the **70** late write commands overlap.
- **37** unresolved patches for **30** paths preserved without applying.
- **54** other Rust, JSON, TOML and JavaScript literal writes preserved.
- **89** other file-change candidates preserved; **4** copied-source
  candidates have verified upstream locators, and **1** local-path candidate
  is preserved in redacted form.
- **509** draft validation-related command candidates preserved;
  command selection and captured output do not establish acceptance.
- **287** implementation draft file-change records preserved across seven
  worktrees; their recorded changes preceded recorded branch pushes.
- **310** historical Python literal-write drafts preserved, including **5**
  with paths redacted; validation remains unverified.
- **2,363** unverified helper candidates preserved as inert text, plus
  **23** with machine-specific paths redacted; their originals remain private.
- **155** historical Markdown write fragments preserved, overlapping with
  other recovered notes; appends are individual fragments.
- **19** late audit notes and **5** queue or lens values preserved as
  transcript-derived text or normalized values, not original file bytes.
- **3,146** normalized printed contract values spanning **1,722** distinct
  contract IDs preserved; values overlap and may be field projections.
- **4,080** queue-related outputs and command contexts preserved;
  **0** records remain deferred.
- **1,699** ledger-related outputs and matching command bodies preserved;
  machine-specific paths were redacted where present.
- **117** printed ledger-related JSON values and arrays preserved as historical fragments.
- **538** normalized printed audit documents preserved, plus **8** with
  personal paths redacted; their original normalized bytes remain private.
- **483** source identities, representing **432** distinct blobs, verified
  across the earlier three source-restoration collections.
- **387** recorded source lines at the final frontier match four pinned files;
  the original registration bytes and final authority remain missing.

These categories overlap. They are not a percentage of the audit completed or
an independently verified number of unique findings.

## What remains

The continuation's final ledger, closure and delta queues, selected-state
pointers, complete registrations and receipts, and exact checkpoint 3821
remain unrecovered. A reported checkpoint or source hash alone cannot restore
its contents. The private transcript snapshot contains additional candidate
file changes, command outputs and literal writes; not all have been examined
or safely published. The original audit goal is paused and no repair work or
new audit credit was performed during recovery.

The [recovery README](README.md) and [resume instructions](AUDIT-RESUME.md)
explain provenance, restrictions and restoration. The older [progress
report](PROGRESS.md) is a historical audit report, not this recovery report.
A fresh shallow clone of commit `181691738a0c494159c4537e2055486a200f8c75`
verified the published evidence collections, including 236 individually
manifested files, 3,146 contract values and 538 printed documents. A separate original-source clone restored 268 source
identities and rejected an existing destination. These checks do not restore
or verify the missing final ledger.

A second private copy of the bounded transcript snapshot was created in a
separate local workspace and verified against its SHA-256. It is on the same
machine, is not a remote or independent backup, and must not be published
without privacy review. A same-machine Git bundle at commit `dd285873` was
independently restored and verified; later local commits require a newer
bundle. A local archive verification also passed for the unpublished packet.
The search-index builder produced 27,671 overlapping searchable records and
refused an existing destination. The index is a navigation aid only.

Next, continue tracing transcript outputs and remaining file-change candidates
against known hashes and references. Publication is on hold until the user
authorizes it. Preserve the raw transcript privately and retain each record's historical source revision.

Six additional registrations were reconstructed from recorded helper
templates and bounded timestamp searches. All six match independently cited
hashes; their source rows added 11 Git-verified identities. The source-object
restorer verified all 313 identities in a fresh destination.

A same-machine bundle verified at `e340b081a1e9833cdf08f96633c59d2b18d1f8dd`
contains earlier local recovery commits; it predates the six registrations
above and is neither a remote nor an independent backup.

Three historical probe receipts were reconstructed from printed output and
matched their cited hashes. They document the historical probes and do not
establish that the full implementation passed acceptance.

A scan of successful command outputs recovered 37 more JSON values matching
cited hashes, including registrations, qualifications and verification
records. The output does not independently establish their historical paths,
latest-state authority or acceptance.

Two recovered registrations supplied four additional source rows; all four
matched their recorded Git hashes. The restorer verified 317 registered
identities in a fresh destination.

Four further historical dependencies were found by following citations from
recovered dependency objects. Their hashes and parent citations are verified;
they include earlier selected-lens and lens-state snapshots, not final state.

The package-wide verifier now checks the four recursively recovered objects
and their hash-verified parent citations; the central README links the set.

The search index now includes 80 text dependency objects and four recursive
dependencies; the compressed crate remains hash-verified but is not indexed.
The latest local bundle covers commit `f84e912`; it is on the same machine
and predates subsequent local commits. The registration count is corrected
to 22 timestamp-reconstructed files.

A prospective lens 2 catalog-owner registration was reconstructed from its
recorded template, Git source bytes and bounded timestamp. It matches the
hash printed by its historical command; this does not establish current
authority or grant audit credit.

Twelve more lens 2 support registrations match hashes printed by their
historical commands. Reconstruction used recorded template changes and Git
bytes; a later patch conflict stops the batch and remains unresolved.

Across the printed-hash registrations, 48 additional source identities
were added to the registration manifest. A fresh restoration verified all
365 identities and 347 distinct objects in that manifest. The search index
contains 28,127 overlapping records after these additions.

The lens 2 catalog-owner read and linked qualification were reconstructed
from exact registrations and recorded templates. Both match hashes printed
by the historical command; the qualification preserves 19 comparisons and
their open obligations. No current authority or fresh audit credit is claimed.

Five more lens 2 read records and their five linked qualifications were
reconstructed from recovered registrations and recorded templates. All ten
match hashes printed by their historical commands; their open obligations and
no-credit limits remain intact.

The proposed lens 2 state 016 was reconstructed exactly from its predecessor
and the recovered evidence chain. Its hash matches the printed proposal and
the historical adoption record identifies those bytes as state 016. It is an
earlier state, not the final selected state; 16 pending contracts remain.

The linked lens 2 progress proposal also matches its printed hash. Its
59 source rows, 37 contract records and 16 pending records are recovered
as historical evidence, not the final ledger or permission to resume repairs.

The protocol-format proposal bundle, its verification and progress record
were reconstructed to their printed hashes. The bundle preserves the 59
source rows, 37 contract updates and 16 pending records from this earlier
lens 2 checkpoint; it did not itself close audit cases.

A fresh bundle restoration exposed line-ending changes in printed evidence.
Git attributes now preserve those exact bytes; a fresh checkout of commit
`a19f54d` passed the complete package verifier. This is a same-machine
restoration, not a remote or independent backup.

Five historical verification records and five progress records in the lens 2
chain were reconstructed to hashes printed by their commands. The final one
links the recovered proposal to the preceding progress record. These are
historical records, not fresh validation or the final checkpoint.

The preceding owner-evidence integrity verification was reconstructed to its
printed hash. Its recorded pass describes the historical run and does not
substitute for fresh validation.

The earlier lens 2 progress record now also matches its printed hash,
connecting the recovered progress sequence to the owner qualification and
its historical verification. The cited checkpoint remains a hash reference.

The initial lens 2 progress record was reconstructed to the filename and hash
printed by its historical write command. Its cited checkpoint 481 is still
unavailable; the recorded path for this progress file comes from the writer.

A later support registration was recovered from its complete printed body and
matched to the separately printed hash. It preserves historical checkpoint 489
context and does not establish the final state.

A transcript-wide comparison recovered 130 further JSON bodies matching
separately printed references after excluding known original, progress and
recursive objects. The manifest preserves reference and body ordinals; full
original command outputs remain private, and current authority is unverified.

The 130 added JSON bodies and their cited path/hash references were checked
against the bounded local transcript snapshot. The source restorer verified
368 manifest identities in a fresh destination; the search index contains
28,288 overlapping records. These checks do not restore final authority.

A fresh checkout from the same-machine bundle at `e79a414` passed the
package verifier with all 130 added JSON bodies. This is not a remote or
independent backup.

Ten linked JSON bodies matched hashes cited in recovered parents. They
include checkpoint 476 and a registration from checkpoint 3778; neither
restores the final checkpoint or grants audit credit.

The ten linked bodies were also checked against their bounded transcript
outputs. Checkpoint 476 identifies a ledger hash and size, but the ledger
bytes and the final checkpoint 3821 remain unavailable.

The checkpoint 3778 corpse-family note was recovered from a recorded literal
write and matches the recovered decision citation. The historical command was
not executed; the note does not decide product policy or restore final state.

Four supporting text objects matched cited hashes and basenames, including a
historical helper preserved as inert evidence. None establishes final
authority or authorizes execution.

A filename/hash scan recovered 19 JSON bodies printed elsewhere in the
transcript. Filenames are preserved as printed; original paths, final
authority and fresh validation are not inferred.

All 19 filename/hash lines and corresponding bodies were checked against
the bounded local transcript. This verifies the matches, not original paths
or current audit authority.

One previously preserved historical helper candidate matches a printed
filename/hash line exactly. It is unreviewed, remains inert, and its original
path and current audit authority are not established by the filename.

A terrain-stage registration was reconstructed from its recorded template
and Git bytes. It matches the hash cited by the recovered qualification;
this preserves historical source scope without granting current audit credit.

The terrain registration supplied one additional Git-verified source
identity. The source restorer verified 369 manifest identities in a fresh
destination; the exact helper reference was checked against the local
transcript and candidate bytes.

Three lens 2 support registrations from checkpoint 487 were reconstructed
using recorded authority hashes, Git source bytes and bounded timestamps.
Each matches its printed hash; historical scope and no-credit limits apply.

A fresh source restoration verified 373 identities in the registration
manifest. The local search index contains 28,327 overlapping records.

The checkpoint 487 fame-parity read, qualification and progress records
match printed hashes. They retain four registered source rows, five
same-lens reuse references and the original conditional source-level witness;
no runtime or database witness is claimed.

The checkpoint 487 fame proposal bundle and follow-up progress record also
match printed hashes. The bundle contains four source rows and its named
contract update; proposed state 022 is still a hash reference, not recovered
state bytes.

The historical lens 2 numeric successor state 017 and its progress record
match printed hashes. The state preserves six pending evidence lists; the
six underlying reads and qualifications remain hash references and do not
establish final authority or new audit credit.

Historical lens 2 state 018 was reconstructed from state 017, printed
references and three recorded pending lists. It matches its printed hash;
the underlying reads and qualifications remain unavailable as bytes.

Historical lens 2 state 019 matches its printed hash and preserves 16
reconciled obligation groups with their remaining questions. The full
crosswalk and native-order read and qualification are still hash references;
this is not the final selected state or new audit credit.

Historical lens 2 state 020 matches its printed hash. It records the scoped
organization-training source disposition and preserves the repair and policy
obligations. Some underlying receipts and qualifications remain hash references;
this historical state does not establish the final audit authority.

Historical lens 2 states 021 and 022 and the book owner-comparison record
match their printed hashes. The book comparison preserves two distinct queue
entries and does not establish an alias. The Fame state records a conditional
source finding, not a runtime incident. The full queues, final selected state,
and runtime acceptance remain unavailable.

The late frontier lineage connects checkpoints 3817–3821, their queue
predecessors, preserved notes, recorded scopes and outstanding obligations.
It records historical command output, not a fresh run or recovered authority.

The recovered Lens 4 receipt supplies the historical errantry comparison
used at checkpoint 3817. It does not recreate the newer source registration,
queue, ledger or final checkpoint, or add new audit credit.

The receipt’s mandatory qualification 170 is restored to its cited hash.
Its cited source-scope and range-overlay versions remain unavailable; the
available same-path versions have different hashes and are not substitutes.

The qualification’s raw-record path resolves in the available Git snapshot.
Its recorded hash pins that candidate only; the qualification does not cite
a hash for it and explicitly withholds credit for premature comparisons.

The printed queue 4151 entry 2708 cites three restored original registrations
and receipts. Their hashes match, but this predates the later adoptions and
does not reconstruct the final queue or transfer coverage between lenses.

The checkpoint 3802 terminal-interpolation registration now matches the
recovered qualification’s citation. It records bounded source observations;
the qualification retains unresolved native admissibility and runtime gates.

The checkpoint 3802 qualification’s three registered observations were
matched against recorded output and the pinned Git sources. This does not
establish runtime acceptance or close its remaining native obligations.

Four historical registrations were reconstructed from recorded creation
commands, preserved receipts and pinned source bytes. Each matches a hash
cited by an already recovered parent; the final audit authority remains open.

Seven scene-readiness and rendering registrations were reconstructed from
pinned Git sources and the checksum-verified Bevy archive. They match their
recovered parent citations; no final authority or audit credit is implied.

The focused older lockfile-delta registration matches its recovered parent’s
citation. Correction: its full-lockfile successor was already recovered under
a hash-prefixed filename and matches its own recorded path and hash. Its rows
also match the historical derivation; neither record establishes final state.

The Lens 6 scatter-shader registration and its prospective parent now match
the linked hashes. The parent contains 12 source rows, of which 10 shader rows
appear in the lens projection. Neither record establishes final audit state.

Two independent Lens 11 and Lens 12 chunk-instancing registrations match
their recovered citations. They preserve the eight registered source files
and explicit older-revision absences; cross-lens verdict transfer remains barred.

A Lens 3 chunk-instancing registration matches its printed packet citation.
The cited packet is normalized transcript evidence, not a recovered final
authority, and no cross-lens verdict transfer follows.

A Lens 7 chunk registration matches a hash in queue 1068 output. That
output was intentionally sliced, so it does not restore the full queue entry.

Six historical registrations were reconstructed to hashes in recorded
outputs. Five citations occur in sliced queue output, which does not restore
the full queue; the sixth is a direct registration output.

The Lens 11 pool-reducer registration matches a hash in a recorded Python
representation of a later qualification; this does not recover final authority.

The Lens 11 reducer-rollback registration matches a hash in the same
recorded later qualification as the pool-reducer registration.

The printed reducer-rollback qualification preserves four source observations
and cites the two reconstructed registrations. Its representation was complete
within the recorded display bound; it is not an original-byte recovery.

A supplemental source manifest links 43 identities to the newly recovered
registrations and restores their hash-verified Git blobs. These are additional
to the 373-entry manifest but may overlap other source-restoration collections.

Two Bevy source files cited by the recovered native registration are now
preserved as exact archive-member bytes. Their hashes and archive checksum
match the registration and pinned lockfile; audit credit remains separate.

Ten additional native archive members are preserved and hash-linked to their
registrations. They are available without the original machine’s crate cache;
source preservation does not confer audit credit or final authority.

Six tangent-generation source files are preserved from a checksum-verified
archive and match their recovered registration. Their inclusion does not
establish final authority or add source-read credit.

Fifty-three native source objects cited in printed documents are preserved
from checksum-verified archives. Those documents are normalized transcript
evidence; the source objects do not establish final audit authority or credit.

Three rawzip source objects are preserved from a checksum-matched archive
against printed registration rows. The original registration file itself is
not recovered, and no final audit authority follows.

Seven native source objects from four archives match individual complete
rows in recorded output. Those rows do not establish complete registrations
or final authority.

A further 248 source identities were matched to Git and recorded output
rows. Two Burn files unavailable in the project Git repository matched their
printed hashes in a dependency checkout and are preserved separately. None
of these observations establishes final authority or transfers audit credit.

Fifteen alternate-field source identities, including binary assets, match
recorded output rows and pinned Git bytes. Their restoration does not imply
read attribution, audit credit or final authority.

Source counts are now deduplicated across seven Git-source manifests and
seven preserved dependency collections. They measure recoverable source
objects, not audit completion or the missing final authority.

The eight published draft commits now have a local incremental Git bundle.
An isolated restoration verified all eight refs and refused an existing
destination. The restored repository depends on the source object store;
none of the drafts is claimed reviewed, integrated or runtime-accepted.

Historical selected-lenses snapshot 440 is reconstructed byte-for-byte against
its cited SHA-256, using all 13 printed values and a timestamp search within the
recorded preparation execution window. Referenced state bytes for lenses 1,
3–11 are verified; states 2, 12 and 13 and the complete final selection remain
missing. Snapshot 440 is historical, not proof of the final selection.

A Lens 2 source-reuse qualification is reconstructed to its printed SHA-256
from six original receipts. It preserves 21 previously qualified source rows;
it is a dependency of the later Lens 2 state chain, not the missing state.

At checkpoint 3802, a printed CURRENT prefix points to recovered snapshot 440.
The statically derived writers for checkpoints 3803–3821 preserve the prior
selection, and all 19 runs reported preflight, apply and verify success. This
supports an inference that 3821 retained snapshot 440; the final CURRENT bytes
and possible unobserved intervening changes remain unverified. No historical
writer was executed during recovery.

The dependency inventory now links 39 subsequently recovered objects to its
earlier missing citations, including snapshot 440 and several Lens 2 records.
These are overlapping evidence counts; nine of the 39 are newly recovered
program sources, preserved as inert evidence.

An unimplemented capture-persistence and active-view repair contract was
reconstructed against its printed SHA-256. Its six repair requirements and
acceptance obligations are historical evidence, not an implemented fix.

A prospective Bevy asset-event source plan was reconstructed to its cited
hash using the preserved archive-member bytes and recorded Windows newlines.
It records a locator and explicitly grants no source-read credit.

A historical Cargo.lock delta was recovered from a transcript output and
reproduced from its two pinned Git revisions, matching the cited SHA-256.
It is source evidence and gives no new audit credit.

A fresh same-machine clone of local commit `cc7b9a08752a9547490c1d446360187917421a36`
passed the recovery verifier. Its draft restorer recreated all eight pinned
refs using the source repository as an object alternate, and refused an
existing destination. This does not test an independent machine or remove the
source-object-store dependency.

The final selection-lineage trace now includes the individual JSON report
fragments for all 19 checkpoint runs. Verification also derives each string
replacement from its recorded source expression rather than trusting the
transformation list alone. These remain historical self-reports.

Four complete printed audit values were reconstructed to their cited hashes,
including an AI-precision repair plan and its source qualification. The plan
remains unimplemented; the records do not establish current authority.

Two registrations cited by the newly recovered grass and AI-precision
qualifications were reconstructed to their recorded hashes. Seven pinned
source objects accompany them; six overlap earlier source inventories. These
references are outside the older 212-record dependency count and do not change
its then-unmatched hashes or grant new audit credit.

Historical Lens 13 state 388 and its adoption plan match recorded SHA-256
hashes. The plan cites the state predecessor, and the recovered verification
cites both the state and receipt. The predecessor and receipt remain missing;
this is not the later selected Lens 13 state or final authority.

Six additional historical Lens 13 states match verification hashes using
predecessor and receipt hashes from preserved plans and verification records.
A seventh reconstruction matched an already preserved state. Referenced
receipts and predecessors are not thereby recovered, and these states do not
establish the later selected state or current audit authority.

Nine further historical Lens 13 states were reconstructed to printed hashes
using separately printed state, predecessor and receipt references. The
referenced predecessor and receipt bytes are not implied recovered. These
states predate the final selected state and give no new audit credit.

A successor capture repair contract was reconstructed to its recorded hash.
It explicitly supersedes the previously recovered version, adding native error
cause and checked buffered-completion requirements. It remains historical and
unimplemented; the cited native packet bytes remain separately unverified.

Three related historical capture-family repair contracts were reconstructed
to their printed hashes. They cover animation artifact certification, plant
diagnostics and remaining callers and cite the native persistence contract.
Their cited source packets and other dependencies remain separately qualified;
no repair or acceptance work was performed.

The plant and diagnostic contract includes an uncredited source scope whose
pinned source bytes are now preserved and hash verified. This does not grant
new audit credit or establish runtime behavior.

A historical capture-view mapping repair contract was reconstructed to its
printed hash. The recovered animation certification contract cites it. Its
native packet, qualification and navigation dependencies remain separately
qualified; no implementation or runtime acceptance follows.

The historical capture-owner repair contract and its five-case facet
comparison were reconstructed to printed hashes. The contract cites the
recovered scene persistence successor, and all three recovered capture-family
contracts cite the owner contract. Case verdicts remain independent; the
referenced source packets and queue bytes are not thereby recovered.

The later remaining-capture-callers repair proposal was reconstructed to its
printed hash and linked to its predecessor. It preserves continuous request
cadence while separately tracking required outcomes. Its cited resolutions and
source packets remain separately qualified; no implementation occurred.

The later building capture identity proposal was reconstructed to its printed
hash, preserving 12 repair requirements and seven acceptance items. Its prior
proposal, source packets and resolution remain separately qualified; no
implementation or runtime acceptance followed from recovery.

Three prospective building-source registrations were reconstructed to
their printed hashes with 20 distinct pinned source objects. Their recorded
scopes are uncredited and do not establish implementation or runtime
acceptance. The corresponding packets were recovered subsequently.

The three building identity read packets were reconstructed to their printed
hashes and linked to the recovered proposal, registrations and source rows.
Their recorded assessments are historical and explicitly uncredited; source
displays were not re-executed during recovery.

The historical capture fingerprint proposal was reconstructed to its printed
hash, preserving eight repair requirements and six acceptance items. Its
source packets were subsequently recovered; the resolutions remain separate.
No implementation or runtime acceptance followed from recovery.

Two fingerprint source registrations and their two read packets were
reconstructed to printed hashes and linked to the recovered proposal and six
pinned source objects. Three source hashes overlap other manifests. The
recorded assessments are uncredited; displays were not re-executed.

The capture-callers source registration and read packet were reconstructed to
their printed hashes and linked to the later proposal and four pinned source
objects. The recorded assessments are uncredited; source displays were not
re-executed. Some source objects overlap other collections.

All 17 output chunks cited by the three building read packets were located.
Their 4,674 recorded line occurrences match the preserved pinned source bytes;
this is a comparison of historical output, not a new source read or audit credit.

Nine output chunks cited by the fingerprint and capture-callers read packets
were located. Their 2,531 recorded line occurrences match preserved pinned
source bytes; this comparison adds no source-read or audit credit.
