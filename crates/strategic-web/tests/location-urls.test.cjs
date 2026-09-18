const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const window = {};
vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../static/location-urls.js"), "utf8"), { window });
const urls = window.strategicLocationUrls;

test("location matching respects decoded identity and path boundaries", () => {
  const state = { kind: "settlement", id: "willowmere" };
  for (const path of ["", "/places/inn", "/places/inn/rest", "/party/1", "/places/inn/fireplace"]) {
    assert.equal(urls.contains(`/locations/settlement/willowmere${path}`, state), true);
  }
  for (const path of ["/locations/settlement/willowmere-2", "/locations/settlement/willowmere2/places/inn", "/settlements/willowmere", "/locations/case-site/willowmere"]) {
    assert.equal(urls.contains(path, state), false);
  }
  const site = { kind: "case_site", id: "site:forest/北" };
  assert.equal(urls.root(site), "/locations/case-site/site%3Aforest%2F%E5%8C%97");
  assert.equal(urls.contains(`${urls.root(site)}/enemy`, site), true);
  assert.equal(urls.contains("/locations/case-site/%ZZ", site), false);
  assert.equal(urls.root({ kind: "unknown", id: "town" }), null);
});

test("the session camp includes child pages without accepting similarly named paths", () => {
  const state = { kind: "camp", id: null };
  assert.equal(urls.root(state), "/locations/camp");
  assert.equal(urls.contains("/locations/camp", state), true);
  assert.equal(urls.contains("/locations/camp/fireplace", state), true);
  assert.equal(urls.contains("/locations/camps", state), false);
  assert.equal(urls.contains("/camp", state), false);
});

test("live state stays on camp fixtures and navigates away from another settlement", async () => {
  const source = fs.readFileSync(path.join(__dirname, "../static/live-state.js"), "utf8");
  for (const [pathname, state, expected] of [
    ["/locations/camp/fireplace", { kind: "camp", id: null, path: "/locations/camp" }, []],
    ["/locations/settlement/willowmere-2/places/inn", { kind: "settlement", id: "willowmere", path: "/locations/settlement/willowmere" }, ["/locations/settlement/willowmere"]],
  ]) {
    const navigated = [];
    const marker = { dataset: { liveRevision: "1" } };
    let observe;
    vm.runInNewContext(source, {
      window: { strategicLocationUrls: urls,
        strategicBackgroundFetch: async () => ({ ok: true, json: async () => state }),
        strategicNavigate: (path) => navigated.push(path),
        reportStrategicError: (error) => { throw error; } },
      location: { pathname },
      document: { querySelector: (selector) => selector.startsWith("#strategic-live-") ? marker : null,
        querySelectorAll: () => [], addEventListener() {}, dispatchEvent() {} },
      MutationObserver: class { constructor(callback) { observe = callback; } observe() {} },
      CustomEvent: class { constructor(type) { this.type = type; } },
    });
    marker.dataset.liveRevision = "2";
    observe();
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(navigated, expected);
  }
});
