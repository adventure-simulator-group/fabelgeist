const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const staticRoot = path.join(__dirname, "../static");
const styles = ["base", "reset", "layout", "components", "strategic", "architecture", "utilities", "workspace"];
const shell = (content) => `<!doctype html><html><head>${styles.map(name =>
  `<link rel="stylesheet" href="/static/css/${name}.css">`).join("")}</head>
  <body><div id="strategic-page" class="app"><header class="top-bar">Riverdale</header>
  <nav class="workspace-bar"><strong class="workspace-title">Character</strong><a href="/" data-workspace-location>Location</a>
  <a data-workspace-inventory hidden>Party inventory</a><button data-workspace-chat hidden>Conversation</button></nav>
  <div class="main-grid"><aside class="left-sidebar">Attributes</aside>
  <main class="center-content settlement-main">${content}</main>
  <aside class="right-sidebar">Summary</aside></div></div>
  <script src="/static/character-action-dialog.js"></script><script src="/static/workspace.js"></script></body></html>`;

async function openFixture(browser, content, width = 1280) {
  const page = await browser.newPage({ viewport: { width, height: 720 } });
  await page.route("**/*", route => {
    const url = new URL(route.request().url());
    if (url.pathname.startsWith("/static/")) {
      const file = path.join(staticRoot, url.pathname.slice(8));
      return fs.existsSync(file) ? route.fulfill({ path: file }) : route.abort();
    }
    if (url.hostname !== "interface.test") return route.abort();
    return route.fulfill({ contentType: "text/html; charset=utf-8", body: shell(content) });
  });
  await page.goto("http://interface.test/");
  return page;
}

test("action dialogs escape scene clipping and retain readable actions and focus", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    for (const width of [1280, 390]) {
      const page = await openFixture(browser, `<div class="character-action-overlay" data-character-action-dialog>
        <a class="character-action-backdrop" href="/" aria-label="Close"></a>
        <section class="character-action-dialog forage-dialog" role="dialog" aria-modal="true" aria-labelledby="title" tabindex="-1">
        <header class="character-action-dialog-header"><h2 id="title">Forage nearby</h2>
        <a class="character-action-dialog-close" aria-label="Close foraging" href="/">×</a></header>
        <p role="alert">The search could not be completed.</p>
        <div class="modal-actions"><a class="btn btn-secondary character-action-dialog-close" href="/">Return</a></div>
        </section></div>`, width);
      const state = await page.locator("[role=dialog]").evaluate(dialog => {
        const rect = dialog.getBoundingClientRect();
        const action = dialog.querySelector(".btn");
        return {
          root: dialog.parentElement.parentElement.id,
          contained: rect.left >= 0 && rect.right <= innerWidth && rect.top >= 0 && rect.bottom <= innerHeight,
          fits: action.scrollWidth <= action.clientWidth + 1,
          topmost: dialog.contains(document.elementFromPoint(rect.left + 20, rect.top + 20)),
        };
      });
      assert.equal(state.root, "strategic-page");
      assert.equal(state.contained, true);
      assert.equal(state.fits, true);
      assert.equal(state.topmost, true);
      await page.locator(".btn").focus();
      await page.keyboard.press("Tab");
      assert.equal(await page.locator("a[aria-label='Close foraging']").evaluate(el => el === document.activeElement), true);
      if (process.env.UX_CAPTURE_DIR) {
        fs.mkdirSync(process.env.UX_CAPTURE_DIR, { recursive: true });
        await page.screenshot({ path: path.join(process.env.UX_CAPTURE_DIR, `forage-${width}.png`) });
      }
      await page.close();
    }
  } finally { await browser.close(); }
});

test("departure field accepts valid 24-hour times and rejects ambiguous input", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const source = fs.readFileSync(path.join(__dirname, "../src/templates/settlement/travel.rs"), "utf8");
    const pattern = source.match(/pattern="([^"]+)"/)[1];
    const page = await openFixture(browser, `<form><label for="time">Departure (24-hour HH:MM)</label>
      <input id="time" type="text" required pattern="${pattern}" maxlength="5"></form>`);
    for (const [value, valid] of [["08:00", true], ["23:59", true], ["24:00", false], ["8:00", false], ["12:60", false], ["", false]]) {
      await page.locator("#time").fill(value);
      assert.equal(await page.locator("#time").evaluate(el => el.checkValidity()), valid, value);
    }
  } finally { await browser.close(); }
});

test("management tasks preserve the central model space and let players toggle conversation", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await openFixture(browser, `<figure class="service-visual service-visual-chest"></figure>
      <div class="party-portrait-overlay"><div class="party-portrait"><a class="party-portrait-select" href="/member">
      <span class="party-portrait-initial">S</span><span class="party-portrait-name">Sebastian Berndes</span></a></div></div>
      <section class="settlement-chat">Conversation content</section>`);
    const panes = await page.locator(".main-grid").evaluate(grid => [...grid.querySelectorAll("aside")].map(el => el.getBoundingClientRect().width));
    assert.ok(panes.every(width => width >= 304));
    const modelSpace = await page.locator('.main-grid').evaluate(grid => {
      const left = grid.querySelector('.left-sidebar').getBoundingClientRect();
      const center = grid.querySelector('main').getBoundingClientRect();
      const right = grid.querySelector('.right-sidebar').getBoundingClientRect();
      return center.width >= 320 && center.height >= 300 && left.right <= center.left && center.right <= right.left;
    });
    assert.equal(modelSpace, true, 'side panels leave a full-height central character stage');
    assert.equal(await page.locator(".settlement-chat").isVisible(), false);
    await page.locator("[data-workspace-chat]").click();
    assert.equal(await page.locator(".settlement-chat").isVisible(), true);
    assert.equal(await page.locator('.settlement-chat').evaluate(chat => {
      const right = document.querySelector('.right-sidebar').getBoundingClientRect();
      return chat.getBoundingClientRect().left >= right.left;
    }), true, 'conversation stays over its side panel, clear of the character model');
    await page.locator("[data-workspace-chat]").click();
    assert.equal(await page.locator(".settlement-chat").isVisible(), false);
    const name = await page.locator(".party-portrait-name").evaluate(el => ({ width: el.scrollWidth <= el.clientWidth + 1, height: el.scrollHeight <= el.clientHeight + 1 }));
    assert.deepEqual(name, { width: true, height: true });
  } finally { await browser.close(); }
});
