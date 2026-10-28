import continuation as c
for i,reg,ranges in [(2153,1007,[[1,64]]),(2154,1008,[[1,299]]),(2155,1009,[[1,121]]),(2156,1010,[[1,151]])]:
 p=c.register_original(i,ranges,f'REGISTRATION-ADLER-{reg:03}.json','Prospective independent lens11 exact metadata union: actual terrain feature collection producer, inferred roads typed/native route projection and source errors, authored world schema enumeration generator, canonical BasisPoints owner. No source body credit until read, no productexecution or other-lens transfer.')
 print(i,reg,'EOF',len(p.read_bytes().decode().splitlines()))
