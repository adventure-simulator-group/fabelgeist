import continuation as c
spec=[(2054,898,[[1,60]]),(2055,899,[[1,648]]),(2056,900,[[1,29]]),(2057,901,[[1,163]]),(2058,902,[[1,427]]),(2059,903,[[1,89]]),(2060,904,[[1,167]]),(2061,905,[[1,51]]),(2062,906,[[1,726]]),(2063,907,[[1,158]]),(2064,908,[[52,320]]),(2065,909,[[1,303]])]
for i,reg,ranges in spec:
 p=c.register_original(i,ranges,f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union including changed full; actual server input/iteration/mission cause/authority/native contracts and source acceptance only.')
 print(i,reg,ranges,'EOF',len(p.read_bytes().decode().splitlines()))
