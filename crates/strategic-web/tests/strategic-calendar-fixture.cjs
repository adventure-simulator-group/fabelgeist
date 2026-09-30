const fs = require("node:fs");
const path = require("node:path");

const calendarSource = fs.readFileSync(
  path.join(__dirname, "..", "..", "adventuresim-world-schema", "src", "calendar.rs"),
  "utf8",
);
const strategicTimeSource = fs.readFileSync(
  path.join(__dirname, "..", "..", "adventuresim-core", "src", "strategic_time.rs"),
  "utf8",
);

const integerConstant = (name, source, resolving = new Set()) => {
  if (resolving.has(name)) throw new Error(`cyclic strategic-time constant ${name}`);
  const declaration = new RegExp(`pub const ${name}: u(?:16|64) = ([^;]+);`).exec(source);
  if (!declaration) throw new Error(`missing strategic-time constant ${name}`);
  const dependencies = new Set(resolving).add(name);
  return declaration[1]
    .split("*")
    .map((factor) => factor.trim().replace(/\s+as\s+u(?:16|64)$/, ""))
    .map((factor) => /^\d[\d_]*$/.test(factor)
      ? Number(factor.replaceAll("_", ""))
      : integerConstant(factor, source, dependencies))
    .reduce((product, factor) => product * factor, 1);
};

globalThis.strategicCalendar = Object.freeze({
  minutesPerDay: integerConstant("MINUTES_PER_DAY", calendarSource),
  daysPerYear: integerConstant("DAYS_PER_YEAR", calendarSource),
  lunarCycleMinutes: integerConstant("LUNAR_CYCLE_MINUTES", strategicTimeSource),
});
module.exports = require("../static/strategic-calendar.js");
