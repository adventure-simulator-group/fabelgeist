const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const source = fs.readFileSync(path.join(__dirname, "../static/regional-city-request.js"), "utf8")
  .replace(/^import .*;\r?\n/m, "").replace("export function", "function");
const createRegionalCityRequests = Function("prepareRegionalCity",
  `${source}\nreturn createRegionalCityRequests;`)(() => { throw Error("Unexpected preparation"); });
const request = name => ({ source: "a".repeat(64),
  place: `place:v1:settlement:${Buffer.from(name).toString("hex")}` });
const response = document => ({ ok: true, text: async () => document });
const turn = () => new Promise(resolve => setImmediate(resolve));
const deferred = () => { let resolve; const promise = new Promise(yes => { resolve = yes; });
  return { promise, resolve }; };

test("panning cancels obsolete city preparation before publishing the latest opaque document", async () => {
  const blocked = deferred(), prepared = [], installed = [], cancelled = [], signals = [];
  const oldDocument = '{"seed":18446744073709551615,"city":"old"}';
  const latestDocument = '{"seed":18446744073709551614,"city":"latest"}';
  const cities = createRegionalCityRequests({
    runtimePromise: Promise.resolve({ wasm_cancel_generation: value => cancelled.push(value) }),
    fetchCity: url => response(url.includes(encodeURIComponent(request("old").place))
      ? oldDocument : latestDocument),
    prepareCity: async (_, document, { signal }) => {
      prepared.push(document); signals.push(signal);
      if (document === oldDocument) await blocked.promise;
      return document === oldDocument ? "old-ticket" : "latest-ticket";
    },
    install: value => installed.push(value),
  });
  const obsolete = cities.request(request("old")); await turn();
  const intermediate = cities.request(request("intermediate")); await turn();
  const latest = cities.request(request("latest")); await turn();
  assert.deepEqual(prepared, [oldDocument]);
  assert.equal(signals[0].aborted, true);
  blocked.resolve();
  assert.equal((await obsolete).status, "superseded");
  assert.equal((await intermediate).status, "superseded");
  assert.equal((await latest).status, "prepared");
  assert.deepEqual(prepared, [oldDocument, latestDocument]);
  assert.deepEqual(cancelled, ["old-ticket"]);
  assert.deepEqual(installed, [{ document: latestDocument, preparation: "latest-ticket" }]);
});

test("failed city refinement retries explicitly and warm return cancels remote fetching", async () => {
  const blocked = deferred(), installed = [], signals = [];
  let fetches = 0, available = false;
  const cities = createRegionalCityRequests({ runtimePromise: Promise.resolve({}),
    fetchCity: (url, { signal }) => {
      fetches++; signals.push(signal);
      if (url.includes(encodeURIComponent(request("remote").place))) return blocked.promise;
      return available ? response("resident") : { ok: false, status: 503 };
    }, prepareCity: async () => "ticket", install: value => installed.push(value),
  });
  assert.equal((await cities.request(request("resident"))).status, "failed");
  assert.equal((await cities.request(request("resident"))).status, "failed");
  assert.equal(fetches, 1, "per-frame requests do not retry a failed city");
  available = true;
  assert.equal((await cities.retry()).status, "prepared");
  cities.cancel();
  const obsolete = cities.request(request("remote")); await turn();
  assert.equal((await cities.request(request("resident"))).status, "reused");
  assert.equal(signals.at(-1).aborted, true);
  blocked.resolve(response("obsolete"));
  assert.equal((await obsolete).status, "superseded");
  assert.equal(installed.length, 1, "warm return needs no new products or installation");
  assert.throws(() => cities.request({ ...request("resident"), place: "place:v1:case-site:aa" }),
    /Invalid focused settlement request/);
});
