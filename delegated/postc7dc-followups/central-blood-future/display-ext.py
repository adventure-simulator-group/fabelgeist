import hashlib,json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent;reg=(p/'EXT-REGISTRATION.json').read_bytes();r=json.loads(reg);groups={'A':['core-filth'],'B':['native-filth'],'C':['native-attempts']};group=sys.argv[1];out=[];rows=[]
for tag in groups[group]:
 e=next(x for x in r['entries'] if x['tag']==tag);b=(p/e['capture']).read_bytes();assert hashlib.sha256(b).hexdigest()==e['sha256'];lines=b.decode().splitlines()
 for lo,hi in e['planned_ranges']:
  out.append(f'--- {tag} {lo}-{hi} ---\n');out.extend(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1));rows.append({'tag':tag,'range':[lo,hi]})
d=''.join(out);(p/f'EXT-DISPLAY-{group}.txt').write_text(d);(p/f'EXT-RECEIPT-{group}.json').write_text(json.dumps({'registration_sha256':hashlib.sha256(reg).hexdigest(),'display_sha256':hashlib.sha256(d.encode()).hexdigest(),'rows':rows},indent=2)+'\n');print(d,end='')
