# Shared strategic calendar

`calendar::CalendarYear` is a positive year in the strategic calendar. Construct
it with `CalendarYear::new` or deserialize it; zero and negative years are
rejected. Use `birth_year_for_age` for age-derived naming and record years.

`calendar::StrategicMinute` is an absolute minute since the start of calendar
year 1544. `WORLD_START_MINUTE` identifies the game's August 20 starting date.
Use its methods for calendar conversion, elapsed time, and bounded arithmetic.
Durations and journey-local movement minutes are separate quantities and are
not absolute strategic instants.

Stored absolute instants use `StrategicMinute`, including SpacetimeDB rows.
Transport adapters convert generated client wire structs to the shared type
before calendar calculations. Queries over typed instants filter rows and
select the earliest matching instant explicitly. Historical birth minutes may
precede the strategic epoch and remain signed coordinates; calendar methods
own calculations that combine them with an absolute `StrategicMinute`.

The strategic web shell supplies calendar constants to the browser. Its
`strategic-calendar.js` adapter owns browser-only date, clock, and lunar
presentation calculations; other browser scripts call that adapter instead of
repeating calendar formulas.

Run `python3 scripts/check_calendar_api.py` from the repository root to check
for local wrappers, raw domain declarations, and browser aliases of absolute
minutes. The Rust quality check uses parsed syntax to reject arithmetic on
unwrapped calendar values and arithmetic inside calendar constructors outside
the owning module. Both checks run in the repository lint workflow. Keep raw
conversions in source, storage, or presentation boundary adapters; ordinary
domain logic should retain the shared types.
