import continuation as c
for i,n,reg in [(2043,86,887),(2044,325,888),(2045,105,889),(2046,672,890),(2047,100,891),(2048,1126,892),(2049,248,893),(2050,11,894),(2051,252,895),(2052,29,896),(2053,95,897)]:
 p=c.register_original(i,[[1,n]],f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union/changed full actual combat/door/equipment retained type/error/native roles and source acceptance, no productexecution.')
 print(i,reg,n,'EOF',len(p.read_bytes().decode().splitlines()))
