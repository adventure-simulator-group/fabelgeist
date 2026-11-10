#!/usr/bin/env python3
"""Build a local full-text index of recovered evidence without adopting it."""
import json
from pathlib import Path
import sqlite3
import subprocess
import sys

if len(sys.argv) != 2:
    raise SystemExit("usage: build_recovery_index.py NEW_INDEX.sqlite")
root = Path(__file__).parent
output = Path(sys.argv[1]).absolute()
subprocess.run([sys.executable, str(root / "verify_recovered.py")], check=True)
with output.open("xb"):
    pass
connection = sqlite3.connect(output)
try:
    connection.execute("CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
    connection.execute("INSERT INTO metadata VALUES ('status', 'INCOMPLETE')")
    connection.execute("CREATE VIRTUAL TABLE records USING fts5(collection UNINDEXED, ordinal UNINDEXED, body)")
    count = 0
    for path in sorted((root / "recovered").rglob("*.jsonl")):
        collection = path.relative_to(root).as_posix()
        with path.open(encoding="utf-8") as stream:
            for line in stream:
                item = json.loads(line)
                ordinal = next((item[k] for k in (
                    "transcript_ordinal", "transcript_command_ordinal",
                    "transcript_output_ordinal", "transcript_filechange_ordinal",
                    "ordinal", "last_filechange_ordinal") if k in item), "")
                body = json.dumps(item, ensure_ascii=False, separators=(",", ":"))
                connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                                   (collection, str(ordinal), body))
                count += 1
    for relative in ("recovered/transcript-hash-matched", "recovered/transcript-notes",
                     "recovered/transcript-queue-fragments"):
        directory = root / relative
        manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
        for item in manifest["files"]:
            name = item["file"]
            if Path(name).name != name:
                raise ValueError("Unsafe manifest filename")
            text = (directory / name).read_text(encoding="utf-8")
            connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                               (relative + "/" + name, "", text))
            count += 1
    directory = root / "recovered/hash-matched-dependencies"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        try:
            body = (directory / item["file"]).read_text(encoding="utf-8")
        except UnicodeDecodeError:
            # The recovered compressed crate is preserved and hash-verified,
            # but cannot be indexed as plain text.
            continue
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/hash-matched-dependencies/" + item["file"],
                            str(item["candidate_source_ordinal"]), body))
        count += 1
    directory = root / "recovered/recursive-dependencies"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest:
        body = (directory / "objects" / item["sha256"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/recursive-dependencies/objects/" + item["sha256"],
                            str(item["ordinal"]), body))
        count += 1
    directory = root / "recovered/printed-hash-registrations"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / "objects" / item["sha256"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/printed-hash-registrations/objects/" + item["sha256"],
                            str(item["ordinal"]), body))
        count += 1
    directory = root / "recovered/printed-hash-audit-records"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / "objects" / item["sha256"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/printed-hash-audit-records/objects/" + item["sha256"],
                            str(item["ordinal"]), body))
        count += 1
    directory = root / "recovered/printed-reference-json"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / item["file"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/printed-reference-json/" + item["file"],
                            str(item["printed_body_ordinal"]), body))
        count += 1
    directory = root / "recovered/recursive-printed-json"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / item["file"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/recursive-printed-json/" + item["file"],
                            str(item["printed_body_ordinal"]), body))
        count += 1
    directory = root / "recovered/linked-literal-evidence"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / item["file"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/linked-literal-evidence/" + item["file"],
                            str(item.get("literal_write_ordinal", item.get("source_ordinal", ""))), body))
        count += 1
    directory = root / "recovered/printed-filename-json"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / item["file"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/printed-filename-json/" + item["file"],
                            str(item["printed_body_ordinal"]), body))
        count += 1
    directory = root / "recovered/printed-filename-text"
    manifest = json.loads((directory / "MANIFEST.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        body = (directory / item["file"]).read_text(encoding="utf-8")
        connection.execute("INSERT INTO records(collection, ordinal, body) VALUES (?, ?, ?)",
                           ("recovered/printed-filename-text/" + item["file"],
                            str(item["candidate_ordinal"]), body))
        count += 1
    connection.execute("UPDATE metadata SET value = 'complete' WHERE key = 'status'")
    connection.execute("INSERT INTO metadata VALUES ('records', ?)", (str(count),))
    connection.commit()
    print("Indexed", count, "overlapping historical evidence records")
except Exception:
    connection.rollback()
    raise
finally:
    connection.close()
