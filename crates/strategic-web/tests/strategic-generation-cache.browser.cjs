const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

test("local products survive reload and invalidate on input, revision, or corruption", async () => {
  const server = http.createServer((request, response) => {
    if (request.url === "/cache.js") {
      response.setHeader("Content-Type", "text/javascript");
      response.end(fs.readFileSync(path.join(__dirname, "../static/strategic-generation-cache.js")));
    } else { response.setHeader("Content-Type", "text/html"); response.end("<!doctype html><title>Cache test</title>"); }
  });
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    await page.evaluate(async () => {
      const { openGeneratedCache } = await import("/cache.js");
      const cache = await openGeneratedCache("generator-A");
      cache.put("seed:18446744073709551615", new Uint8Array([1, 2, 3]));
      await cache.close();
    });
    await page.reload();
    const result = await page.evaluate(async () => {
      const { openGeneratedCache } = await import("/cache.js");
      const cache = await openGeneratedCache("generator-A");
      const hit = Array.from(await cache.get("seed:18446744073709551615"));
      const changedInput = await cache.get("seed:18446744073709551614");
      const changed = await openGeneratedCache("generator-B");
      const changedRevision = await changed.get("seed:18446744073709551615");
      await changed.close();
      await new Promise((resolve, reject) => {
        const request = indexedDB.open("fabelgeist-generated-assets", 1);
        request.onsuccess = () => {
          const db = request.result, tx = db.transaction("products", "readwrite");
          tx.oncomplete = () => { db.close(); resolve(); }; tx.onerror = () => reject(tx.error);
          const store = tx.objectStore("products"), records = store.getAll();
          records.onsuccess = () => { const record = records.result[0]; record.bytes[0] ^= 1; store.put(record); };
        };
      });
      const corrupted = await cache.get("seed:18446744073709551615");
      cache.put("seed:18446744073709551615", new Uint8Array([4, 5]));
      await cache.close();
      const repaired = await openGeneratedCache("generator-A");
      const regenerated = Array.from(await repaired.get("seed:18446744073709551615"));
      await repaired.remove("seed:18446744073709551615");
      const removed = await repaired.get("seed:18446744073709551615");
      await repaired.close();
      return { hit, changedInput, changedRevision, corrupted, regenerated, removed };
    });
    assert.deepEqual(result, { hit: [1, 2, 3], changedInput: undefined, changedRevision: undefined,
      corrupted: undefined, regenerated: [4, 5], removed: undefined });
    assert.equal(await page.evaluate(async () => {
      const { openGeneratedCache } = await import("/cache.js");
      const unavailable = await openGeneratedCache("A", { open() { throw new DOMException("disabled", "SecurityError"); } });
      unavailable.put("job", new Uint8Array([1])); await unavailable.close();
      const cache = await openGeneratedCache("quota-test");
      const original = IDBObjectStore.prototype.put;
      IDBObjectStore.prototype.put = () => { throw new DOMException("full", "QuotaExceededError"); };
      try { cache.put("job", new Uint8Array([2])); await cache.close(); }
      finally { IDBObjectStore.prototype.put = original; }
      return true;
    }), true);
  } finally { await browser.close(); await new Promise(resolve => server.close(resolve)); }
});
