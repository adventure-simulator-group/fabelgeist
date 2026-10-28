import continuation as c
for i,n,reg in [(2003,164,847),(2004,108,848),(2005,42,849),(2006,38,850),(2007,55,851),(2008,7,852),(2009,197,853),(2010,330,854),(2011,185,855),(2012,62,856)]:
 p=c.register_original(i,[[1,n]],f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union/changed full: actual source/capture/admission/error and diagnostic acceptance roles; source reads only, no projectexecution.')
 print(i,reg,n,'EOF',len(p.read_bytes().decode().splitlines()))
