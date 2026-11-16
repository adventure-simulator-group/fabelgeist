const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const fixtures = process.env.UX_CAPTURE_DIR;
const staticRoot = path.resolve(__dirname, "../static");

test("rendered town entry states remain readable and actionable at desktop and narrow widths",
  { skip: !fixtures }, async () => {
    const browser = await chromium.launch({ headless: true });
    try {
      for (const width of [1280, 390]) {
        for (const state of ["ready", "preparing", "unavailable"]) {
          const page = await browser.newPage({ viewport: { width, height: 844 } });
          await page.route("**/*", async (route) => {
            const url = new URL(route.request().url());
            if (url.hostname !== "town-entry.test") return route.abort();
            if (url.pathname.startsWith("/static/")) {
              const file = path.join(staticRoot, url.pathname.slice(8));
              return fs.existsSync(file) ? route.fulfill({ path: file }) : route.abort();
            }
            if (url.pathname !== "/") return route.abort();
            return route.fulfill({ contentType: "text/html; charset=utf-8",
              body: fs.readFileSync(path.join(fixtures, `town-scene-${state}.html`), "utf8") });
          });
          await page.goto("http://town-entry.test/");
          await page.evaluate(() => document.fonts.ready);
          const status = page.locator("#settlement-scene-status");
          assert.equal(await status.getAttribute("aria-live"), "polite");
          const layout = await page.locator("main").evaluate((main) => {
            const bounds = main.getBoundingClientRect();
            return { x: bounds.x, width: bounds.width, background: getComputedStyle(main).backgroundColor,
              documentWidth: document.documentElement.scrollWidth, viewport: innerWidth };
          });
          assert.ok(layout.x >= 0 && layout.x + layout.width <= width + 1);
          assert.ok(layout.documentWidth <= layout.viewport + 1);
          assert.notEqual(layout.background, "rgba(0, 0, 0, 0)");
          const action = state === "ready" ? page.getByRole("link", { name: "Enter town", exact: true })
            : state === "unavailable" ? page.getByRole("button", { name: "Try again", exact: true })
              : page.getByRole("link", { name: "Return", exact: true });
          await action.click({ trial: true });
          await action.focus();
          assert.equal(await action.evaluate((element) => element === document.activeElement), true);
          if (state === "ready") {
            assert.equal(await action.getAttribute("data-character-id"), "41");
            assert.equal(await action.getAttribute("data-server-addr"), "127.0.0.1:6000");
          }
          await page.screenshot({ path: path.join(fixtures, `town-scene-${state}-${width}.png`) });
          await page.close();
        }
      }
    } finally {
      await browser.close();
    }
  });
