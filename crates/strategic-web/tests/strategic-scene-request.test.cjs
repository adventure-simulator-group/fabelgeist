const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../static/strategic-scene-request.js"), "utf8")
  .replace(/^import .*;\r?\n/m, "").replace("export function", "function");
const createSceneRequests = Function("prepareGeneratedScene", `${source}\nreturn createSceneRequests;`)(
  () => { throw Error("Unexpected generator"); });
const deferred = () => { let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject }; };
const response = input => ({ ok: true, text: async () => input });
const request = location => ({ location, settlement: location, venues: { places: [], people: [] } });
const turn = () => new Promise(resolve => setImmediate(resolve));

test("obsolete scene responses cannot prepare or replace the latest destination", async () => {
  const first = deferred(), installed = [], prepared = [], states = [], signals = [];
  const scenes = createSceneRequests({ runtimePromise: Promise.resolve({wasm_cancel_generation(){}}),
    fetchScene: (url, { signal }) => {
      signals.push(signal); return url.endsWith("first") ? first.promise : response("new scene");
    }, prepareScene: async (_, input) => prepared.push(input),
    install: value => installed.push(value), changed: state => states.push(state),
  });
  const obsolete = scenes.request(request("first")); await turn();
  const latest = scenes.request(request("latest"));
  assert.deepEqual(await latest, { status: "prepared", location: "latest" });
  first.resolve(response("old scene"));
  assert.deepEqual(await obsolete, { status: "superseded" });
  assert.equal(signals[0].aborted, true);
  assert.deepEqual(prepared, ["new scene"]);
  assert.deepEqual(installed, [{ location: "latest", input: "new scene", preparation: 1 }]);
  assert.deepEqual(scenes.state, { phase: "prepared", location: "latest" });
  assert.ok(Object.isFrozen(states[0]));
});

test("replacement waits for cancelled generation ownership and drops intermediate requests", async () => {
  const first = deferred(), installed = [], prepared = [], signals = [];
  const scenes = createSceneRequests({ runtimePromise: Promise.resolve({wasm_cancel_generation(){}}),
    fetchScene: url => response(url.split("=").at(-1)),
    prepareScene: async (_, input, _venues, { signal }) => {
      prepared.push(input); signals.push(signal); if (input === "first") await first.promise;
    }, install: value => installed.push(value), changed() {},
  });
  const obsolete = scenes.request(request("first")); await turn();
  const intermediate = scenes.request(request("intermediate")); await turn();
  const latest = scenes.request(request("latest")); await turn();
  assert.deepEqual(prepared, ["first"]);
  assert.equal(signals[0].aborted, true);
  first.resolve();
  assert.deepEqual(await obsolete, { status: "superseded" });
  assert.deepEqual(await intermediate, { status: "superseded" });
  assert.deepEqual(await latest, { status: "prepared", location: "latest" });
  assert.deepEqual(prepared, ["first", "latest"]);
  assert.deepEqual(installed.map(value => value.location), ["latest"]);
});

test("a failed destination does not retry every frame or disable later destinations", async () => {
  let fetches = 0;
  const cause = Error("Scene source unavailable");
  const scenes = createSceneRequests({ runtimePromise: Promise.resolve({wasm_cancel_generation(){}}),
    fetchScene: url => { fetches++; if (url.endsWith("first")) throw cause; return response("new"); },
    prepareScene: async () => {}, install() {}, changed() {},
  });
  assert.deepEqual(await scenes.request(request("first")), { status: "failed", location: "first", cause });
  assert.equal(scenes.state.phase, "failed");
  await scenes.request(request("first")); assert.equal(fetches, 1);
  assert.equal((await scenes.request(request("latest"))).status, "prepared");
  assert.equal(fetches, 2);
});

test("returning to the resident document cancels pending work and reuses preparation", async () => {
  const blocked = deferred(), installed = [];
  let generations = 0;
  const scenes = createSceneRequests({ runtimePromise: Promise.resolve({wasm_cancel_generation(){}}),
    fetchScene: url => url.endsWith("first") ? response("resident") : blocked.promise,
    prepareScene: async () => { generations++; }, install: value => installed.push(value), changed() {},
  });
  await scenes.request(request("first"));
  const obsolete = scenes.request(request("latest")); await turn();
  assert.deepEqual(await scenes.request(request("first")), { status: "reused", location: "first" });
  blocked.resolve(response("obsolete"));
  assert.equal((await obsolete).status, "superseded");
  assert.equal(generations, 1); assert.equal(installed.length, 1);
  assert.deepEqual(scenes.state, { phase: "prepared", location: "first" });
});

test("hiding the view cancels preparation and suppresses obsolete failures", async () => {
  const blocked = deferred(), states = [];
  const scenes = createSceneRequests({ runtimePromise: Promise.resolve({wasm_cancel_generation(){}}),
    fetchScene: () => blocked.promise, prepareScene: async () => {},
    install() { throw Error("Obsolete installation"); }, changed: state => states.push(state),
  });
  const obsolete = scenes.request(request("first")); await turn(); scenes.cancel();
  blocked.reject(Error("Old request failed"));
  assert.deepEqual(await obsolete, { status: "superseded" });
  assert.deepEqual(scenes.state, { phase: "idle" });
  assert.deepEqual(states.map(state => state.phase), ["loading", "idle"]);
});
