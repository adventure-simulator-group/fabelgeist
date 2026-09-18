const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const { parseHTML } = require("linkedom");

const source = fs.readFileSync(path.join(__dirname, "..", "static", "building-state.js"), "utf8");

function fixture(href, active = "organization-merchants-lubeck") {
  const { window, document } = parseHTML(`<html><body>
    <main id="strategic-page">
      <nav data-settlement-id="lubeck">
        <a class="nav-tab" data-service-id="map" data-building-id="map"></a>
        <a class="nav-tab" data-service-id="inn" data-building-id="inn"></a>
        <a class="nav-tab active" data-service-id="organization"
          data-building-id="organization-merchants-lubeck"></a>
      </nav>
      <a id="party-link" href="/locations/settlement/lubeck/party/7">Party</a>
      <form id="party-form" action="/locations/settlement/lubeck/party/7/social"></form>
    </main>
  </body></html>`);
  document.querySelectorAll(".nav-tab").forEach((tab) =>
    tab.classList.toggle("active", tab.dataset.buildingId === active));
  const location = { href, origin: "http://game.test" };
  const replacements = [];
  const history = {
    state: null,
    replaceState(_state, _title, url) {
      replacements.push(url.toString());
    },
  };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "..", "static", "location-urls.js"), "utf8"), { window });
  vm.runInNewContext(source, {
    window,
    document,
    location,
    history,
    URL,
    MutationObserver: window.MutationObserver,
    Node: window.Node,
  });
  return { window, document, location, replacements };
}

test("exact organization state survives links, forms, remounts, and live insertion", async () => {
  const view = fixture(
    "http://game.test/locations/settlement/lubeck/party/7?building=organization-merchants-lubeck",
  );
  const organization = view.document.querySelector('[data-building-id="organization-merchants-lubeck"]');
  assert.equal(organization.classList.contains("active"), true);
  assert.equal(view.document.querySelectorAll(".nav-tab.active").length, 1);
  assert.equal(view.document.querySelector("#party-link").getAttribute("href"),
    "/locations/settlement/lubeck/party/7?building=organization-merchants-lubeck");
  assert.equal(view.document.querySelector("#party-form").getAttribute("action"),
    "/locations/settlement/lubeck/party/7/social?building=organization-merchants-lubeck");

  view.document.dispatchEvent(new view.window.Event("strategic-page-mounted"));
  assert.equal(organization.classList.contains("active"), true);
  assert.equal(view.document.querySelectorAll(".nav-tab.active").length, 1);
  const live = view.document.createElement("a");
  live.href = "/locations/settlement/lubeck/party/9";
  view.document.querySelector("#strategic-page").append(live);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(live.getAttribute("href"),
    "/locations/settlement/lubeck/party/9?building=organization-merchants-lubeck");
});

test("an invalid requested identity is removed and cannot become active", () => {
  const view = fixture(
    "http://game.test/locations/settlement/lubeck/party/7?building=organization-foreign-town",
  );
  assert.equal(view.replacements.length, 1);
  assert.equal(view.replacements[0], "http://game.test/locations/settlement/lubeck/party/7");
  assert.equal(view.document.querySelector('[data-building-id="organization-foreign-town"]'), null);
  assert.equal(view.document.querySelectorAll(".nav-tab.active").length, 1);
});

test("the map opens party panels without inventing a physical building", () => {
  const view = fixture("http://game.test/locations/settlement/lubeck/party/7?building=map&medical=surgery#limb", "map");
  assert.deepEqual(view.replacements, ["http://game.test/locations/settlement/lubeck/party/7?medical=surgery#limb"]);
  assert.equal(view.document.querySelector("#party-link").getAttribute("href"),
    "/locations/settlement/lubeck/party/7");
});

test("fireplace building state survives mount and remount without rewriting history", () => {
  const view = fixture(
    "http://game.test/locations/settlement/lubeck/places/inn/fireplace", "inn",
  );
  const inn = view.document.querySelector('[data-building-id="inn"]');
  assert.equal(inn.classList.contains("active"), true);
  assert.deepEqual(view.replacements, []);

  view.document.dispatchEvent(new view.window.Event("strategic-page-mounted"));
  assert.equal(inn.classList.contains("active"), true);
  assert.equal(view.document.querySelectorAll(".nav-tab.active").length, 1);
  assert.deepEqual(view.replacements, []);
});

test("fireplace identity comes from the place, and stale building queries are removed", () => {
  const view = fixture(
    "http://game.test/locations/settlement/lubeck/places/inn/fireplace?building=forge", "inn",
  );
  assert.equal(view.replacements.length, 1);
  assert.equal(
    view.replacements[0],
    "http://game.test/locations/settlement/lubeck/places/inn/fireplace",
  );
  assert.equal(view.document.querySelectorAll(".nav-tab.active").length, 1);
});

test("building context preserves other queries and fragments and stays in its settlement", async () => {
  const view = fixture("http://game.test/locations/settlement/lubeck/party/7?building=inn", "inn");
  const page = view.document.querySelector("#strategic-page");
  const link = view.document.createElement("a");
  link.href = "/locations/settlement/lubeck/party/9?medical=surgery#limb";
  const foreign = view.document.createElement("a");
  foreign.href = "/locations/settlement/lubeck-2/party/9";
  page.append(link, foreign);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(link.getAttribute("href"), "/locations/settlement/lubeck/party/9?medical=surgery&building=inn#limb");
  assert.equal(foreign.getAttribute("href"), "/locations/settlement/lubeck-2/party/9");
});
