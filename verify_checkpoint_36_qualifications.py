#!/usr/bin/env python3
"""Verify historical rig pin mismatches and qualified role exclusion."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();cp=json.loads((R/'DELEGATED-EVIDENCE-CHECKPOINT-36.json').read_text());b=next(x for x in cp['bundles'] if x['bundle_id']=='anime-helmets-2609-2610')
with tarfile.open(R/b['archive']) as t:
 def gz(path):return gzip.decompress(t.extractfile('history-774/continuation/'+path).read())
 old=json.loads(gz('root/admission03-scope-reconciliation-046.json.gz'));new=json.loads(gz('root/admission03-scope-reconciliation-050.json.gz'));h=json.load(t.extractfile('historical-index.json'))['rig_registration_pin_qualification'];reg=gz('rig/admission03/source-range-registration.jsonl.gz');qual=gz('rig/admission03/source-qualifications.jsonl.gz')
 def pin(d,name):return next(x for x in d['input_pins'] if x['supplier']=='rig' and x['record'].endswith(name))
 p46=pin(old,'source-range-registration.jsonl');p50=pin(new,'source-range-registration.jsonl');q46=pin(old,'source-qualifications.jsonl');q50=pin(new,'source-qualifications.jsonl')
 assert p46==h['root046_registration_pin'] and p50==h['root050_registration_pin'] and q46==h['root046_qualification_pin'] and q50==h['root050_qualification_pin']
 assert p46['sha256']!=sha(b''.join(reg.splitlines(keepends=True)[:35]))==h['later_registration_first35_sha256']
 assert p50['sha256']!=sha(b''.join(reg.splitlines(keepends=True)[:139]))==h['later_registration_first139_sha256']
 assert q46['sha256']==sha(b''.join(qual.splitlines(keepends=True)[:31])) and q50['sha256']==sha(b''.join(qual.splitlines(keepends=True)[:45]))
 assert json.loads(reg.splitlines()[51])==h['row52'] and json.loads(reg.splitlines()[54])==h['row55'] and old['at']<h['row52']['at'] and old['at']<h['row55']['at']
 x=next(x for x in new['role_qualifications'] if x.get('registration_id_resolved')=='RIG-ADMISSION03-REGISTER-055')['original_record'];assert x['id']=='RIG-ADMISSION03-SOURCE-QUALIFICATION-016' and x['excluded_incidental_display']==[[391,391]] and [392,414] in x['material_ranges']
print('Verified 046/050 distinct registration pins, matching qualification prefixes and original line-391 exclusion; no missing original is inferred recovered.')
