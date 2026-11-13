#!/usr/bin/env python3
"""Read-only identity and public restoration-receipt check."""
import argparse,hashlib,json,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("unoptimized interpreter required")
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument("--ciphertexts",required=True,type=Path);ap.add_argument("--prior-nonces",type=Path);a=ap.parse_args()
def sh(b):return hashlib.sha256(b).hexdigest()
r=json.loads((p/"RESTORATION.json").read_bytes());raw=(a.ciphertexts/"MANIFEST.json").read_bytes()
if r["schema"]!="726-postb111-privacy-restoration-v1" or sh(raw)!=r["ciphertext_manifest_sha256"]:raise RuntimeError("receipt/manifest identity")
m=json.loads(raw);rows=m["objects"]
if len(rows)!=r["collections"] or r["restore_exit"]!=0 or any(r["selected_replay_exits"]) or any(x["exit"] or x["stderr_bytes"] for x in r["selected_replay_checks"]):raise RuntimeError("receipt results")
nonces=set()
for x in rows:
 b=(a.ciphertexts/x["ciphertext"]).read_bytes()
 if len(b)!=x["ciphertext_bytes"] or sh(b)!=x["ciphertext_sha256"] or len(b)<28 or b[:12] in nonces:raise RuntimeError("ciphertext identity or nonce")
 nonces.add(b[:12])
if len(nonces)!=14:raise RuntimeError("new nonce count")
checks=["receipt and 14 ciphertext identities", "14 new nonces mutually distinct"]
if a.prior_nonces:
 old=json.loads(a.prior_nonces.read_bytes())["objects"];previous={bytes.fromhex(x["nonce_hex"]) for x in old}
 if len(previous)!=536 or previous & nonces:raise RuntimeError("prior nonce collision")
 checks.append("distinct from supplied 536-object prior nonce inventory")
print(json.dumps({"checks":checks,"qualification":"This read-only check does not decrypt, recreate the restore, verify prior inventory provenance without its separate inputs, or establish independent custody or product behavior."},indent=2))
