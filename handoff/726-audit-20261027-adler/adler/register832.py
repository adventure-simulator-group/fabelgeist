import continuation as c,bookkeeping as b
spec=[(1989,832,[[1,207]]),(1990,833,[[1,56]]),(1991,834,[[1,71]]),(1992,835,[[1,168]]),(1993,836,[[1,89]]),(1994,837,[[1,203]]),(1995,838,[[37,140]]),(1996,839,[[1,105]]),(1997,840,[[1,102]]),(1998,841,[[1,346]]),(1999,842,[[1,57]]),(2000,843,[[1,978]]),(2001,844,[[1,42]]),(2002,845,[[1,534]])]
for i,reg,ranges in spec:
 p=c.register_original(i,ranges,f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union including changed full, diagnostic/provider/authority types and complete native/error roles, source reads only.')
 print(i,reg,ranges,'EOF',len(p.read_bytes().decode().splitlines()))
