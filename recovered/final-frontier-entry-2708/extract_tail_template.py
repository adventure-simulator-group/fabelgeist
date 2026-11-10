#!/usr/bin/env python3
"""Statically extract a registration template; never execute historical code."""
import ast
import hashlib
import json
from pathlib import Path

here = Path(__file__).resolve().parent
root = here.parent.parent
source_hash = "c745b617fe6b91b054b4f968565f295d03f9839eb329c4957d86306d65969858"
source = (here / source_hash).read_bytes()
if hashlib.sha256(source).hexdigest() != source_hash:
    raise ValueError("Source checksum mismatch")
record = None
with (root / "recovered/late-command-history/RECORDS.jsonl").open(encoding="utf8") as stream:
    for line in stream:
        item = json.loads(line)
        if item["transcript_ordinal"] == 418957:
            record = item
            break
if record is None:
    raise ValueError("Registration command unavailable")
command = record["command_text"]
if hashlib.sha256(command.encode()).hexdigest() != "122f82e155f978d696e16d699b2c0586e0175933d49c2350b16f7a686ea453fc":
    raise ValueError("Registration command hash mismatch")
program = command.split("@'\n", 1)[1].rsplit("\n'@", 1)[0]
tree = ast.parse(program)
env = {"rev": "6533e444fa37552927070cc8d8190907fd3b413e",
       "path": "crates/adventuresim-core/src/errantry.rs", "h": source_hash}

def value(node):
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, ast.Name) and node.id in env:
        return env[node.id]
    if isinstance(node, ast.Dict):
        return {value(k): value(v) for k, v in zip(node.keys, node.values)}
    if isinstance(node, ast.List):
        return [value(item) for item in node.elts]
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        return value(node.left) + value(node.right)
    if isinstance(node, ast.Call) and ast.unparse(node) == "len(data.splitlines())":
        return len(source.splitlines())
    if isinstance(node, ast.Call) and ast.unparse(node) == "datetime.datetime.now(datetime.timezone.utc).isoformat()":
        return "UNRECOVERED_ORIGINAL_TIMESTAMP"
    raise ValueError("Unrecognized expression in registration template")

static_assignments = {n.targets[0].id: n.value for n in tree.body
                      if isinstance(n, ast.Assign) and len(n.targets) == 1
                      and isinstance(n.targets[0], ast.Name)}
if (ast.literal_eval(static_assignments["rev"]) != env["rev"]
        or ast.literal_eval(static_assignments["path"]) != env["path"]
        or ast.unparse(static_assignments["h"]) != "hashlib.sha256(data).hexdigest()"
        or ast.unparse(static_assignments["data"]) != "subprocess.check_output(['git', 'show', rev + ':' + path])"):
    raise ValueError("Registration command source derivation mismatch")

assignments = [n for n in tree.body if isinstance(n, ast.Assign) and len(n.targets) == 1
               and isinstance(n.targets[0], ast.Name) and n.targets[0].id == "j"]
if len(assignments) != 1:
    raise ValueError("Expected one registration assignment")
template = value(assignments[0].value)
print(json.dumps(template, indent=2, ensure_ascii=False))
