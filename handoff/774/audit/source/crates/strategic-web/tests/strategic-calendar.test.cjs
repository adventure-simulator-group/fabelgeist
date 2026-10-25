const assert = require("node:assert/strict");
const test = require("node:test");
const calendar = require("./strategic-calendar-fixture.cjs");

test("browser calendar preserves day, clock, and non-leap rollover", () => {
  assert.equal(calendar.dayOfYear(0), 1);
  assert.equal(calendar.formatClock(0), "00:00");
  assert.equal(calendar.dayOfYear(calendar.addDays(0, calendar.daysPerYear)), 1);
  assert.equal(calendar.formatClock(calendar.addMinutes(0, calendar.minutesPerDay + 61)), "01:01");
  assert.deepEqual(calendar.calendarDate(31 * calendar.minutesPerDay), {
    weekday: "Thursday", day: 1, month: "February", isSunday: false,
  });
});

test("browser calendar owns rail boundaries and lunar phase", () => {
  assert.equal(calendar.dayStart(1_500), calendar.minutesPerDay);
  assert.equal(calendar.nextMidnightAtOrAfter(1_500), 2 * calendar.minutesPerDay);
  assert.equal(calendar.minuteOfDay(1_500), 60);
  assert.equal(calendar.lunarPhase(calendar.lunarCycleMinutes), 0);
  assert.equal(calendar.lunarPhase(calendar.lunarCycleMinutes / 2), 0.5);
  assert.equal(calendar.elapsedSince(1_500, 1_440), 60);
  assert.equal(calendar.elapsedSince(1_440, 1_500), 0);
  assert.equal(calendar.minutesUntilDailyTimeWithMinimum(1_430, 10, 1_440), 1_460);
  assert.equal(calendar.minutesUntilDailyTimeWithMinimum(1_430, 10, 1), 20);
});
