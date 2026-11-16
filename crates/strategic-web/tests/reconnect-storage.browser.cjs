const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

test("reconnect capability survives reload, rotates, and remains scoped to its tab and scene", async () => {
  const source = fs.readFileSync(path.resolve(__dirname,
    "../../adventuresim-tactical-netcode/src/client/reconnect.rs"), "utf8");
  const snippet = source.match(/inline_js = "([\s\S]*?)"\)\]/);
  assert.ok(snippet, "the browser uses the production wasm-bindgen storage adapter");
  const browser = await chromium.launch({ headless: true });
  try {
    const context = await browser.newContext();
    await context.route("http://reconnect.test/**", async (route) => {
      const module = new URL(route.request().url()).pathname === "/reconnect.js";
      await route.fulfill({
        contentType: module ? "text/javascript" : "text/html",
        body: module ? snippet[1] : `<script type="module">
          import * as reconnect from '/reconnect.js'; window.reconnect = reconnect;
        </script>`,
      });
    });
    const page = await context.newPage();
    await page.goto("http://reconnect.test/");
    await page.waitForFunction(() => window.reconnect);
    await page.evaluate(() => window.reconnect.browser_store("ws://town-a", "41", "first-capability"));
    await page.reload();
    await page.waitForFunction(() => window.reconnect);
    assert.equal(await page.evaluate(() => window.reconnect.browser_load("ws://town-a", "41")), "first-capability");
    assert.equal(await page.evaluate(() => window.reconnect.browser_load("ws://town-b", "41")), "");
    assert.equal(await page.evaluate(() => window.reconnect.browser_load("ws://town-a", "42")), "");
    const otherTab = await context.newPage();
    await otherTab.goto("http://reconnect.test/");
    await otherTab.waitForFunction(() => window.reconnect);
    assert.equal(await otherTab.evaluate(() => window.reconnect.browser_load("ws://town-a", "41")), "");
    await page.evaluate(() => window.reconnect.browser_store("ws://town-a", "41", "rotated-capability"));
    await page.reload();
    await page.waitForFunction(() => window.reconnect);
    assert.equal(await page.evaluate(() => window.reconnect.browser_load("ws://town-a", "41")), "rotated-capability");
    await page.evaluate(() => window.reconnect.browser_remove("ws://town-a", "41"));
    await page.reload();
    await page.waitForFunction(() => window.reconnect);
    assert.equal(await page.evaluate(() => window.reconnect.browser_load("ws://town-a", "41")), "");
  } finally {
    await browser.close();
  }
});
