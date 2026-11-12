from pathlib import Path
import json,sys,hashlib,datetime
p=Path(__file__).resolve().parent
label,rel,start,end=sys.argv[1],sys.argv[2],int(sys.argv[3]),int(sys.argv[4])
reg=json.loads((p/"SOURCE-REGISTRATION.json").read_text())
assert any(start>=a and end<=b for a,b in reg["windows"][rel]), (rel,start,end)
raw=(p/"sources"/rel).read_bytes(); lines=raw.decode().splitlines(); assert 1<=start<=end<=len(lines)
text=f"SOURCE {rel}:{start}-{end}\n"+"".join(f"{i:4} {lines[i-1]}\n" for i in range(start,end+1))
out=p/("SOURCE-DISPLAY-"+label+".txt")
with out.open("x") as f:f.write(text)
receipt={"display":out.name,"source":rel,"range":[start,end],"source_sha256":hashlib.sha256(raw).hexdigest(),"display_sha256":hashlib.sha256(text.encode()).hexdigest(),"displayed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"status":"untruncated requested window"}
with (p/("SOURCE-RECEIPT-"+label+".json")).open("x") as f:json.dump(receipt,f,indent=2);f.write("\n")
print(text,end="")
