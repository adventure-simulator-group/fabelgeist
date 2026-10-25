"""Save authored current-source operation comparisons; no project execution."""
import json
from pathlib import Path

C = Path(__file__).resolve().parents[1]
destination = C / "root/packet-payload-158.json"
assert not destination.exists()
lengths = {"recipe/generic_blade.rs": 113, "model/generic_blade.rs": 90,
           "model/lofted_blade.rs": 9, "recipe/validation/melee.rs": 240,
           "model/placement.rs": 156, "recipe/blade_profile.rs": 158,
           "model/placement/anchors.rs": 137, "model/blade_sections.rs": 274}
roles = [{
    "source_id": "pr-working:crates/adventuresim-weapon-model/src/" + name,
    "actual_compared_ranges": [[1, length]], "complete_scoped_comparison": True,
    "source_evidence": "Original complete owner/producer scoped union and actual EOF verified before reading. Fresh literal entire currentPR body in bounded untruncated slices; independently authored04 complete operation/product/computed-domain comparison. No new unregistered extent/helper body or peer verdict credit.",
} for name, length in lengths.items()]
roles.append({
    "source_id": "physical-p16:crates/adventuresim-weapon-model/src/model/placement.rs",
    "actual_compared_ranges": [[26, 156]], "complete_scoped_comparison": True,
    "source_evidence": "Exact effective material26-156 from original admission registration verified and alloverlays checked. Actual full emptyPR/P16 diff +fresh literal fullPR counterpart and independent same operations within26-156. No imports/structs1-25 or whole156 counterpart credit.",
})
payload = {
    "packet_number": 158, "source_roles": roles,
    "support_only_source_roles": [{
        "source_id": "pr-working:crates/adventuresim-weapon-model/src/recipe/quantities.rs",
        "actual_compared_ranges": [[1, 98]],
        "source_evidence": "Fresh literal entire98 re-compared for actual Ratio admissibility; already current04complete, not recounted."
    }],
    "groups": {
        "quantity-authority": "Actual private f64 Metres/Ratio/Degrees/Radians constructors and custom Deserialize require finite only, canonicalize negativezero, derive default0, and Eq/Hash use canonicalbits. Thus signed/geometric finite representation is not arbitrary physical admissibility. Count publicu16 is native capacity0..65535 with checked decoder, actual shape budget/minima elsewhere. Direction typed two signs with exact wire-1/+1 and explicitFrom intoi8 is a warranted native codec, not interior identity erasure.",
        "generic-blade-form": "Generic BladeParameters body/point widths, derivative andsection thickness use authored ratios and dimensions, with independent nonnegativepoint envelope. validate_form requires positive taper, singleedge[-1,1], tipfraction[0,1], point whenzero-tip, section/fullthickness rule and belly>-1 plus interiorpoint start/roundness. Actual dimensions layer156 supplies genericabs<=128 ratio caps; do not infer same caps for LoftedBlade. Actual generic model rechecks form, adaptivelyrefines complete sections and triangulates caps before positivewinding. Each supported nonrecessed ring has fixed case cardinality; native Region and reduction child correctness remain separate.",
        "lofted-profile-gates": "LoftedBlade wrapper projects typed input into raw kernel BladeProfile and Detail.samples(count,min4). Profilevalidate enforces recessediffFuller, optionalpointstart/roundness and complete groove longitudinal/entry/exit associations; for no point/fuller it returnsOk without globallybounding taper/belly. Ratio constructor permits anyfinitef64; local union dimensions onlytaper>0/belly>-1, so commonabs<=128 bound must not be assumed. Bodywidth scales width, plan, tapercurve andbellfactor; physical/raw renderer representations still require computed finite/extent bounds. Actual allRecipe graphs/ranged/shields, shape.range/frame registration, reduction/cap geometry must be traced before claiming complete accepted hugegeometry/forgeoverflow. No new unique finding at this partial graph stage.",
        "melee-relational-admission": "Melee dispatcher adds BladeProfilevalidate/fullerdimensions, gripanatomy, profilemonotonicity, correlatedsocketfacets/segments/crenellation/wall, pommelratio/twist/ornament and maceprofiles/cusp/flanges. Many width/ratio/scalar helpers alone are deliberately incomplete. Mace explicit len>=2 precedes first/last, socket profilegate proves nonempty beforelast/index; actual earlyreturn sequencing justifies those local accesses. Optional features require their whole correlated input, not generic positive wrappers or duplicateunchecked pairedfields.",
        "placement-and-identity": "resolve clones original recipe, applies facing, normalizes componentids only in clone, inserts canonical shaft/root/grip frames, then validates contact. Existing original-versus-normalized-ID projection graph remains real repair dependency for derivedhead lookup152/153; not claimed new witness here. Component insert resolves dimensionalfiniteness of computedoffset via Metres::new, resolves actual shape.range twice, applies mounting/attachment/seating/socketfit and registers frames before returning owned result. Failures affect only clone/temporarygraph. P16exact26-156 includesoperations but excludesresolved struct/import1-25. Actual missing newextents declaration has only been signature-navigated, no unregistered bodycredit.",
        "anchor-frame-relations": "attached stretchrequires positiveYtarget andKnuckleBow, localnearzero andknownframes; attachment computes rotatedanchor, targetoffset andowner-inward overlap rather than translating nominalYonly. register stores base/bottom/top/center along transformedlocalY and specialgrip/shield/heel/ricasso/socket frames. These correlate frameposition/rotation/owner; rawString dotnamespace backed by componentgrammar separately. Existing Root04 source proves axialframes, not a full transverse boundingbox for every shape, so frameextent20check alone doesnotclear arbitrary computedwidegeometry.",
        "section-cardinality-and-error": "Loft baseline n=Detail.samples(u16count,min4), successfulRecipecap256 givesn4..512 forHigh, so0..=n nonempty and denominatornonzero. Featurestations starts0 andincludes positivevalidatedlength. rings0/first/last rely on this actual producer, notrawVecindex. AdaptiveSection recursion cappedf64mantissadepth and append triangleestimate, failedconstructnotpublished. Recessed ringcardinality and reduction dependencies require actual groovebody, not genericassumption. face dedupsdegeneratevertices thenrejectsnonfinitepoints beforetriangleappend; successfulfinitef64face doesnotprove laterf32castrepresentability. cap Region and allSolidwinding/volume conditions separate.",
        "limits": "Nine additionaldistinct04roles575->584, findings60 unchanged. Quantitysupport rereadnotrecounted. MissingPRmodel/extents remainsprospective directsource if actualrangeproof stillneeded; no fullforeign/otherhelpergrowth. All237/233/231/priorqualifications retained; current0682849+245/307queue. No codeedits/execution/newtests or otherlensclearance; all thirteeninitialaudits thenhandover163, implementationdeferred."
    }, "findings": []
}
destination.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"payload": str(destination), "authored_source_roles": len(roles)}))
