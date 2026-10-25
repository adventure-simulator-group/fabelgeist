"""Qualify exactly Social027 nineteen plus Naming052 five producer roles."""
from pathlib import Path
from collections import Counter
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[1]
WAVE = ROOT / "records/final-acceptance-20261021/wave-01/producer-supplement-after-owner"
MAP = OUT / "expanded-producer-delta-map-049.json"
packet = json.loads(MAP.read_text())
context = json.loads((ROOT / "continuation-context.json").read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_path(source):
    variant, rel = source.split(":", 1)
    return Path(context["snapshots"][variant]) / rel


SOCIAL = "CAM-PPS-SOCIAL-027"
NAMING = "CAM-PPS-NAMING-STARTING-052"
rows = [(i, row) for i, row in enumerate(packet["delta_observation_map"]) if row["parent_trace"] in (SOCIAL, NAMING)]
assert len(rows) == 24 and Counter(row["parent_trace"] for _, row in rows) == {SOCIAL: 19, NAMING: 5}

# Explicit per-observation role reconciliation. These are not source-file read
# counts: parent source proofs and their substantive actual-receiver narratives
# are consumed only for the role described here.
roles = {
    27: ([1, 4], "Canonical thirteen-axis personality enums, demographics and Personality record -> DB canonical re-exports -> chat/prayer/discovery consumers. Actual typed enum is lost into retained i8 chat/prayer APIs (existing propagation finding); perceived axis-local i8 belief and private Sex differ. Conviction strength/non-neutral count are newly computed numeric results, not previously checked leaves erased."),
    44: ([0, 1, 2, 3, 4], "Core social action/topic/claim/prayer/chat records and numerical operations -> actual complete DB social and Web witnesses. Typed classifications survive where stated; retained raw chat personality/cooldown action, elapsed/familiarity/affinity/effect roles remain existing repair. Preserve broad public numeric/unknown input normalization, signed affinity, effectiveness above1 and actual final decay/scaling/roll kernels."),
    45: ([0, 3], "SocialActionKind authored enum -> strict manual command parse -> typed automatic choice -> actual manual/automatic DB operations. Automatic action still stable-id String -> same internal parse (F019). Canonical command/Rally and deception/Reframe spelling, unknown/whitespace error ordering and real native reducer text port preserved."),
    46: ([2, 4], "Affinity anchor/elapsed -> actual exponential half-life, bounded signed gain and realized clamp-difference equations -> DB relationship/presence receipt. Fresh full60 source/fixtures confirms real zero and signed changes; missing required caller clocks remain F023, not excused by numerical0 normalization."),
    47: ([3, 4], "Typed ClaimChallengeOutcome plus actual realized affinity delta -> WitnessClaimResolution enum payload -> optional DB resolution -> generated row -> Web claim_view -> actual JSON/DOM. Option carries unresolved versus resolved; signed realized change remains raw owner strengthening. No duplicate resolved flag or sentinel inferred."),
    48: ([3, 4], "Witness assessment/challenge typed approach/direction and broader sensitivity/roll inputs -> actual confidence/strength/scoring kernels -> DB claim record -> Web CSS/text. Successful assessed strength clamp differs diagnosed-axis NaN confidence; preserve raw public normalization and unknown assessment alias policy rather than blanket UnitInterval."),
    55: ([2, 4], "Actual shared_language_coefficient loads both skills/attribute caps -> best_common_oral_language_capped -> scalar coefficient -> social effect. Fresh2144-2173 shows missing rows/caps become0 under existing policy; do not mislabel that as observed fluency. Fraction is new computed output, not a peeled checked fraction; retain capability/observation owner requirements."),
    83: ([1], "Actual CharacterPersonality1-160 re-exports Core enums, retains typed visible traits/demographics and separate signed private score schema. This scoped producer reaches DB social canonical enum-to-i8 seam. Only1-160 material role, not the entire larger personality file or its partial set_score body, is reconciled."),
    109: ([0, 1, 2, 3, 4, 5], "Complete frozen P16 social3805 and explicit current PR signature/logging union -> admission, manual/automatic actions, chat/prayer/affinity/presence, testimony/challenge and native row/reducer recipients. Existing F019 typed action/personality erasure, F006 concrete causes and F023 missing clocks/health remain. Actual stable IDs at final native receipt/cooldown fields differ internal raw forwarding; preserve exact ordering/privacy/replay/draw/rollback policies."),
    110: ([0, 5], "Original action authority fixtures -> strict canonical action/native request -> current typed operation result and SDK state checks. Parent personally read full154 source; unchanged fixture values/cases/assertions retained. Known typed aggregate versus string comparison is an API adaptation obligation, not passed validation."),
    111: ([0, 5], "Fresh31 admission source parses SocialActionKind before actor/target/self/living/co-location policy. ParseSocialActionKindError is formatted before internal return (F006); preserve unknown-token diagnostic and first-failure order. Actual manual string wire ingress does not justify automatic typed action round trip."),
    112: ([3, 5], "Original full379 witness/social authority fixtures -> challenge/retry/resolution/native persistence/feedback recipients. Parent current producer graph preserves all original inputs and optional-resolution/alias policy. Existing map_err(String::as_str) on typed aggregate remains source adaptation finding; no test executed."),
    113: ([0, 5], "Original automatic85 fixtures exercise actual typed automatic selection -> application path/native social state; same F019 loss remains in that receiving operation. Preserve existing actor eligibility, conditional draw order and oracle population; source-read evidence only."),
    114: ([0, 3, 4], "Fresh47 source keeps SocialActionKind/SocialTopic/PersonalityAxis through actual discovery matching and ordered axis vector; self-only branch, deliberate empty Reassure and fixed fallback axes stay explicit. Axis word extraction occurs only inside actual owned RNG context, not retained raw selection state."),
    115: ([4], "Fresh54 ActionDraws stores Seed and participants; typed axis survives until actual StreamId RNG context words. testimony uses Seed bytes solely in owned length-framed derive, other helpers consume Seed directly. Returned random floats are new draws consumed by actual social numerical comparisons, not retained canonical Seed bits erased."),
    147: ([4], "Fresh language214-243 computes capped shared hours/fluency coefficient and deterministic tie order; language_scaled_effect performs final finite/clamp multiplication. OralLanguage stays typed through selection; coefficient is changed output. Preserve missing source policy and original float order, broader nonfinite behavior, not arbitrary wrapper narrowing."),
    156: ([0, 1, 4, 5], "StartingCharacterSpec1936 authored/native seed, PersonalNameIdentity, traits/profession/loadout -> actual name generation, life/training and DB creation -> immutable tactical request/mailbox/spawn. Root054 plus final pure8 closes previously pending LifeSimulation receiver. Fresh886-984 confirms checked Seed and training result flow; raw128-bit roster seed, biological/body mass, raw religion/organization/skill keys and physical catalogue union remain distinct connected findings."),
    175: ([3], "Fresh complete build121: sorted actual file paths/text -> framed SHA; external JSON-compatible YAML decode -> raw schema -> fragment merge -> validator -> actual generated JSON/Rust file. These are concrete filesystem/Serde/hash/output ports; internal validated-name membership/raw keys still require propagation. Build panic Display is final output, not a waiver for earlier concrete SamplingError erasure."),
    176: ([3], "Externally decoded full schema199 retains typed cultural/sex/register/tradition/record-unit enums but open definition/reference IDs and frequency/provenance records remain raw across validation and generation. Real external field names/JSON are native format; successful membership must survive later internal selectors. Source strengthening and tuple/aggregate design remain separate later lenses."),
    177: ([2, 3], "Full validator576 -> indexed name membership, positive/unique compatible frequencies and provenance -> actual build accept/reject. Fresh270-340 confirms per-family form checked_add; family/surname total overflow can still produce runtime SamplingError. Existing NameCatalogError::Sampling(String) loses that concrete runtime cause; static validator string diagnostics are a separate strengthening obligation."),
    178: ([3], "Full derivations214 validation and fresh1-90: repertoire/family/form/surname raw target tuple + exact frequency -> known-target, source observation and complete-coverage checks -> build output. Unknown rejected authored key evidence stays available; no arbitrary closed enum or successful-membership waiver. Optional tuple shape is later aggregate review, not unread producer graph."),
    179: ([1, 2, 3, 5], "Exact Web175-310/939-1030/1333-1503 -> claim/native query observations -> ConversationView/JSON and NPC social availability. Rig033 full dialogue29 closes actual callbacks/build_view/JSON/DOM graph, preserving F016 malformed authoritative omission and F023 observation losses. Real absent relationship maps Uncertain; final safe response does not justify internal failed-query default."),
    180: ([3, 5], "Fresh claims1-127 matches exact event/order/text, carries optional typed WitnessClaimResolution and maps generated outcome enum to canonical enum before JSON. Witness approach admits canonical ClaimChallengeApproach and immediately encodes at actual HTTP reducer JSON; final conflict/bad-request statuses are real response policy. Earlier grant extraction/build_view malformed data remain existing whole-loop findings."),
    186: ([3], "Fresh JS480-555 receives claim JSON, sends actual HTTP challenge command, writes actual CSS/ARIA/text and completion DOM. Assessment text chooses CSS/text fallback at this browser adapter, not a Rust semantic re-admission. Preserve unknown insight fallback, full token/revision matching and stale-view request recovery; unrelated JS bodies not credited."),
}
assert set(roles) == {i for i, _ in rows}

fresh = [
    ("physical-p16:crates/adventuresim-core/src/personality.rs", [[1, 202]]),
    ("physical-p16:crates/adventuresim-stdb-module/src/personality.rs", [[1, 160]]),
    ("physical-p16:crates/adventuresim-stdb-module/src/social/admission.rs", [[1, 31]]),
    ("physical-p16:crates/adventuresim-stdb-module/src/social/draws.rs", [[1, 54]]),
    ("physical-p16:crates/adventuresim-stdb-module/src/social/discovery.rs", [[1, 47]]),
    ("physical-p16:crates/adventuresim-core/src/social/claim_resolution.rs", [[1, 69]]),
    ("physical-p16:crates/adventuresim-core/src/social/affinity.rs", [[1, 60]]),
    ("physical-p16:crates/adventuresim-world-schema/src/language.rs", [[214, 243]]),
    ("physical-p16:crates/adventuresim-stdb-module/src/character.rs", [[2144, 2173]]),
    ("pr-working:crates/strategic-web/src/routes/dialogue/claims.rs", [[1, 127]]),
    ("pr-working:crates/strategic-web/static/dialogue-client.js", [[480, 555]]),
    ("pr-working:crates/adventuresim-core/src/starting_character.rs", [[886, 984]]),
    ("pr-working:crates/adventuresim-world-schema/build.rs", [[1, 121]]),
    ("pr-working:crates/adventuresim-world-schema/src/name_catalog_schema.rs", [[1, 110]]),
    ("pr-working:crates/adventuresim-world-schema/src/name_catalog_validation.rs", [[270, 340]]),
    ("pr-working:crates/adventuresim-world-schema/src/name_catalog_validation/derivations.rs", [[1, 90]]),
]
fresh_proofs = []
for source, ranges in fresh:
    path = source_path(source)
    eof = len(path.read_bytes().splitlines())
    assert all(1 <= a <= b <= eof for a, b in ranges)
    fresh_proofs.append({"source_id": source, "sha256": sha(path), "read_ranges": ranges, "actual_EOF": eof,
                         "origin": "Fresh actual bounded source displayed/reviewed in this producer-only turn; no inference beyond these ranges"})

dispositions = []
for i, row in rows:
    path = source_path(row["canonical_source_id_candidate"])
    assert sha(path) == row["sha256"]
    assert all(1 <= a <= b <= len(path.read_bytes().splitlines()) for a, b in row["material_contract_ranges"])
    contract_indices, trace = roles[i]
    social = row["parent_trace"] == SOCIAL
    dispositions.append({
        "delta_observation_index": i, "source_id": row["canonical_source_id_candidate"], "sha256": row["sha256"],
        "path": row["path"], "material_ranges": row["material_contract_ranges"],
        "parent_source_proof": {"record": row["record"], "receipt": row["parent_trace"], "json_pointer": row["json_pointer"]},
        "parent_actual_contract_refs": [row["parent_trace"] + "/contracts/" + str(n) for n in contract_indices],
        "actual_producer_recipient_reconciliation": trace,
        "fresh_source_proof_indices": [j for j, p in enumerate(fresh_proofs) if p["source_id"] == row["canonical_source_id_candidate"]],
        "receiving_closure_groups": (["social-time-mental-language-native", "dialogue-native-and-browser"] if social else ["name-build-runtime", "starting-life-and-authoritative-projection"]),
        "findings": (["CAM-PROP-F019", "CAM-PROP-F006", "CAM-PROP-F023", "CAM-OWN-SOCIAL-037"] if social else ["CAM-OWNER-F005", "CAM-PROP-F006", "CAM-PROP-F002", "CAM-PROP-F021"]),
        "disposition": "bounded-producer-role-accounted-with-findings",
        "remaining_actual_unread": [],
        "proof_limit": "Exact current producer proof and substantive role/receiver narrative reused; fresh reads only where listed. No full-file, counterpart, owner-fit or acceptance credit from hash/range alone."
    })

used = [MAP, WAVE / "camera/traces.jsonl", WAVE / "rig/traces.jsonl", WAVE / "root/pure-food-life-partial.json",
        ROOT / "continuation/root/pure8-packet.json", ROOT / "continuation/camera/traces.jsonl", OUT / "web-request-reconciliation-039.json"]
result = {
    "id": "RIG-CONT-EXPANDED-PRODUCER-ROLE-DELTA-051", "lens": "audit-producer-propagation-only",
    "status": "assigned24-roles-exhausted-with-findings",
    "inputs": [{"path": str(path.relative_to(ROOT)), "sha256": sha(path)} for path in used],
    "assigned_observations": {SOCIAL: 19, NAMING: 5}, "accounted_observations": 24,
    "fresh_source_proofs": fresh_proofs, "observation_dispositions": dispositions,
    "receiving_closure_groups": {
        "social-time-mental-language-native": ["CAM-PPS-SOCIAL-027/contracts/0-5 + actual_receivers", "CAM-PPS-CONDITION-WEATHER-009", "CAM-PPS-TIME-ROSTER-014 / NATIVE-PS-CAMERA-TIME-PACKET-011", "CAM-PPS-POPULATION-024"],
        "dialogue-native-and-browser": ["RIG-PPS-033/paths/0-7 + current_shared_exact_proofs", "RIG-PPS-033/actually_unread=[]", "RIG-PPS-F016 preserved; no malformed observation fallback accepted"],
        "name-build-runtime": ["CAM-PPS-NAMING-STARTING-052/contracts/0-3", "Existing canonical Seed/StreamId/weighted sampling actual kernel and NameCatalogError cause proof, preserving order/hash/framing/unknown IDs"],
        "starting-life-and-authoritative-projection": ["ROOT-PPS-PURE-FOOD-LIFE-PARTIAL-054/producer_to_receiver/6-9", "CONT-ROOT-PP-PURE8-001/new_receiver_dispositions/2", "CAM-PPS-STRATEGIC-PROJECTION-007 creation source", "RIG-CONT-PPS-WEB-REQUEST-RECONCILIATION-039/source_dispositions indices714/715 and actual mailbox/spawn joins", "CONT-CAMERA-PP-008/historical_reconciliation", "NATIVE-PS-SERVER-EQUIPMENT-PROJECTION-CLOSURE-027 exact source/recipient graph"]
    },
    "connected_repairs_preserved": [
        "Camera whole canonical social manual/automatic action/personality/cause/clock loops retain existing claims; typed action must reach internal execution without stable-id reparse.",
        "Camera/rig dialogue whole observation and provenance/query/HTTP callback owners retain F016/F006/F022/F023; true final JSON/DOM/reducer text ports do not excuse internal loss.",
        "Root/camera canonical name catalogue error/ID/membership and current P16 physical catalogue owners; Native combined pure training/life/curriculum function; Camera DB creation/physical biological mass and Native terminal causes; rig catalogue/hash/DOM graph. No new ownership transfer.",
        "Preserve exact current PR/P16 union, typed P15 equipment chronology, all existing fixture inputs/cases/assertions, full seed bits/framing/draw order, private versus perceived state, realzero versus unavailable and original broad input policies."
    ],
    "remaining_actual_unread": [], "first_unfinished_assigned_cell": None,
    "limits": ["16 fresh bounded source reads supplement existing current producer body proofs; not a claim to independently reread social3805/Core1909/starting1936/name validation576 entirely.", "Existing findings remain mandatory; no source/family, behavior, owner-fit or overall lens clearance.", "Only assigned24 rows qualified. Root six narrow qualifiers, Native boundary15 and Camera remainder remain separate."],
    "source_or_authoritative_ledger_edits": 0, "project_commands_or_tests": 0, "overall_lens_clearance": "not awarded"
}
target = OUT / "expanded-producer-role-delta-051.json"
if target.exists():
    raise SystemExit("Append-only target exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"assigned": len(rows), "accounted": len(dispositions), "fresh_bounded_proofs": len(fresh_proofs), "remaining_actual_unread": result["remaining_actual_unread"]}))
print("sha256=" + sha(target))
