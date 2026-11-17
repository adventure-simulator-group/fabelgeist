const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const origin = "http://live-dialogue.test";
const staticRoot = path.resolve(__dirname, "../static");
const tacticalRoot = path.resolve(__dirname, "../../adventuresim-stdb-module/static");
const resident = "18446744073709551614";
const session = "dialogue:1:scene:11:1";
const panel = `<div class="side-panel" data-side-panel="left" data-panel-context="storefront-a">
  <nav role="tablist" class="side-panel-tabs"><button type="button" role="tab" id="trade-tab" data-panel-tab="trade" aria-controls="trade-pane" aria-selected="true">Trade</button>
  <button type="button" role="tab" id="services-tab" data-panel-tab="services" aria-controls="services-pane" aria-selected="false">Services</button></nav>
  <section id="trade-pane" data-panel-pane="trade">Old shop</section><section id="services-pane" data-panel-pane="services" hidden>Services</section></div>`;
const chat = `<section class="settlement-chat" data-dialogue-chat data-dialogue-catalog-revision="catalog" data-local-chat-kind="npc" data-local-chat-subject="old-npc" data-local-chat-location="market">
  <div class="settlement-chat-layout"><div class="settlement-chat-conversation">
  <header class="conversation-dock-header"><label><input type="checkbox" checked data-chat-filter="local">Local</label></header>
  <section data-dialogue-category-panel="lore"></section>
  <div class="settlement-chat-messages" aria-live="polite"><div data-chat-channel="local">Old conversation</div></div>
  <div class="settlement-chat-composer"><input aria-label="Response"><button type="button">Send</button></div>
  </div></div></section>`;

function transcript(id = session, revision = 1, name = "Resident") {
  return { session_id: id, revision, participants: [{ actor_id: resident, display_name: name }],
    catalog_revision: "catalog", open_prompt: null, order_errantry_offer: false,
    topics: [{ id: "news", label: "News", category: "lore" }],
    events: [{ speaker_name: name, speaker_is_player: false, fragments: [
      { fragment: { kind: "text", value: `${name} revision ${revision}. ` } },
      { fragment: { kind: "topic", topic: "news", label: "News" } },
    ] }],
  };
}

async function open(page, read = async route => route.fulfill({ json: transcript() })) {
  const errors = [], requests = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.route(`${origin}/**`, async route => {
    const request = route.request(), url = new URL(request.url());
    if (url.pathname.startsWith("/static/")) return route.fulfill({ path: path.join(staticRoot, url.pathname.slice(8)) });
    if (url.pathname.startsWith("/tactical/")) return route.fulfill({ path: path.join(tacticalRoot, url.pathname.slice(10)) });
    if (url.pathname.startsWith("/api/")) {
      requests.push({ path: url.pathname, method: request.method() });
      if (url.pathname.startsWith("/api/dialogue/")) return read(route);
      return route.fulfill({ json: url.pathname === "/api/local-chat/incoming" ? [] : { messages: [] } });
    }
    return route.fulfill({ contentType: "text/html", body: `<!doctype html><html><head>
      ${["base", "reset", "layout", "components", "strategic", "architecture", "utilities", "workspace", "side-panels", "portraits", "chat-dock", "readability", "legends", "stats", "strategic-scene"].map(name => `<link rel="stylesheet" href="/static/css/${name}.css">`).join("")}
      </head><body data-tactical-active><div id="strategic-render-surface"><canvas id="game-canvas" tabindex="0"></canvas></div>
      <div id="strategic-page" class="app" data-strategic-workspace><div class="main-grid"><aside class="left-sidebar">${panel}</aside><main class="center-content"></main><aside class="right-sidebar"></aside></div><div data-chat-dock>${chat}</div></div>
      <script>window.commands=[]; window.reported=[]; window.ticks=0; window.originalCanvas=document.querySelector('canvas');
        setInterval(()=>ticks++,16); window.strategicFetch=(...args)=>fetch(...args); window.strategicBackgroundFetch=(key,...args)=>fetch(...args);
        window.queueStrategicInitialLoad=fn=>Promise.resolve().then(fn); window.reportStrategicError=e=>reported.push(e.message);</script>
      <script src="/tactical/tactical-input.js"></script><script src="/static/chat-dock.js"></script><script src="/static/side-panels.js"></script>
      <script src="/static/dialogue-client.js"></script><script src="/static/local-chat.js"></script>
      <script type="module">import {installLiveDialogue} from '/static/live-dialogue.js';
        import {installLiveDialoguePanel} from '/static/live-dialogue-panel.js';
        import {installTacticalHud} from '/tactical/tactical-hud.js';
        installLiveDialogue(payload=>commands.push(payload)); installLiveDialoguePanel();
        installTacticalHud({container:document.querySelector('#strategic-render-surface'),command:payload=>commands.push(payload)});
        window.ready=true;</script></body></html>` });
  });
  await page.goto(origin);
  await page.waitForFunction(() => window.ready);
  await page.bringToFront();
  return { errors, requests };
}

const notice = (page, detail) => page.evaluate(detail => hud_view("dialogue", JSON.stringify(detail)), detail);
const begin = (page, id = session) => notice(page, { type: "opened", session_id: id, resident_id: resident });

for (const width of [1280, 600]) test(`confirmed live conversation keeps the scene and isolates input at ${width}px`, async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width, height: 800 } });
    let nextView = transcript();
    const { errors, requests } = await open(page, route => route.fulfill({ json: nextView }));
    assert.equal(await page.locator("[data-panel-tab='dialogue']").count(), 0);
    assert.equal(requests.some(request => request.method === "POST"), false);
    await begin(page);
    await page.getByText("Resident revision 1.", { exact: false }).waitFor();
    const closeBounds = await page.getByRole("button", { name: "End conversation" }).boundingBox();
    assert.ok(closeBounds.height < 60, "The close control must leave the panel's height for dialogue");
    if (process.env.DIALOGUE_REVIEW_DIR) {
      fs.mkdirSync(process.env.DIALOGUE_REVIEW_DIR, { recursive: true });
      await page.screenshot({ path: path.join(process.env.DIALOGUE_REVIEW_DIR, `conversation-${width}.png`) });
    }
    assert.equal(await page.locator("[data-local-chat-subject]").getAttribute("data-local-chat-subject"), resident);
    assert.equal(await page.getByRole("tab", { name: "Trade", exact: true }).count(), 0);
    const ticks = await page.evaluate(() => ticks);
    await page.getByRole("tab", { name: "Conversation", exact: true }).press("ArrowRight");
    assert.equal(await page.getByRole("tab", { name: "Conversation", exact: true }).evaluate(el => el === document.activeElement), true);
    await page.getByRole("textbox", { name: "Response" }).fill("Unported free text");
    await page.getByRole("textbox", { name: "Response" }).press("Enter");
    await page.locator("[data-dialogue-topic]").last().click();
    const command = await page.evaluate(() => commands.at(-1));
    assert.equal(command.type, "dialogue");
    assert.deepEqual(command.request.action.Topic.topic_id, "news");
    assert.equal(command.request.session_id, session);
    const prompt = transcript(session, 2);
    prompt.open_prompt = { id: `${session}:prompt:0`, mode: "YesNo", min_choices: 1, max_choices: 1, choices: [{ id: "yes", label: "Yes" }, { id: "no", label: "No" }] };
    nextView = prompt;
    await notice(page, { type: "action", session_id: session, action_id: command.request.action.Topic.action_id, result: { Ok: null } });
    await page.getByRole("button", { name: "Yes", exact: true }).click();
    const answer = await page.evaluate(() => commands.at(-1).request.action.Answer);
    assert.deepEqual(answer.choice_ids, ["yes"]);
    assert.equal(answer.prompt_row_id, `${session}:prompt:0`);
    assert.equal(await page.evaluate(() => tacticalUiOwnsInput()), true);
    await page.waitForFunction(previous => ticks > previous, ticks);
    assert.equal(await page.evaluate(() => originalCanvas === document.querySelector('#game-canvas')), true);
    await page.getByRole("button", { name: "End conversation" }).click();
    assert.equal(await page.evaluate(() => commands.at(-1).request.action), "Close");
    assert.equal(await page.locator("[data-panel-tab='trade']").getAttribute("aria-selected"), "true");
    assert.equal(await page.evaluate(() => document.activeElement === originalCanvas && document.body.hasAttribute('data-tactical-active')), true);
    assert.equal(requests.some(request => request.method === "POST"), false);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("late transcript reads and receipts cannot reopen a closed or replaced conversation", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    let delayed;
    const other = "dialogue:1:scene:11:2";
    const { errors } = await open(page, route => {
      if (decodeURIComponent(new URL(route.request().url()).pathname).endsWith(session)) { delayed = route; return; }
      return route.fulfill({ json: transcript(other, 1, "Other resident") });
    });
    const requested = page.waitForRequest(request => request.url().includes(encodeURIComponent(session)));
    await begin(page);
    await requested;
    await begin(page, other);
    await page.getByText("Other resident revision 1.", { exact: false }).waitFor();
    await delayed.fulfill({ json: transcript() });
    await notice(page, { type: "closed", session_id: session });
    assert.equal(await page.locator("[data-panel-tab='dialogue']").isVisible(), true);
    assert.equal(await page.locator(".settlement-chat-messages").textContent(), "[--:--] Other resident: Other resident revision 1. News");
    await page.getByRole("tab", { name: "Conversation" }).press("Escape");
    await notice(page, { type: "action", session_id: other, action_id: "late", result: { Ok: null } });
    assert.equal(await page.locator("[data-panel-tab='dialogue']").count(), 0);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("sidebar and whole-page refreshes reattach the current confirmed transcript", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const initialView = transcript();
    initialView.open_prompt = { id: `${session}:prompt:0`, mode: "Single", min_choices: 1, max_choices: 1, choices: [{ id: "yes", label: "Yes" }, { id: "no", label: "No" }] };
    const { errors } = await open(page, route => route.fulfill({ json: initialView }));
    await begin(page);
    await page.getByText("Resident revision 1.", { exact: false }).waitFor();
    await page.getByLabel('Yes', { exact: true }).check();
    await page.evaluate(async () => {
      document.dispatchEvent(new CustomEvent('strategic-live-regions-refreshed', {detail:{regions:['right-sidebar']}}));
      await Promise.resolve();
    });
    assert.equal(await page.getByLabel('Yes', { exact: true }).isChecked(), true);
    await page.evaluate(html => {
      document.querySelector('.left-sidebar').innerHTML = html;
      document.dispatchEvent(new CustomEvent('strategic-live-regions-refreshed', {detail:{regions:['left-sidebar']}}));
    }, panel);
    await page.getByText("Resident revision 1.", { exact: false }).waitFor();
    assert.equal(await page.getByLabel('Yes', { exact: true }).isChecked(), true);
    await page.evaluate(({ panel, chat }) => {
      document.dispatchEvent(new Event('strategic-page-unmounting'));
      document.querySelector('.left-sidebar').innerHTML = panel;
      document.querySelector('[data-chat-dock]').innerHTML = chat;
      document.dispatchEvent(new Event('strategic-page-mounted'));
    }, { panel, chat });
    await page.getByText("Resident revision 1.", { exact: false }).waitFor();
    assert.equal(await page.locator("[data-panel-tab='dialogue']").count(), 1);
    assert.equal(await page.evaluate(() => originalCanvas === document.querySelector('#game-canvas')), true);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("a settlement without venue tabs opens the confirmed conversation and closes during its pending read", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    let delayed;
    const { errors, requests } = await open(page, route => { delayed = route; });
    await page.evaluate(() => {
      document.querySelector('.left-sidebar').innerHTML = '<p data-settlement-summary>Settlement</p>';
      const chat = document.querySelector('[data-dialogue-chat]');
      chat.removeAttribute('data-local-chat-subject');
      chat.removeAttribute('data-dialogue-catalog-revision');
      document.dispatchEvent(new Event('strategic-page-mounted'));
    });
    const requested = page.waitForRequest(request => request.url().includes(encodeURIComponent(session)));
    await begin(page);
    await requested;
    await page.getByRole('button', { name: 'End conversation' }).click();
    await delayed.fulfill({ json: transcript() });
    await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 30)));
    assert.equal(await page.locator('[data-side-panel]').count(), 0);
    assert.equal(await page.locator('[data-settlement-summary]').isVisible(), false); // The scene still owns tactical presentation.
    assert.equal(await page.locator('[data-chat-dock] [data-dialogue-chat]').count(), 1);
    assert.equal(await page.locator('.settlement-chat-messages').textContent(), '');
    assert.equal(requests.some(request => request.method === 'POST'), false);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("a party social dock retains the shared conversation renderer for native contact", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const { errors } = await open(page);
    await page.evaluate(() => {
      document.querySelector('main.center-content').innerHTML = '<section class="settlement-chat" data-social-conversation aria-label="Party conversation">Party conversation</section>';
      document.dispatchEvent(new Event('strategic-page-mounted'));
    });
    assert.equal(await page.locator('[data-dialogue-chat]').count(), 0);
    await begin(page);
    await page.getByText('Resident revision 1.', { exact: false }).waitFor();
    await page.getByLabel('Local', { exact: true }).uncheck();
    assert.equal(await page.locator('[data-dialogue-scripted]').isVisible(), false);
    await page.getByLabel('Local', { exact: true }).check();
    assert.equal(await page.locator('[data-dialogue-scripted]').isVisible(), true);
    await page.getByRole('button', { name: 'End conversation' }).click();
    assert.equal(await page.locator('[data-chat-dock] [data-dialogue-chat]').isHidden(), true);
    assert.equal(await page.locator('[data-chat-dock] [data-social-conversation]').textContent(), 'Party conversation');
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("closing after sidebar replacement restores the tab selected before conversation", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const { errors } = await open(page);
    await page.getByRole('tab', { name: 'Services', exact: true }).click();
    await begin(page);
    await page.getByText('Resident revision 1.', { exact: false }).waitFor();
    await page.evaluate(html => {
      document.querySelector('.left-sidebar').innerHTML = html;
      document.dispatchEvent(new CustomEvent('strategic-live-regions-refreshed', {detail:{regions:['left-sidebar']}}));
    }, panel);
    await page.getByRole('button', { name: 'End conversation' }).click();
    assert.equal(await page.getByRole('tab', { name: 'Services', exact: true }).getAttribute('aria-selected'), 'true');
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

test("an open character dialog keeps Escape ownership ahead of conversation", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const { errors } = await open(page);
    await begin(page);
    await page.getByText('Resident revision 1.', { exact: false }).waitFor();
    await page.evaluate(() => {
      const modal = document.createElement('section');
      modal.setAttribute('role', 'dialog');
      modal.setAttribute('aria-label', 'Character inspection');
      modal.innerHTML = '<button type="button">Close inspection</button>';
      modal.style.cssText = 'position:fixed;top:0;right:0;z-index:5000;background:#181713';
      document.body.append(modal);
      modal.addEventListener('keydown', event => {
        if (event.key === 'Escape') { event.preventDefault(); modal.remove(); }
      });
      modal.querySelector('button').focus();
    });
    await page.getByRole('button', { name: 'Close inspection' }).press('Escape');
    assert.equal(await page.getByRole('dialog').count(), 0);
    assert.equal(await page.getByRole('tab', { name: 'Conversation' }).isVisible(), true);
    assert.equal(await page.evaluate(() => commands.some(command => command.request?.action === 'Close')), false);
    await page.getByRole('tab', { name: 'Conversation' }).press('Escape');
    assert.equal(await page.locator('[data-panel-tab="dialogue"]').count(), 0);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
