// Tactical HUD and loading screen in DOM. The wasm client computes what they
// show and calls the window.hud_* functions defined here, only when a value
// changed (crates/adventuresim-tactical-client/src/hud.rs). A host page calls
// installTacticalHud before it boots the client, because the client calls
// these functions from its first frame.

const ASSETS = new URL("./assets/", import.meta.url);
const SVG = "http://www.w3.org/2000/svg";
const CONTROLS =
  "WASD to move | Caps Lock: jog | Shift: sprint | Space to jump | " +
  "Aim + Space + WASD: quickstep | Release Left Alt without WASD: prone/get up | " +
  "Left Alt + WASD: dive (slide while sprinting) | Downed WASD: tank controls | " +
  "Hold Space: align with camera | Hold Space + A/D: keep rolling | " +
  "Mouse to look around | F9 to toggle camera";
const LIMBS = ["Head", "Chest", "Stomach", "Left Arm", "Right Arm", "Left Leg", "Right Leg"];
const ATTACK_RESULT_SECONDS = 4;

function element(tag, className, parent, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  parent?.append(node);
  return node;
}

function svgElement(tag, attributes, parent) {
  const node = document.createElementNS(SVG, tag);
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  parent.append(node);
  return node;
}

/** Builds both overlays inside `container`, the game canvas's parent.
 *  `command(payload)` forwards a browser command to the running client. */
export function installTacticalHud({ container, command }) {
  const stylesheet = document.createElement("link");
  stylesheet.rel = "stylesheet";
  stylesheet.href = new URL("./tactical-hud.css", import.meta.url).href;
  document.head.append(stylesheet);

  const root = element("div", "tactical-hud", container);
  root.hidden = true;
  const wheels = element("div", "hud-wheels", root);
  const outcome = element("div", "hud-outcome", root);
  outcome.hidden = true;
  const crosshair = element("div", "hud-crosshair", root);
  crosshair.hidden = true;
  const aimBlocked = element("div", "hud-aim-blocked", crosshair, "MUZZLE BLOCKED");
  aimBlocked.hidden = true;
  const attackInfo = element("div", "hud-attack-info", root);
  const texts = {
    "attack-timer": element("div", "hud-attack-timer", attackInfo),
    guard: element("div", "hud-guard", root),
  };
  const attackResult = element("div", "hud-attack-result", attackInfo);
  element("div", "hud-controls", root, CONTROLS);
  const limbs = element("div", "hud-limbs", element("div", "hud-player", root));
  element("div", "title", limbs, "Limbs");
  const limbValues = LIMBS.map((name) => {
    const row = element("div", "", limbs, `${name}: `);
    return element("span", "", row);
  });
  const slots = element("div", "hud-slots", root);
  slots.hidden = true;
  // A left click selects a slot while a hand still grabs. The grab's held
  // button gives the canvas pointer capture and the client cancels its
  // pointerdown, so the click reaches no slot node and fires no mouse event:
  // it arrives as a pointermove whose button changed (pointerdown when no
  // other button is held). Hit-test the slot HUD at the pointer instead.
  // With the pointer locked in play the position is frozen, so this only
  // works with the cursor free (Esc).
  const pressSlotHud = (event) => {
    const leftPressed = event.button === 0 && (event.buttons & 1) === 1;
    if (!leftPressed || root.hidden || slots.hidden || document.pointerLockElement) return;
    const target = document.elementsFromPoint(event.clientX, event.clientY)
      .find((node) => slots.contains(node) && node.closest("[data-press]"))
      ?.closest("[data-press]");
    if (!target) return;
    const press = target.dataset.press === "other-hand"
      ? { kind: "other-hand" }
      : { kind: "slot", input: target.dataset.press };
    command({ type: "press-slot-hud", press });
  };
  window.addEventListener("pointerdown", pressSlotHud, true);
  window.addEventListener("pointermove", pressSlotHud, true);

  const loading = createLoadingScreen(container, command);
  const icons = new Map();
  let following = [];
  let attackResultTimer;

  window.hud_visible = (on) => { root.hidden = !on; };
  window.hud_reticle = (state) => {
    crosshair.hidden = state === "hidden";
    aimBlocked.hidden = state !== "blocked";
  };
  window.hud_text = (id, text) => { texts[id].textContent = text; };
  window.hud_limbs = (percent) => {
    percent.forEach((value, index) => { limbValues[index].textContent = `${value}%`; });
  };
  window.hud_icon = (key, width, height, rgba) => {
    // The bytes are a view of wasm memory: copy them before anything else runs.
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    canvas.getContext("2d").putImageData(
      new ImageData(new Uint8ClampedArray(rgba), width, height), 0, 0);
    icons.set(key, canvas.toDataURL());
  };
  const views = {
    wheels(view) {
      following = drawWheels(wheels, view);
    },
    outcome(view) {
      outcome.hidden = !view;
      if (!view) return;
      outcome.textContent = view.label;
      outcome.className = `hud-outcome ${view.tone}`;
    },
    "attack-result"(view) {
      attackResult.replaceChildren(
        "Attacking ",
        Object.assign(document.createElement("span"), {
          className: "target", textContent: view.target,
        }),
        ": ",
        Object.assign(document.createElement("span"), {
          className: view.verdict, textContent: view.verdict === "success" ? "success" : "fail",
        }),
        `\n\n${view.detail}`,
      );
      attackResult.style.setProperty("--player-color", view.color);
      attackResult.classList.remove("showing");
      void attackResult.offsetWidth; // restart the show-in animation
      attackResult.classList.add("showing");
      clearTimeout(attackResultTimer);
      attackResultTimer = setTimeout(
        () => attackResult.replaceChildren(), ATTACK_RESULT_SECONDS * 1000);
    },
    slots(view) {
      slots.hidden = !view;
      if (view) drawSlots(slots, view, icons);
    },
    loading: loading.show,
  };
  window.hud_view = (id, json) => views[id](JSON.parse(json));
  // Called every frame an enemy's wheel moves: only a transform changes, so
  // the browser recomposites without restyling, laying out or repainting.
  window.hud_wheel_positions = (centres) => {
    following.forEach((wheel, index) => {
      const x = centres[index * 2] - wheel.half;
      const y = centres[index * 2 + 1] - wheel.half;
      wheel.node.style.transform = `translate(${x}px, ${y}px)`;
    });
  };
}

/** Incapacitation wheels: a dark track, a red glow while incapacitated, then
 *  the arcs clockwise from the top, as the client clipped them. Each wheel is
 *  its own small SVG drawn around its centre: the player's sits at the screen
 *  centre and each enemy's is moved by hud_wheel_positions. Returns the
 *  following wheels in the order their positions arrive. */
function drawWheels(container, view) {
  const following = [];
  container.replaceChildren();
  for (const wheel of view.wheels) {
    const r = wheel.radius;
    const half = Math.ceil(r + wheel.width / 2 + 3);
    const svg = svgElement("svg", {
      class: wheel.follows ? "hud-wheel" : "hud-wheel centred",
      width: half * 2,
      height: half * 2,
      viewBox: `${-half} ${-half} ${half * 2} ${half * 2}`,
    }, container);
    if (wheel.follows) {
      svg.style.transform = "translate(-9999px, -9999px)";
      following.push({ node: svg, half });
    }
    svgElement("circle", { class: "track", r, "stroke-width": wheel.width }, svg);
    if (wheel.incapacitated) {
      svgElement("circle", { class: "incapacitated", r, "stroke-width": wheel.width + 6 }, svg);
    }
    let angle = -Math.PI / 2;
    for (const arc of wheel.arcs) {
      const end = angle + arc.share * Math.PI * 2;
      const attributes = { class: "arc", stroke: arc.color, "stroke-width": wheel.width };
      if (arc.share >= 0.9999) {
        svgElement("circle", { ...attributes, r }, svg);
      } else {
        const large = arc.share > 0.5 ? 1 : 0;
        svgElement("path", {
          ...attributes,
          d: `M ${r * Math.cos(angle)} ${r * Math.sin(angle)} ` +
            `A ${r} ${r} 0 ${large} 1 ${r * Math.cos(end)} ${r * Math.sin(end)}`,
        }, svg);
      }
      angle = end;
    }
  }
  return following;
}

function iconElement(icon, className, icons) {
  const node = element("div", `hud-icon ${className}`);
  if (icon.kind === "atlas") {
    node.style.backgroundImage = `url("${new URL(icon.path, ASSETS)}")`;
    node.style.backgroundSize = `${icon.columns * 100}% ${icon.rows * 100}%`;
    node.style.backgroundPosition = `${(icon.column / (icon.columns - 1)) * 100}% ` +
      `${(icon.row / (icon.rows - 1)) * 100}%`;
  } else if (icon.kind === "asset") {
    node.style.backgroundImage = `url("${new URL(icon.path, ASSETS)}")`;
  } else {
    node.classList.add("generated");
    const url = icons.get(icon.key);
    if (url) node.style.backgroundImage = `url("${url}")`;
  }
  return node;
}

/** The QWERTY slot grid and the hands row. Cells of one item that spans
 *  several locations are joined between their icons. */
function drawSlots(slots, view, icons) {
  const grid = element("div", "hud-slot-grid");
  grid.style.setProperty("--columns", view.columns);
  grid.style.setProperty("--rows", view.rows);
  const joined = [];
  for (const cell of view.cells) {
    const node = element("div", "hud-slot", grid);
    node.classList.toggle("eligible", cell.eligible);
    node.classList.toggle("selected", cell.selected);
    node.classList.toggle("refused", cell.refused);
    node.style.gridRow = cell.row + 1;
    node.style.gridColumn = cell.column + 1;
    node.title = cell.tooltip;
    element("span", "key", node, cell.key);
    if (cell.icon) {
      const icon = iconElement(cell.icon, "", icons);
      node.append(icon);
      if (cell.join !== null) joined.push({ cell, icon });
    }
    node.dataset.press = cell.input;
  }

  const hands = element("div", "hud-hands");
  const active = iconElement(view.active, "active", icons);
  active.title = view.active_tooltip;
  hands.append(active);
  const other = element("button", "", hands);
  other.type = "button";
  other.title = "Move or swap with the other hand";
  other.append(iconElement(view.other, "", icons));
  other.dataset.press = "other-hand";
  if (view.pickup) {
    const pickup = iconElement(view.pickup.icon, "pickup", icons);
    pickup.title = view.pickup.tooltip;
    hands.append(pickup);
  }
  slots.replaceChildren(grid, hands);

  const origin = grid.getBoundingClientRect();
  for (const [index, first] of joined.entries()) {
    for (const second of joined.slice(index + 1)) {
      if (first.cell.join !== second.cell.join) continue;
      const rows = Math.abs(first.cell.row - second.cell.row);
      const columns = Math.abs(first.cell.column - second.cell.column);
      if (rows + columns !== 1) continue;
      const [a, b] = [first.icon, second.icon].map((icon) => icon.getBoundingClientRect());
      const [left, right] = a.left < b.left ? [a, b] : [b, a];
      const [top, bottom] = a.top < b.top ? [a, b] : [b, a];
      const join = element("div", "hud-slot-join", grid);
      if (rows === 0) {
        Object.assign(join.style, {
          left: `${left.right - origin.left}px`, width: `${right.left - left.right}px`,
          top: `${left.top + left.height / 2 - 2.5 - origin.top}px`, height: "5px",
        });
      } else {
        Object.assign(join.style, {
          top: `${top.bottom - origin.top}px`, height: `${bottom.top - top.bottom}px`,
          left: `${top.left + top.width / 2 - 2.5 - origin.left}px`, width: "5px",
        });
      }
    }
  }
}

function createLoadingScreen(container, command) {
  const root = element("div", "tactical-loading", container);
  root.hidden = true;
  element("div", "title", root, "Fabelgeist");
  const stages = element("div", "stages", root);
  stages.style.display = "contents";
  const failure = element("div", "failure", root);
  failure.hidden = true;
  const message = element("div", "message", failure);
  const buttons = element("div", "buttons", failure);
  const retry = element("button", "", buttons, "Retry");
  const exit = element("button", "", buttons, "Exit");
  retry.type = exit.type = "button";
  retry.addEventListener("click", () => command({ type: "retry-tactical-load" }));
  // Each host page owns where leaving tactical play goes.
  exit.addEventListener("click", () => window.exitTactical?.());
  const fills = [];

  const show = (view) => {
    if (view.state === "hidden") {
      root.hidden = true;
      root.classList.remove("revealing");
      return;
    }
    root.hidden = false;
    if (view.state === "revealing") {
      root.style.setProperty("--reveal-seconds", `${view.seconds}s`);
      root.classList.add("revealing");
      return;
    }
    root.classList.remove("revealing");
    failure.hidden = view.state !== "failed";
    if (view.state === "failed") {
      message.textContent = view.message;
      return;
    }
    view.stages.forEach((stage, index) => {
      if (!fills[index]) {
        const row = element("div", "stage", stages);
        const label = element("div", "", row);
        const fill = element("div", "fill", element("div", "track", row));
        fills[index] = { label, fill };
      }
      fills[index].label.textContent = stage.label;
      fills[index].fill.style.width = `${stage.fraction * 100}%`;
      fills[index].fill.classList.toggle("finished", stage.finished);
    });
  };
  return { show };
}
