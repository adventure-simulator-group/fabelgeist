"""Compare saved GitHub metadata, ZIPs and logs; never execute project code."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import re
import shlex
import sys
import tomllib
import zipfile

reports = Path(__file__).resolve().parent
stem = sys.argv[1]
expected = json.loads((reports / f"P06-hosted-contract-expectations-{stem}.json").read_text())
run = json.loads((reports / f"P06-hosted-{stem}-run.json").read_text())
job = json.loads((reports / f"P06-hosted-{stem}-job.json").read_text())
artifacts = json.loads((reports / f"P06-hosted-{stem}-artifacts.json").read_text())["artifacts"]
head = expected["head"]
assert run["id"] == expected["run"] == job["run_id"]
assert run["head_sha"] == job["head_sha"] == head
assert run["status"] == job["status"] == "completed"
assert run["conclusion"] == job["conclusion"] == "success"
assert all(s["conclusion"] == "success" for s in job["steps"] if s["number"] < 80)
worktree = reports.parent / "rig-review-774/worktree"
assert hashlib.sha256((worktree / ".github/workflows/rust-quality.yml").read_bytes()).hexdigest() == expected["workflow_sha256"]
ansi = re.compile(r"\x1b\[[0-9;]*m")
summary = re.compile(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", re.M)

def clean(text):
    return re.sub(r"^\d{4}-\d\d-\d\dT\S+ ?", "", ansi.sub("", text), flags=re.M)

def results(text):
    parsed = []
    doctests = False
    for line in clean(text).splitlines():
        if re.match(r"\s*Doc-tests\s", line):
            doctests = True
        elif re.match(r"\s*Running\s", line):
            doctests = False
        match = summary.fullmatch(line.split('; measured')[0]) or summary.match(line)
        if match:
            state, passed, failed, ignored = match.groups()
            assert state == "ok" and int(failed) == 0, line
            parsed.append({"passed": int(passed), "failed": int(failed), "ignored": int(ignored), "doctest": doctests})
    assert parsed and sum(x["passed"] for x in parsed) > 0
    return {
        "passed": sum(x["passed"] for x in parsed),
        "failed": sum(x["failed"] for x in parsed),
        "ignored": sum(x["ignored"] for x in parsed),
        "doctest": sum(x["passed"] for x in parsed if x["doctest"]),
        "summaries": parsed,
        "ignored_cases": [line for line in clean(text).splitlines() if re.match(r"^test .* \.\.\. ignored", line)],
    }

verified_artifacts = []
archives = {}
for name, filename in [
    (f"rig-contracts-{head}", f"rig-contracts-{stem}.zip"),
    (f"constraint-validation-{run['id']}-{run['run_attempt']}", f"constraint-validation-{run['id']}.zip"),
]:
    found = [item for item in artifacts if item["name"] == name]
    assert len(found) == 1
    artifact = found[0]
    assert not artifact["expired"]
    assert artifact["workflow_run"]["head_sha"] == head
    digest = hashlib.sha256((reports / filename).read_bytes()).hexdigest()
    assert artifact["digest"] == "sha256:" + digest
    verified_artifacts.append({"id": artifact["id"], "name": name, "sha256": digest})
    archives[name] = zipfile.ZipFile(reports / filename)
rig = archives[f"rig-contracts-{head}"]
assert len(rig.namelist()) == len(set(rig.namelist()))
assert rig.read("revision.txt").decode().strip() == head
labels = expected["required_labels"] + expected["additional_log_labels"]
assert len(labels) == len(set(labels)) == expected["expected_log_count"]
assert {n for n in rig.namelist() if n.endswith('.log')} == {x + '.log' for x in labels}
logs = {label: results(rig.read(label + '.log').decode()) for label in labels}
cells = json.loads(rig.read("required-cells.json"))
assert len(cells) == len(expected["required_labels"])
assert [c["cell"] for c in cells] == expected["required_labels"]
for cell in cells:
    observed = logs[cell["cell"]]
    assert cell["revision"] == head and cell["accepted"] is True
    assert cell["log"] == 'target/rig-contracts/' + cell["cell"] + '.log'
    for key in ["passed", "failed", "ignored"]:
        assert cell[key] == observed[key]
commands = expected["creator_command_vectors"]
assert {n for n in rig.namelist() if n.endswith('.command.txt')} == {x + '.command.txt' for x in commands}
for label, arguments in commands.items():
    assert shlex.split(rig.read(label + '.command.txt').decode()) == arguments
job_log = clean((reports / f"P06-hosted-{stem}-job.log").read_text())
observed_fixed = {}
for line in job_log.splitlines():
    line = line.removeprefix('##[group]Run ').strip()
    if not line.startswith('cargo test ') or ' | tee target/rig-contracts/' not in line:
        continue
    command, log = line.split(' | tee target/rig-contracts/', 1)
    command = command.removesuffix(' 2>&1')
    label = log.strip().removesuffix('.log')
    if label in expected['fixed_test_command_vectors']:
        assert shlex.split(command) == expected['fixed_test_command_vectors'][label]
        observed_fixed[label] = shlex.split(command)
assert observed_fixed == expected['fixed_test_command_vectors']
constraint = archives[f"constraint-validation-{run['id']}-{run['run_attempt']}"]
for name in ['source.txt', 'pr-head.txt']:
    assert constraint.read(name).decode().strip() == head
constraint_results = results(constraint.read('tests.log').decode())
baseline = rig.read('quality-baseline.toml')
tomllib.loads(baseline.decode())
(reports / f"official-quality-baseline-{stem}.toml").write_bytes(baseline)
rig_passed = sum(x['passed'] for x in logs.values())
rig_doc = sum(x['doctest'] for x in logs.values())
report = {
    'verified_at': datetime.now(timezone.utc).isoformat(),
    'head': head, 'run': run['id'], 'job': job['id'],
    'artifacts': verified_artifacts,
    'required_cells': len(cells), 'logs': logs,
    'creator_command_vectors': len(commands), 'fixed_command_vectors': len(observed_fixed),
    'rig_selected_executions': rig_passed,
    'rig_doctests': rig_doc,
    'rig_ignored': sum(x['ignored'] for x in logs.values()),
    'constraint': constraint_results,
    'combined_selected_executions': rig_passed + constraint_results['passed'],
    'combined_doctests': rig_doc + constraint_results['doctest'],
    'official_baseline_sha256': hashlib.sha256(baseline).hexdigest(),
    'acceptance_limits': 'Selected executions, not unique tests or whole-suite/renderer/live-DB coverage. Relevant official baseline reductions still need separate inspection. No blanket PR canary acceptance.',
}
(reports / f"P06-hosted-artifact-verification-{stem}.json").write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: v for k, v in report.items() if k not in ['logs', 'constraint']}, indent=2))
