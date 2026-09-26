const test = require("node:test");
const assert = require("node:assert/strict");
const { restBooking } = require("../static/action-previews.js");
const calendar = { minutesPerDay: 1440, daysPerYear: 365 };

test("lodging previews show the selected cost and the game's one-based calendar", () => {
  assert.equal(restBooking(2, 2, 8 * 60, calendar), "4 coin for lodging. Wake on day 3, 08:00.");
  assert.equal(restBooking(1, 0, 364 * 1440 + 23 * 60 + 59, calendar), "0 coin for lodging. Wake on day 1, 23:59.");
  assert.equal(restBooking(1, 2, undefined, calendar), "2 coin for lodging. Wake time unavailable.");
  assert.equal(restBooking(1.5, 2, 0, calendar), "Enter a whole number of days.");
});
