(() => {
  function restBooking(days, dailyCost, start, calendar) {
    if (!Number.isInteger(days) || days < 1) return "Enter a whole number of days.";
    const cost = `${days * dailyCost} coin for lodging`;
    if (!Number.isFinite(start)) return `${cost}. Wake time unavailable.`;
    const end = calendar.addDays(start, days);
    return `${cost}. Wake on day ${calendar.dayOfYear(end)}, ${calendar.formatClock(end)}.`;
  }
  if (typeof module !== "undefined") module.exports = { restBooking };
  if (typeof document === "undefined") return;
  const render = () => document.querySelectorAll("[data-rest-daily-cost]").forEach(service => {
    const duration = service.querySelector("[data-rest-duration-input]");
    const output = service.querySelector("[data-rest-booking-preview]");
    if (!duration || !output) return;
    output.textContent = restBooking(Number(duration.value), Number(service.dataset.restDailyCost),
      window.strategicCharacterMinutes, window.strategicCalendar);
  });
  document.addEventListener("input", event => {
    if (event.target.matches("[data-rest-duration-input]")) render();
  });
  ["strategic-page-mounted", "strategic-live-regions-refreshed", "strategic-time-ready"].forEach(name => document.addEventListener(name, render));
  render();
})();
