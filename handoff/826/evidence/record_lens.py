import json, pathlib, hashlib, sys
p=pathlib.Path('target/pr826-audit/ledger.json'); ledger=json.loads(p.read_text()); record=json.load(sys.stdin)
for row in record['inventory']:
    row.setdefault('repair','not started' if row['disposition'].startswith('F') else 'not required')
    row.setdefault('validation','pending')
    for source in row.get('sources',[]):
        file=source.split(':')[0]
        if pathlib.Path(file).is_file():
            row.setdefault('fingerprints',{})[file]=hashlib.sha256(pathlib.Path(file).read_bytes()).hexdigest()
            if file not in [x['path'] for x in ledger['scope']['changed']+ledger['scope']['affected_unchanged']]:
                ledger['scope']['affected_unchanged'].append({'path':file,'sha256':row['fingerprints'][file]})
ledger['waves'][0]['lenses'][record['lens']].update(record)
ledger['next']=record.get('next','Continue next initial lens; no source repairs before all thirteen inventories complete.')
p.write_text(json.dumps(ledger,indent=2)+'\n')
