#!/usr/bin/env python3
"""Verify selected historical rows without equating whole-file identities."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());reg=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];cp=load('DELEGATED-EVIDENCE-CHECKPOINT-27.json');bundles={b['bundle_id']:b for b in cp['bundles']};q=load('QUALIFICATION-REGISTRATION-HASH-MISMATCHES-27.json');assert len(q['entries'])==36 and len(q['roles'])==36 and cp['registration_hash_qualification']=='QUALIFICATION-REGISTRATION-HASH-MISMATCHES-27.json'
for x in q['entries']:
 r=reg[x['role']];rows=r.get('scope_registration',[])+r.get('parallel_reached_registrations',[])+([r['parallel_reached_registration']] if r.get('parallel_reached_registration') else []);source=next(y for y in rows if y and y['id']==x['registration_id']);assert source['sha256']==x['registry_recorded_whole_file_sha256'] and source['record']==x['registry_record'] and source['row_1based']==x['row_1based'] and x['selected_row_matches_embedded_original_record']
 b=bundles[x['bundle']];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  m=members[x['archive_member']];assert not m['changed'];raw=t.extractfile(x['archive_member']).read();body=gzip.decompress(raw) if x['archive_member'].endswith('.gz') else raw;h=hashlib.sha256(body).hexdigest();assert h==x['f82_whole_file_sha256']!=x['registry_recorded_whole_file_sha256'];parsed=[json.loads(z) for z in body.splitlines()];assert parsed[x['row_1based']-1]==source['original_record'];p=json.load(t.extractfile('git-proofs.json'));ps=p.get('proofs',p.get('paths',[]));assert any(y['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee' and y['path'].endswith(source['record']) and y.get('blob',y.get('git_blob'))==x['f82_git_blob'] and y['sha256']==h for y in ps)
assert not q['equal_whole_file_entries'] and not q['unproved_in_packet']
assert 'not byte-identical' in q['qualification'] and 'cause is unknown' in q['qualification'];print('Verified 36 matching selected registration rows with distinct registry-cited and f82 whole-file hashes; no original whole-file match or extra credit inferred.')
