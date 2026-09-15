const test = require("node:test");
const assert = require("node:assert/strict");
const { hydrateProceduralEquipmentIcons } = require("../static/inventory-browser.js");

test("weapon and armor portraits display color images only after loading", () => {
  const originalImage = global.Image;
  const pending = [];
  global.Image = class {
    constructor() { this.events = {}; pending.push(this); }
    addEventListener(event, callback) { this.events[event] = callback; }
  };
  try {
    function row(portrait, dataset) {
      const properties = new Map();
      const classes = new Set();
      const icon = { dataset: {}, isConnected: true, style: { setProperty: (k, v) => properties.set(k, v) }, classList: { add: c => classes.add(c) } };
      return {
        dataset, icon, properties, classes,
        matches: () => true,
        querySelector: selector => selector === ".inventory-item-type .game-icon" ? icon
          : selector === "[data-equipment-portrait]" ? (portrait ? { dataset: { equipmentPortrait: portrait } } : null)
          : {},
      };
    }
    const weapon = row(null, { personalInventoryId: "12" });
    hydrateProceduralEquipmentIcons(weapon);
    assert.equal(pending[0].src, "/api/weapon-icons/personal/12.png");
    assert.equal(weapon.classes.size, 0);
    pending[0].events.load();
    assert.equal(weapon.properties.get("--equipment-portrait"), 'url("/api/weapon-icons/personal/12.png")');
    assert.ok(weapon.classes.has("equipment-portrait"));
    hydrateProceduralEquipmentIcons(weapon);
    assert.equal(pending.length, 1, "hydrating the same row does not fetch twice");

    const armor = row("/static/equipment-icons/barbute--worn.png", {});
    hydrateProceduralEquipmentIcons(armor);
    assert.equal(pending[1].src, "/static/equipment-icons/barbute--worn.png");
    pending[1].events.load();
    assert.ok(armor.classes.has("equipment-portrait"));

    const missing = row(null, { partyInventoryId: "9" });
    hydrateProceduralEquipmentIcons(missing);
    pending[2].events.error();
    assert.equal(missing.classes.size, 0, "missing instances retain their catalog icon");

    const stale = row(null, { personalInventoryId: "10" });
    hydrateProceduralEquipmentIcons(stale);
    stale.icon.dataset.proceduralEquipmentIcon = "/new-row.png";
    pending[3].events.load();
    assert.equal(stale.classes.size, 0, "a stale request cannot replace a new row's image");
  } finally {
    global.Image = originalImage;
  }
});
