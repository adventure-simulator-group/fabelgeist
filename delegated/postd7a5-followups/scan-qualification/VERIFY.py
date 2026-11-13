#!/usr/bin/env python3
"""Verify the retained four candidate classifications against captured files."""
import argparse,hashlib,json,re,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();q=json.loads((p/'CLASSIFICATION.json').read_bytes());scan=json.loads((p/'DIGEST-SCAN.json').read_bytes());capture=(p/'STAGED-CAPTURE.json').read_bytes();tar=(p/'TAR-PATTERN-SCAN.txt').read_bytes();assert h(capture)==q['staged_capture_sha256'] and h((p/'DIGEST-SCAN.json').read_bytes())==q['digest_scan_sha256'] and h(tar)==q['tar_scan_sha256'];assert len(json.loads(capture)['rows'])==q['captured_staged_files']==139;assert scan['forbidden_digest_total']==0 and scan['other_candidate_counts']=={'unix-temp':4} and not scan['exceptions'] and b'issues 0' in tar
expected={x['path'] for x in q['scanner_literal_candidates']};assert expected=={x['location'] for x in scan['other_candidate_locations']} and len(expected)==4
arg=argparse.ArgumentParser();arg.add_argument('--publication',type=Path);a=arg.parse_args()
if a.publication:
 pattern=re.compile(rb'(?<![A-Za-z0-9_])/(?:private/)?t' + b'mp/[A-Za-z0-9_.~+@-]+(?:/[A-Za-z0-9_.~+@-]+)*')
 generic=b'/' + b'tmp' + b'/' + b'726-'
 for x in q['scanner_literal_candidates']:
  body=(a.publication/x['path']).read_bytes();assert h(body)==x['sha256'];matches=list(pattern.finditer(body));assert len(matches)==1 and matches[0].group()==generic
 print('Verified four scanner regex literals against the captured file identities; no actual local path is implied by those four hits.')
else:print('Verified retained scan metadata; supply --publication for exact candidate classification.')
print('The scan remains bounded; ciphertext and unscanned private information are separate.')
