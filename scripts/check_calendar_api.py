"""Keep calendar ownership in the shared world schema module."""

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
CALENDAR = Path("crates/adventuresim-world-schema/src/calendar.rs")
GENERATED = Path("crates/adventuresim-stdb-client/src")
BROWSER_CALENDAR = Path("crates/strategic-web/static/strategic-calendar.js")
FORBIDDEN = (
    re.compile(r"\b(?:struct|type)\s+(?:NameBirthYear|WorldMinute|ChildBirthMinute)\b"),
    re.compile(
        r"\bconst\s+(?:MINUTES_PER_DAY|DAYS_PER_YEAR|MINUTES_PER_YEAR|"
        r"WORLD_START_DAY_OF_YEAR|WORLD_START_MINUTE)\b"
    ),
    re.compile(
        r"\bstrategic_time::(?:MINUTES_PER_DAY|DAYS_PER_YEAR|MINUTES_PER_YEAR|"
        r"WORLD_START_DAY_OF_YEAR|WORLD_START_MINUTE)\b"
    ),
    re.compile(r"\bfn\s+(?:world_year_at|birth_year_from_age)\b"),
)
LOCAL_CALENDAR_WRAPPER = re.compile(
    r"\bstruct\s+(?:\w*Year\s*\(\s*i32|"
    r"\w*(?:World|Strategic|Birth|Record)Minute\s*\(\s*u64)\b",
    re.MULTILINE,
)
CORE_RAW_ABSOLUTE_FIELD = re.compile(
    r"\bpub\s+(?:\w+_at|(?:now|world|starting|current)_minute):\s*"
    r"(?:Option<)?u64\b"
)
RAW_ABSOLUTE_ARITHMETIC = re.compile(
    r"\b(?:character_time|patient_time|actor_time|time)\.minutes\s*"
    r"(?:[+/%-]|\.(?:checked|saturating)_(?:add|sub)\b)"
)
CLIENT_RAW_MINUTE_ORDERING = re.compile(
    r"\bsort_by_key\([^;\n]*\.minutes\b|"
    r"\.map\(\|[^|]+\|[^;]*?\.minutes\)\s*\.max\(",
    re.DOTALL,
)
STDB_RAW_ABSOLUTE_FIELD = re.compile(
    r"\bpub\s+(\w*(?:_at|_minute|_through|_from|_until|_to|_since)):\s*"
    r"(?:Option<)?u64\b"
)
STDB_RAW_MINUTE_INDEX = re.compile(r"\bpub\s+\w*minute_index:\s*u64\b")
BROWSER_RAW_CALENDAR_ARITHMETIC = re.compile(
    r"\b(?:absoluteMinute|characterMinutes|officialMinutes|currentMinutes)\s*"
    r"[%/]\s*(?:DAY|DAY_MINUTES|LUNAR_CYCLE)\b|"
    r"Math\.(?:floor|ceil)\([^)]*/\s*(?:DAY|DAY_MINUTES|calendar\.minutesPerDay)\)|"
    r"\bdays\s*\*\s*calendar\.minutesPerDay\b|"
    r"\b(?:nextBoundary\s*-\s*absolute|midnight\s*-\s*departure|"
    r"departure\s*\+\s*total\s*\*|state\.start\s*\+\s*hours\s*\*|"
    r"normalizeMinute\(targetMinute\)\s*-\s*currentTod)|"
    r"\(\s*target\s*-\s*current\s*\+\s*(?:DAY|DAY_MINUTES)\s*\)"
    r"\s*%\s*(?:DAY|DAY_MINUTES)|"
    r"\b(?:departureMinute|arrivalMinute|worldMinute|absoluteMinute|"
    r"characterMinutes|officialMinutes)\s*[+-]\s*[\w$]+\b"
)
BROWSER_ABSOLUTE_IDENTIFIERS = frozenset({
    "absoluteMinute", "characterMinutes", "officialMinutes",
    "departureMinute", "arrivalMinute", "worldMinute",
    "strategicCharacterMinutes",
})
BROWSER_ALIAS_ASSIGNMENT = re.compile(
    r"\b(?:const|let|var)\s+(?P<alias>[A-Za-z_$][\w$]*)\s*=\s*"
    r"(?:(?:window|globalThis)\.)?(?P<source>[A-Za-z_$][\w$]*)\s*(?:;|$)"
)
RAW_CALENDAR_DAY_CONSTRUCTION = re.compile(
    r"StrategicMinute::new\([^;]{0,180}"
    r"(?:\b\w+\s*\*\s*MINUTES_PER_DAY|"
    r"\b\w+\.saturating_mul\(MINUTES_PER_DAY\))",
    re.DOTALL,
)
RAW_CALENDAR_DAY_WRAP = re.compile(
    r"u64::from\([^)]*\)\s*\+\s*\w+\s*\)\s*%\s*MINUTES_PER_DAY"
)
RAW_ABSOLUTE_TUPLE_RETURN = re.compile(
    r"\bfn\s+\w+\s*\([^{};]*?\b\w+:\s*StrategicMinute"
    r"[^{};]*?\)\s*->\s*Option<\(String,\s*u64\)>",
    re.DOTALL,
)
RAW_ABSOLUTE_RESULT_RETURN = re.compile(
    r"\bfn\s+\w*(?:temporal_scope|departure_time|canonical_courtship_pair)"
    r"\s*\([^{};]*?\)\s*->\s*Result<\s*(?:Option<\s*)?u64\b",
    re.DOTALL,
)
RAW_TYPED_MINUTE_RETURN = re.compile(
    r"\bfn\s+\w+\s*\([^{};]*?\b(?P<minute>\w+):\s*StrategicMinute"
    r"[^{};]*?\)\s*->\s*(?:Result|Option)<[^{};]*u64[^{};]*>"
    r"\s*\{[^}]{0,500}(?:Ok|Some)\([^;{}]*"
    r"\b(?P=minute)\.get\(\)",
    re.DOTALL,
)
RAW_TYPED_MINUTE_DIRECT_RETURN = re.compile(
    r"\bfn\s+\w+\s*\([^{};]*?\b(?P<minute>\w+):\s*StrategicMinute"
    r"[^{};]*?\)\s*->\s*u64\s*\{\s*(?:return\s+)?"
    r"(?P=minute)\.get\(\)\s*;?\s*\}",
    re.DOTALL,
)
RAW_TYPED_MINUTE_LOCAL_RETURN = re.compile(
    r"\bfn\s+\w+\s*\([^{};]*?\b(?P<minute>\w+):\s*StrategicMinute"
    r"[^{};]*?\)\s*->\s*u64\s*\{\s*let\s+(?P<raw>\w+)"
    r"\s*=\s*(?P=minute)\.get\(\);\s*(?P=raw)\s*\}",
    re.DOTALL,
)
RAW_MINUTE_ROUND_TRIP = re.compile(
    r"\blet\s+(?P<raw>\w+)\s*=\s*[^;]{0,120}\.get\(\);"
    r"\s*let\s+\w+\s*=\s*StrategicMinute::new\(\s*(?P=raw)\s*\)",
    re.DOTALL,
)
RAW_DAY_ADVANCE = re.compile(
    r"\.(?:saturating|checked)_add_minutes\(\s*(?:(?:u64::from\([^)]*\)|\d+)"
    r"\s*\*\s*)?(?:adventuresim_world_schema::calendar::)?MINUTES_PER_DAY\b(?!\s*/(?![/*]))",
    re.DOTALL,
)
RAW_DAY_RETREAT = re.compile(
    r"\.saturating_sub_minutes\(\s*(?:(?:u64::from\([^)]*\)|\d+)"
    r"\s*\*\s*)?(?:adventuresim_world_schema::calendar::)?MINUTES_PER_DAY\b(?!\s*/(?![/*]))"
)
RAW_YEAR_ADVANCE = re.compile(
    r"\.(?:saturating|checked)_add_minutes\(\s*(?:(?:u64::from\([^)]*\)|\d+)"
    r"\s*\*\s*)?(?:adventuresim_world_schema::calendar::)?MINUTES_PER_YEAR"
)
RAW_DAILY_CLOCK_WRAP = re.compile(
    r"\(\s*\w+\s*\+\s*MINUTES_PER_DAY\s*-\s*\w+\s*\)"
    r"\s*%\s*MINUTES_PER_DAY"
)
RAW_DAILY_WINDOW_RANGE = re.compile(
    r"\b(?:\w+\.)?start_minute\s*<=\s*minute\s*&&\s*"
    r"minute\s*<\s*(?:\w+\.)?end_minute"
)
RAW_LITERAL_DAY_ADVANCE = re.compile(
    r"\.(?:saturating|checked)_add_minutes\(\s*(?:24\s*\*\s*60|"
    r"60\s*\*\s*24|1_?440)\s*\)"
)
RAW_PERIOD_START = re.compile(
    r"StrategicMinute::new\(\s*\w+\s*\*\s*\w*INTERVAL_MINUTES\s*\)"
)
RAW_FROZEN_DAY_COMPOSITION = re.compile(
    r"\.day_start\(\)\s*\.saturating_add_minutes\("
    r"\s*u64::from\([^)]*minute_of_day\)\)\s*\.wrapping_day_offset\(",
    re.DOTALL,
)
RAW_CASE_ORDERING_KEY = re.compile(
    r"IntoIterator<Item\s*=\s*\([^)]*DomainCaseStatus,\s*u64\)>"
)
STDB_LOCAL_MINUTE_FIELDS = {
    "available_at_elapsed_minute",
    "camp_elapsed_minute",
    "camp_movement_minute",
    "elapsed_start_minute",
    "journey_elapsed_minute",
    "journey_movement_minute",
    "movement_minute",
}
RAW_DOMAIN_DECLARATION = re.compile(
    r"\b(?:absolute_minute|lunar_phase_minute|world_minute|strategic_minute|"
    r"current_minute|starting_minute|now_minutes|interval_end_minute|"
    r"preview_departure_minute|configured_starting_minute|"
    r"departure_minute|weather_interval_start|next_due_minute|"
    r"effective_minute|resolved_minute|accepted_at|paid_at|"
    r"active_from|active_until|gap_from|gap_to|"
    r"occurred_at_minute|expires_at_minute|"
    r"birth_year|world_year|calendar_year|recorded_at|latest_update_at)"
    r":\s*(?:Option<)?(?:u64|i64|i32|u32)\b"
)
# These declarations are command-line and manifest serialization boundaries.
# An additional raw declaration in one of these files still fails the check.
BOUNDARY_RAW_COUNTS = {
    Path("crates/adventuresim-world-import/src/manifest.rs"): 1,
    Path("crates/adventuresim-tactical-server-dispatcher/src/bin/materialize-real-world-scene.rs"): 1,
    Path("crates/adventuresim-tactical-client/src/tactical_scene_viewer_main.rs"): 1,
    Path("crates/adventuresim-tactical-client/src/tactical_scene_viewer.rs"): 1,
    Path("crates/adventuresim-tactical-client/src/tactical_sky_viewer.rs"): 1,
    Path("crates/adventuresim-tactical-client/src/tactical_scene_viewer/manifest.rs"): 1,
}
REQUIRED_SHARED_FIELDS = {
    Path("crates/adventuresim-world-schema/src/lib.rs"):
        "pub world_year: calendar::CalendarYear",
    Path("crates/adventuresim-world-schema/src/person_names.rs"):
        "pub birth_year: CalendarYear",
    Path("crates/adventuresim-tactical-core/src/scene_input/descriptor.rs"):
        "pub lunar_phase_minute: StrategicMinute",
    Path("crates/adventuresim-tactical-core/src/scene_input/environment.rs"):
        "pub absolute_minute: StrategicMinute",
    Path("crates/adventuresim-stdb-module/src/tactical.rs"):
        "pub lunar_phase_minute: StrategicMinute",
    Path("crates/adventuresim-world-import/src/builder.rs"):
        "year: CalendarYear",
    Path("crates/adventuresim-world-import/src/draft.rs"):
        "pub(crate) year: CalendarYear",
    Path("crates/adventuresim-world-import/src/sources/viabundus/chronology.rs"):
        "from: Option<CalendarYear>",
    Path("crates/strategic-web/src/medical.rs"):
        "pub administered_at: StrategicMinute",
    Path("crates/adventuresim-stdb-module/src/time/clock.rs"):
        "pub fn refresh_clock(ctx: &ReducerContext) -> Result<StrategicMinute, String>",
    Path("crates/adventuresim-stdb-module/src/time/activities.rs"):
        "let starting_minute = character_time.minutes",
    Path("crates/adventuresim-stdb-module/src/time/settlement_rest.rs"):
        "let starting_minute = character_time.minutes",
    Path("crates/adventuresim-stdb-module/src/local_problem.rs"):
        "fn official_minute(ctx: &ReducerContext) -> StrategicMinute",
    Path("crates/adventuresim-stdb-module/src/investigation/claims.rs"):
        "fn official_minute(ctx: &ReducerContext) -> StrategicMinute",
    Path("crates/adventuresim-stdb-module/src/investigation/capabilities.rs"):
        "fn character_strategic_minute(ctx: &ReducerContext, character_id: u64) -> StrategicMinute",
    Path("crates/adventuresim-stdb-module/src/residence.rs"):
        "fn residence_now(ctx: &ReducerContext, character_id: u64) -> Result<StrategicMinute, String>",
    Path("crates/adventuresim-stdb-module/src/relationship/model.rs"):
        ") -> Result<StrategicMinute, String> {",
    Path("crates/adventuresim-strategic-sim/src/live_core/discovery_policy.rs"):
        "DomainCaseStatus, StrategicMinute)",
    Path("crates/adventuresim-stdb-module/src/corpse.rs"):
        "fn now(ctx: &ReducerContext, actor_id: u64) -> Result<StrategicMinute, String>",
    Path("crates/adventuresim-strategic-sim/src/live_core/failure.rs"):
        "fn public_party_elapsed_max(&self, party_id: &str) -> StrategicMinute",
    Path("crates/adventuresim-strategic-sim/src/live_core/expedition_policy.rs"):
        "pub(super) elapsed_minutes: StrategicMinute",
    Path("crates/strategic-web/src/routes/mod.rs"):
        ") -> Result<StrategicMinute, String> {",
}
REQUIRED_TYPED_RETURNS = {
    Path("crates/adventuresim-stdb-module/src/time/settlement_rest.rs"):
        ") -> Result<Option<StrategicMinute>, String> {",
    Path("crates/adventuresim-stdb-module/src/residence.rs"):
        ") -> Option<(String, StrategicMinute)> {",
}


def browser_alias_arithmetic(contents: str) -> list[int]:
    """Find arithmetic on local aliases of absolute browser minutes."""
    aliases = set(BROWSER_ABSOLUTE_IDENTIFIERS)
    findings = []
    for number, line in enumerate(contents.splitlines(), 1):
        for assignment in BROWSER_ALIAS_ASSIGNMENT.finditer(line):
            if assignment.group("source") in aliases:
                aliases.add(assignment.group("alias"))
        for alias in aliases - BROWSER_ABSOLUTE_IDENTIFIERS:
            escaped = re.escape(alias)
            if re.search(rf"\b{escaped}\s*[+*/%-]|[+*/%-]\s*{escaped}\b", line):
                findings.append(number)
                break
    return findings


def main() -> int:
    violations = []
    for source in sorted((ROOT / "crates").rglob("*.rs")):
        relative = source.relative_to(ROOT)
        if relative == CALENDAR or relative.is_relative_to(GENERATED):
            continue
        contents = source.read_text()
        test_tail = re.search(r"#\[cfg\(test\)\]\s*mod tests\b", contents)
        production = contents[:test_tail.start()] if test_tail else contents
        if (
            relative.parts[:2] != ("crates", "adventuresim-stdb-module")
            and "tests" not in relative.parts
        ):
            raw_lines = [
                number for number, line in enumerate(production.splitlines(), 1)
                if RAW_DOMAIN_DECLARATION.search(line)
            ]
            allowed = BOUNDARY_RAW_COUNTS.get(relative, 0)
            if len(raw_lines) != allowed:
                violations.append(
                    f"{relative}: found {len(raw_lines)} raw calendar declarations; "
                    f"expected {allowed} explicit boundary declarations"
                )
        for match in LOCAL_CALENDAR_WRAPPER.finditer(contents):
            number = contents.count("\n", 0, match.start()) + 1
            violations.append(f"{relative}:{number}: use the shared calendar types")
        for match in RAW_ABSOLUTE_ARITHMETIC.finditer(production):
            number = production.count("\n", 0, match.start()) + 1
            violations.append(f"{relative}:{number}: move absolute-minute arithmetic to calendar")
        for pattern, reason in (
            (RAW_CALENDAR_DAY_CONSTRUCTION, "construct calendar days through StrategicMinute"),
            (RAW_CALENDAR_DAY_WRAP, "wrap day time through StrategicMinute"),
            (RAW_ABSOLUTE_TUPLE_RETURN, "keep absolute-minute tuple values typed"),
            (RAW_ABSOLUTE_RESULT_RETURN, "return typed absolute minutes"),
            (RAW_TYPED_MINUTE_RETURN, "return typed absolute minutes"),
            (RAW_TYPED_MINUTE_DIRECT_RETURN, "return typed absolute minutes"),
            (RAW_TYPED_MINUTE_LOCAL_RETURN, "return typed absolute minutes"),
            (RAW_MINUTE_ROUND_TRIP, "keep absolute minutes typed through domain logic"),
            (RAW_DAY_ADVANCE, "advance calendar days through StrategicMinute"),
            (RAW_DAY_RETREAT, "retreat calendar days through StrategicMinute"),
            (RAW_YEAR_ADVANCE, "advance calendar years through StrategicMinute"),
            (RAW_DAILY_CLOCK_WRAP, "wrap daily clock time through StrategicMinute"),
            (RAW_DAILY_WINDOW_RANGE, "check daily windows through StrategicMinute"),
            (RAW_LITERAL_DAY_ADVANCE, "advance calendar days through StrategicMinute"),
            (RAW_PERIOD_START, "construct period starts through StrategicMinute"),
            (RAW_FROZEN_DAY_COMPOSITION, "compose frozen day time through StrategicMinute"),
            (RAW_CASE_ORDERING_KEY, "keep case ordering timestamps typed"),
        ):
            for match in pattern.finditer(production):
                number = production.count("\n", 0, match.start()) + 1
                violations.append(f"{relative}:{number}: {reason}")
        if relative.parts[:2] in {
            ("crates", "adventuresim-strategic-sim"),
            ("crates", "strategic-web"),
        }:
            for match in CLIENT_RAW_MINUTE_ORDERING.finditer(production):
                number = production.count("\n", 0, match.start()) + 1
                violations.append(
                    f"{relative}:{number}: convert wire minutes before ordering"
                )
        for number, line in enumerate(contents.splitlines(), 1):
            if line.lstrip().startswith("//"):
                continue
            if relative.parts[:2] == ("crates", "adventuresim-core") and CORE_RAW_ABSOLUTE_FIELD.search(line):
                violations.append(f"{relative}:{number}: type absolute domain minutes")
            if relative.parts[:2] == ("crates", "adventuresim-stdb-module"):
                if STDB_RAW_MINUTE_INDEX.search(line):
                    violations.append(f"{relative}:{number}: type stored calendar indexes")
                match = STDB_RAW_ABSOLUTE_FIELD.search(line)
                if match and match.group(1) not in STDB_LOCAL_MINUTE_FIELDS and not (
                    match.group(1) == "start_minute"
                    and relative == Path("crates/adventuresim-stdb-module/src/strategic/authority_model.rs")
                ):
                    violations.append(f"{relative}:{number}: type stored absolute minutes")
            if any(pattern.search(line) for pattern in FORBIDDEN):
                violations.append(f"{relative}:{number}: use the shared calendar API")
    for relative, declaration in REQUIRED_SHARED_FIELDS.items():
        if declaration not in (ROOT / relative).read_text():
            violations.append(f"{relative}: restore shared calendar field {declaration}")
    for relative, declaration in REQUIRED_TYPED_RETURNS.items():
        if declaration not in (ROOT / relative).read_text():
            violations.append(f"{relative}: restore typed calendar return {declaration}")
    for source in sorted((ROOT / "crates/strategic-web/static").glob("*.js")):
        relative = source.relative_to(ROOT)
        if relative == BROWSER_CALENDAR:
            continue
        contents = source.read_text()
        for match in BROWSER_RAW_CALENDAR_ARITHMETIC.finditer(contents):
            number = contents.count("\n", 0, match.start()) + 1
            violations.append(f"{relative}:{number}: use the browser calendar adapter")
        for number in browser_alias_arithmetic(contents):
            violations.append(f"{relative}:{number}: use the browser calendar adapter for aliased minutes")
    if violations:
        print("\n".join(violations), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
