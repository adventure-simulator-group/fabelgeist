// Compare portable recipe search with one or more native `heraldry-lab mix` results.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {pathToFileURL} from 'node:url';

const [site, ...results] = process.argv.slice(2);
if (!site || !results.length) throw new Error('Usage: node paint-parity.mjs SITE NATIVE_RESULT_JSON...');
const pkg = resolve(site, 'pkg');
const {default: init, solve_heraldry_paint} = await import(pathToFileURL(join(pkg, 'adventuresim_heraldry_studio.js')));
await init({module_or_path: await WebAssembly.compile(readFileSync(join(pkg, 'adventuresim_heraldry_studio_bg.wasm')))});
for (const path of results) {
  const {request, result} = JSON.parse(readFileSync(path, 'utf8'));
  assert.deepEqual(JSON.parse(solve_heraldry_paint(JSON.stringify(request))), result);
}
console.log(`Native/WASM paint recipes, errors and exact quotes agree for ${results.length} scenarios.`);
