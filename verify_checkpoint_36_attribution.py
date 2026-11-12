import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;q=json.loads((R/'QUALIFICATION-CHECKPOINT36-ATTRIBUTION-AND-COMPRESSED.json').read_text());expected=[('READ-L11-426','REGISTRATION-SEGOVB-1517',[[86,171]]),('READ-L11-427','REGISTRATION-SEGOVB-1529',[[1,85]])]
for x,(read,reg,ranges) in zip(q['historical_receipts'],expected):
 arc,member=x['receipt_member'].split('!',1)
 with tarfile.open(R/arc) as t:body=t.extractfile(member).read()
 assert hashlib.sha256(body).hexdigest()==x['receipt_public_sha256'];r=json.loads(body);s=next(y for y in r['sources'] if y.get('original_index')==2606)
 assert read in member and reg in s['registration'] and s['registration_sha256']==x['registration_original_sha256'] and s['fresh_personal_body_reads']==s['assessed_ranges']==x['recorded_ranges']==ranges and s['sha256']==x['historical_source_sha256']
assert 'later public compressed members are path-redacted reconstructions' in q['public_compression_qualification'] and 'remain missing' in q['checkpoint3821']
print('Verified separate recorded Segovb scopes and public compression qualification; no personal read credit inferred.')
