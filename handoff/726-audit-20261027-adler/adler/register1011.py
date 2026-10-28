import continuation as c
for i,reg,ranges in [(2157,1011,[[1,112],[140,285]]),(2158,1012,[[1,698]]),(2159,1013,[[1,512]]),(2160,1014,[[1,49]]),(2161,1015,[[1,83]]),(2162,1016,[[1,358]]),(2163,1017,[[120,308],[315,530]]),(2164,1018,[[1,330],[1633,1690],[1768,1909],[2670,3510]])]:
 p=c.register_original(i,ranges,f'REGISTRATION-ADLER-{reg:03}.json','Prospective independent lens11 exact metadata union for shared world-schema canonical bestiary/calendar/geographic/economy/language/source identity types, constructors, serialization/admission and actual consuming contracts. No body credit before read; no product execution or other-lens transfer.')
 print(i,reg,'EOF',len(p.read_bytes().decode().splitlines()))
