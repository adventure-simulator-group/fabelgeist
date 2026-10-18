const test = require("node:test");
const assert = require("node:assert/strict");
const { chromium } = require("playwright");
const { withDeadline } = require("./startup-deadline.cjs");

test("a stalled page snapshot cannot retain the owned benchmark browser", {timeout: 45_000}, async () => {
  let server, browser;
  try {
    server = await chromium.launchServer({headless: true,
      channel: process.env.STRATEGIC_BROWSER_CHANNEL || (process.platform === "win32" ? "msedge" : undefined),
      args: ["--disable-gpu"]});
    browser = await withDeadline(chromium.connect(server.wsEndpoint()), "browser connection");
    const page = await withDeadline(browser.newPage(), "test page creation");
    await page.setContent("<!doctype html><title>Bounded snapshot</title>");
    assert.equal(await page.title(), "Bounded snapshot");
    await assert.rejects(withDeadline(page.evaluate(() => new Promise(() => {})), "stalled snapshot", 20),
      /stalled snapshot exceeded/);
  } finally {
    await withDeadline(browser?.close(), "browser connection close", 5_000).catch(() => {});
    await withDeadline(server?.kill(), "owned process termination", 5_000);
  }
  assert.notEqual(server.process().exitCode, null, "the owned browser process survived cleanup");
});
