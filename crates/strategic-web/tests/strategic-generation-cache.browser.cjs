const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

test("local products survive reload and invalidate on input, revision, or corruption", async () => {
  const server = http.createServer((request, response) => {
    if (["/cache.js", "/strategic-generation-write-queue.js"].includes(request.url)) {
      response.setHeader("Content-Type", "text/javascript");
      response.end(fs.readFileSync(path.join(__dirname, "../static/", request.url === "/cache.js" ? "strategic-generation-cache.js" : "strategic-generation-write-queue.js")));
    } else { response.setHeader("Content-Type", "text/html"); response.end("<!doctype html><title>Cache test</title>"); }
  });
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  const browser = await chromium.launch({ headless: true,
    ...(process.platform === "win32" ? { channel: "msedge" } : {}) });
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
      const originalGet = IDBObjectStore.prototype.get;
      let missReads = 0, changedInput;
      IDBObjectStore.prototype.get = function (...args) {
        missReads++; return originalGet.apply(this, args);
      };
      try { changedInput = await cache.get("seed:18446744073709551614"); }
      finally { IDBObjectStore.prototype.get = originalGet; }
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
      return { hit, changedInput, missReads, changedRevision, corrupted, regenerated, removed };
    });
    assert.deepEqual(result, { hit: [1, 2, 3], changedInput: undefined, missReads: 0, changedRevision: undefined,
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
    const pageErrors = [];
    page.on("pageerror", error => pageErrors.push(error.message));
    assert.equal(await page.evaluate(async () => {
      const { openGeneratedCache } = await import("/cache.js");
      const seed = await openGeneratedCache("delayed-completion");
      seed.put("present", new Uint8Array([2])); await seed.close();
      const cache = await openGeneratedCache("delayed-completion");
      const originalTimer = window.setTimeout, originalAbort = IDBTransaction.prototype.abort;
      let attemptedAbort = false;
      // Deterministically put the timeout ahead of the completion event for a
      // transaction which is already no longer abortable.
      window.setTimeout = (callback, delay, ...args) => {
        if (delay === 2_000) { queueMicrotask(callback); return 0; }
        return originalTimer(callback, delay, ...args);
      };
      IDBTransaction.prototype.abort = () => {
        attemptedAbort = true; throw new DOMException("finished", "InvalidStateError");
      };
      try { return (await cache.get("present"))?.[0] === 2 && attemptedAbort; }
      finally {
        window.setTimeout = originalTimer; IDBTransaction.prototype.abort = originalAbort;
        await cache.close();
      }
    }), true);
    assert.deepEqual(pageErrors, []);
    assert.equal(await page.evaluate(async () => {
      const { openGeneratedCache } = await import("/cache.js");
      const seed = await openGeneratedCache("recover-read-timeout");
      seed.put("present", new Uint8Array([2])); await seed.close();
      const cache = await openGeneratedCache("recover-read-timeout");
      const originalTimer = window.setTimeout, originalAbort = IDBTransaction.prototype.abort;
      let aborted = false;
      window.setTimeout = (callback, delay, ...args) => {
        if (delay === 2_000) { queueMicrotask(callback); return 0; }
        return originalTimer(callback, delay, ...args);
      };
      IDBTransaction.prototype.abort = function () {
        originalAbort.call(this); aborted = true;
      };
      try { await cache.get("present"); }
      finally { window.setTimeout = originalTimer; IDBTransaction.prototype.abort = originalAbort; }
      // A timed-out read must not disable persistence for the rest of the city.
      cache.put("after-timeout", new Uint8Array([7, 8]));
      await cache.close();
      const reopened = await openGeneratedCache("recover-read-timeout");
      const restored = await reopened.get("after-timeout");
      await reopened.close();
      return aborted && restored?.[0] === 7 && restored?.[1] === 8;
    }), true);
    assert.equal(await page.evaluate(async () => {
      const { openGeneratedCache } = await import("/cache.js");
      const cache = await openGeneratedCache("pending-removal");
      const original = new Uint8Array([1, 2, 3]);
      cache.put("removed-before-flush", original);
      const ownershipTransferred = original.byteLength === 0;
      await cache.remove("removed-before-flush");
      cache.put("latest", new Uint8Array([4]));
      cache.put("latest", new Uint8Array([5, 6]));
      await cache.close();
      const reopened = await openGeneratedCache("pending-removal");
      const removed = await reopened.get("removed-before-flush");
      const latest = await reopened.get("latest");
      await reopened.close();
      return ownershipTransferred && removed === undefined && latest[0] === 5 && latest[1] === 6;
    }), true);
    assert.deepEqual(pageErrors, []);
  } finally { await browser.close(); await new Promise(resolve => server.close(resolve)); }
});
