const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const origin = "http://side-panels.test";
const staticRoot = path.resolve(__dirname, "../static");
const tacticalRoot = path.resolve(__dirname, "../../adventuresim-stdb-module/static");
const reviewRoot = process.env.PANEL_REVIEW_DIR;

function panel(side, context, panes) {
  return `<div class="side-panel" data-side-panel="${side}" data-panel-context="${context}">
    <nav role="tablist" class="side-panel-tabs" aria-label="${side} panel">${panes.map(([id], index) =>
      `<button type="button" role="tab" class="side-panel-tab" id="${side}-${context}-${id}-tab"
        aria-controls="${side}-${context}-${id}-panel" aria-selected="${index === 0}" tabindex="${index === 0 ? 0 : -1}"
        data-panel-tab="${id}">${id}</button>`).join("")}</nav>
    ${panes.map(([id, content], index) => `<section role="tabpanel" class="side-panel-content" tabindex="0"
      id="${side}-${context}-${id}-panel" aria-labelledby="${side}-${context}-${id}-tab"
      data-panel-pane="${id}" ${index ? "hidden" : ""}>${content}</section>`).join("")}</div>`;
}

const stock = `<table class="trade-inventory-table"><tbody><tr class="trade-inventory-row" data-merchant-item="bandage" data-merchant-sell-price="2">
  <td class="inventory-item-name"><span class="inventory-item-label" data-item-name="bandage">Bandage</span>
    <span class="inventory-row-actions"><button type="button" data-merchant-buy="bandage" data-merchant-buy-price="4" data-count="99">Buy bandage</button></span></td>
  <td class="inventory-count">99</td><td class="inventory-gold">4</td></tr></tbody></table>`;
const owned = `<table class="trade-inventory-table"><tbody>
  <tr class="trade-inventory-row" data-merchant-item="bandage" data-inventory-quantity="2" data-target="0"><td>Bandage</td><td class="inventory-count">2</td></tr>
  <tr class="trade-inventory-row" data-merchant-item="coin"><td><span data-item-name="coin">Coin</span></td><td class="inventory-count">100</td></tr>
  </tbody></table>`;

function fixture(context = "shop-a") {
  const inventory = `<div class="inventory-owner-panel" data-inventory-tabs>
    <nav class="inventory-owner-tabs" role="tablist" aria-label="Trading inventory">
      <button type="button" class="inventory-owner-tab active" role="tab" aria-selected="true" tabindex="0" data-inventory-tab="player">Player</button>
      <button type="button" class="inventory-owner-tab" role="tab" aria-selected="false" tabindex="-1" data-inventory-tab="party">Party</button>
    </nav><div data-inventory-pane="player">${owned}</div><div data-inventory-pane="party" hidden>${owned}</div></div>`;
  return `<div class="main-grid"><aside class="left-sidebar">${panel("left", context, [["trade", stock], ["services", "<h2>Services</h2><button>Repair</button>"]])}</aside>
    <main class="center-content"><form id="merchant-offer" class="party-offer" hidden action="/${context}/offer" method="post">
      <span class="party-offer-summary"></span><input type="hidden" name="return_to" value="/${context}">
      <input type="hidden" name="inventory_scope" value="player"><button type="button" data-cancel-trade="merchant">Cancel</button><button type="submit" disabled>Offer</button>
    </form></main><aside class="right-sidebar">${panel("right", "character-7", [["inventory", inventory], ["character", "<h2>Anna</h2><label>Search <input type='search' aria-label='Search'></label>"]])}</aside></div>`;
}

async function open(page) {
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.route(`${origin}/**`, route => {
    const url = new URL(route.request().url());
    if (url.pathname.startsWith("/static/")) return route.fulfill({ path: path.join(staticRoot, url.pathname.slice(8)) });
    if (url.pathname.startsWith("/tactical/")) return route.fulfill({ path: path.join(tacticalRoot, path.basename(url.pathname)) });
    return route.fulfill({ contentType: "text/html", body: `<!doctype html><html><head>
      ${["base", "reset", "layout", "strategic", "workspace", "strategic-scene", "side-panels"].map(name => `<link rel="stylesheet" href="/static/css/${name}.css">`).join("")}
      </head><body data-tactical-active><div id="strategic-render-surface"><canvas id="game-canvas" tabindex="0"></canvas></div>
      <div id="strategic-page" class="app" data-strategic-workspace>${fixture()}</div>
      <script>window.originalCanvas = document.querySelector('canvas'); window.ticks = 0;
        window.sceneIdentity = {}; setInterval(() => window.ticks++, 16);</script>
      <script src="/tactical/tactical-input.js"></script><script src="/static/side-panels.js"></script><script src="/static/party-trade.js"></script>
      </body></html>` });
  });
  await page.goto(origin);
  await page.bringToFront();
  return errors;
}

test("independent panel tabs retain drafts, keyboard state and the running scene", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    const errors = await open(page);
    await page.getByRole("button", { name: "Buy bandage" }).click();
    assert.equal(await page.locator("[name='buy_quantities']").inputValue(), "1");
    const ticks = await page.evaluate(() => window.ticks);
    const left = page.locator("[data-side-panel='left']");
    const right = page.locator("[data-side-panel='right']");
    await left.getByRole("tab", { name: "services", exact: true }).click();
    await right.getByRole("tab", { name: "character", exact: true }).click();
    assert.equal(await left.locator("[data-panel-pane='trade']").isVisible(), false);
    assert.equal(await page.locator("[name='buy_quantities']").inputValue(), "1");
    await left.getByRole("tab", { name: "services", exact: true }).press("Home");
    assert.equal(await left.getByRole("tab", { name: "trade", exact: true }).getAttribute("aria-selected"), "true");
    assert.equal(await right.getByRole("tab", { name: "character", exact: true }).getAttribute("aria-selected"), "true");
    await right.getByRole("tab", { name: "character", exact: true }).press("ArrowRight");
    assert.equal(await right.getByRole("tab", { name: "inventory", exact: true }).evaluate(el => el === document.activeElement), true);
    await right.getByRole("tab", { name: "Player", exact: true }).press("End");
    assert.equal(await page.locator("[name='inventory_scope']").inputValue(), "party");
    assert.equal(await right.getByRole("tab", { name: "Party", exact: true }).getAttribute("aria-selected"), "true");
    await page.waitForFunction(value => window.ticks > value, ticks);
    assert.equal(await page.evaluate(() => originalCanvas === document.querySelector("canvas")), true);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("subject changes clear staged trades and live refresh restores the selected tab", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = await open(page);
    await page.getByRole("button", { name: "Buy bandage" }).click();
    await page.locator("[data-side-panel='left'] [data-panel-tab='services']").click();
    await page.evaluate(() => {
      const root = document.querySelector("[data-side-panel='left']");
      root.replaceWith(root.cloneNode(true));
      document.dispatchEvent(new CustomEvent("strategic-live-regions-refreshed", { detail: { regions: ["left-sidebar"] } }));
    });
    assert.equal(await page.locator("[data-side-panel='left'] [data-panel-tab='services']").getAttribute("aria-selected"), "true");
    assert.equal(await page.locator("[data-side-panel='left'] [data-panel-tab='services']").evaluate(el => el === document.activeElement), true);
    await page.evaluate(html => {
      document.dispatchEvent(new Event("strategic-page-unmounting"));
      document.querySelector("#strategic-page").innerHTML = html;
      document.dispatchEvent(new Event("strategic-page-mounted"));
    }, fixture("shop-b"));
    assert.equal(await page.locator("[data-side-panel='left'] [data-panel-tab='trade']").getAttribute("aria-selected"), "true");
    assert.equal(await page.evaluate(() => strategicTradeUi.state.merchantDraft.size), 0);
    await page.getByRole("button", { name: "Buy bandage" }).click();
    assert.equal(await page.locator("[name='buy_quantities']").inputValue(), "1");
    assert.equal(await page.locator("#merchant-offer").getAttribute("action"), "/shop-b/offer");
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("HTML owns input during editing and panels remain usable at narrow widths", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = await open(page);
    for (const width of [1280, 390]) {
      await page.setViewportSize({ width, height: 844 });
      await page.locator("[data-side-panel='right'] [data-panel-tab='character']").click();
      await page.getByRole("searchbox", { name: "Search" }).fill("wasd");
      assert.equal(await page.evaluate(() => tacticalUiOwnsInput()), true);
      await page.mouse.move(width / 2, 100);
      assert.equal(await page.evaluate(() => tacticalUiOwnsInput()), true);
      await page.locator("#game-canvas").click({ position: { x: width / 2, y: 100 } });
      assert.equal(await page.evaluate(() => tacticalUiOwnsInput()), false);
      const bounds = await page.locator("[data-side-panel='right']").boundingBox();
      assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= width + 1);
      if (reviewRoot) {
        fs.mkdirSync(reviewRoot, { recursive: true });
        await page.screenshot({ path: path.join(reviewRoot, `panels-${width}.png`) });
      }
    }
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
