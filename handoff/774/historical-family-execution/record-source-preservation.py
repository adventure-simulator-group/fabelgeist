"""Record Git/source metadata; this does not run or parse Rust/project checks."""
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import re
import subprocess

REPORTS = Path(__file__).resolve().parent
PRIMARY = REPORTS.parents[2]
WORKTREE = REPORTS.parent / "rig-review-774/worktree"
SOURCE_BASE = "82cb2a3acccc852bba6a7ef9c706c337ba9fe765"


def git(*arguments):
    return subprocess.check_output(["git", *arguments], cwd=WORKTREE)


paths = set(git("diff", SOURCE_BASE, "--no-renames", "--name-only", "-z").decode().split("\0"))
paths |= set(git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0"))
paths.discard("")
test_pattern = re.compile(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*(?:async\s+)?fn\s+(\w+)")
assertion_pattern = re.compile(
    r"\b(?:assert|assert_eq|assert_ne|debug_assert|debug_assert_eq|debug_assert_ne)\s*!"
)
inventory = json.loads((REPORTS / "P06-before-test-inventory.json").read_text())["files"]
old_names, new_names = Counter(), Counter()
old_count = new_count = 0
files = {}
for name in sorted(paths):
    if not name.endswith(".rs"):
        continue
    result = subprocess.run(
        ["git", "show", SOURCE_BASE + ":" + name], cwd=WORKTREE, capture_output=True, check=False
    )
    original = result.stdout if result.returncode == 0 else b""
    current = (WORKTREE / name).read_bytes() if (WORKTREE / name).is_file() else b""
    if name in inventory:
        assert hashlib.sha256(original).hexdigest() == inventory[name]["sha256"], name
    old_text, new_text = original.decode(), current.decode()
    before, after = test_pattern.findall(old_text), test_pattern.findall(new_text)
    old_names.update(before)
    new_names.update(after)
    old_asserts = len(assertion_pattern.findall(old_text))
    new_asserts = len(assertion_pattern.findall(new_text))
    old_count += old_asserts
    new_count += new_asserts
    files[name] = {
        "state": "deleted" if not current else "new" if not original else "modified",
        "sha256": hashlib.sha256(current).hexdigest() if current else None,
        "test_names_before": before,
        "test_names_after": after,
        "assertions_before": old_asserts,
        "assertions_after": new_asserts,
    }
upstream_path = REPORTS / "P06-upstream-source-deltas.json"
upstream = json.loads(upstream_path.read_text()) if upstream_path.exists() else None
if upstream:
    incoming = upstream["incoming_main"]
    ancestor = subprocess.run(
        ["git", "merge-base", "--is-ancestor", incoming, "HEAD"], cwd=WORKTREE
    )
    if ancestor.returncode:
        assert git("rev-parse", "MERGE_HEAD").decode().strip() == incoming
    for name, delta in upstream["files"].items():
        if name in paths:
            old_names.update(delta["test_names_added"])
            old_names.subtract(delta["test_names_removed"])
            old_count += delta["assertion_delta"]
    old_names = +old_names

authorized_fixture_renames = [
    {
        "before": "unweighted_physics_preserves_rest_shape_and_ignores_invalid_skin_weights",
        "after": "unweighted_physics_preserves_rest_shape",
        "source_commit": "6aee89ebf17bf3e823ff9003bca82596f608da0f",
        "authority": "execution-ledger.json#events fixture_contract_decision_and_provenance",
        "limit": "Same physical assertions; invalid unused skin payload is excluded by the new explicit binding state. No executed rejection coverage claimed.",
    },
]
for rename in authorized_fixture_renames:
    assert old_names[rename["before"]] == 1
    assert new_names[rename["before"]] == 0
    assert old_names[rename["after"]] == 0
    assert new_names[rename["after"]] == 1
    old_names[rename["before"]] -= 1
    old_names[rename["after"]] += 1
old_names = +old_names
assert old_names == new_names, {
    "removed": list((old_names - new_names).elements()),
    "added": list((new_names - old_names).elements()),
}
production_rejections = [
    {
        "before": "crates/adventuresim-tactical-client/src/tactical_scene_viewer/furniture_capture.rs",
        "after": "crates/adventuresim-tactical-client/src/tactical_scene_viewer/furniture_capture.rs",
        "assertion": 'assert!(\n        !market.is_empty(),\n        "furniture review requires generated market vendors"\n    );',
        "rejection": 'if market.is_empty() {\n        return Err(FurnitureCaptureError::MissingVendors.into());\n    }',
    },
    {
        "before": "crates/adventuresim-tactical-client/src/tactical_scene_viewer/interior_furniture_capture.rs",
        "after": "crates/adventuresim-tactical-client/src/tactical_scene_viewer/interior_furniture_capture/catalog.rs",
        "assertion": 'assert!(\n        layout.instances.is_empty(),\n        "catalog fixture must have no placed furniture"\n    );',
        "rejection": 'if !layout.instances.is_empty() {\n        return Err(CatalogAdmissionError::AlreadyPlaced.into());\n    }',
    },
]
for change in production_rejections:
    before = git("show", SOURCE_BASE + ":" + change["before"]).decode()
    after = (WORKTREE / change["after"]).read_text()
    assert before.count(change["assertion"]) == 1, change["before"]
    assert after.count(change["rejection"]) == 1, change["after"]
    assert change["assertion"] in before.split("#[cfg(test)]")[0], change["before"]
assert old_count == new_count + len(production_rejections), (old_count, new_count)
ledger = json.loads((REPORTS / "execution-ledger.json").read_text())
plan = json.loads((REPORTS.parent / "pr774-canary-audit-20261020/migration-plan.json").read_text())
assert {item["id"]: item["exit_requirement"] for item in ledger["items"]} == {
    item["id"]: item["exit"] for item in plan["items"]
}
path = REPORTS / "P06-source-progress.json"
progress = json.loads(path.read_text())
progress.update(
    recorded_at=datetime.now(timezone.utc).isoformat(),
    source_head=git("rev-parse", "HEAD").decode().strip(),
    published_head=git("rev-parse", "origin/codex/shared-rig-identity-726").decode().strip(),
    published_head_authority="Locally recorded remote-tracking branch; no network or CI request from this metadata helper.",
    source_comparison_base=SOURCE_BASE,
    changed_files=files,
    test_names_in_changed_responsibilities=sum(old_names.values()),
    assertion_occurrences_before_after=[old_count, new_count],
    production_assertions_replaced_with_typed_rejections=production_rejections,
    authorized_fixture_renames=authorized_fixture_renames,
    assertion_metadata_limit="All test names preserved except the one explicit authorized representational-exclusion rename; assertion counts preserved except exactly two verified production assertions becoming typed rejections. This is not a parsed or executed test verdict.",
    upstream_source_delta_report="P06-upstream-source-deltas.json" if upstream else None,
    scope="Source metadata includes incoming main; upstream test/assertion changes are independently accounted for and are not new tests authored for this goal.",
    all_twenty_original_exit_requirements_preserved=True,
)
path.write_text(json.dumps(progress, indent=2) + "\n")
print(json.dumps({
    "changed_rust_paths": len(files),
    "existing_test_names_preserved": sum(old_names.values()),
    "remaining_assertion_occurrences_preserved": new_count,
    "verified_production_assertions_replaced_with_typed_rejections": len(production_rejections),
    "all_twenty_exit_requirements_preserved": True,
    "coverage": "Static source metadata only; this helper establishes no executed validation.",
}, indent=2))
