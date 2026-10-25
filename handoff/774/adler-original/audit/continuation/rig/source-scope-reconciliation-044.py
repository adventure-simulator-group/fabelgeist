"""Reconcile explicit receipt metadata, never scan source semantics or award coverage."""
from collections import Counter, defaultdict
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).parent


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def fingerprint(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


INDEX_PATH = ROOT / "continuation/producer-reached-role-index.json"
LOCATORS_PATH = ROOT / "continuation/source-locators.json"
index = read_json(INDEX_PATH)
locators = read_json(LOCATORS_PATH)
context = read_json(ROOT / "continuation-context.json")
frozen = locators["matched"]
by_id = {r["source_id"]: r for r in frozen}
by_hash = defaultdict(list)
for row in frozen:
    if row.get("sha256"):
        by_hash[row["sha256"]].append(row)


def canonical_path_identity(source_id):
    """Revision contexts remain distinct; only dependency package spelling normalizes."""
    prefix, _, suffix = source_id.partition(":")
    if prefix in context["snapshots"]:
        return "snapshot:" + prefix + ":" + suffix.replace("\\", "/")
    if prefix in ("foreign", "native-dependency", "pinned-registry"):
        return "registry:" + suffix.replace("\\", "/")
    if prefix.startswith("pinned-"):
        return "registry:" + prefix[7:] + "/" + suffix.replace("\\", "/")
    return source_id


by_canonical = defaultdict(list)
for row in frozen:
    by_canonical[canonical_path_identity(row["source_id"])].append(row)

file_cache = {}


def file_metadata(path):
    path = Path(path)
    if not path.is_absolute():
        # Locators may be repository-relative, rather than artifact-root-relative.
        path = Path.cwd() / path
    key = str(path.resolve())
    if key not in file_cache:
        if not path.is_file():
            file_cache[key] = {"path": key, "exists": False}
        else:
            data = path.read_bytes()
            file_cache[key] = {
                "path": key,
                "exists": True,
                "sha256": hashlib.sha256(data).hexdigest(),
                "actual_line_count": len(data.splitlines()),
            }
    return file_cache[key]


registry_roots = set()
for row in frozen:
    path = row.get("resolved_path", "").replace("\\", "/")
    if "/registry/src/" in path:
        head, tail = path.split("/registry/src/", 1)
        registry_roots.add(Path(head + "/registry/src/" + tail.split("/")[0]))


def candidates(source_id, source_context):
    result = []
    if source_id in by_id:
        result.append(("exact-frozen-source-id", by_id[source_id]["resolved_path"]))
    for row in by_canonical[canonical_path_identity(source_id)]:
        result.append(("normalized-package-source-id", row["resolved_path"]))
    prefix, _, suffix = source_id.partition(":")
    if prefix in context["snapshots"]:
        result.append(("exact-snapshot-context", str(Path(context["snapshots"][prefix]) / suffix)))
    canonical = canonical_path_identity(source_id)
    if canonical.startswith("registry:"):
        rel = canonical[len("registry:"):]
        for root in sorted(registry_roots):
            result.append(("exact-package-version-registry-path", str(root / rel)))
    # Existing receipt paths are locators only, never source-read evidence.
    for key in ("resolved_path", "path", "actual_path"):
        value = source_context.get(key)
        if not isinstance(value, str):
            continue
        if value.startswith("SNAPSHOTS/"):
            parts = value.split("/", 2)
            if len(parts) == 3 and parts[1] in context["snapshots"]:
                result.append(("receipt-snapshot-path", str(Path(context["snapshots"][parts[1]]) / parts[2])))
        elif value.startswith("CARGO_REGISTRY/"):
            for root in sorted(registry_roots):
                result.append(("receipt-registry-path", str(root / value[len("CARGO_REGISTRY/"):])))
        elif Path(value).is_absolute():
            result.append(("receipt-absolute-path", value))
    seen = set()
    rows = []
    for basis, path in result:
        metadata = file_metadata(path)
        if metadata["path"] not in seen:
            rows.append(dict(metadata, locator_basis=basis))
            seen.add(metadata["path"])
    return rows


def ranges_report(raw, line_count):
    if not raw:
        return {"status": "absent", "original": raw, "normalized": None}
    flat = len(raw) == 2 and all(isinstance(v, int) for v in raw)
    normalized = [raw] if flat else raw
    problems = []
    for pair in normalized:
        if not isinstance(pair, list) or len(pair) != 2 or not all(isinstance(v, int) for v in pair):
            problems.append({"range": pair, "reason": "not-two-integer-endpoints"})
        elif pair[0] < 1 or pair[1] < pair[0]:
            problems.append({"range": pair, "reason": "invalid-one-based-inclusive-range"})
        elif line_count is not None and pair[1] > line_count:
            problems.append({"range": pair, "reason": "end-after-actual-EOF", "actual_EOF": line_count})
    return {
        "original": raw,
        "normalized": normalized,
        "flat_pair_normalized": flat,
        "status": "endpoint-conflict" if problems else "qualified-endpoints-only",
        "problems": problems,
        "authority": "Recorded read scope only; byte/range validity never establishes material-role or owner-fit clearance.",
    }


def ref(row):
    return {k: row.get(k) for k in ("record", "receipt", "json_pointer")}


qualified = index["explicit_source_range_observations"]
groups = defaultdict(list)
for row in qualified:
    if not row["frozen_source_identity"] or not row["frozen_fingerprint_match"]:
        groups[(row["source_id"], row["sha256"])].append(row)

variant_rows = []
for (source_id, claimed_sha), observations in sorted(groups.items()):
    candidate_rows = candidates(source_id, observations[0]["source_context"])
    matching = [r for r in candidate_rows if r.get("sha256") == claimed_sha]
    existing = [r for r in candidate_rows if r["exists"]]
    resolved = matching[0] if matching else (existing[0] if existing else None)
    canonical_frozen = by_canonical[canonical_path_identity(source_id)]
    canonical_matches = [r for r in canonical_frozen if r.get("sha256") == claimed_sha]
    if matching and canonical_matches:
        category = "metadata-normalization-package-alias"
        reason = "Same exact package/version/relative path and SHA as named frozen variant; prefix spelling normalizes identity only."
    elif matching:
        category = "genuinely-new-reached-source-variant"
        reason = "Concrete pinned snapshot/package path and recorded bytes verified, but this source-context/path/hash is absent from frozen scope; material role and owner-fit remain pending."
    elif resolved:
        category = "unresolved-recorded-fingerprint-conflict"
        reason = "Concrete candidate exists but recorded SHA does not match its bytes. Do not replace the SHA, transfer ranges or grant coverage silently."
    else:
        category = "unresolved-source-locator"
        reason = "No exact pinned source locator found; preserve obligation and receipt reference."
    actual_lines = resolved.get("actual_line_count") if resolved else None
    rows = []
    for row in observations:
        rows.append(dict(ref(row), source_context=row["source_context"], range_metadata=ranges_report(row["read_ranges"], actual_lines)))
    same_bytes = [{"source_id": r["source_id"], "sha256": r["sha256"], "resolved_path": r["resolved_path"]} for r in by_hash[claimed_sha]]
    variant_rows.append({
        "source_id": source_id, "recorded_sha256": claimed_sha,
        "recorded_sha256_is_64_hex": bool(re.fullmatch("[0-9a-fA-F]{64}", claimed_sha or "")),
        "canonical_locator_identity": canonical_path_identity(source_id),
        "classification": category, "reason": reason,
        "resolved_candidate": resolved, "all_candidate_locators": candidate_rows,
        "frozen_alias_candidates": canonical_matches,
        "same_byte_frozen_counterpart_candidates": same_bytes,
        "same_byte_limit": "A PR/P16 or different-purpose counterpart is not an identity alias or semantic proof merely because bytes match.",
        "observations": rows, "material_owner_fit_status": "pending; no coverage awarded",
    })

receipt_cache = {}


def receipt_document(record, receipt):
    if record not in receipt_cache:
        path = ROOT / record
        if path.suffix == ".jsonl":
            receipt_cache[record] = [json.loads(line) for line in path.read_text(encoding="utf-8-sig").splitlines() if line.strip()]
        else:
            receipt_cache[record] = [read_json(path)]
    documents = receipt_cache[record]
    for i, document in enumerate(documents, 1):
        if receipt in (document.get("id"), document.get("trace_id"), document.get("receipt_id"), "row-" + str(i)):
            return document
    return None


def resolve_pointer(document, pointer):
    value = document
    for component in pointer.strip("/").split("/") if pointer else []:
        component = component.replace("~1", "/").replace("~0", "~")
        value = value[int(component)] if isinstance(value, list) else value[component]
    return value


qualified_by_id = defaultdict(list)
for row in qualified:
    qualified_by_id[row["source_id"]].append(row)

unqualified_rows = []
for ordinal, row in enumerate(index["incomplete_metadata_observations"]):
    metadata = row["source_context"]
    source_id = row["source_id"]
    missing_hash = not row.get("sha256")
    missing_ranges = not row.get("read_ranges")
    keys = set(metadata)
    if metadata.get("state") == "incomplete_not_started":
        category = "explicit-unread-progress-navigation"
    elif "current_variants" in row["json_pointer"]:
        category = "nested-variant-group-owns-range"
    elif "inventory_status" in keys or "trace_ref" in keys:
        category = "range-free-disposition-cross-reference"
    elif "evidence" in keys or "historical_exact_source_proofs" in keys:
        category = "range-free-supplier-evidence-cross-reference"
    elif "prior_read_ranges" in keys:
        category = "prior-range-plus-explicit-unread-recipient"
    elif missing_hash and not missing_ranges:
        category = "bounded-range-without-explicit-fingerprint"
    elif "read_status" in keys:
        category = "range-free-additional-variant-status"
    else:
        category = "range-free-variant-comparison-metadata"
    inherited = []
    try:
        document = receipt_document(row["record"], row["receipt"])
        parts = row["json_pointer"].split("/")
        for n in range(len(parts) - 1, 0, -1):
            pointer = "/".join(parts[:n])
            ancestor = resolve_pointer(document, pointer)
            if isinstance(ancestor, dict) and (ancestor.get("read_ranges") or ancestor.get("ranges")):
                # A range on a containing variant group is candidate inheritance only.
                inherited.append({
                    "record": row["record"], "receipt": row["receipt"], "json_pointer": pointer,
                    "source_id": ancestor.get("source_id"), "sha256": ancestor.get("sha256"),
                    "read_ranges": ancestor.get("read_ranges", ancestor.get("ranges")),
                    "proof_mode": ancestor.get("proof_mode"),
                    "limit": "Exact parent group metadata candidate; counterpart role/range reconciliation required, no automatic inheritance or coverage.",
                })
                break
    except (KeyError, ValueError, IndexError, TypeError):
        pass
    exact_candidates = []
    for candidate in qualified_by_id[source_id]:
        if row.get("sha256") and candidate["sha256"] != row["sha256"]:
            continue
        exact_candidates.append(dict(ref(candidate), sha256=candidate["sha256"], read_ranges=candidate["read_ranges"]))
    locs = candidates(source_id, metadata)
    existing = [r for r in locs if r["exists"]]
    unqualified_rows.append({
        "input_ordinal": ordinal, **ref(row), "source_id": source_id,
        "classification": category, "missing_explicit_hash": missing_hash,
        "missing_explicit_ranges": missing_ranges,
        "original_metadata": metadata,
        "locator_candidate_only": existing[0] if existing else None,
        "ancestor_range_candidates": inherited,
        "same_source_qualified_metadata_candidates": exact_candidates,
        "disposition": "Preserved unresolved metadata reference. Follow its exact parent/linked source proof during material-role/owner-fit reconciliation; no synthesized read or coverage.",
    })

result = {
    "id": "RIG-CONT-PPS-SOURCE-SCOPE-RECONCILIATION-044",
    "kind": "Read-only identity/range metadata reconciliation; no semantic source scanning, scope mutation or coverage award",
    "input_files": [{"path": str(p.relative_to(ROOT)), "sha256": fingerprint(p)} for p in (INDEX_PATH, LOCATORS_PATH, ROOT / "continuation-context.json")],
    "method": "Group outside-frozen explicit observations by exact source_id/SHA; resolve exact snapshot or package/version paths; hash concrete bytes and count line endpoints; compare frozen identities separately from same-byte counterpart candidates. Preserve all source_context and receipt/json-pointer references. Classify all unqualified entries and expose exact containing/linked metadata candidates without retroactive read credit.",
    "summary": {
        "frozen_source_variants": len(frozen),
        "qualified_input_observations": len(qualified),
        "outside_frozen_unique_source_id_hash": len(variant_rows),
        "outside_frozen_observations": sum(len(r["observations"]) for r in variant_rows),
        "variant_classification_counts": dict(Counter(r["classification"] for r in variant_rows)),
        "range_status_counts": dict(Counter(o["range_metadata"]["status"] for r in variant_rows for o in r["observations"])),
        "unqualified_input_observations": len(unqualified_rows),
        "unqualified_schema_reason_counts": dict(Counter(r["classification"] for r in unqualified_rows)),
    },
    "new_variant_identity_and_range_map": variant_rows,
    "unqualified_observation_schema_map": unqualified_rows,
    "limits": [
        "This reconciles metadata only. Genuinely new source variants still require explicit material-role inventory and owner-fit disposition before authoritative scope admission.",
        "Hash conflict, invalid digest, missing locator, absent range or range beyond EOF remain explicit unresolved qualifications. No clipping or inferred full-file reads.",
        "A qualified receipt may contain inherited or duplicated supplier evidence. Metadata count is not source-read count, coverage or lens clearance.",
        "Frozen source-locators, source scope and authoritative audit/execution ledgers remain untouched.",
    ],
    "source_edits": 0, "tests_or_project_commands": 0, "overall_lens_clearance": "not awarded",
}

target = OUT / "source-scope-reconciliation-044.json"
if target.exists():
    raise SystemExit("Append-only target already exists")
target.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result["summary"], indent=2))
print("output_sha256=" + fingerprint(target))
