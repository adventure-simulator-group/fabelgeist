"""Append the first explicitly reviewed owner-fit roles, not inferred coverage."""
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).parent
ROOT = OUT.parents[2]
queue = json.loads((ROOT / "continuation/rig/owner-fit-delta-queue.json").read_text(encoding="utf-8-sig"))["sources"]
scope = json.loads((ROOT / "continuation/cumulative-source-scope-after-producer.json").read_text(encoding="utf-8-sig"))
files = {row["source_id"]: row for row in scope["files"]}


def append(name, value):
    with (OUT / name).open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(value, separators=(",", ":")) + "\n")


roles = {
    0: [
        {"role": "PNG encoding cause identity", "definition": "png0.17.16 encoder20-139 EncodingError::{IoError,Format,Parameter,LimitsExceeded}",
         "use_and_recipient": "tiny-skia Pixmap172->PixmapRef391->Encoder/write_header414/write_image_data415 -> weapon icon studio107-113 -> IconError::Png(String)112",
         "classification": "exact canonical native cause match; existing migration gap confirmed",
         "comparison": "The exact lower operation returns EncodingError, including original IO/format/parameter payload. No unit/frame/clock/range conversion applies. IconError derives Debug/Error only, so no Clone/Eq/Serde barrier. Final PNG bytes are a true format boundary, but intermediate icon result still needs this concrete cause. Native EncodingError::cause exposes IO only; retain all variants anyway.",
         "repair": "Use existing png::EncodingError payload in weapon-model IconError and preserve through actual icon generation/cache/render consumers. Add direct same-version PNG dependency if naming transitive type requires it; no duplicate error wrapper or String adapter.",
         "prior_semantic_carry": "RIG-OWNER-CLIENT-EQUIPMENT-026/matches/6 exact cause/producer/error recipient compared against fresh native20-165 and application icon102-113/studio107-114; prior finding remains, not whole-file credit."}
    ],
    1: [
        {"role": "Native PNG package/version contract", "definition": "tiny-skia0.11.4 Cargo46-47 depends png0.17",
         "use_and_recipient": "Pixmap::encode_png return type -> application IconError cause",
         "classification": "intentional package metadata boundary; exact dependency evidence for existing owner",
         "comparison": "The version literal is a native manifest selector, not gameplay quantity or canonical schema ID. It confirms PNG0.17 error identity at actual encoder API; no additional runtime scalar owner applies.",
         "repair": "Keep PNG0.17 cause available in the lowest icon owning crate; manifest coordination remains Root-owned.",
         "prior_semantic_carry": "RIG-OWNER-CLIENT-EQUIPMENT-026 PNG cause contract, after fresh46-47 comparison."}
    ],
    2: [
        {"role": "Native pixmap dimensions, RGBA bytes and encoding cause", "definition": "tiny-skia Pixmap155-182 / PixmapRef385-430 actual encoding body",
         "use_and_recipient": "weapon studio sizeu16 -> IntSize positivity -> Pixmap::from_vec -> demultiplied native RGBA8 -> PNG Encoder(width,height)/Rgba/Eight/write_image_data",
         "classification": "actual native image-format boundary plus exact concrete EncodingError match",
         "comparison": "Pixel dimensions and RGBA8 are native raster units, not metre extents or linear garment colors. Native premultiplied color conversion is within the real codec. App WeaponIconSpec size/supersampling purpose remains distinct from image dimensions; no positive length or UnitInterval blanket. Newly reached385-430 proves unchanged concrete PNG cause available before application formatting.",
         "repair": "Preserve concrete PNG cause through icon operation, reuse WeaponIconSpec aggregate in caches; keep native byte/dimension conversion at actual pixmap/encoder calls.",
         "prior_semantic_carry": "RIG-OWNER-CLIENT-EQUIPMENT-026/matches/1,6 and native boundary, compared to fresh155-182/385-430; no unrelated pixmap module claim."}
    ],
    160: [
        {"role": "Staged byte address/extent and asynchronous readback outcomes", "definition": "GPU buffer/extent.rs1-54 BufferByteOffset(allu64,START0,checked_after), BufferByteLength(allu64 incl0); Buffer.size in buffer/mod.rs16-40; current ReadbackError/ReadbackState",
         "use_and_recipient": "Buffer.size -> typed StagedRange -> checked aligned layout -> native create_buffer/copy_buffer_to_buffer -> map callback -> typed Pending/Ready/Failed/Taken -> mapped byte slicing",
         "classification": "existing owners already retained correctly at scoped roles; native extraction justified individually",
         "comparison": "Offsets are byte addresses, lengths are byte extents; zero remains valid logical extent/empty batch, allocation independently uses alignment minimum. Not ScanElementCount/u32 or positive physical length. Padding is actual wgpu byte-layout arithmetic, re-admitted changed byte extent. Native copy/allocation/slice endpoints require numbers; retained ranges stay typed. Readback errors keep exact native mapping/poll causes; bool readiness is ordinary predicate, not parallel state authority. Vec ordering indexes are local native result order, not rig identity.",
         "repair": "No new owner propagation repair in this readback role; preserve existing types/cause payloads and byte order. Any overflow/admission/atomicity questions belong later operations/admission lenses, no acceptance awarded here.",
         "prior_semantic_carry": "No prior owner-fit source proof for this new variant; fresh full294 owner comparison and explicit GPU definition dependencies supply current evidence."}
    ],
    161: [
        {"role": "Sort record byte requirement and internal copy signature", "definition": "BufferByteLength(allu64 incl0) and BufferByteOffset exact existing GPU owners; compute already depends fabelgeist-gpu",
         "use_and_recipient": "RadixSort::record251 count*4 -> raw needed -> Buffer.size comparisons252 and KernelBatch::copy_buffer301-302 -> raw helper319 -> actual wgpu copy328",
         "classification": "exact owner match; raw internal byte contract remains a migration gap",
         "comparison": "Computed needed is a new byte extent, same meaning/width/validzero as BufferByteLength, not count or address. The private copy helper is application orchestration before actual wgpu port. It can compare typed Buffer.size and carry BufferByteLength to final native call; no dependency barrier or duplicate wrapper needed. Preserve early count<=1 no-op, pass parity and exact copying order.",
         "repair": "Retain BufferByteLength for needed and KernelBatch copy parameter and every connected caller; extract only at actual native copy. Existing GPU owner remains lowest coherent shared crate.",
         "prior_semantic_carry": "New source role. Fresh sort220-305 plus actual helper308-345 and owner definitions; no coverage inferred from prior producer RIG-PPS-009."},
        {"role": "Typed histogram scan count versus sorting/dispatch/bit roles", "definition": "ScanElementCount prefix_sum19-64 is zero-allowedu32 count of u32 scan elements; byte_length produces checked byte-purpose output",
         "use_and_recipient": "RADIX*tiles -> ScanElementCount -> PrefixSum::record240-269 -> typed scratch/count and BufferByteLength comparison; raw count/bits/pass/shift/tiles otherwise feed actual sort policy and WGSL parameters",
         "classification": "exact existing ScanElementCount retained for scan receiver; mismatch for unrelated sort count, workgroups or bit positions",
         "comparison": "Histogram scan length legitimately begins as new computed count and enters its real owner. Sorting key/value count, significant bits clamped1..32, workgroup tiles and pass/shift roles are different dimensions/collection authority despiteu32. RigJointCount/strand count and nonzero wrappers do not fit early0/1 no-op or native broad bit normalization. No canonical sort/bit-purpose owner established in this bounded definition comparison; newly meaningful count/policy roles remain explicit owner inventory requirements rather than forced Scan aliases.",
         "repair": "Preserve ScanElementCount; keep sort/bit/dispatch roles distinct in shared compute owner work without narrowing accepted native inputs. Any new owner requires complete relevant source/recipient comparison before implementation.",
         "prior_semantic_carry": "No carry claimed; fresh exact scan definition and receiving body compared."}
    ]
}

for index, comparisons in roles.items():
    source = queue[index]
    scoped = files[source["source_id"]]
    row = {
        "id": f"RIG-OWNER-DELTA-COVERAGE-{index:03d}", "queue_index": index,
        "source_id": source["source_id"], "sha256": source["sha256"],
        "scope_record": "continuation/cumulative-source-scope-after-producer.json",
        "producer_role_refs": [{k: r[k] for k in ("record", "receipt", "json_pointer")} for r in scoped.get("producer_reached_contract_observations", [])],
        "role_comparisons": comparisons,
        "discovery": "bounded-exact-role-owner-fit-accounted-with-findings", "remaining_actual_unread_for_listed_roles": [],
        "acceptance": "Carry exact canonical owner through all application fields/signatures/collections/errors until proven native operation; preserve unit/frame/authority/input/validzero and original arithmetic/fixtures. No duplicate owner or primitive forwarding adapter.",
        "source_edits": 0, "validation": "none; source only", "overall_clearance": "not awarded"
    }
    append("coverage.jsonl", row)

append("findings.jsonl", {
    "id": "RIG-OWNER-DELTA-F001", "kind": "exact-existing-owner-migration-gap",
    "sources": ["sort/mod.rs251-257,301-302", "kernel/mod.rs319-329"],
    "owner": "fabelgeist_gpu::BufferByteLength",
    "source_backed_comparison": "Same byte extent/all-u64/zero policy as actual Buffer.size and native copy extent, distinct byte offset. Compute already imports GPU crate, so no dependency barrier.",
    "connected_repair": "Typed sort needed byte extent -> typed KernelBatch copy parameter -> every actual internal caller -> native wgpu copy only; do not merely convert at call sites.",
    "dependency": "Existing canonical GPU extent owner; coordinate lower batch helper once across all calling families.",
    "acceptance": "No retained/internal raw byte-length parameter or needed extent on this graph; unchanged copying order/no-op/pass parity/native bits; existing tests only.",
    "status": "discovered; no source repair or validation"
})
append("findings.jsonl", {
    "id": "RIG-OWNER-DELTA-CARRIED-PNG-001", "dedup": "RIG-OWNER-CLIENT-EQUIPMENT-026/matches/6",
    "kind": "existing-owner-finding-confirmed-after-exact-new-role-comparison",
    "source": "weapon-model icon.rs112 and icon/studio.rs113",
    "owner": "png0.17::EncodingError",
    "repair": "Retain exact native cause in IconError and connected receiving cache/result graph; same-version direct dependency if needed. No Clone/Eq/Serde restriction in actual IconError.",
    "acceptance": "Original IO/format/parameter/limit payload survives until actual terminal output; preserve true PNG bytes/native pixel conversion and all existing oracles.",
    "status": "carried finding; no source repair or validation"
})
append("progress.jsonl", {
    "id": "RIG-OWNER-DELTA-PROGRESS-002", "lens": "audit-existing-type-owners", "status": "incomplete",
    "assigned": 235, "accounted": 5, "accounted_queue_indices": [0, 1, 2, 160, 161],
    "first_unfinished_queue_index": 3,
    "active_dependency_extensions": "additional-sources.jsonl entries001-003; GPU extent/Buffer, scan count/actual record, KernelBatch copy and compute manifest",
    "new_findings": ["RIG-OWNER-DELTA-F001"], "carried_findings": ["RIG-OWNER-DELTA-CARRIED-PNG-001"],
    "source_edits": 0, "validation": "none", "overall_lens_clearance": "not awarded"
})
print(json.dumps({"appended_sources": list(roles), "first_unfinished": 3}))
