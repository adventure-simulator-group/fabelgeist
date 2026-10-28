import continuation as c
for i,n,reg in [(2013,213,857),(2014,263,858),(2015,270,859),(2016,85,860),(2017,325,861),(2018,1248,862),(2019,287,863),(2020,287,864),(2021,198,865),(2022,370,866),(2023,43,867),(2024,48,868),(2025,46,869),(2026,1148,870),(2027,674,871),(2028,62,872)]:
 p=c.register_original(i,[[1,n]],f'REGISTRATION-ADLER-{reg:03}.json','Independent lens11 full metadata union/changed full retained type/error/native complete operation and source acceptance; source reads only.')
 print(i,reg,n,'EOF',len(p.read_bytes().decode().splitlines()))
