#!/usr/bin/env python3
"""Verify the additive correction of two previously published lens labels."""
import json
from pathlib import Path
R=Path(__file__).resolve().parent; load=lambda p:json.loads((R/p).read_text())
d=load('CORRECTION-10C-LENS-LABELS.json');base=load('BASELINE-95E-20261110.json');rows={x['id']:x for x in base['newly_assessed_ledger']['entries']}
assert d['prior_published_commit']=='58b86a9bebea0986c156224f00e143399e6c2ea3' and len(d['corrections'])==2
for x in d['corrections']:
 assert x['prior_published_lens']==10 and x['prior_published_additional_current_lenses']==[] and rows[x['id']]['lens']==x['corrected_current_lens'] and rows[x['id']]['additional_current_lenses']==x['corrected_additional_current_lenses']
assert base['lens_names'][4]=='aggregates' and base['lens_names'][7]=='behavior preservation' and base['lens_names'][9]=='derivability'
print('Verified two additive lens label corrections without changing prior evidence or coverage.')
import gzip,hashlib
x=d['prior_evidence']; old=gzip.decompress((R/x['source_gzip']).read_bytes());c=(R/x['commit_object']).read_bytes();t=(R/x['root_tree_object']).read_bytes()
oid=lambda k,b:hashlib.sha1(k.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
assert hashlib.sha256((R/x['source_gzip']).read_bytes()).hexdigest()==x['compressed_sha256'] and hashlib.sha256(old).hexdigest()==x['uncompressed_sha256'] and len(old)==x['uncompressed_bytes']
assert oid('commit',c)==d['prior_published_commit'] and c.splitlines()[0]==b'tree '+x['root_tree_git_sha1'].encode() and oid('tree',t)==x['root_tree_git_sha1'] and oid('blob',old)==x['blob_git_sha1']
entries={};pos=0
while pos<len(t):
 end=t.index(b'\0',pos);mode,name=t[pos:end].split(b' ',1);entries[name.decode()]=(mode.decode(),t[end+1:end+21].hex());pos=end+21
assert entries[x['path']]==(x['mode'],x['blob_git_sha1'])
old_rows={r['id']:r for r in json.loads(old)['newly_assessed_ledger']['entries']}
for r in d['corrections']:assert old_rows[r['id']]['lens']==r['prior_published_lens'] and old_rows[r['id']]['additional_current_lenses']==r['prior_published_additional_current_lenses']
print('Verified exact prior baseline through its original commit/tree/blob membership.')
