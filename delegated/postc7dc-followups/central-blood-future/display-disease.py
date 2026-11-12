import hashlib,json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent;reg=(p/'DISEASE-REGISTRATION.json').read_bytes();r=json.loads(reg);e=r['entry'];b=(p/e['capture']).read_bytes();assert hashlib.sha256(b).hexdigest()==e['sha256'];lines=b.decode().splitlines();groups={'A':[0,1],'B':[2,3]};group=sys.argv[1];out=[];rows=[]
for index in groups[group]:
 lo,hi=e['planned_ranges'][index];out.append(f'--- native-disease {lo}-{hi} ---\n');out.extend(f'{i:4d} {lines[i-1]}\n' for i in range(lo,hi+1));rows.append([lo,hi])
d=''.join(out);(p/f'DISEASE-DISPLAY-{group}.txt').write_text(d);(p/f'DISEASE-RECEIPT-{group}.json').write_text(json.dumps({'registration_sha256':hashlib.sha256(reg).hexdigest(),'display_sha256':hashlib.sha256(d.encode()).hexdigest(),'ranges':rows},indent=2)+'\n');print(d,end='')
