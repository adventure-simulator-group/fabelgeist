import bookkeeping as b
l=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
print([k for k in l['discovery_findings_and_dependencies'] if any(z in k for z in ['FORECAST','WHEEL','INCAP','FATIGUE'])])
