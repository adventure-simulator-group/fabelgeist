#!/usr/bin/env python3
"""Verify the q2330 path-safe source-evidence successor, not product behavior."""
import argparse,hashlib,json,subprocess,sys,tomllib
from pathlib import Path
if sys.flags.optimize: raise RuntimeError("Assertions require an unoptimized interpreter")
P=Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
load=lambda n:json.loads((P/n).read_bytes())
m=load("MANIFEST.json");rows=m["files"];actual={x.relative_to(P).as_posix() for x in P.rglob("*") if x.is_file() and x.name!="MANIFEST.json"};assert actual==set(rows)
for name,row in rows.items():
 p=P/name;assert not p.is_symlink();b=p.read_bytes();assert len(b)==row["bytes"] and H(b)==row["sha256"]
bound=load("PRIVACY-BOUNDARY.json");raw=(P/"ORIGINAL-MANIFEST.json").read_bytes();assert H(raw)==bound["original_manifest_sha256"];old=json.loads(raw)["files"];old={x["path"]:x for x in old} if isinstance(old,list) else old;omitted={x["file"]:x for x in bound["omitted"]};assert len(omitted)==2
for name,row in old.items():
 if name in omitted:assert not (P/name).exists() and row["sha256"]==omitted[name]["original_file_sha256"] and row["bytes"]==omitted[name]["bytes"]
 else:
  b=(P/name).read_bytes();assert len(b)==row["bytes"] and H(b)==row["sha256"]
assert set(rows)==(set(old)-set(omitted))|{"ORIGINAL-MANIFEST.json","PRIVACY-BOUNDARY.json","VERIFY-PUBLIC-SUCCESSOR.py"}
lock=tomllib.loads((P/"sources/current-Cargo.lock").read_text());packages={(x["name"],x["version"]):x for x in lock["package"]}
for capture,pkg,ver in [("NATIVE-CAPTURE.json","serde_json","1.0.151"),("CORE-CAPTURE.json","serde_core","1.0.229")]:
 x=load(capture);x=x["items"][1] if "items" in x else x;source=(P/(x.get("source_file") or x["file"])).read_bytes();assert H(source)==(x.get("source_sha256") or x["sha256"]);assert packages[(pkg,ver)]["checksum"]==x["archive_sha256"]==omitted[x["archive_file"]]["original_file_sha256"]
ap=argparse.ArgumentParser();ap.add_argument("--original-packet",type=Path);ap.add_argument("--repo",type=Path);a=ap.parse_args()
if a.original_packet:
 original=a.original_packet.resolve();assert H((original/"MANIFEST.json").read_bytes())==bound["original_manifest_sha256"];cmd=[sys.executable,"-B",str(original/"VERIFY-EVIDENCE.py")];
 if a.repo:cmd.extend(["--repo",str(a.repo.resolve())])
 subprocess.run(cmd,check=True)
else:
 assert not a.repo;print("NOT REPEATED original verifier or archive-to-member check: supply separately held --original-packet and optional --repo.")
print("Verified unchanged original members, qualified archive omissions and selected source/lock identities; no product behavior or independent custody established.")
