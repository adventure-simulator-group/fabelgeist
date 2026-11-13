#!/usr/bin/env python3
"""Verify a published evidence successor and optional exact predecessors."""
import argparse
import hashlib
import json
import os
import stat
import subprocess
import sys
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
ap = argparse.ArgumentParser()
ap.add_argument("--publication", type=Path, required=True)
ap.add_argument("--predecessor", type=Path)
ap.add_argument("--ancestor", type=Path)
ap.add_argument("--object-alternate", type=Path)
ap.add_argument("--object-inventory", type=Path)
ap.add_argument("--object-inventory-sha256")
a = ap.parse_args()
assert not a.ancestor or a.predecessor
assert all((a.object_alternate, a.object_inventory, a.object_inventory_sha256)) or not any((a.object_alternate, a.object_inventory, a.object_inventory_sha256))
assert not a.object_alternate or a.predecessor
root = a.publication.resolve()
stage = root / "delegated/post8b-followups"
guard = stage / "checkpoint-guard"
assert Path(__file__).resolve() == guard / "VERIFY.py" and not guard.is_symlink()
assert {x.name for x in guard.iterdir()} == {"VERIFY.py", "STAGE-MANIFEST.json"}
sha = lambda b: hashlib.sha256(b).hexdigest()
driver_path = root / "check_published_evidence_privacy.py"
assert driver_path.is_file() and not driver_path.is_symlink()
assert sha(driver_path.read_bytes()) == "b787d6726b40aa0a49cb34325763c6d4ca5434f2830a845629e934e61ba6d41f"
manifest_path = guard / "STAGE-MANIFEST.json"
assert manifest_path.is_file() and not manifest_path.is_symlink()
manifest_raw = manifest_path.read_bytes()
assert sha(manifest_raw) == "b236abdb65fa542f74a4b0f1e8568ed8c6e753cb6f15e014be0b1914ce542222"
manifest = json.loads(manifest_raw)
assert manifest["schema"] == "726-post8b-installed-stage-v1"
actual = set()
for path in stage.rglob("*"):
    assert not path.is_symlink()
    if path.is_file() and guard not in path.parents:
        actual.add(path.relative_to(stage).as_posix())
    else:
        assert path.is_file() or path.is_dir()
assert actual == set(manifest["files"])
for name, row in manifest["files"].items():
    rel = PurePosixPath(name)
    assert not rel.is_absolute() and ".." not in rel.parts and str(rel) == name
    path = stage / name
    assert path.is_file() and not path.is_symlink()
    data = path.read_bytes()
    assert (len(data), sha(data), bool(stat.S_IMODE(path.stat().st_mode) & stat.S_IXUSR)) == (row["bytes"], row["sha256"], row["owner_executable"])
env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
env.update(PYTHONDONTWRITEBYTECODE="1", PYTHONNOUSERSITE="1", GIT_TERMINAL_PROMPT="0",
           GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_NO_REPLACE_OBJECTS="1",
           GIT_OPTIONAL_LOCKS="0", GIT_CONFIG_COUNT="4", GIT_CONFIG_KEY_0="gc.auto", GIT_CONFIG_VALUE_0="0",
           GIT_CONFIG_KEY_1="maintenance.auto", GIT_CONFIG_VALUE_1="false",
           GIT_CONFIG_KEY_2="core.fsmonitor", GIT_CONFIG_VALUE_2="false",
           GIT_CONFIG_KEY_3="core.hooksPath", GIT_CONFIG_VALUE_3=os.devnull)
env.pop("PYTHONOPTIMIZE", None)
if a.object_alternate:
    alternate, inventory_path = a.object_alternate, a.object_inventory
    assert alternate.is_absolute() and alternate.resolve() == alternate and alternate.is_dir() and not alternate.is_symlink()
    assert ":" not in str(alternate) and "\n" not in str(alternate)
    assert inventory_path.is_absolute() and inventory_path.resolve() == inventory_path and inventory_path.is_file() and not inventory_path.is_symlink()
    expected_inventory_sha = a.object_inventory_sha256
    assert len(expected_inventory_sha) == 64 and all(c in "0123456789abcdef" for c in expected_inventory_sha)
    inventory_raw = inventory_path.read_bytes()
    assert sha(inventory_raw) == expected_inventory_sha
    inventory = json.loads(inventory_raw)
    assert inventory["schema"] == "726-supplied-git-object-alternate-inventory-v1"
    actual_files, actual_dirs = {}, []
    for path in sorted(alternate.rglob("*")):
        assert not path.is_symlink()
        name = path.relative_to(alternate).as_posix()
        if path.is_file():
            data = path.read_bytes()
            actual_files[name] = {"bytes": len(data), "sha256": sha(data)}
        else:
            assert path.is_dir()
            actual_dirs.append(name)
    assert actual_files == inventory["files"] and actual_dirs == inventory["directories"]
    env["GIT_ALTERNATE_OBJECT_DIRECTORIES"] = str(alternate)
    print("The explicitly supplied alternate matched its separately supplied inventory; other shared Git object stores remain separate dependencies.")
for checkout in (a.predecessor, a.ancestor):
    if checkout:
        checkout = checkout.resolve()
        assert checkout.is_dir()
        top = subprocess.run(["git", "-c", "gc.auto=0", "-c", "maintenance.auto=false", "rev-parse", "--show-toplevel"],
                             cwd=checkout, env=env, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout.decode().strip()
        assert Path(top).resolve() == checkout
def run(relative, *args):
    subprocess.run([sys.executable, "-B", str(stage / relative), *map(str, args)], cwd=root, env=env, check=True)
for script in ("public-transports/VERIFY.py", "later-transports/VERIFY.py", "late-public-transports/VERIFY.py",
               "supplemental-public-transports/VERIFY.py", "additional-public-transports/VERIFY.py",
               "followup-public-transports/VERIFY.py", "calendar-public-transports/VERIFY.py",
               "q3021-peer-public-transports/VERIFY.py", "installer-review-public-transports/VERIFY.py",
               "pending-eight/VERIFY.py", "pending-eight-correction/VERIFY.py", "pending-two/VERIFY.py",
               "pending-calendar-bestiary/VERIFY.py", "reconciliation/VERIFY.py", "review-binding/VERIFY.py",
               "q3033-public-transports/VERIFY.py", "q3023-public-transports/VERIFY.py",
               "guard-v4-review-public-transports/VERIFY.py", "custody-encrypted/VERIFY-SPLIT.py",
               "custody-second-encrypted/VERIFY.py", "custody-third-encrypted/VERIFY.py",
               "custody-fourth-encrypted/VERIFY.py"):
    run(script)
run("custody-receipt/VERIFY-RECEIPT.py")
# The first ciphertext group is transported in lossless pieces. The separate
# streaming verifier checks all original ciphertext identities and nonces;
# bind the unchanged original controls to the retained receipt as well.
assert sha((stage / "custody-receipt/MANIFEST.json").read_bytes()) == "2e00f372fd1695cae27a53e253b8d70bf647b5c518fe2d1df7ca9fd8d7442a17"
first_binding = json.loads((stage / "custody-receipt/CIPHERTEXT-BINDING.json").read_bytes())
assert set(first_binding["auxiliary_metadata"]) == {"README.md", "VERIFY.py"}
first_original = stage / "custody-encrypted/ORIGINAL-MANIFEST.json"
assert first_binding["ciphertext_manifest_sha256"] == sha(first_original.read_bytes())
for name, row in first_binding["auxiliary_metadata"].items():
    assert name in {"README.md", "VERIFY.py"}
    path = stage / "custody-encrypted" / ("ORIGINAL-" + name)
    assert path.is_file() and not path.is_symlink()
    raw = path.read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
first_scan = json.loads((stage / "custody-receipt/SCAN.json").read_bytes())
assert first_scan["schema"] == "726-post8b-finite-public-scan-v1"
assert first_scan["ciphertext_manifest_sha256"] == sha((stage / "custody-encrypted/ORIGINAL-MANIFEST.json").read_bytes())
expected_first_patterns = {"aws_access_key", "credential_assignment", "email", "file_url", "github_token", "home_path", "private_key", "temporary_path", "windows_user_path"}
assert set(first_scan["pattern_counts"]) == expected_first_patterns and all(value == 0 for value in first_scan["pattern_counts"].values())
assert set(first_scan["private_derived_matches"]) == {"private_path_or_digest", "key_form", "nonce_form"} and all(value == 0 for value in first_scan["private_derived_matches"].values())
assert set(first_scan["known_path_digest_corpus_matches"]) == {"original_059", "q2332", "gpu"} and all(value == 0 for value in first_scan["known_path_digest_corpus_matches"].values())
assert first_scan["match_count"] == first_scan["output_raw_hex_base64_key_matches"] == 0 and first_scan["positive_controls_pass"] and first_scan["windows_controls_pass"]
assert (first_scan["source_text_files_scanned"], first_scan["source_files_skipped_large"], first_scan["source_files_skipped_binary"]) == (2316, 8, 260)

# Git preserves executable bits but not the receipt's original full POSIX modes.
# Check both retained generations as Git bytes, and check the recorded restore.
for label, expected_schema in (("custody-second-receipt-v1", "726-post8b-second-receipt-v1"),
                               ("custody-second-receipt-v2", "726-post8b-second-receipt-v2")):
    receipt = stage / label
    recorded = json.loads((receipt / "MANIFEST.json").read_bytes())
    assert recorded["schema"] == expected_schema and recorded["directories"] == []
    rows = {row["path"]: row for row in recorded["files"]}
    assert {x.name for x in receipt.iterdir()} == set(rows) | {"MANIFEST.json"}
    for name, row in rows.items():
        assert len(PurePosixPath(name).parts) == 1
        path = receipt / name
        assert path.is_file() and not path.is_symlink()
        data = path.read_bytes()
        assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
        assert bool(stat.S_IMODE(path.stat().st_mode) & stat.S_IXUSR) == bool(int(row["mode"], 8) & 0o100)
    restoration = json.loads((receipt / "RESTORATION.json").read_bytes())
    assert restoration["schema"] == "726-post8b-second-restoration-v1"
    assert restoration["ciphertext_manifest_sha256"] == sha((stage / "custody-second-encrypted/MANIFEST.json").read_bytes())
    assert (restoration["collections"], restoration["source_trees"], restoration["immediate_snapshots"], restoration["control_collections"]) == (53, 49, 3, 1)
    assert (restoration["source_regular_files"], restoration["source_directories_including_roots"], restoration["control_regular_files"]) == (4786, 226, 16)
    assert restoration["restore_exit"] == 0 and restoration["matched_file_bytes_and_modes"] and restoration["matched_directory_and_root_modes"] and restoration["source_before_after_equal"]
    assert (restoration["prior_nonce_count"], restoration["new_nonce_count"], restoration["cumulative_nonce_count"]) == (818, 53, 871)
    scan = json.loads((receipt / "PUBLIC-SCAN.json").read_bytes())
    assert scan["positive_controls_pass"] and scan["source_files_skipped_binary"] == 52 and scan["source_files_skipped_large"] == 0
    assert scan["source_text_files_scanned"] == 4734 and scan["ciphertexts_key_scanned"] == 53
    expected_patterns = {"aws_access_key", "credential_assignment", "email", "file_url", "github_token", "home_path", "private_key", "temporary_path", "windows_user_path"}
    assert set(scan["pattern_counts"]) == expected_patterns and all(value == 0 for value in scan["pattern_counts"].values())
    assert all(scan[k] == 0 for k in ("private_source_literal_or_digest_hits", "key_form_hits_in_metadata", "nonce_form_hits_in_metadata", "ciphertext_key_form_hits", "baseline059_private_digest_hits", "q2332_private_digest_hits", "gpu_path_or_prefix_digest_hits"))
first_receipt = stage / "custody-second-receipt-v1"
second_receipt = stage / "custody-second-receipt-v2"
correction = json.loads((second_receipt / "RECEIPT-CORRECTION.json").read_bytes())
assert correction["schema"] == "726-post8b-second-receipt-correction-v1"
assert correction["predecessor_manifest_sha256"] == sha((first_receipt / "MANIFEST.json").read_bytes())
assert (first_receipt / "RESTORATION.json").read_bytes() == (second_receipt / "RESTORATION.json").read_bytes()
first_scan_version = json.loads((first_receipt / "PUBLIC-SCAN.json").read_bytes())
second_scan_version = json.loads((second_receipt / "PUBLIC-SCAN.json").read_bytes())
assert first_scan_version.pop("schema") == "726-post8b-second-public-scan-v1"
assert second_scan_version.pop("schema") == "726-post8b-second-public-scan-v2" and first_scan_version == second_scan_version
# The public third receipt records an authenticated same-machine restore. Git
# does not preserve its original full POSIX modes, so recheck bytes and the
# owner-executable bit here; do not claim that authentication is repeated.
third_receipt = stage / "custody-third-receipt"
third_manifest_raw = (third_receipt / "MANIFEST.json").read_bytes()
assert sha(third_manifest_raw) == "0c422e7021d34ee0636c5ea8b0caf5ed5546cfaa07350731861d13322eb9a175"
third_manifest = json.loads(third_manifest_raw)
assert third_manifest["schema"] == "726-post8b-third-custody-receipt-v1" and third_manifest["directories"] == []
third_rows = {row["path"]: row for row in third_manifest["files"]}
assert {x.name for x in third_receipt.iterdir()} == set(third_rows) | {"MANIFEST.json"}
for name, row in third_rows.items():
    assert len(PurePosixPath(name).parts) == 1
    path = third_receipt / name
    assert path.is_file() and not path.is_symlink()
    data = path.read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
    assert bool(stat.S_IMODE(path.stat().st_mode) & stat.S_IXUSR) == bool(int(row["mode"], 8) & 0o100)
third_restore = json.loads((third_receipt / "RESTORATION.json").read_bytes())
assert third_restore["schema"] == "726-post8b-third-restoration-v1"
assert third_restore["ciphertext_manifest_sha256"] == sha((stage / "custody-third-encrypted/MANIFEST.json").read_bytes())
assert third_restore["collections"] == {"source_trees": 93, "immediate_file_snapshots": 2, "pre_capture_controls": 1, "authenticated_objects": 96}
assert (third_restore["source_regular_files"], third_restore["source_directories_including_roots"], third_restore["control_regular_files"]) == (10701, 859, 43)
assert third_restore["restore_exit"] == 0 and third_restore["matched_file_bytes_and_modes"] and third_restore["matched_directory_and_root_modes"] and third_restore["source_before_after_equal"]
assert (third_restore["prior_unique_nonces"], third_restore["new_unique_nonces"], third_restore["cumulative_unique_nonces"]) == (871, 96, 967)
third_scan = json.loads((third_receipt / "SCAN-INITIAL.json").read_bytes())
assert third_scan["schema"] == "726-post8b-third-public-metadata-scan-initial-v1"
assert set(third_scan["pattern_counts"]) == expected_first_patterns and all(value == 0 for value in third_scan["pattern_counts"].values())
assert set(third_scan["private_derived_matches"]) == {"private_path_or_digest", "key_form", "nonce_form"} and all(value == 0 for value in third_scan["private_derived_matches"].values())
assert third_scan["ciphertext_key_form_matches"] == third_scan["match_count"] == 0 and third_scan["positive_controls_pass"]
assert (third_scan["source_text_files_scanned"], third_scan["source_files_skipped_large"], third_scan["source_files_skipped_binary"], third_scan["ciphertexts_key_scanned"]) == (10545, 3, 153, 96)
schema_qualification = (stage / "CUSTODY-THIRD-SCHEMA-QUALIFICATION.json").read_bytes()
assert sha(schema_qualification) == "480376664886affd69f60de95adb84e53bf6ac585f5afd0076532f0b7ba2377e"
schema_data = json.loads(schema_qualification)
assert schema_data["schema"] == "726-post8b-third-custody-inherited-label-qualification-v1"
assert (schema_data["collections"], schema_data["regular_files"], schema_data["directories_including_roots"]) == (95, 10701, 859)
correction = json.loads((stage / "Q3023-TRANSPORT-CORRECTION.json").read_bytes())
assert correction["schema"] == "726-q3023-transport-scope-correction-v1" and correction["original_readme"] == "q3023-public-transports/README.md"
original_readme = (stage / correction["original_readme"]).read_bytes()
assert sha(original_readme) == correction["original_readme_sha256"] and correction["original_wording"] in original_readme.decode()
assert "If supplied to the selected consumers" in correction["corrected_wording"]
# The fourth receipt records an authenticated same-machine restore. Check
# Git bytes and owner-executable bits here, not original full POSIX modes.
fourth_receipt = stage / "custody-fourth-receipt"
fourth_manifest_raw = (fourth_receipt / "MANIFEST.json").read_bytes()
assert sha(fourth_manifest_raw) == "46254bf58ab5f506a0f22eec44678f07ea0cbeb32426b56ce1a35545ad0dfca7"
fourth_manifest = json.loads(fourth_manifest_raw)
assert fourth_manifest["schema"] == "726-post8b-fourth-custody-receipt-v1" and fourth_manifest["directories"] == []
fourth_rows = {row["path"]: row for row in fourth_manifest["files"]}
assert {x.name for x in fourth_receipt.iterdir()} == set(fourth_rows) | {"MANIFEST.json"}
for name, row in fourth_rows.items():
    assert len(PurePosixPath(name).parts) == 1
    path = fourth_receipt / name
    assert path.is_file() and not path.is_symlink()
    data = path.read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
    assert bool(stat.S_IMODE(path.stat().st_mode) & stat.S_IXUSR) == bool(int(row["mode"], 8) & 0o100)
fourth_restore = json.loads((fourth_receipt / "RESTORATION.json").read_bytes())
assert fourth_restore["schema"] == "726-post8b-fourth-restoration-v1"
assert fourth_restore["ciphertext_manifest_sha256"] == sha((stage / "custody-fourth-encrypted/MANIFEST.json").read_bytes())
assert fourth_restore["collections"] == {"source_trees": 30, "immediate_file_snapshots": 1, "pre_capture_controls": 1, "authenticated_objects": 32}
assert (fourth_restore["source_regular_files"], fourth_restore["source_directories_including_roots"], fourth_restore["control_regular_files"]) == (2062, 248, 42)
assert fourth_restore["restore_exit"] == 0 and fourth_restore["matched_file_bytes_and_modes"] and fourth_restore["matched_directory_and_root_modes"] and fourth_restore["source_before_after_equal"]
assert (fourth_restore["prior_unique_nonces"], fourth_restore["new_unique_nonces"], fourth_restore["cumulative_unique_nonces"]) == (967, 32, 999)
fourth_scan = json.loads((fourth_receipt / "SCAN-INITIAL.json").read_bytes())
assert fourth_scan["schema"] == "726-post8b-fourth-public-metadata-scan-initial-v1"
assert set(fourth_scan["pattern_counts"]) == expected_first_patterns and all(value == 0 for value in fourth_scan["pattern_counts"].values())
assert set(fourth_scan["private_derived_matches"]) == {"private_path_or_digest", "key_form", "nonce_form"} and all(value == 0 for value in fourth_scan["private_derived_matches"].values())
assert fourth_scan["ciphertext_key_form_matches"] == fourth_scan["match_count"] == 0 and fourth_scan["positive_controls_pass"]
assert (fourth_scan["source_text_files_scanned"], fourth_scan["source_files_skipped_large"], fourth_scan["source_files_skipped_binary"], fourth_scan["ciphertexts_key_scanned"]) == (2050, 5, 7, 32)
# Bind the optional nested predecessor executables and their inventory anchors
# directly before the candidate verifier can launch them. These pins do not
# remove concurrent filesystem replacement or shared-object dependencies.
if a.predecessor and a.ancestor:
    nested_pins = [({'verify_post8ca_checkpoint.py': 'b52a3c6d3f20d94dcae073a8a4fad7c2851066fc96665c19d64276d9cbfebf94', 'delegated/post8ca-followups/final-guard/VERIFY.py': '58f3a7710c40874504b6a1f757fc8629f96bc2760f1abb4c4c0748e08a4964fe', 'delegated/post8ca-followups/final-guard/PINS.json': '5b41da119a95aa46dd9ba99cfb732a288c47034e90322a184bf5f0c715a0ad01', 'delegated/post8ca-followups/STAGE-CAPTURE.json': 'e50f1c8401897d2fca9c3cb050be3214b718a05127a9ada72ce7ae6301b51088', 'delegated/post8ca-followups/ADOPTION-STATUS.json': '40b6534d6fcd769ca4147e855978f811e2eb3f89aa417936ce4940b33142e965'}, 'predecessor'), ({'verify_post76fe_checkpoint.py': '8f5d68f704bf20693b038d31be16c941f40332457eb0184ca09400e2dcbc6c3f', 'delegated/post76fe-followups/final-guard/VERIFY.py': 'ac6b8ff0879d806710925af66420e5d350ffdd8f73b9d817c94f92d4a5cfc33f', 'delegated/post76fe-followups/final-guard/PINS.json': '0ebd265d0e5c62d6ae42a71317f346e24395ecafccae9887a0972d5ba0cdfe14', 'delegated/post76fe-followups/STAGE-CAPTURE.json': 'ff005088252ffa8f8b82fb9fea1f2b5ac407f335aa90da6105ed411881797651', 'delegated/post76fe-followups/ADOPTION-STATUS.json': '30e75f08e91ffe7d6d96fb23e97b47ca05d499a68fb962518ebc248b1dfbf327', 'delegated/post76fe-followups/closing/VERIFY.py': '1264ca113b81ec0237e7a8a5c440d562ccebb37b77c2608b16f69529d7a274f9', 'delegated/post76fe-followups/closing/CAPTURE.json': '4131c02d57b8a3cd2cba51be9341d70ca284d7154eeb90687998f10e36c50bca', 'delegated/post76fe-followups/closing/SCAN.json': '477eb9b397b4cff32c8bb22c0b08de947738bdb7aeb72761ae0da56dd8a45437'}, 'ancestor')]
    for pins, label in nested_pins:
        checkout = (a.predecessor if label == "predecessor" else a.ancestor).resolve()
        assert checkout.is_dir() and not checkout.is_symlink()
        for name, expected in pins.items():
            path = checkout / name
            assert path.is_file() and not path.is_symlink() and path.resolve() == path
            assert sha(path.read_bytes()) == expected
candidate_args = ["--publication", root]
if a.predecessor:
    candidate_args += ["--predecessor", a.predecessor.resolve()]
if a.ancestor:
    candidate_args += ["--ancestor", a.ancestor.resolve()]
run("application-candidate-v2/VERIFY.py", *candidate_args)
print("Staged bytes and Git executable bits, public transport identities, recorded custody and exact candidate installation matched. Full receipt modes, authentication, prior-nonce comparison and independent custody are not repeated here; see their separately retained evidence.")
print("Main is point-in-time. Missing checkpoint 3821, SOURCE004, 73 nontext/no-positive-EOF contracts, semantic audit closure and product behavior remain open.")
