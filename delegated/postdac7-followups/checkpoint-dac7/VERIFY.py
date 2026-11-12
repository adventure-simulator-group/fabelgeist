#!/usr/bin/env python3
"""Verify same-machine checkpoint recovery evidence without product execution."""
import hashlib,json,sys
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((P/'MANIFEST.json').read_bytes());assert m['schema']=='726-checkpoint-dac7-recovery-evidence-v1'
for row in m['files']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts;p=P/n;assert p.is_file() and not p.is_symlink();b=p.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);assert bool(p.stat().st_mode&0o111)==bool(int(row['mode'],8)&0o111)
rec=json.loads((P/'recovery/RECOVERY-RECORD.json').read_bytes());pre=json.loads((P/'prepublication/RESULT.json').read_bytes());log=(P/'recovery/evidence.log').read_bytes();old=(P/'prepublication/evidence.log').read_bytes();compare=json.loads((P/'precommit/RECORD.json').read_bytes());capture=json.loads((P/'capture/CAPTURE.json').read_bytes());digest=json.loads((P/'staged-digest/OUTPUT.json').read_bytes());pattern=(P/'staged-pattern/OUTPUT.log').read_bytes()
assert rec['checkpoint']==m['checkpoint'] and rec['previous_checkpoint']==m['parent'] and rec['fetch_head_asserted_before_checkout']==m['checkpoint'] and rec['recovery_driver_exit']==0 and rec['post_replay_head']==m['checkpoint'] and rec['post_replay_clean'] and rec['symbolic_ref_exit_after']==1
assert rec['prepublication_and_recovery_transcripts_byte_identical'] and log==old and (len(log),H(log))==(rec['transcript_bytes'],rec['transcript_sha256'])==(pre['transcript_bytes'],pre['transcript_sha256']) and pre['exit']==0
assert rec['assertion_preflight']==pre['assertion_preflight']=={'assertions_enabled':True,'optimization_level':0}
assert compare['parent']==m['parent'] and compare['staged_files']==148 and compare['aggregate_names_modes_and_sha256']==capture['aggregate'] and compare['driver_transcript_sha256']==H(log) and compare['main_recheck']['observed_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009'
assert digest['forbidden_digest_total']==0 and not digest['other_candidate_counts'] and not digest['exceptions'] and b'issues 0' in pattern
print('Verified recorded dac7 same-machine recovery, prepublication transcript, staging identity and bounded scans; independent custody, nested flags, ciphertext and product behavior remain separate.')
