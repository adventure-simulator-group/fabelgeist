import bookkeeping as b
l=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
print([k for k in l['discovery_findings_and_dependencies'] if any(s in k for s in ['EQUIPMENT','ITEM','PICKUP','ANCHOR'])])
