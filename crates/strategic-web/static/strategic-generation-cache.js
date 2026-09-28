// Disposable, locally generated static assets. Never stores simulation state.
const DATABASE = "fabelgeist-generated-assets";
const STORE = "products";
const CACHE_FORMAT = "gzip-cbor-v1";
const STORAGE_TIMEOUT_MS = 2_000;
const MAX_CACHE_BYTES = 512 * 1024 * 1024;
const MAX_PRODUCT_BYTES = 128 * 1024 * 1024;
const digest = async bytes => Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)),
  byte => byte.toString(16).padStart(2, "0")).join("");

async function decode(record) {
  const reader = new Blob([record.bytes]).stream().pipeThrough(new DecompressionStream("gzip")).getReader();
  const chunks = [];
  let length = 0;
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      length += value.byteLength;
      if (length > record.decodedSize) throw new Error("Cached product exceeds its size bound");
      chunks.push(value);
    }
    if (length !== record.decodedSize) throw new Error("Cached product was truncated");
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return bytes;
  } finally { await reader.cancel().catch(() => {}); }
}

export async function openGeneratedCache(revision, storage) {
  let database, disabled = false;
  const writes = new Set();
  try {
    storage ??= globalThis.indexedDB;
    if (!storage) disabled = true;
    if (storage) database = await new Promise((resolve, reject) => {
      const request = storage.open(DATABASE, 1);
      const timer = setTimeout(() => { disabled = true; reject(new Error("Cache open timed out")); }, STORAGE_TIMEOUT_MS);
      request.onupgradeneeded = () => {
        request.result.createObjectStore(STORE, { keyPath: "key" }).createIndex("used", "used");
      };
      request.onsuccess = () => {
        clearTimeout(timer);
        if (disabled) request.result.close(); else resolve(request.result);
      };
      request.onerror = () => { clearTimeout(timer); reject(request.error); };
      request.onblocked = () => { clearTimeout(timer); reject(new Error("Cache blocked")); };
    });
  } catch { disabled = true; }
  if (database) database.onversionchange = () => { disabled = true; database.close(); };

  async function transaction(mode, operation) {
    if (disabled) return undefined;
    try {
      return await new Promise((resolve, reject) => {
        const tx = database.transaction(STORE, mode);
        const timer = setTimeout(() => { tx.abort(); reject(new Error("Cache transaction timed out")); }, STORAGE_TIMEOUT_MS);
        let result;
        tx.oncomplete = () => { clearTimeout(timer); resolve(result); };
        tx.onabort = tx.onerror = () => { clearTimeout(timer); reject(tx.error); };
        try { operation(tx.objectStore(STORE), value => { result = value; }); }
        catch (error) { clearTimeout(timer); tx.abort(); reject(error); }
      });
    } catch { disabled = true; return undefined; }
  }
  const key = job => digest(new TextEncoder().encode(`${CACHE_FORMAT}\n${revision}\n${job}`));
  async function remove(job) {
    const id = await key(job);
    await transaction("readwrite", store => store.delete(id));
  }
  async function prune() {
    // Walk newest first without copying every large blob into a JS array.
    await transaction("readwrite", store => {
      let total = 0;
      const request = store.index("used").openCursor(null, "prev");
      request.onsuccess = () => {
        const cursor = request.result;
        if (!cursor) return;
        const size = cursor.value.size;
        if (!Number.isSafeInteger(size) || size < 0 || size > MAX_PRODUCT_BYTES) cursor.delete();
        else {
          total += size;
          if (total > MAX_CACHE_BYTES) cursor.delete();
        }
        cursor.continue();
      };
    });
  }
  return {
    async get(job) {
      if (disabled) return undefined;
      const id = await key(job);
      const record = await transaction("readonly", (store, done) => {
        const request = store.get(id); request.onsuccess = () => done(request.result);
      });
      if (!record) return undefined;
      if (record.revision !== revision || !(record.bytes instanceof Uint8Array)
        || record.size !== record.bytes.byteLength || record.size > MAX_PRODUCT_BYTES
        || !Number.isSafeInteger(record.decodedSize) || record.decodedSize < 0 || record.decodedSize > MAX_PRODUCT_BYTES
        || await digest(record.bytes) !== record.checksum) {
        await remove(job); return undefined;
      }
      try { return await decode(record); }
      catch { await remove(job); return undefined; }
    },
    put(job, bytes) {
      if (disabled || bytes.byteLength > MAX_PRODUCT_BYTES) return;
      const write = (async () => {
        const compressed = new Uint8Array(await new Response(new Blob([bytes]).stream()
          .pipeThrough(new CompressionStream("gzip"))).arrayBuffer());
        const [id, checksum] = await Promise.all([key(job), digest(compressed)]);
        await transaction("readwrite", store => store.put({ key: id, revision,
          bytes: compressed, checksum, size: compressed.byteLength,
          decodedSize: bytes.byteLength, used: Date.now() }));
      })().catch(() => { disabled = true; });
      writes.add(write); write.finally(() => writes.delete(write));
    },
    remove,
    async close() {
      await Promise.all(writes); await prune(); database?.close(); disabled = true;
    },
  };
}
