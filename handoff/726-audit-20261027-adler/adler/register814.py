import continuation as c
for i,n,reg in [(1971,490,814),(1972,514,815),(1973,1514,816),(1974,178,817),(1975,475,818),(1976,242,819)]:
 c.register_original(i,[[1,n]],f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union: complete retained owner/admission/native and error roles, source acceptance only; no product execution.')
 print(i,reg,n)
