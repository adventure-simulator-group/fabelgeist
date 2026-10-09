const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const staticRoot = path.join(__dirname, "../static");
const styles = ["base", "reset", "layout", "components", "strategic", "architecture", "utilities", "workspace", "readability", "legends", "portraits", "chat-dock"];
const shell = (content) => `<!doctype html><html><head>${styles.map(name =>
  `<link rel="stylesheet" href="/static/css/${name}.css">`).join("")}</head>
  <body><div id="strategic-page" class="app" data-strategic-workspace><header class="top-bar">Riverdale</header>
  <div class="main-grid"><aside class="left-sidebar">Attributes</aside>
  <main class="center-content settlement-main">${content}</main>
  <aside class="right-sidebar">Summary</aside></div><aside data-chat-dock><section class="settlement-chat"><div class="settlement-chat-messages"></div></section></aside></div>
  <script src="/static/chat-dock.js"></script><script src="/static/character-action-dialog.js"></script><script src="/static/portrait-navigation.js"></script></body></html>`;

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

test("management tasks keep bottom-center chat visible through menu replacement", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await openFixture(browser, `<figure class="service-visual service-visual-chest"></figure>
      <div class="party-portrait-overlay"><div class="party-portrait"><a class="party-portrait-select" href="/member">
      <span class="party-portrait-initial">S</span><span class="party-portrait-name">Sebastian Berndes</span></a></div></div>
      <section class="settlement-chat"><div class="settlement-chat-messages">Conversation content</div></section>`);
    const panes = await page.locator(".main-grid").evaluate(grid => [...grid.querySelectorAll("aside")].map(el => el.getBoundingClientRect().width));
    assert.ok(panes.every(width => width >= 304));
    const modelSpace = await page.locator('.main-grid').evaluate(grid => {
      const left = grid.querySelector('.left-sidebar').getBoundingClientRect();
      const center = grid.querySelector('main').getBoundingClientRect();
      const right = grid.querySelector('.right-sidebar').getBoundingClientRect();
      return center.width >= 320 && center.height >= 300 && left.right <= center.left && center.right <= right.left;
    });
    assert.equal(modelSpace, true, 'side panels leave a full-height central character stage');
    assert.equal(await page.locator(".settlement-chat").count(), 1);
    assert.equal(await page.locator(".settlement-chat").isVisible(), true);
    assert.equal(await page.locator('[data-workspace-chat]').count(), 0);
    assert.equal(await page.locator('.settlement-chat').evaluate(chat => {
      const center = document.querySelector('main.center-content').getBoundingClientRect();
      const bounds = chat.getBoundingClientRect();
      return bounds.left >= center.left && bounds.right <= center.right
        && Math.abs((bounds.left + bounds.right) - (center.left + center.right)) < 2;
    }), true, 'chat is centered between the panels');
    await page.evaluate(() => {
      sessionStorage.setItem('fabelgeist.conversation-expanded', 'false');
      document.dispatchEvent(new Event('strategic-page-mounted'));
    });
    assert.equal(await page.locator('.settlement-chat').isVisible(), true, 'an old collapsed preference cannot hide chat');
    await page.locator('.settlement-chat-messages').evaluate(el => el.append(' Party message'));
    const name = await page.locator(".party-portrait-name").evaluate(el => ({ width: el.scrollWidth <= el.clientWidth + 1, height: el.scrollHeight <= el.clientHeight + 1 }));
    assert.deepEqual(name, { width: true, height: true });
    await page.route('**/quests', route => route.fulfill({ contentType: 'text/html', body:
      '<aside class="left-sidebar" data-journal-case-index><button data-journal-case-select="case">Case</button></aside><main class="center-content" data-journal-case-log>Journal menu</main><aside class="right-sidebar" data-journal-context>Context</aside>' }));
    await page.evaluate(() => {
      window.strategicFetch = (...args) => fetch(...args);
      document.querySelector('header').insertAdjacentHTML('beforeend', '<a href="/quests" data-journal-tab>Journal</a>');
    });
    await page.addScriptTag({ path: path.join(staticRoot, 'journal-tab.js') });
    await page.locator('[data-journal-tab]').click();
    await page.locator('[data-journal-case-log]').waitFor();
    assert.equal(await page.locator('.settlement-chat-messages').innerText(), 'Conversation content Party message');
    await page.locator('[data-journal-tab]').click();
    assert.equal(await page.locator('.settlement-chat-messages').innerText(), 'Conversation content Party message');
    await page.setViewportSize({ width: 390, height: 844 });
    await page.evaluate(() => scrollTo(0, document.body.scrollHeight));
    await page.waitForFunction(() => document.querySelector('[data-chat-dock]').getBoundingClientRect().right <= innerWidth);
    assert.equal(await page.locator('.settlement-chat').evaluate(chat => {
      const r = chat.getBoundingClientRect();
      return r.top >= 0 && r.bottom <= innerHeight;
    }), true, 'chat stays in view on a scrolling narrow menu');

  } finally { await browser.close(); }
});


test("portrait views select one tab across routes, nested treatments and remounts", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await openFixture(browser, `<div class="party-portrait-overlay">
      <div class="party-portrait" data-character-id="7"><nav class="portrait-tabs">
        <a data-portrait-tab="profile" href="/locations/settlement/goslar/party/7">Profile</a>
        <a data-portrait-tab="conversation" href="/locations/settlement/goslar/party/7/social">Conversation</a>
        <a data-portrait-tab="inventory" href="/locations/settlement/goslar/party/7/inventory">Inventory</a>
      </nav></div>
      <div class="party-portrait" data-character-id="8"><nav class="portrait-tabs">
        <a data-portrait-tab="profile" href="/locations/settlement/goslar/party/8/stats">Profile</a>
        <a data-portrait-tab="conversation" href="/locations/settlement/goslar/party/8/social">Conversation</a>
        <a data-portrait-tab="inventory" href="/locations/settlement/goslar/party/8/inventory">Inventory</a>
      </nav></div></div>`);
    for (const [suffix, id, tab] of [['7', '7', 'profile'], ['7/social', '7', 'conversation'],
      ['8/inventory', '8', 'inventory'], ['8/surgery/head', '8', 'profile'], ['8/stats', '8', 'profile']]) {
      await page.evaluate(suffix => {
        history.pushState({}, '', `/locations/settlement/goslar/party/${suffix}?building=inn`);
        document.dispatchEvent(new Event('strategic-page-mounted'));
      }, suffix);
      const selected = page.locator('[data-portrait-tab][aria-current="page"]');
      assert.equal(await selected.count(), 1);
      assert.equal(await selected.getAttribute('data-portrait-tab'), tab);
      assert.equal(await page.locator('.party-portrait.active').getAttribute('data-character-id'), id);
    }
    await page.evaluate(() => {
      history.pushState({}, '', '/locations/settlement/goslar/inn');
      document.dispatchEvent(new Event('strategic-page-mounted'));
    });
    assert.equal(await page.locator('[data-portrait-tab][aria-current="page"]').count(), 0);
    assert.equal(await page.locator('#strategic-page').getAttribute('data-character-view'), null);
  } finally { await browser.close(); }
});

test("live resident portraits use attached conversation tabs and mutually exclusive selection", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await openFixture(browser, `<nav class="scene-interactable-strip" data-npc-strip data-npc-settlement="goslar" data-npc-place="inn" data-npc-location="inn"></nav>
      <section class="settlement-chat" data-local-chat-subject="" data-dialogue-catalog-revision="1"><div class="settlement-chat-messages"></div></section>`);
    await page.evaluate(() => {
      window.strategicLocationUrls = { encode: encodeURIComponent };
      window.reportStrategicError = error => { throw error; };
      window.strategicFetch = async path => path.endsWith('/npcs')
        ? { ok: true, json: async () => [{ id: 'innkeeper', name: 'Anna', initials: 'A', is_default: true }, { id: 'guest', name: 'Benedikt', initials: 'B' }] }
        : new Promise(() => {});
    });
    await page.addScriptTag({ url: '/static/dialogue-client.js' });
    await page.waitForSelector('.resident-portrait.active');
    assert.equal(await page.locator('.resident-portrait .portrait-tab').count(), 2);
    assert.equal(await page.locator('.resident-portrait').first().evaluate(frame => {
      const portrait = frame.querySelector('.party-portrait-initial').getBoundingClientRect();
      const tab = frame.querySelector('.portrait-tab').getBoundingClientRect();
      return Math.abs((portrait.left + portrait.right) - (tab.left + tab.right)) < 2;
    }), true, 'a single conversation spoke aligns beneath the portrait');
    await page.locator('[data-resident-conversation]').nth(1).click();
    assert.equal(await page.locator('[data-resident-conversation][aria-pressed="true"]').count(), 1);
    assert.equal(await page.locator('.resident-portrait.active .settlement-npc-portrait').getAttribute('data-npc-id'), 'guest');
    await page.locator('[data-resident-conversation]').first().focus();
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('.resident-portrait.active .settlement-npc-portrait').getAttribute('data-npc-id'), 'innkeeper');
    assert.equal(await page.locator('.settlement-chat').getAttribute('data-local-chat-subject'), 'innkeeper');
    if (process.env.UX_CAPTURE_DIR) await page.screenshot({ path: path.join(process.env.UX_CAPTURE_DIR, 'resident-tabs-row-1280.png') });
  } finally { await browser.close(); }
});
