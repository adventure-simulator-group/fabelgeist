import continuation as c
for i,n,reg in [(1977,349,820),(1978,36,821),(1979,21,822),(1980,102,823),(1981,142,824),(1982,176,825),(1983,328,826),(1984,153,827),(1985,204,828),(1986,122,829),(1987,48,830),(1988,169,831)]:
 c.register_original(i,[[1,n]],f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union/changed full; native diagnostic retained roles, complete operation/error contracts; source reads only.')
 print(i,reg,n)
