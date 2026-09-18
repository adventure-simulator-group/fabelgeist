const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const origin = "http://location-navigation.test";
const town = "/locations/settlement/willowmere";
const scripts = ["location-urls", "strategic-navigation", "building-state", "live-state"];

function pageHtml(url) {
  const place = url.pathname.split("/places/")[1]?.split("/")[0] || "map";
  const camp = url.pathname.startsWith("/locations/camp");
  const nav = camp ? "" : `<nav data-settlement-id="willowmere">${[
    ["map", town], ["public-square", `${town}/places/public-square`],
    ["residences", `${town}/places/residences`], ["inn", `${town}/places/inn`],
  ].map(([id, href]) => `<a data-building-id="${id}" class="nav-tab ${id === place ? "active" : ""}" href="${href}">${id}</a>`).join("")}</nav>`;
  return `<!doctype html><html><head><title>${place}</title></head><body>
    <canvas id="renderer"></canvas>
    <div id="strategic-live-stream"><span id="strategic-live-revision" data-live-revision="1"></span></div>
    <div id="strategic-page" data-script-profile="strategic" data-path="${url.pathname}">
      ${nav}<main><h1>${place}</h1><div id="details"></div>
      <a id="fireplace" href="${camp ? "/locations/camp" : `${town}/places/inn`}/fireplace">Fireplace</a>
      <a id="party" href="${town}/party/7?medical=surgery#details">Party</a>
      <form id="party-action" action="${town}/party/7/activity?medical=surgery#details"></form>
      </main>
    </div>
    <script>
      window.rendererIdentity = document.querySelector('#renderer');
      window.liveUpdates = 0;
      document.addEventListener('strategic-live-update', () => window.liveUpdates++);
      window.strategicBackgroundFetch = async () => ({ok: true, json: async () => window.navigationState});
      window.reportStrategicError = error => { throw error; };
    </script>
    ${scripts.map(name => `<script src="/static/${name}.js"></script>`).join("")}
    </body></html>`;
}

test("canonical places, history, and live camp updates preserve the strategic document", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    await page.route(`${origin}/**`, route => {
      const url = new URL(route.request().url());
      if (url.pathname.startsWith("/static/")) {
        return route.fulfill({ contentType: "text/javascript", body: fs.readFileSync(
          path.join(__dirname, "..", "static", path.basename(url.pathname)), "utf8") });
      }
      return route.fulfill({ contentType: "text/html", body: pageHtml(url), headers: {
        "X-Strategic-Response": "root", "X-Strategic-Script-Profile": "strategic",
        "X-Strategic-Canonical-Url": `${url.pathname}${url.search}`,
      } });
    });
    const mounted = async pathname => {
      await page.waitForFunction(path => location.pathname === path &&
        document.querySelector("#strategic-page").dataset.path === path, pathname);
      assert.equal(await page.evaluate(() => window.rendererIdentity === document.querySelector("#renderer")), true);
    };
    await page.goto(`${origin}${town}?destination=nearby`);
    for (const place of ["public-square", "residences", "inn"]) {
      await page.locator(`[data-building-id="${place}"]`).click();
      await mounted(`${town}/places/${place}`);
    }
    await page.locator("#fireplace").click();
    await mounted(`${town}/places/inn/fireplace`);
    await page.locator("#party").click();
    await mounted(`${town}/party/7`);
    const partyUrl = new URL(page.url());
    assert.equal(partyUrl.searchParams.get("building"), "inn");
    assert.equal(partyUrl.searchParams.get("medical"), "surgery");
    assert.equal(partyUrl.hash, "#details");
    assert.equal(await page.locator(".nav-tab.active").getAttribute("data-building-id"), "inn");
    assert.match(await page.locator("#party-action").getAttribute("action"), /building=inn#details$/);
    await page.goBack();
    await mounted(`${town}/places/inn/fireplace`);
    await page.goForward();
    await mounted(`${town}/party/7`);
    assert.equal(await page.locator(".nav-tab.active").getAttribute("data-building-id"), "inn");
    await page.evaluate(() => window.strategicNavigate("/locations/camp/fireplace"));
    await mounted("/locations/camp/fireplace");
    await page.evaluate(() => {
      window.navigationState = { kind: "camp", id: null, path: "/locations/camp" };
      document.querySelector("#strategic-live-revision").dataset.liveRevision = "2";
    });
    await page.waitForFunction(() => window.liveUpdates === 1);
    await mounted("/locations/camp/fireplace");
    await page.evaluate(path => window.strategicNavigate(path), `${town}-2/places/inn`);
    await mounted(`${town}-2/places/inn`);
    await page.evaluate(path => {
      window.navigationState = { kind: "settlement", id: "willowmere", path };
      document.querySelector("#strategic-live-revision").dataset.liveRevision = "3";
    }, town);
    await mounted(town);
    assert.deepEqual(errors, []);
  } finally {
    await browser.close();
  }
});
