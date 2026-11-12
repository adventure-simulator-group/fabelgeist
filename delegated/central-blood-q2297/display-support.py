import hashlib, json, pathlib, sys
p = pathlib.Path(__file__).resolve().parent
reg = (p / 'SUPPORT-REGISTRATION.json').read_bytes()
r = json.loads(reg)
groups = {'A': [('core-filth', 0), ('windows', 0)], 'B': [('calendar', 0)], 'C': [('native-disease', 0), ('native-disease', 1)], 'D': [('native-disease', 2), ('native-disease', 3)]}
group = sys.argv[1]
parts = []
rows = []
for tag, index in groups[group]:
    e = next(x for x in r['entries'] if x['tag'] == tag)
    b = (p / e['capture']).read_bytes()
    assert hashlib.sha256(b).hexdigest() == e['sha256']
    lines = b.decode().splitlines()
    a, z = e['planned_ranges'][index]
    parts.append(f'--- {tag} {a}-{z} ---\n')
    parts.extend(f'{i:4d} {lines[i-1]}\n' for i in range(a, z+1))
    rows.append({'tag': tag, 'displayed_range': [a, z]})
d = ''.join(parts)
(p / f'SUPPORT-DISPLAY-{group}.txt').write_text(d)
(p / f'SUPPORT-RECEIPT-{group}.json').write_text(json.dumps({'registration_sha256': hashlib.sha256(reg).hexdigest(), 'display_sha256': hashlib.sha256(d.encode()).hexdigest(), 'rows': rows, 'qualification': 'Only displayed registered ranges are credited to this coordinator.'}, indent=2) + '\n')
print(d, end='')
