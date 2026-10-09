# Shared strategic calendar

## Database serialization boundary

The `spacetimedb` feature derives database serialization through
`spacetimedb-lib`, which owns the shared type representation without linking
the module runtime. Host catalog build scripts can therefore enable the same
feature safely. The database module discovers these types through its tables
and reducers; shared types do not export standalone runtime registration
functions. `just verify-db-client` checks that this boundary produces the
committed client bindings.

## Calendar values

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

## Demographic vocabulary

`Sex` is the shared female/male type for personality, starting characters,
personal names, and sex-specific quest rules. `Culture` currently has only the
German variant because the catalog has only German names. It is distinct from
`OralLanguage` or `WrittenLanguage`.

Unknown private sex in observer-facing quest data is `None`, serialized as
`null`. Generated SpacetimeDB client types remain transport types and require
explicit conversion at client boundaries.

## Imported source identity

`source_package::SourcePackageDigest` identifies an immutable imported source
package with a checked lowercase SHA-256 digest. Geographic producers, scene
capture and map presentation use this same type. Malformed identities return
`SourcePackageDigestError` at the decoding boundary. Geometry content and scene
placement bindings remain separate identities.
