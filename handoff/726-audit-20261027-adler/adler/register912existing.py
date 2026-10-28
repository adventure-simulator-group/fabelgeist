import continuation as c
for i,reg,ranges in [(2814,912,[[1,125]]),(2815,913,[[1,76]]),(2066,914,[[1,99]])]:
 p=c.register_original(i,ranges,f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 existingmetadata union; future2814/2815 actualterminal enrollment/receipt contracts materiallyreached2063/2064 alreadyregistered1..125/1..76. No extension/newglobalidentity/otherlenscredit; receivingproofmustadoptfirstorderedfrontier. Original2066 ordered fullmetadata99.')
 print(i,reg,ranges,'EOF',len(p.read_bytes().decode().splitlines()))
