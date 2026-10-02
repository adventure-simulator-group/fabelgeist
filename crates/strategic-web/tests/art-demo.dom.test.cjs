const assert = require("node:assert/strict");
const test = require("node:test");
const fs = require("node:fs");
const path = require("node:path");
const { parseHTML } = require("linkedom");

async function fixture({ hash = "#city", saveData = false } = {}) {
  const { window, document } = parseHTML(fs.readFileSync(path.join(__dirname, "../static/art-demo/index.html"), "utf8"));
  const catalog = JSON.parse(fs.readFileSync(path.join(__dirname, "../../../assets/art-demo/catalog.json"), "utf8"));
  const location = { hash };
  const navigation = [];
  const commands = [];
  let status = { state: "starting" };
  let poll;
  let frame;
  let boots = 0;
  let nextTimer = 1;
  const timeouts = new Map();
  const history = Object.fromEntries(["pushState", "replaceState"].map((method) => [method, (_, __, hash) => {
    navigation.push([method, hash]);
    location.hash = hash;
  }]));
  const runtime = {
    default: async () => {},
    boot: () => { boots++; },
    command: (json) => commands.push(JSON.parse(json)),
    status: () => JSON.stringify(status),
  };
  Object.assign(globalThis, {
    window, document, location, history,
    fetch: async () => ({ ok: true, json: async () => catalog }),
    requestAnimationFrame: (callback) => { frame = callback; },
    setInterval: (callback) => { poll = callback; return 1; },
    clearInterval: () => { poll = undefined; },
    setTimeout: (callback) => { const id = nextTimer++; timeouts.set(id, callback); return id; },
    clearTimeout: (id) => timeouts.delete(id),
  });
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: { gpu: {}, connection: { saveData } },
  });
  const { mount } = await import("../static/art-demo/viewer.mjs");
  await mount({ loadRuntime: async () => runtime });
  const emit = (target, type, properties = {}) => {
    const event = new window.Event(type, { bubbles: true, cancelable: true });
    Object.assign(event, properties);
    target.dispatchEvent(event);
    return event;
  };
  return {
    document, window, location, navigation, commands, emit,
    canvas: document.querySelector("canvas"),
    loading: document.querySelector("#loading"),
    boots: () => boots,
    status: (value) => { status = value; poll(); },
    frame: (time) => frame(time),
    hasPoll: () => Boolean(poll),
    runTimeout: () => {
      const entry = timeouts.entries().next().value;
      if (!entry) return;
      timeouts.delete(entry[0]);
      entry[1]();
    },
    timeoutCount: () => timeouts.size,
  };
}

test("cold start loads only the selected exhibit before sustained idle", async () => {
  const f = await fixture({ hash: "#henry" });
  assert.deepEqual(f.commands, [{ type: "show", exhibit: "henry" }]);
  assert.equal(f.timeoutCount(), 0);
  f.status({ state: "ready", exhibit: "henry" });
  assert.equal(f.timeoutCount(), 1);
});

test("immediate cross-category selection preempts scheduled speculation", async () => {
  const f = await fixture({ hash: "#henry" });
  f.emit(f.document.querySelector("#tab-weapon"), "click");
  assert.deepEqual(f.commands.at(-1), { type: "show", exhibit: "longsword" });
  assert.equal(f.timeoutCount(), 0);
});

test("pointer and keyboard intent prefetch without changing selection", async () => {
  const f = await fixture({ hash: "#henry", saveData: true });
  const weapon = f.document.querySelector("#tab-weapon");
  f.emit(weapon, "pointerenter");
  assert.equal(f.commands.filter((command) => command.type === "prefetch").length, 0);
  f.status({ state: "ready", exhibit: "henry" });
  assert.deepEqual(f.commands.at(-1), { type: "prefetch", exhibit: "longsword" });
  assert.equal(weapon.getAttribute("aria-selected"), "false");
  assert.equal(f.location.hash, "#henry");
  f.emit(weapon, "focus");
  assert.equal(f.commands.filter((command) => command.type === "prefetch").length, 1);
});

test("sustained idle has a deterministic fallback and respects constrained data", async () => {
  const f = await fixture({ hash: "#henry" });
  f.status({ state: "ready", exhibit: "henry" });
  f.runTimeout();
  assert.deepEqual(f.commands.at(-1), { type: "prefetch", exhibit: "nuremberg" });
  const constrained = await fixture({ hash: "#henry", saveData: true });
  constrained.status({ state: "ready", exhibit: "henry" });
  assert.equal(constrained.timeoutCount(), 0);
});

test("cached return keeps the runtime and canvas and reports timing evidence", async () => {
  const f = await fixture({ hash: "#henry" });
  const canvas = f.canvas;
  f.emit(f.document.querySelector("#tab-weapon"), "click");
  f.status({ state: "ready", exhibit: "longsword" });
  f.emit(f.document.querySelector("#tab-armor"), "click");
  f.status({ state: "ready", exhibit: "henry" });
  assert.equal(f.document.querySelector("canvas"), canvas);
  assert.equal(f.boots(), 1);
  assert.match(f.document.documentElement.dataset.firstSelectedExhibitMilliseconds, /^\d+(\.\d+)?$/);
  assert.match(f.document.documentElement.dataset.bytesBeforeInteraction, /^\d+$/);
});

test("partially loaded city accepts pan input and preserves its canvas across navigation", async () => {
  const f = await fixture();
  f.status({ state: "loading", exhibit: "city", completed: 3, total: 3000 });
  assert.equal(f.loading.hidden, false);
  assert.equal(f.loading.dataset.streaming, "");
  f.emit(f.canvas, "keydown", { code: "KeyW", key: "w" });
  assert.equal(f.commands.at(-1).type, "pan");
  f.emit(f.document.querySelector("#tab-weapon"), "click");
  assert.equal(f.commands.at(-1).exhibit, "longsword");
  assert.deepEqual(f.navigation.at(-1), ["pushState", "#longsword"]);
  f.status({ state: "ready", exhibit: "city" });
  assert.equal(f.loading.hidden, false, "late previous-scene status cannot hide the current progress");
  f.status({ state: "ready", exhibit: "longsword" });
  assert.equal(f.loading.hidden, true);
  f.location.hash = "#city";
  f.emit(f.window, "popstate");
  assert.equal(f.commands.at(-1).exhibit, "city");
  assert.equal(f.document.querySelector("canvas"), f.canvas);
  assert.equal(f.boots(), 1);
});

test("an unavailable exhibit remains navigable and does not become a renderer failure", async () => {
  const f = await fixture();
  f.status({ state: "unavailable", exhibit: "city", message: "A building could not load." });
  assert.equal(f.loading.dataset.error, "");
  assert.equal(f.loading.dataset.streaming, "");
  f.emit(f.document.querySelector("#tab-armor"), "click");
  f.status({ state: "ready", exhibit: "henry" });
  assert.equal(f.loading.hidden, true);
  assert.equal(f.hasPoll(), true);
  assert.equal(f.boots(), 1);
});

test("page restoration resumes status polling without booting another runtime", async () => {
  const f = await fixture();
  f.emit(f.window, "pagehide");
  assert.equal(f.hasPoll(), false);
  f.emit(f.window, "pageshow", { persisted: true });
  assert.equal(f.hasPoll(), true);
  f.status({ state: "loading", exhibit: "city", completed: 20, total: 3000 });
  assert.equal(f.canvas.dataset.state, "loading");
  const event = f.emit(f.document.querySelector(".wordmark"), "click", { button: 0 });
  assert.equal(event.defaultPrevented, true);
  assert.equal(f.commands.at(-1).exhibit, "henry");
  assert.equal(f.boots(), 1);
});
