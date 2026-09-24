// Compare the browser worker entry point with a native CLI export, without a GPU.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {pathToFileURL} from 'node:url';

const [site, recipePath, bakePath] = process.argv.slice(2);
if (!site || !recipePath || !bakePath) {
  throw new Error('Usage: node wasm-parity.mjs SITE_DIRECTORY RECIPE_JSON MATERIAL_BAKE');
}
const pkg = resolve(site, 'pkg');
const {default: init, bake_heraldry} = await import(pathToFileURL(join(pkg, 'adventuresim_heraldry_studio.js')));
const module = await WebAssembly.compile(readFileSync(join(pkg, 'adventuresim_heraldry_studio_bg.wasm')));
await init({module_or_path: module});
const document = JSON.parse(readFileSync(recipePath, 'utf8'));
const native = readFileSync(bakePath);
const size = native.readUInt32LE(0);
const resolution = new Map([[128,'Draft'],[512,'Preview'],[1024,'High'],[2048,'Final']]).get(size);
assert.ok(resolution, 'Native bake has a supported resolution');
const response = bake_heraldry(JSON.stringify({document, resolution, export: true}));
const length = new DataView(response.buffer, response.byteOffset).getUint32(0, true);
const wasm = response.subarray(4, 4 + length);
assert.equal(wasm.length, native.length);
const headerSize = 4 + 64;
assert.deepEqual(Buffer.from(wasm.subarray(0, headerSize)), native.subarray(0, headerSize));
const mapCount = 5;
const mapEnd = headerSize + mapCount * size * size * 4;
let differingBytes = 0, maxByteError = 0, maxHeightError = 0;
for (let i = headerSize; i < mapEnd; i++) {
  if (wasm[i] !== native[i]) differingBytes++;
  maxByteError = Math.max(maxByteError, Math.abs(wasm[i] - native[i]));
}
const values = new DataView(wasm.buffer, wasm.byteOffset);
for (let i = mapEnd; i < length; i += 4) {
  maxHeightError = Math.max(maxHeightError, Math.abs(values.getFloat32(i, true) - native.readFloatLE(i)));
}
assert.ok(maxByteError <= 1, 'RGBA maps agree within one quantization step');
assert.ok(maxHeightError <= 0.00001, 'Height maps agree within 0.00001 mm');
assert.deepEqual(Array.from(response.subarray(4 + length, 8 + length)), [80, 75, 3, 4]);
console.log(JSON.stringify({size, differingBytes, maxByteError, maxHeightError, zipBytes: response.length - length - 4}));
