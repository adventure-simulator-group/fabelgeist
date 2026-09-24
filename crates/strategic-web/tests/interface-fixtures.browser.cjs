const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const directory = process.env.UX_CAPTURE_DIR;
const staticRoot = path.join(__dirname, "../static");
const scripts = new Set([
  "tooltips.js", "chat-dock.js", "chat-resize.js", "character-action-dialog.js", "portrait-navigation.js", "action-previews.js",
  "inventory-browser.js", "party-trade.js", "cooking.js", "physiology-dialog.js", "equipment-toggle.js",
  "numeric-editor.js", "stats-panels.js",
]);

test("rendered interface fixtures fit desktop and narrow viewports", { skip: !directory }, async () => {
  const browser = await chromium.launch({ headless: true });
  const findings = [];
  try {
    const page = await browser.newPage({ reducedMotion: "reduce" });
    await page.route("**/*", route => {
      const url = new URL(route.request().url());
      if (url.hostname !== "review.test") return route.abort();
      if (url.pathname.startsWith("/static/")) {
        const file = path.join(staticRoot, url.pathname.slice(8));
        return fs.existsSync(file) ? route.fulfill({ path: file }) : route.abort();
      }
      const fixtureName = url.pathname === "/locations/settlement/goslar/party/8/inventory" ? "inventory" : url.pathname.slice(1);
      const file = path.join(directory, `${fixtureName}.html`);
      if (!fs.existsSync(file)) return route.fulfill({ status: 404, body: "Fixture unavailable" });
      const body = fs.readFileSync(file, "utf8").replace(/<script\b[^>]*\bsrc="([^"]+)"[^>]*>[\s\S]*?<\/script>/g,
        (tag, src) => scripts.has(path.posix.basename(src.split("?")[0])) ? tag : "");
      return route.fulfill({ contentType: "text/html; charset=utf-8", body });
    });
    for (const width of [1280, 390]) {
      await page.setViewportSize({ width, height: width === 1280 ? 720 : 844 });
      for (const name of ["life-stage", "candidates", "roster", "inventory", "surgery", "notebook", "cooking", "recruitment", "rest", "injured-stats", "examined-stats"]) {
        await page.goto(`http://review.test/${name === "inventory" ? "locations/settlement/goslar/party/8/inventory?building=inn" : name}`);
        await page.evaluate(() => document.fonts.ready);
        await page.mouse.move(0, 0);
        if (name === "notebook") await page.locator("#review-notebook").evaluate(dialog => dialog.showModal());
        if (name === "recruitment") await page.locator('[data-recruitment-dialog]').evaluate(dialog => dialog.showModal());
        await page.screenshot({ path: path.join(directory, `${name}-${width}.png`), fullPage: true });
        if (name === "candidates") {
          assert.equal(await page.locator('.summary-rank-value, .skill-rank-value').count(), 0);
          const tooltip = page.locator('#strategic-tooltip');
          for (const selector of ['.character-summary-icon', '.skill-rank-bar', '.attribute-rank-bar']) {
            const stat = page.locator(selector).first();
            const reading = await stat.getAttribute('aria-valuenow');
            await stat.hover();
            assert.equal(await tooltip.isVisible(), true, `${selector} reveals its reading on hover`);
            assert.match(await tooltip.innerText(), reading ? new RegExp(`${reading} out of 5`) : /\d\.\d/);
            await stat.click();
            await page.mouse.move(640, 10);
            assert.equal(await tooltip.isVisible(), true, `${selector} click keeps the reading open`);
            if (selector === '.skill-rank-bar' && width === 1280) {
              await page.screenshot({ path: path.join(directory, 'stat-reading-1280.png') });
            }
            await page.keyboard.press('Escape');
            assert.equal(await tooltip.isVisible(), false);
            await stat.press('Enter');
            assert.equal(await tooltip.isVisible(), true, `${selector} supports keyboard inspection`);
            await page.keyboard.press('Escape');
            // Touch starts by suppressing the focus tooltip; the tap must still pin it.
            await stat.dispatchEvent('pointerdown', { pointerType: 'touch' });
            await stat.dispatchEvent('click', { detail: 1 });
            assert.equal(await tooltip.isVisible(), true, `${selector} reveals a reading on tap`);
            await page.keyboard.press('Escape');
          }
          // Clicking a different stat switches directly to its reading.
          await page.locator('.character-summary-icon').first().click();
          await page.locator('.skill-rank-bar').first().click();
          assert.match(await tooltip.innerText(), /out of 5 usable rank/);
          await page.keyboard.press('Escape');
        }
        if (name === 'inventory') {
          assert.equal(await page.locator('[data-chat-dock] .settlement-chat').count(), 1);
          assert.equal(await page.locator('[data-chat-dock] .settlement-chat').isVisible(), true);
          const resize = page.locator('[data-chat-dock] .settlement-chat-resize');
          const chatHeight = Number(await resize.getAttribute('aria-valuenow'));
          await resize.press('ArrowUp');
          assert.equal(Number(await resize.getAttribute('aria-valuenow')), chatHeight + 24);
          await resize.press('ArrowDown');
          if (width === 1280) assert.equal(await page.evaluate(() => {
            const left = document.querySelector('.left-sidebar').getBoundingClientRect();
            const right = document.querySelector('.right-sidebar').getBoundingClientRect();
            const chat = document.querySelector('[data-chat-dock]').getBoundingClientRect();
            return chat.left >= left.right && chat.right <= right.left
              && document.querySelector('.right-sidebar .encumbrance').getBoundingClientRect().bottom > chat.top;
          }), true, 'chat is between full-height inventory panels');
          if (width === 390) await page.screenshot({ path: path.join(directory, 'chat-viewport-390.png') });
          for (const side of ['left', 'right']) {
            const frame = page.locator(`.${side}-sidebar .inventory-browser-table-frame`).first();
            for (const end of [true, false]) {
              await frame.evaluate((el, end) => { el.scrollLeft = end ? el.scrollWidth : 0; }, end);
              const columns = await frame.evaluate((el, side) => {
                const viewport = el.getBoundingClientRect();
                return [...el.querySelectorAll('tbody .trade-inventory-row:not([hidden]) .trade-transfer')].map(button => {
                  const r = button.getBoundingClientRect();
                  const cell = button.closest('.inventory-actions-cell');
                  const c = cell.getBoundingClientRect();
                  return {
                    x: r.x,
                    contained: r.left >= viewport.left && r.right <= viewport.right,
                    inner: Math.abs(side === 'left' ? c.right - viewport.right : c.left - viewport.left) < 2,
                    edgeOffset: side === 'left' ? c.right - viewport.right : c.left - viewport.left,
                    reachable: r.bottom > (button.closest('.encumbrance-inventory-scroll')?.getBoundingClientRect().bottom ?? innerHeight) || button.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)),
                    outsideName: !button.closest('.inventory-item-name'),
                  };
                });
              }, side);
              assert.ok(columns.length > 0 && columns.every(c => c.contained && c.inner && c.outsideName), `${side} at ${width}, scrolled=${end}: ${JSON.stringify(columns)}`);
              assert.ok(columns.every(c => Math.abs(c.x - columns[0].x) < 1), 'transfer controls align in one column');
              if (width === 1280) assert.ok(columns.every(c => c.reachable), 'pinned controls remain clickable after horizontal scrolling');
            }
          }
          assert.equal(await page.locator('.workspace-bar').count(), 0);
          const selected = page.locator('[data-portrait-tab][aria-current="page"]');
          assert.equal(await selected.count(), 1);
          assert.equal(await selected.getAttribute('data-portrait-tab'), 'inventory');
          assert.equal(await selected.evaluate(tab => {
            const box = tab.getBoundingClientRect();
            const rail = tab.closest('.party-portrait-overlay').getBoundingClientRect();
            return box.left >= rail.left && box.right <= rail.right;
          }), true, 'the selected view stays visible when the portrait row scrolls');
          assert.equal(await selected.locator('..').locator('..').getAttribute('data-character-id'), '8');
          const controls = await page.locator('.portrait-tab').evaluateAll(tabs => tabs.map(tab => {
            const rect = tab.getBoundingClientRect();
            const face = tab.closest('.party-portrait').querySelector('.party-portrait-initial').getBoundingClientRect();
            const style = getComputedStyle(tab);
            const icon = tab.querySelector('.game-icon, .party-action-icon').getBoundingClientRect();
            const rail = tab.closest('.party-portrait-overlay').getBoundingClientRect();
            const x = icon.left + icon.width / 2, y = icon.top + icon.height / 2;
            const reachable = x < rail.left || x > rail.right || tab.contains(document.elementFromPoint(x, y));
            return reachable && icon.top >= face.bottom - 2 && rect.width >= 24 && rect.height >= 32 && style.opacity === '1' && style.pointerEvents !== 'none';
          }));
          assert.ok(controls.length >= 6 && controls.every(Boolean), 'all view spokes expose upright icons below portraits without hover');
          assert.equal(await page.locator('.party-portrait form').count(), 0, 'membership actions do not masquerade as tabs');
        }
        if (name === "inventory" && width === 1280) {
          const countVisible = await page.locator('.inventory-count').first().evaluate(el => {
            const r = el.getBoundingClientRect();
            return el.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2));
          });
          assert.equal(countVisible, true, 'inventory values stay above the footer and unclipped');
        }
        if (name === "candidates" && width === 1280) {
          const rankColors = await page.locator('.character-summary-icons').evaluate(summary => {
            const colors = [...summary.querySelectorAll('.character-summary-icon')].map(icon => {
              const tier = [...icon.classList].find(name => name.startsWith('skill-rank-tier-')).split('-').pop();
              const segment = document.querySelector(`.rank-color-swatch[data-rank="${tier}"]`);
              return { icon: getComputedStyle(icon).color, segment: getComputedStyle(segment).backgroundColor };
            });
            return { matched: colors.every(color => color.icon === color.segment), distinct: new Set(colors.map(color => color.icon)).size };
          });
          assert.equal(rankColors.matched, true, 'summary icons and skill bars teach the same rank colors');
          assert.ok(rankColors.distinct > 1, 'different ranked summary capabilities retain different colors');
          const zeroColor = await page.evaluate(() => {
            const condition = document.createElement('span');
            condition.className = 'condition-green';
            document.body.append(condition);
            const expected = getComputedStyle(condition).backgroundColor;
            const zero = getComputedStyle(document.querySelector('.party-skill-icon-cell.skill-rank-tier-0 .stat-icon')).backgroundColor;
            const key = [...document.querySelectorAll('.rank-color-swatch')].map(el => getComputedStyle(el).backgroundColor);
            condition.remove();
            return { matched: expected === zero && key[0] === expected, count: new Set(key).size };
          });
          assert.deepEqual(zeroColor, { matched: true, count: 6 }, 'rank zero shares item-condition green within a six-color key');
          const attributes = page.locator('[data-stats-labels=attributes]');
          const skills = page.locator('[data-stats-labels=skills]');
          assert.equal(await attributes.isChecked(), false);
          const eyesight = page.locator('.party-attribute-row:has(.stat-icon-eyesight) .party-attribute-name');
          assert.equal(await eyesight.isVisible(), false);
          assert.equal(await page.locator('.region-health-label').first().isVisible(), false);
          await attributes.focus();
          await page.keyboard.press('Space');
          assert.equal(await eyesight.isVisible(), true);
          assert.equal(await skills.isChecked(), false, 'panel preferences are independent');
          await page.reload();
          assert.equal(await attributes.isChecked(), true, 'labels survive navigation');
          await page.evaluate(() => document.dispatchEvent(new Event('strategic-page-mounted')));
          await skills.check();
          const summaryLabels = await page.locator('.summary-readable-label').allTextContents();
          assert.ok(summaryLabels.every(label => !/\d/.test(label)), 'Show labels adds names without exact ranks');
          await page.screenshot({ path: path.join(directory, 'candidate-labels-1280.png') });
          const skillKey = page.locator('.stats-key').nth(1);
          await skillKey.locator('summary').focus();
          await page.keyboard.press('Enter');
          assert.equal(await skillKey.locator('.rank-scale-key').isVisible(), true);
          await skillKey.scrollIntoViewIfNeeded();
          await page.screenshot({ path: path.join(directory, 'skill-key-1280.png') });
          await page.keyboard.press('Enter');
          const key = page.locator('.stats-key').first();
          await key.locator('summary').focus();
          await page.keyboard.press('Enter');
          assert.equal(await key.getAttribute('open'), '');
          await page.screenshot({ path: path.join(directory, 'candidate-key-1280.png') });
          await page.evaluate(() => {
            Object.defineProperty(window, 'localStorage', { configurable: true, get() { throw new Error('Storage blocked'); } });
            document.dispatchEvent(new Event('strategic-page-mounted'));
          });
          await attributes.uncheck();
          assert.equal(await eyesight.isVisible(), false, 'the control still works with storage blocked');
          await attributes.check();
          assert.equal(await eyesight.isVisible(), true);
          await page.reload();

        }
        if (name === 'injured-stats') {
          const impaired = page.locator('.region-reading[data-impaired=true]').first();
          assert.equal(await impaired.locator('.region-health-symbol').textContent(), '+');
          const treatment = impaired.locator('..').locator('.limb-surgery-button');
          assert.equal(await treatment.isVisible(), false);
          assert.equal(await impaired.locator('p').isVisible(), false);
          await impaired.locator('summary').focus();
          await page.keyboard.press('Enter');
          assert.equal(await impaired.locator('p').isVisible(), true);
          assert.equal(await treatment.isVisible(), true);
          assert.match(await impaired.locator('p').textContent(), /other impairment/);
          assert.doesNotMatch(await impaired.locator('p').textContent(), /sanguine|phlegmatic/);
          await page.screenshot({ path: path.join(directory, `injured-reading-${width}.png`), fullPage: true });
        }
        if (name.endsWith('-stats') || name === 'candidates') {
          const health = await page.locator('.region-reading').evaluateAll(regions => regions.map(region => {
            const row = region.querySelector('summary');
            const bar = region.querySelector('.attribute-health-bar');
            const bounds = row.getBoundingClientRect();
            const meter = bar.getBoundingClientRect();
            const panel = region.closest('.attribute-group').getBoundingClientRect();
            return { fullRow: Math.abs(bounds.width - panel.width) < 2,
              available: bounds.width - meter.width < 55, height: meter.height,
              icons: row.querySelectorAll('.region-body-icon, .region-health-symbol').length };
          }));
          const pairs = await page.locator('.limb-attribute-pair').evaluateAll(pairs => pairs.map(pair => {
            const [left, right] = [...pair.children].map(limb => limb.getBoundingClientRect());
            return Math.abs(left.top - right.top) < 2 && left.right <= right.left;
          }));
          assert.deepEqual(pairs, [true, true], 'arms and legs stay side by side at both widths');
          assert.equal(health.length, 7);
          assert.ok(health.every(row => row.fullRow && row.available && row.height >= 12 && row.icons === 2),
            'each health bar fills its body-region column except its two icons');
        }
        const overflow = await page.evaluate(() => ({
          width: document.documentElement.scrollWidth,
          viewport: innerWidth,
          elements: [...document.querySelectorAll("body *")].filter(el => {
            const box = el.getBoundingClientRect();
            return box.width > 0 && box.right > innerWidth + 2 && getComputedStyle(el).position !== "fixed";
          }).slice(0, 12).map(el => el.className),
        }));
        findings.push({ name, width, overflow });
        await page.evaluate(() => localStorage.clear());
      }
    }
    fs.writeFileSync(path.join(directory, "layout-measurements.json"), JSON.stringify(findings, null, 2));
    for (const finding of findings) {
      assert.ok(finding.overflow.width <= finding.width + 2, `${finding.name} at ${finding.width}: document overflows to ${finding.overflow.width}`);
    }
  } finally { await browser.close(); }
});
