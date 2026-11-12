from pathlib import Path
import sys,json,hashlib,datetime
s=Path(__file__).resolve().parent;label,path,a,b=sys.argv[1],sys.argv[2],int(sys.argv[3]),int(sys.argv[4]);reg=json.loads((s/'MORE-DISPLAY-REGISTRATION.json').read_text());assert any(a>=x and b<=y for x,y in reg['windows'][path]);raw=(s/'sources'/'95e'/path).read_bytes();lines=raw.decode().splitlines();assert 1<=a<=b<=len(lines);text=f'SOURCE {path}:{a}-{b}\n'+''.join(f'{i:4} {lines[i-1]}\n' for i in range(a,b+1));name=f'MORE-DISPLAY-{label}.txt'
with (s/name).open('x') as f:f.write(text)
r={'source':path,'range':[a,b],'source_sha256':hashlib.sha256(raw).hexdigest(),'display':name,'display_sha256':hashlib.sha256(text.encode()).hexdigest(),'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'complete requested range'}
with (s/f'MORE-RECEIPT-{label}.json').open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(text,end='')
