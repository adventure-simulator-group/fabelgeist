const $ = (selector) => document.querySelector(selector);
const canvas = $("#game-canvas");
const loading = $("#loading");
const tabs = [...document.querySelectorAll("[data-category]")];
let catalog = [];
let selected;
let runtime;
let bootError;
let readinessTimer;
const remembered = new Map();

function fail(error) {
  clearInterval(readinessTimer);
  bootError = error;
  loading.hidden = false;
  loading.dataset.error = "";
  $("#loading-message").textContent = error instanceof Error ? error.message : String(error);
}

function send(payload) {
  if (!runtime) return;
  try { runtime.command(JSON.stringify(payload)); } catch (error) { fail(error); }
}

const categoryOf = (exhibit) => exhibit.kind === "scenery" ? exhibit.id : exhibit.kind;

function select(exhibit) {
  heldKeys.clear();
  selected = exhibit;
  const category = categoryOf(exhibit);
  remembered.set(category, exhibit.id);
  tabs.forEach((tab) => {
    const active = tab.dataset.category === category;
    tab.setAttribute("aria-selected", String(active));
    tab.tabIndex = active ? 0 : -1;
  });
  $("#exhibit").setAttribute("aria-labelledby", `tab-${category}`);
  $("#exhibit-title").textContent = exhibit.label;
  $("#technique").textContent = exhibit.technique;
  $("#exhibit-note").textContent = exhibit.note;
  const siblings = catalog.filter((entry) => categoryOf(entry) === category);
  $("#specimen-count").textContent = siblings.length > 1
    ? `${String(siblings.indexOf(exhibit) + 1).padStart(2, "0")} / ${String(siblings.length).padStart(2, "0")}` : "";
  $("#specimens").replaceChildren(...(siblings.length > 1 ? siblings.map((entry) => {
    const button = document.createElement("button");
    button.textContent = entry.label;
    button.setAttribute("role", "tab");
    button.setAttribute("aria-selected", String(entry === exhibit));
    button.setAttribute("aria-controls", "exhibit");
    button.tabIndex = entry === exhibit ? 0 : -1;
    button.addEventListener("click", () => { select(entry); $("#specimens [aria-selected='true']").focus(); });
    return button;
  }) : []));
  const reference = exhibit.reference;
  $(".viewer-layout").dataset.reference = String(Boolean(reference));
  $("#reference").hidden = !reference;
  $("#reference-error").hidden = true;
  $("#reference-photo").hidden = false;
  if (reference) {
    $("#reference-photo").alt = reference.alt;
    $("#reference-photo").src = `/tactical/assets/${reference.image}`;
    $("#reference-source").href = reference.source;
    $("#reference-source").textContent = reference.credit;
  }
  history.replaceState(null, "", `#${exhibit.id}`);
  loading.hidden = false;
  delete loading.dataset.error;
  $("#loading-message").textContent = runtime ? "Preparing exhibit…" : "Starting 3D renderer…";
  send({ type: "show", exhibit: exhibit.id });
  if (bootError) fail(bootError);
}

for (const tab of tabs) {
  tab.addEventListener("click", () => {
    const category = tab.dataset.category;
    const exhibit = catalog.find((entry) => entry.id === remembered.get(category))
      ?? catalog.find((entry) => categoryOf(entry) === category);
    if (exhibit) select(exhibit);
  });
}

function navigateTabs(event) {
  const buttons = [...event.currentTarget.querySelectorAll("button")];
  let index = buttons.indexOf(document.activeElement);
  if (index < 0) return;
  if (event.key === "ArrowRight") index = (index + 1) % buttons.length;
  else if (event.key === "ArrowLeft") index = (index - 1 + buttons.length) % buttons.length;
  else if (event.key === "Home") index = 0;
  else if (event.key === "End") index = buttons.length - 1;
  else return;
  event.preventDefault();
  buttons[index].focus();
  buttons[index].click();
}
$(".categories").addEventListener("keydown", navigateTabs);
$("#specimens").addEventListener("keydown", navigateTabs);

const pointers = new Map();
let pinchDistance;
const distance = () => {
  const [a, b] = [...pointers.values()];
  return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : undefined;
};
canvas.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  canvas.focus({ preventScroll: true });
  canvas.setPointerCapture(event.pointerId);
  pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
  pinchDistance = distance();
});
canvas.addEventListener("pointermove", (event) => {
  const before = pointers.get(event.pointerId);
  if (!before) return;
  pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
  if (pointers.size > 1) {
    const next = distance();
    if (pinchDistance > 0 && next > 0) send({ type: "zoom", delta: Math.log(pinchDistance / next) * 1000 });
    pinchDistance = next;
  } else {
    send({ type: "orbit", delta_x: event.clientX - before.x, delta_y: event.clientY - before.y });
  }
});
for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) {
  canvas.addEventListener(type, (event) => { pointers.delete(event.pointerId); pinchDistance = distance(); });
}
canvas.addEventListener("wheel", (event) => {
  event.preventDefault();
  const scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1;
  send({ type: "zoom", delta: event.deltaY * scale });
}, { passive: false });
canvas.addEventListener("keydown", (event) => {
  const orbit = { ArrowLeft: [-20, 0], ArrowRight: [20, 0], ArrowUp: [0, -20], ArrowDown: [0, 20] }[event.key];
  if (orbit) send({ type: "orbit", delta_x: orbit[0], delta_y: orbit[1] });
  else if (["+", "=", "-", "_"].includes(event.key)) send({ type: "zoom", delta: ["+", "="].includes(event.key) ? -100 : 100 });
  else return;
  event.preventDefault();
});
window.addEventListener("keydown", (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey || event.target.closest("input, textarea, select, [contenteditable]")) return;
  if (["KeyW", "KeyA", "KeyS", "KeyD"].includes(event.code)) {
    if (!event.repeat && loading.hidden) send({ type: "pan",
      right: Number(event.code === "KeyD") - Number(event.code === "KeyA"),
      forward: Number(event.code === "KeyW") - Number(event.code === "KeyS"), seconds: 1 / 60 });
    heldKeys.add(event.code);
    event.preventDefault();
    return;
  }
});
const heldKeys = new Set();
let previousFrame;
function panFrame(now) {
  const seconds = previousFrame === undefined ? 0 : Math.min((now - previousFrame) / 1000, 0.05);
  previousFrame = now;
  if (heldKeys.size && loading.hidden) send({ type: "pan",
    right: Number(heldKeys.has("KeyD")) - Number(heldKeys.has("KeyA")),
    forward: Number(heldKeys.has("KeyW")) - Number(heldKeys.has("KeyS")), seconds });
  requestAnimationFrame(panFrame);
}
requestAnimationFrame(panFrame);
window.addEventListener("keyup", (event) => heldKeys.delete(event.code));
canvas.addEventListener("blur", () => heldKeys.clear());
window.addEventListener("blur", () => heldKeys.clear());
document.addEventListener("visibilitychange", () => { heldKeys.clear(); previousFrame = undefined; });
$("#reference-photo").addEventListener("error", () => {
  $("#reference-photo").hidden = true;
  $("#reference-error").hidden = false;
});
window.addEventListener("hashchange", () => {
  const entry = catalog.find((entry) => entry.id === location.hash.slice(1));
  if (entry && entry !== selected) select(entry);
});
window.addEventListener("pagehide", () => clearInterval(readinessTimer));
window.addEventListener("error", (event) => fail(event.error ?? "The renderer stopped. Reload to retry."));
window.addEventListener("unhandledrejection", (event) => fail(event.reason));

async function start() {
  const response = await fetch("/tactical/assets/art-demo/catalog.json");
  if (!response.ok) throw new Error("Collection unavailable. Run just build-wasm and reload.");
  catalog = await response.json();
  select(catalog.find((entry) => entry.id === location.hash.slice(1)) ?? catalog[0]);
  if (!navigator.gpu) throw new Error("This showcase requires WebGPU. Open it in a WebGPU-enabled browser over HTTPS or localhost.");
  const module = await import("/tactical/wasm/art-demo.js");
  await module.default();
  module.boot();
  runtime = module;
  send({ type: "show", exhibit: selected.id });
  readinessTimer = setInterval(() => {
    try {
      const status = JSON.parse(runtime.status());
      canvas.dataset.state = status.state;
      if (status.exhibit) canvas.dataset.exhibit = status.exhibit;
      if (status.state === "ready" && status.exhibit === selected.id) loading.hidden = true;
      else if (status.state === "loading" && status.total > 0 && status.exhibit === selected.id)
        $("#loading-message").textContent = `Loading city · ${status.completed.toLocaleString()} / ${status.total.toLocaleString()} buildings`;
      else if (status.state === "failed") fail(status.message);
    } catch (error) { clearInterval(readinessTimer); fail(error); }
  }, 200);
}

start().catch((error) => { bootError = error; fail(error); });
