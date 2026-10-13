// Layout and semantic identity bridge for the single persistent Bevy canvas.
import { createSceneRequests } from "./strategic-scene-request.js";
const kinds = { "public-square": "square", residences: "residence", keep: "keep",
  merchants: "market", weapons: "smith", armor: "armor", clothing: "tailor",
  herbalist: "apothecary", books: "books", inn: "inn", religion: "church",
  church: "church" };
const peopleRequests = new Map();
let rosterLocation;

export function loadPeople(path) {
  if (!peopleRequests.has(path)) {
    const promise = (window.strategicFetch || fetch)(path, { headers: { Accept: "application/json" } })
      .then(response => {
        if (!response.ok) throw new Error(`Could not load people (${response.status})`);
        return response.json();
      }).catch(error => {
        if (peopleRequests.get(path) === promise) peopleRequests.delete(path);
        throw error;
      });
    peopleRequests.set(path, promise);
  }
  return peopleRequests.get(path);
}

// Clip to the window and every scrolling ancestor, retaining the full camera
// projection so partial portraits and the street never stretch when scrolled.
export function canvasRect(element, full = element?.getBoundingClientRect()) {
  if (!element || !full || full.width <= 0 || full.height <= 0 || !element.getClientRects().length) return null;
  let left = Math.max(0, full.left), top = Math.max(0, full.top);
  let right = Math.min(innerWidth, full.right), bottom = Math.min(innerHeight, full.bottom);
  for (let parent = element.parentElement; parent; parent = parent.parentElement) {
    if (parent instanceof HTMLDetailsElement && !parent.open
      && !parent.querySelector(":scope > summary")?.contains(element)) return null;
    const style = getComputedStyle(parent), rect = parent.getBoundingClientRect();
    if (/(auto|scroll|hidden|clip)/.test(style.overflowX)) {
      left = Math.max(left, rect.left); right = Math.min(right, rect.right);
    }
    if (/(auto|scroll|hidden|clip)/.test(style.overflowY)) {
      top = Math.max(top, rect.top); bottom = Math.min(bottom, rect.bottom);
    }
  }
  if (right <= left || bottom <= top) return null;
  const ratio = devicePixelRatio;
  const x = Math.ceil(left * ratio), y = Math.ceil(top * ratio);
  return { x, y, width: Math.floor(right * ratio) - x, height: Math.floor(bottom * ratio) - y,
    full_width: Math.round(full.width * ratio), full_height: Math.round(full.height * ratio),
    offset_x: x - full.left * ratio, offset_y: y - full.top * ratio };
}

// Canvas output sits above the document, so preserve foreground HTML surfaces
// by subtracting their rectangles from the final compositor mask.
function subtract(rect, cover) {
  const left = Math.max(rect.x, cover.x), top = Math.max(rect.y, cover.y);
  const right = Math.min(rect.x + rect.width, cover.x + cover.width);
  const bottom = Math.min(rect.y + rect.height, cover.y + cover.height);
  if (left >= right || top >= bottom) return [rect];
  return [
    { x: rect.x, y: rect.y, width: rect.width, height: top - rect.y },
    { x: rect.x, y: bottom, width: rect.width, height: rect.y + rect.height - bottom },
    { x: rect.x, y: top, width: left - rect.x, height: bottom - top },
    { x: right, y: top, width: rect.x + rect.width - right, height: bottom - top },
  ].filter(part => part.width > 0 && part.height > 0);
}

const foregroundSelector = "[data-chat-dock], .settlement-chat, .settlement-location, .top-bar-right, .character-switcher-menu";

export function installStrategicScene(command, runtimePromise) {
  const surface = document.querySelector("#strategic-render-surface");
  let frame, generation = 0, selected, previewForge = false, roster = [], rosterPending = false, lastPayload;
  let loadedLocation, statusTimer, refreshTimer, runtime, retainedView, changedAt = performance.now();
  let refreshPending = false;
  let revision = 0;
  const equipment = new Map();
  let equipmentPending = false, equipmentEpoch = 0, equipmentError = false;
  const metrics = { navigations: [], bootStarted: performance.now() };
  window.strategicRendererMetrics = metrics;
  const status = document.createElement("div");
  status.id = "strategic-scene-status"; status.setAttribute("role", "status");
  status.textContent = "Loading scene…"; document.body.append(status);
  const schedule = () => { if (!frame) frame = requestAnimationFrame(sync); };
  const resize = new ResizeObserver(schedule);
  const observe = new MutationObserver(schedule);
  const fail = error => {
    status.hidden = false; status.textContent = "Scene unavailable";
    console.error("strategic scene failed", error);
  };
  const sceneRequests = createSceneRequests({ runtimePromise,
    install: ({ location, input }) => command({ type: "prepare-strategic-scene", location, input_json: input }),
    changed: state => { if (state.phase === "failed") fail(state.cause); schedule(); },
  });
  runtimePromise.then(value => { runtime = value; schedule(); }).catch(fail);

  function resetEquipment() {
    equipment.clear(); equipmentEpoch++; equipmentPending = false; equipmentError = false;
  }

  async function loadEquipment(ids) {
    if (equipmentPending || equipmentError) return;
    equipmentPending = true;
    const epoch = equipmentEpoch;
    try {
      for (let offset = 0; offset < ids.length; offset += 256) {
        const path = `/api/scene-equipment?characters=${ids.slice(offset, offset + 256).join(",")}`;
        const response = await (window.strategicFetch || fetch)(path, { cache: "no-store", headers: { Accept: "application/json" } });
        if (!response.ok) throw new Error(`Could not load equipment (${response.status})`);
        const appearances = await response.json();
        if (epoch !== equipmentEpoch) return;
        for (const appearance of appearances) equipment.set(appearance.id, appearance.equipment);
      }
      if (ids.some(id => !equipment.has(id))) throw new Error("Incomplete character equipment response");
    } catch (error) {
      if (epoch === equipmentEpoch) { equipmentError = true; fail(error); }
    } finally {
      if (epoch === equipmentEpoch) { equipmentPending = false; schedule(); }
    }
  }

  async function preload(nav, places) {
    const location = nav.dataset.settlementId;
    if (rosterLocation !== location) { peopleRequests.clear(); rosterLocation = location; }
    rosterPending = true;
    const current = generation;
    try {
      const entries = await Promise.all(places.map(async place => {
        const path = `/api/locations/settlement/${encodeURIComponent(location)}/places/${encodeURIComponent(place.id)}/npcs`;
        return (await loadPeople(path)).map(person => ({ id: String(person.id), place: place.id, presentation: "scene" }));
      }));
      if (current !== generation) return;
      roster = entries.flat(); rosterPending = false; schedule();
    } catch (error) { if (current === generation) { rosterPending = false; fail(error); } }
  }

  function mount() {
    document.body.removeAttribute("data-strategic-scene-ready");
    generation++; selected = undefined; previewForge = false; lastPayload = undefined;
    changedAt = performance.now();
    observe.disconnect(); resize.disconnect();
    const page = document.querySelector("#strategic-page");
    if (!page) return;
    observe.observe(page, { childList: true, subtree: true, attributes: true,
      attributeFilter: ["class", "hidden", "open", "aria-pressed", "data-character-id"] });
    resize.observe(page);
    page.querySelectorAll(foregroundSelector).forEach(element => resize.observe(element));
    const nav = page.querySelector(".settlement-services[data-settlement-id]");
    if (nav) {
      resize.observe(nav);
      nav.querySelectorAll("[data-building-id]").forEach(link => resize.observe(link));
    }
    const places = placeList(nav);
    if (!nav && hasScene(page)) {
      loadedLocation = undefined; roster = []; rosterPending = false;
    }
    if (nav && loadedLocation !== nav.dataset.settlementId) {
      resetEquipment();
      loadedLocation = nav.dataset.settlementId; roster = [];
      preload(nav, places);
    } else if (rosterPending && nav) preload(nav, places);
    schedule();
  }

  async function refreshRoster() {
    if (rosterPending) {
      refreshTimer = setTimeout(refreshRoster, 200);
      return;
    }
    refreshPending = false;
    resetEquipment();
    const nav = document.querySelector(".settlement-services[data-settlement-id]");
    if (!nav) { schedule(); return; }
    peopleRequests.clear();
    await preload(nav, placeList(nav));
    if (refreshPending) refreshTimer = setTimeout(refreshRoster, 200);
  }

  function placeList(nav) {
    return [...(nav?.querySelectorAll('[data-building-id]:not([data-building-id="map"])') || [])]
      .map(link => ({ id: link.dataset.buildingId, kind: kinds[link.dataset.serviceId] || "guild" }));
  }

  function hasScene(page) {
    return page.querySelector('.settlement-services[data-settlement-id], [data-bevy-character], [data-bevy-current-character], .fireplace-stage, .party-member-stage, .npc-description-stage, [data-bevy-scene="forge"]');
  }

  function hideScene() {
    if (sceneRequests.state.phase === "loading") sceneRequests.cancel();
    clearTimeout(statusTimer);
    setClip([]); status.hidden = true;
    if (retainedView) {
      const view = { ...retainedView, street: null, portraits: [], stage: null, forge: null, selected: null };
      const payload = JSON.stringify(view);
      if (payload !== lastPayload) {
        lastPayload = payload;
        command({ type: "sync-strategic-view", view: { ...view, revision: ++revision } });
      }
    }
    document.body.setAttribute("data-strategic-scene-ready", "");
    if (changedAt !== undefined) {
      metrics.navigations.push({ path: location.pathname, milliseconds: performance.now() - changedAt });
      changedAt = undefined;
    }
  }

  function sceneWindow(page, forge) {
    const host = forge || page.querySelector(".npc-description-stage, .party-member-stage, .settlement-overview, .fireplace-stage");
    if (!host || page.querySelector(".settlement-map-main")) return null;
    let element = host.querySelector(":scope > .bevy-scene-window");
    if (!element) {
      element = document.createElement("div"); element.className = "bevy-scene-window";
      element.setAttribute("aria-hidden", "true");
      const portraits = host.querySelector(":scope > .party-portrait-overlay");
      if (portraits) portraits.after(element); else host.prepend(element);
      resize.observe(element);
    }
    return element;
  }

  function streetWindow(nav) {
    const links = [...(nav?.querySelectorAll('[data-building-id]:not([data-building-id="map"])') || [])];
    if (!links.length) return null;
    const layout = metrics.state?.street;
    if (layout) {
      const height = links[0].getBoundingClientRect().height - 34;
      for (const link of links) {
        const bay = layout.bays.find(bay => bay.id === link.dataset.buildingId);
        if (bay) link.style.setProperty("--street-bay-width", `${bay.width / layout.height * height}px`);
      }
    }
    const first = links[0].getBoundingClientRect();
    const last = links.at(-1).getBoundingClientRect();
    const full = { left: first.left, right: last.right, top: first.top,
      bottom: first.bottom - 34, width: last.right - first.left, height: first.height - 34 };
    return { element: links[0], rect: canvasRect(links[0], full) };
  }

  function sync() {
    frame = undefined;
    if (document.body.hasAttribute("data-tactical-active")) return;
    const page = document.querySelector("#strategic-page");
    if (!page) return;
    if (!hasScene(page)) { hideScene(); return; }
    const nav = page.querySelector(".settlement-services[data-settlement-id]");
    const locationId = nav?.dataset.settlementId || location.pathname.split("/").slice(0, 4).join("/");
    const places = placeList(nav);
    if (!places.length) places.push({ id: "camp", kind: "camp" });
    if (rosterPending) return;
    const active = nav?.querySelector('[data-building-id].active')?.dataset.buildingId || places[0].id;
    const people = new Map(roster.map(person => [person.id, person]));
    const portraits = [];
    const portraitWindows = [];
    const currentId = (page.querySelector("[data-active-character][data-character-id]")
      || page.querySelector(".character-switcher-option.is-current[data-character-id]"))?.dataset.characterId;
    for (const element of page.querySelectorAll("[data-bevy-current-character]")) {
      if (currentId) element.dataset.bevyCharacter = currentId;
    }
    for (const element of page.querySelectorAll("[data-bevy-character]")) {
      const id = element.dataset.bevyCharacter;
      if (!/^\d+$/.test(id)) continue;
      const rect = canvasRect(element);
      if (rect) { portraits.push({ id, rect }); portraitWindows.push({ rect, element }); }
      if (!people.has(id)) people.set(id, { id, place: active === "map" ? places[0].id : active,
        presentation: id === currentId || element.closest(".party-portrait, .settlement-npc-portrait") ? "scene" : "portrait" });
    }
    const activeMember = page.querySelector(".party-portrait.active[data-character-id]");
    if (rosterPending) return;
    if (sceneRequests.state.phase !== "prepared" || sceneRequests.state.location !== locationId) {
      sceneRequests.request({ location: locationId, settlement: nav?.dataset.settlementId,
        venues: { places, people: [...people.values()] } });
      return;
    }
    const missingEquipment = [...people.keys()].filter(id => !equipment.has(id));
    if (missingEquipment.length) {
      document.body.removeAttribute("data-strategic-scene-ready");
      loadEquipment(missingEquipment);
      return;
    }
    for (const person of people.values()) person.equipment = equipment.get(person.id);
    const focus = selected || page.querySelector('.settlement-npc-portrait[aria-pressed="true"]')?.dataset.npcId || activeMember?.dataset.characterId;
    const forge = page.querySelector('[data-bevy-scene="forge"]');
    const scene = sceneWindow(page, forge);
    const showForge = Boolean(forge && previewForge);
    if (forge) forge.dataset.sceneMode = showForge ? "forge" : "character";
    const street = streetWindow(nav);
    const rect = canvasRect(scene);
    const view = { location: nav?.dataset.settlementId || location.pathname.split("/").slice(0, 4).join("/"),
      places, people: [...people.values()], active_place: active === "map" ? null : active,
      selected: focus || null, street: street?.rect || null, stage: showForge ? null : rect, forge: showForge ? rect : null, portraits };
    retainedView = view;
    const payload = JSON.stringify(view);
    if (payload !== lastPayload) {
      lastPayload = payload;
      command({ type: "sync-strategic-view", view: { ...view, revision: ++revision } });
    }
    setClip([street, {rect, element: scene}, ...portraitWindows].filter(view => view?.rect));
    if (!runtime) return;
    clearTimeout(statusTimer);
    checkReady();
  }

  function setClip(windows) {
    const ratio = devicePixelRatio;
    const covers = [...document.querySelectorAll(foregroundSelector)]
      .filter(element => getComputedStyle(element).visibility !== "hidden")
      .map(element => ({element, rect: canvasRect(element)})).filter(cover => cover.rect);
    const visible = windows.flatMap(view => covers.filter(cover => !cover.element.contains(view.element))
      .reduce((parts, cover) => parts.flatMap(rect => subtract(rect, cover.rect)), [view.rect]));
    const path = visible.map(rect => {
      const x = rect.x / ratio, y = rect.y / ratio;
      const right = (rect.x + rect.width) / ratio, bottom = (rect.y + rect.height) / ratio;
      return `M${x},${y}H${right}V${bottom}H${x}Z`;
    }).join(" ");
    surface.style.clipPath = path ? `path('${path}')` : "inset(100%)";
  }

  function checkReady() {
    if (document.body.hasAttribute("data-tactical-active")) return;
    const state = JSON.parse(runtime.wasm_strategic_status());
    const streetChanged = JSON.stringify(state.street) !== JSON.stringify(metrics.state?.street);
    metrics.state = state;
    if (streetChanged) schedule();
    if (state.error) { fail(new Error(state.error)); return; }
    const ready = sceneRequests.state.phase === "prepared" && !rosterPending && !equipmentPending && !equipmentError && state.ready && state.revision === revision;
    status.hidden = ready || metrics.navigations.length > 0;
    document.body.toggleAttribute("data-strategic-scene-ready", ready);
    if (ready && changedAt !== undefined) {
      metrics.navigations.push({ path: location.pathname, milliseconds: performance.now() - changedAt });
      if (metrics.navigations.length > 100) metrics.navigations.shift();
      changedAt = undefined;
    }
    if (!ready) statusTimer = setTimeout(checkReady, 100);
  }

  document.addEventListener("strategic-page-mounted", mount);
  document.addEventListener("strategic-page-unmounting", () => {
    if (sceneRequests.state.phase === "loading") sceneRequests.cancel();
    surface.style.clipPath = "inset(100%)"; clearTimeout(statusTimer);
  });
  document.addEventListener("strategic-character-selected", event => {
    document.body.removeAttribute("data-strategic-scene-ready");
    selected = String(event.detail.id); previewForge = false; changedAt = performance.now(); schedule();
  });
  document.addEventListener("strategic-forge-selected", () => { previewForge = true; schedule(); });
  document.addEventListener("strategic-tactical-started", () => {
    if (sceneRequests.state.phase === "loading") sceneRequests.cancel();
    lastPayload = undefined; clearTimeout(statusTimer); status.hidden = true;
  });
  document.addEventListener("strategic-tactical-ended", schedule);
  document.addEventListener("strategic-live-regions-refreshed", schedule);
  document.addEventListener("strategic-live-update", event => {
    if (event.target !== document) return;
    refreshPending = true;
    clearTimeout(refreshTimer);
    refreshTimer = setTimeout(refreshRoster, 200);
  });
  document.addEventListener("portrait-view-selected", schedule);
  document.addEventListener("scroll", schedule, true);
  window.addEventListener("resize", schedule);
  window.addEventListener("keydown", event => { if (event.key === "Escape") schedule(); });
  mount();
}
