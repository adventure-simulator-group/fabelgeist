const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const staticRoot = path.join(__dirname, "..", "static");
const styles = ["base", "reset", "layout", "components", "strategic", "architecture", "utilities", "workspace", "readability", "legends"];
const skins = ["map-board", "civic-court", "merchant-hall", "hearth-room", "sanctuary", "ironbound-store", "domestic-cabinet"];

function fixture(skin, forge = false) {
  return `<!doctype html><html><head>${styles.map(name =>
    `<link rel="stylesheet" href="/static/css/${name}.css">`).join("")}</head><body>
    <div id="strategic-render-surface"><canvas id="game-canvas"></canvas></div>
    <div id="strategic-page" class="app" data-architectural-family="harz" data-place-skin="${skin}">
      <header class="top-bar settlement-top-bar"><nav class="settlement-services" aria-label="Places">
        ${skins.map(name => `<a href="#${name}" class="nav-tab"><span class="service-tab-label">${name}</span></a>`).join("")}
      </nav></header>
      <div class="main-grid">
        <aside class="left-sidebar"><section class="sidebar-section"><h3 class="sidebar-header">Settlement</h3><dl class="location-stat-list"><div><dt>Population</dt><dd>approximately 12,000</dd></div><div><dt>Also known as</dt><dd>Goselare, Gosilare, Goslaria, Goslarie, Gosler, Goslern, Gosseler, Gozlare</dd></div></dl><button class="btn">Rest</button></section></aside>
        <main class="center-content settlement-main">
          <div class="party-portrait-overlay"><a href="#party">Party inventory</a></div>
          <nav class="scene-interactable-strip"><button class="btn">Talk to the host</button></nav>
          <section class="npc-description-stage ${forge ? "forge-description-stage" : ""}"><h2>The host</h2><p>${"A long description that must remain readable. ".repeat(12)}</p></section>
          <section class="settlement-chat" aria-label="Settlement chat">
            <div class="settlement-chat-resize" role="separator" aria-label="Resize chat"><span></span></div>
            <div class="settlement-chat-layout"><div class="settlement-chat-conversation">
              <header class="conversation-dock-header">
                <div class="settlement-chat-filters" role="group" aria-label="Visible chat channels">
                  ${["Local", "Party", "Settlement", "DMs", "Guild", "Info"].map(label =>
                    `<label class="chat-channel-filter"><input type="checkbox" checked aria-label="${label}"><span>${label[0]}</span></label>`).join("")}
                </div>
                <div class="conversation-dock-tools">
                  <button type="button" class="affinity-face" aria-label="Reserved regard">&#x1f641;</button>
                  <div class="conversation-tabs" role="tablist" aria-label="Conversation topics">
                    ${["Quests", "Lore", "Recent Tidings", "Of Thee"].map((label, index) =>
                      `<button type="button" role="tab" class="conversation-tab" aria-selected="${index === 1}" aria-label="${label}">${label[0]}</button>`).join("")}
                  </div>
                </div>
              </header>
              <section class="dialogue-category-panel"><ul class="dialogue-category-topic-list"><li><a href="#profession">Profession</a></li></ul></section>
              <div class="settlement-chat-messages"><p data-chat-channel="local">A conversation remains readable in the fitted bay.</p></div>
              <div class="settlement-chat-composer"><div class="settlement-chat-input-shell"><input aria-label="Local message" placeholder="Message Goslar (Local)"></div><button class="btn btn-primary" aria-label="Send message">Send</button></div>
            </div></div>
          </section>
        </main>
        <aside class="right-sidebar"><section class="sidebar-section"><h3 class="sidebar-header">Inventory</h3><button class="btn">Inspect inventory</button></section></aside>
      </div>
    </div></body></html>`;
}

test("constructed bays reflow without clipping reading surfaces or keyboard controls", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const context = await browser.newContext({ hasTouch: true, reducedMotion: "reduce" });
    const page = await context.newPage();
    await page.route("http://architecture.test/**", route => {
      const url = new URL(route.request().url());
      if (!url.pathname.startsWith("/static/")) {
        return route.fulfill({ contentType: "text/html", body: fixture(url.searchParams.get("skin"), url.searchParams.has("forge")) });
      }
      const file = path.join(staticRoot, url.pathname.slice("/static/".length));
      return fs.existsSync(file) ? route.fulfill({ path: file }) : route.abort();
    });
    // 720 CSS pixels also exercises the reflow of a 1440px screen at 200% zoom.
    for (const [width, height] of [[1440, 900], [1172, 800], [1440, 600], [1024, 768], [390, 844], [320, 740], [720, 450]]) {
      await page.setViewportSize({ width, height });
      let trimHeight;
      for (const skin of skins) {
        await page.goto(`http://architecture.test/?skin=${skin}`);
        const geometry = await page.evaluate(() => {
          const description = document.querySelector(".npc-description-stage");
          const board = description.getBoundingClientRect();
          const bay = description.closest("main").getBoundingClientRect();
          return {
            withinBay: board.left >= bay.left && board.right <= bay.right,
            overflow: document.documentElement.scrollWidth > innerWidth,
            clipped: description.scrollHeight > description.clientHeight + 1,
            surface: getComputedStyle(description).backgroundColor,
          };
        });
        assert.equal(geometry.overflow, false, `${skin} at ${width}px overflows`);
        assert.equal(geometry.withinBay, true, `${skin} at ${width}px escapes its bay`);
        assert.equal(geometry.clipped, false, `${skin} at ${width}px clips the description`);
        assert.equal(geometry.surface, "rgb(25, 26, 22)");
        const trim = await page.locator("main").evaluate(el => getComputedStyle(el, "::before").height);
        trimHeight ??= trim;
        assert.equal(trim, trimHeight, "changing place keeps the trim scale stable");
        const factsFit = await page.locator(".location-stat-list").evaluate(el => {
          const board = el.closest(".sidebar-section").getBoundingClientRect();
          return [...el.querySelectorAll("dt,dd")].every(fact => {
            const rect = fact.getBoundingClientRect();
            return rect.left >= board.left && rect.right <= board.right && fact.scrollWidth <= fact.clientWidth + 1;
          });
        });
        assert.equal(factsFit, true, `settlement facts fit at ${width}px`);
        const scrolling = await page.locator("main").evaluate(el => ({
          overflows: el.scrollHeight > el.clientHeight,
          behavior: getComputedStyle(el).overflowY,
        }));
        if (scrolling.overflows) assert.equal(scrolling.behavior, "auto", "long descriptions leave the conversation reachable");
        const dialogue = await page.locator(".settlement-chat").evaluate(chat => {
          const messages = chat.querySelector(".settlement-chat-messages").getBoundingClientRect();
          const composer = chat.querySelector(".settlement-chat-composer").getBoundingClientRect();
          const input = chat.querySelector(".settlement-chat-composer input").getBoundingClientRect();
          const panel = chat.querySelector(".dialogue-category-panel").getBoundingClientRect();
          const tabs = chat.querySelector(".conversation-tabs");
          const chatBounds = chat.getBoundingClientRect();
          const overlaps = (one, two) => one.left < two.right && one.right > two.left
            && one.top < two.bottom && one.bottom > two.top;
          return {
            messagesWidth: messages.width,
            composerWidth: composer.width,
            inputWidth: input.width,
            topicsVisible: getComputedStyle(tabs).display !== "none"
              && [...tabs.querySelectorAll("[role=tab]")].every(tab => tab.getBoundingClientRect().width > 0),
            panelVisible: panel.width > 0 && panel.height >= 44,
            panelClipped: panel.scrollHeight > panel.clientHeight + 1,
            regionsOverlap: overlaps(messages, panel) || overlaps(composer, panel),
            contained: [messages, composer, panel].every(rect => rect.left >= chatBounds.left && rect.right <= chatBounds.right),
          };
        });
        assert.ok(dialogue.messagesWidth >= 160, `message stream is ${dialogue.messagesWidth}px wide at ${width}px`);
        assert.ok(dialogue.composerWidth >= 160, `message composer is ${dialogue.composerWidth}px wide at ${width}px`);
        assert.ok(dialogue.inputWidth >= 80, `message input is ${dialogue.inputWidth}px wide at ${width}px`);
        assert.equal(dialogue.topicsVisible, true, `topic controls remain available at ${width}px`);
        assert.equal(dialogue.panelVisible, true, `active topic remains available at ${width}px`);
        assert.equal(dialogue.panelClipped, false, `active topic is not buried at ${width}px`);
        assert.equal(dialogue.regionsOverlap, false, `dialogue regions do not overlap at ${width}px`);
        assert.equal(dialogue.contained, true, `dialogue regions stay inside the chat at ${width}px`);
        await page.getByRole("button", { name: "Rest", exact: true }).focus();
        const outline = await page.locator(":focus").evaluate(el => getComputedStyle(el).outlineWidth);
        assert.equal(outline, "3px");
        if (width <= 1100) {
          const height = await page.getByRole("button", { name: "Rest", exact: true }).evaluate(el => el.getBoundingClientRect().height);
          assert.ok(height >= 44, "touch controls retain their minimum height");
        }
      }
    }
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("http://architecture.test/?skin=merchant-hall");
    const originalRail = await page.locator(".left-sidebar").evaluate(sidebar => ({
      paddingInlineStart: getComputedStyle(sidebar).paddingInlineStart,
      sectionPadding: getComputedStyle(sidebar.querySelector(".sidebar-section")).padding,
    }));
    await page.evaluate(() => {
      document.querySelector(".left-sidebar").innerHTML = `<section class="sidebar-section"><div class="inventory-browser" data-inventory-browser="left"><div class="inventory-browser-table-frame"><table class="trade-inventory-table"><tbody><tr><td style="min-width:16rem">Bandage</td><td style="min-width:4rem">3 coin</td></tr></tbody></table></div></div></section>`;
    });
    await page.addScriptTag({ path: path.join(staticRoot, "inventory-browser.js") });
    const inventoryRail = await page.locator(".left-sidebar").evaluate(sidebar => ({
      paddingInlineStart: getComputedStyle(sidebar).paddingInlineStart,
      sectionPadding: getComputedStyle(sidebar.querySelector(".sidebar-section")).padding,
    }));
    assert.deepEqual(inventoryRail, originalRail, "inventory content keeps the shared rail inset");
    const measure = () => page.evaluate(() => {
      window.strategicInventoryBrowser.syncPanelWidth(document.querySelector(".inventory-browser"));
      return parseFloat(document.querySelector(".main-grid").style.getPropertyValue("--inventory-left-width"));
    });
    const initialWidth = await measure();
    await page.evaluate(() => {
      const cell = document.createElement("td");
      cell.style.minWidth = "8rem";
      cell.textContent = "Additional detail";
      document.querySelector("tr").append(cell);
    });
    assert.ok(await measure() > initialWidth, "additional columns grow the rail");
    await page.evaluate(() => document.querySelector("tr td:last-child").remove());
    const restoredWidth = await measure();
    assert.ok(
      Math.abs(restoredWidth - initialWidth) < 4,
      `removing columns restores the rail from ${restoredWidth}px to ${initialWidth}px`,
    );
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.waitForFunction(() => !document.querySelector(".main-grid").style.getPropertyValue("--inventory-left-width"));
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.waitForFunction(() => !!document.querySelector(".main-grid").style.getPropertyValue("--inventory-left-width"));
    const widenedWidth = await measure();
    assert.ok(
      Math.abs(widenedWidth - initialWidth) < 4,
      `widening a stacked page restores the rail from ${widenedWidth}px to ${initialWidth}px`,
    );
    await page.evaluate(() => {
      const sidebar = document.querySelector(".left-sidebar");
      sidebar.classList.add("smith-wares-column");
      sidebar.innerHTML = `<div class="merchant-stock-stack"><div class="merchant-stock-area"><section class="sidebar-section"><h3 class="sidebar-header">Stock</h3><div class="smith-wares-scroll">${Array(60).fill('<p>Bandage — 3 coin</p>').join("")}</div></section></div><button>Trade</button></div>`;
    });
    const stock = await page.locator(".smith-wares-scroll").evaluate(el => ({
      scrollable: el.scrollHeight > el.clientHeight,
      withinRail: el.getBoundingClientRect().bottom <= el.closest("aside").getBoundingClientRect().bottom,
    }));
    assert.deepEqual(stock, { scrollable: true, withinRail: true });
    await page.goto("http://architecture.test/?skin=workshop&forge");
    const opening = await page.evaluate(() => ["#strategic-page", ".main-grid", ".settlement-main"].map(selector => {
      const style = getComputedStyle(document.querySelector(selector));
      return [style.backgroundColor, style.backgroundImage];
    }));
    assert.deepEqual(opening, Array(3).fill(["rgba(0, 0, 0, 0)", "none"]));
  } finally {
    await browser.close();
  }
});
