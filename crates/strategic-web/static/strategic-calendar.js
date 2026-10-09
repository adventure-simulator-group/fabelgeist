(() => {
  const source = globalThis.strategicCalendar;
  const { minutesPerDay: DAY, daysPerYear: DAYS_PER_YEAR, lunarCycleMinutes: LUNAR_CYCLE } = source;
  const WEEKDAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
  const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
  const MONTH_DAYS = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  const modulo = (value, size) => ((value % size) + size) % size;
  const dayIndex = (minute) => Math.floor(Number(minute) / DAY);
  const minuteOfDay = (minute) => modulo(Math.floor(Number(minute)), DAY);
  const dayOfYear = (minute) => modulo(dayIndex(minute), DAYS_PER_YEAR) + 1;
  const dayStart = (minute) => dayIndex(minute) * DAY;
  const nextMidnightAtOrAfter = (minute) => Math.ceil(Number(minute) / DAY) * DAY;
  const addMinutes = (minute, elapsed) => Number(minute) + Number(elapsed);
  const addDays = (minute, days) => addMinutes(minute, Number(days) * DAY);
  const elapsedSince = (later, earlier) => Math.max(0, Number(later) - Number(earlier));
  const minutesUntilDailyTimeWithMinimum = (current, target, minimumMinutes) => {
    const currentTod = minuteOfDay(Math.round(Number(current)));
    const targetTod = minuteOfDay(Math.round(Number(target)));
    const minimum = Math.max(1, Math.round(Number(minimumMinutes) || 1));
    let duration = modulo(targetTod - currentTod, DAY);
    if (duration < minimum) duration += Math.ceil((minimum - duration) / DAY) * DAY;
    return duration;
  };
  const formatClock = (minute) => {
    const value = minuteOfDay(minute);
    return `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
  };
  const calendarDate = (minute) => {
    let remaining = dayOfYear(minute) - 1;
    let monthIndex = 0;
    while (remaining >= MONTH_DAYS[monthIndex]) {
      remaining -= MONTH_DAYS[monthIndex];
      monthIndex += 1;
    }
    const weekdayIndex = modulo(dayIndex(minute), WEEKDAYS.length);
    return {
      weekday: WEEKDAYS[weekdayIndex],
      day: remaining + 1,
      month: MONTHS[monthIndex],
      isSunday: weekdayIndex === 6,
    };
  };
  const lunarPhase = (minute) => modulo(Number(minute), LUNAR_CYCLE) / LUNAR_CYCLE;
  const calendar = Object.freeze({
    ...source, addDays, addMinutes, calendarDate, dayIndex, dayOfYear, dayStart,
    elapsedSince, formatClock, lunarPhase, minuteOfDay, minutesUntilDailyTimeWithMinimum,
    nextMidnightAtOrAfter,
  });
  globalThis.strategicCalendar = calendar;
  if (typeof module !== "undefined") module.exports = calendar;
})();
