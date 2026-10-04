"""Exercise calendar guard coverage for typed domain and storage fields."""

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import check_calendar_api


class CalendarApiGuardTests(unittest.TestCase):
    def test_day_projection_is_allowed_only_in_its_exact_shared_owner(self) -> None:
        projection = (
            "pub const fn start(self) -> StrategicMinute { "
            "StrategicMinute::new(self.0.saturating_mul(MINUTES_PER_DAY)) }"
        )
        # A consumer cannot copy even the same operation and receiver spelling.
        result, errors = self.check_fixture(projection)
        self.assertEqual(result, 1)
        self.assertIn("construct calendar days", errors)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owner = root / check_calendar_api.CALENDAR_DAY
            owner.parent.mkdir(parents=True)
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {}),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
            ):
                owner.write_text(projection)
                self.assertEqual(check_calendar_api.main(), 0)
                owner.write_text(
                    projection + "\nfn copied(day: u64) -> StrategicMinute { "
                    "StrategicMinute::new(day.saturating_mul(MINUTES_PER_DAY)) }"
                )
                with contextlib.redirect_stderr(io.StringIO()) as errors:
                    self.assertEqual(check_calendar_api.main(), 1)
                self.assertIn("construct calendar days", errors.getvalue())
                # Owning this conversion does not exempt other calendar leaks.
                owner.write_text(projection + "\nfn copied(absolute_minute: u64) {}")
                with contextlib.redirect_stderr(io.StringIO()) as errors:
                    self.assertEqual(check_calendar_api.main(), 1)
                self.assertIn("raw calendar declarations", errors.getvalue())

    def test_residence_clock_retains_character_identity_and_shared_minute(self):
        relative = Path("crates/adventuresim-stdb-module/src/residence.rs")
        declaration = check_calendar_api.REQUIRED_SHARED_FIELDS[relative]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / relative
            source.parent.mkdir(parents=True)
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {
                    relative: declaration,
                }),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
            ):
                source.write_text(declaration + " { todo!() }")
                self.assertEqual(check_calendar_api.main(), 0)
                for invalid in (
                    declaration.replace(
                        "adventuresim_core::identity::CharacterId", "u64"
                    ),
                    declaration.replace("Result<StrategicMinute", "Result<u64"),
                ):
                    with self.subTest(declaration=invalid):
                        source.write_text(invalid + " { todo!() }")
                        with contextlib.redirect_stderr(io.StringIO()) as errors:
                            self.assertEqual(check_calendar_api.main(), 1)
                        self.assertIn("restore shared calendar field", errors.getvalue())

    def test_departure_clock_retains_members_minute_and_concrete_error(self):
        relative = Path("crates/adventuresim-stdb-module/src/time/departure.rs")
        declaration = check_calendar_api.REQUIRED_SHARED_FIELDS[relative]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / relative
            source.parent.mkdir(parents=True)
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {
                    relative: declaration,
                }),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
            ):
                source.write_text(declaration + " todo!() }")
                self.assertEqual(check_calendar_api.main(), 0)
                for invalid in (
                    declaration.replace(
                        "adventuresim_core::identity::CharacterId", "u64"
                    ),
                    declaration.replace("Result<StrategicMinute", "Result<u64"),
                    declaration.replace("DepartureClockError", "String"),
                    declaration.replace("Result<StrategicMinute", "Result<Option<StrategicMinute>"),
                ):
                    with self.subTest(declaration=invalid):
                        source.write_text(invalid + " todo!() }")
                        with contextlib.redirect_stderr(io.StringIO()) as errors:
                            self.assertEqual(check_calendar_api.main(), 1)
                        self.assertIn("restore shared calendar field", errors.getvalue())

    def test_partial_day_durations_are_not_whole_day_advances(self):
        for operation in ("saturating_add_minutes", "checked_add_minutes", "saturating_sub_minutes"):
            for prefix in ("", "adventuresim_world_schema::calendar::"):
                with self.subTest(operation=operation, prefix=prefix):
                    self.assertEqual(
                        self.check_fixture(f"let noon = now.{operation}({prefix}MINUTES_PER_DAY / 2);"),
                        (0, ""),
                    )
                    result, errors = self.check_fixture(
                        f"let tomorrow = now.{operation}({prefix}MINUTES_PER_DAY /* one day */);"
                    )
                    self.assertEqual(result, 1)
                    self.assertIn("calendar days", errors)

    def check_fixture(self, rust: str, browser: str = "") -> tuple[int, str]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            module = root / "crates/adventuresim-stdb-module/src/model.rs"
            script = root / "crates/strategic-web/static/travel-planner.js"
            module.parent.mkdir(parents=True)
            script.parent.mkdir(parents=True)
            module.write_text(rust)
            script.write_text(browser)
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {}),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
                contextlib.redirect_stderr(io.StringIO()) as errors,
            ):
                result = check_calendar_api.main()
            return result, errors.getvalue()

    def test_rejects_equivalent_absolute_minute_leaks_and_accepts_owned_operations(self):
        rejected = (
            (
                "fn next_due(now: StrategicMinute) -> Result<u64, String> { Ok(now.get()) }",
                "",
                "return typed absolute minutes",
            ),
            (
                "fn next_due(moment: StrategicMinute) -> u64 { moment.get() }",
                "",
                "return typed absolute minutes",
            ),
            (
                "fn next_due(moment: StrategicMinute) -> u64 { let raw = moment.get(); raw }",
                "",
                "return typed absolute minutes",
            ),
            (
                "fn timeline(now: StrategicMinute) -> Option<(u64, u64)> { Some((1, now.get())) }",
                "",
                "return typed absolute minutes",
            ),
            (
                "let due = now.get(); let current = StrategicMinute::new(due);",
                "",
                "keep absolute minutes typed",
            ),
            (
                "let due = now.saturating_add_minutes(u64::from(days) * MINUTES_PER_DAY);",
                "",
                "advance calendar days",
            ),
            (
                "let tomorrow = now.saturating_add_minutes(MINUTES_PER_DAY);",
                "",
                "advance calendar days",
            ),
            (
                "let midnight = now.saturating_add_minutes(24 * 60);",
                "",
                "advance calendar days",
            ),
            (
                "let morning = now.checked_add_minutes(MINUTES_PER_DAY - dusk + dawn);",
                "",
                "advance calendar days",
            ),
            (
                "let yesterday = now.saturating_sub_minutes(3 * MINUTES_PER_DAY);",
                "",
                "retreat calendar days",
            ),
            (
                "let next = now.saturating_add_minutes(MINUTES_PER_YEAR);",
                "",
                "advance calendar years",
            ),
            (
                "let wait = (start + MINUTES_PER_DAY - current) % MINUTES_PER_DAY;",
                "",
                "wrap daily clock time",
            ),
            (
                "let present = start_minute <= minute && minute < end_minute;",
                "",
                "check daily windows",
            ),
            (
                "let start = StrategicMinute::new(interval * WEATHER_INTERVAL_MINUTES);",
                "",
                "construct period starts",
            ),
            (
                "let now = anchor.day_start().saturating_add_minutes("
                "u64::from(start_minute_of_day)).wrapping_day_offset(elapsed);",
                "",
                "compose frozen day time",
            ),
            ("", "const arrival = departureMinute + durationMinutes;", "browser calendar adapter"),
            (
                "",
                "const clock = window.strategicCharacterMinutes; const due = clock + wait;",
                "browser calendar adapter",
            ),
            (
                "",
                "const clock = window.strategicCharacterMinutes; const copied = clock;"
                " const due = copied + wait;",
                "browser calendar adapter",
            ),
        )
        for rust, browser, reason in rejected:
            with self.subTest(reason=reason, rust=rust, browser=browser):
                result, errors = self.check_fixture(rust, browser)
                self.assertEqual(result, 1)
                self.assertIn(reason, errors)

        accepted = (
            "fn remaining_minutes(now: StrategicMinute) -> Result<u64, String> {"
            " Ok(u64::from(now.minute_of_day())) }",
            "fn next_due(now: StrategicMinute) -> Result<StrategicMinute, String> { Ok(now) }",
            "let current = now.saturating_add_days(u64::from(days));",
            "let yesterday = now.saturating_sub_days(1);",
            "let next = now.saturating_add_years(1);",
            "let wait = now.minutes_until_time_of_day(start_minute);",
            "let present = now.contains_daily_window(start_minute, end_minute);",
            "let start = StrategicMinute::checked_period_start_for_index("
            "interval, WEATHER_INTERVAL_MINUTES);",
            "let now = anchor.with_wrapped_time_of_day(start_minute_of_day, elapsed);",
        )
        for rust in accepted:
            with self.subTest(rust=rust):
                self.assertEqual(self.check_fixture(rust), (0, ""))
        self.assertEqual(
            self.check_fixture("", "const arrival = calendar.addMinutes(departureMinute, durationMinutes);"),
            (0, ""),
        )
        self.assertEqual(
            self.check_fixture(
                "",
                "const clock = window.strategicCharacterMinutes;"
                " const due = calendar.addMinutes(clock, wait);",
            ),
            (0, ""),
        )

    def test_rejects_raw_domain_parameters_and_stored_absolute_minutes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            core = root / "crates/adventuresim-core/src/strategic_state.rs"
            module = root / "crates/adventuresim-stdb-module/src/model.rs"
            browser = root / "crates/strategic-web/static/travel-planner.js"
            client = root / "crates/strategic-web/src/routes/investigation.rs"
            core.parent.mkdir(parents=True)
            module.parent.mkdir(parents=True)
            browser.parent.mkdir(parents=True)
            client.parent.mkdir(parents=True)
            core.write_text(
                "fn parse(accepted_at: Option<u64>, resolved_minute: Option<u64>) {}\n"
            )
            module.write_text(
                "pub effective_minute_index: u64,\n"
                "pub active_from: u64,\n"
                "pub gap_to: Option<u64>,\n"
            )
            browser.write_text("const day = Math.floor(absoluteMinute / DAY);\n")
            client.write_text(
                "rows.sort_by_key(|row| row.recorded_at.minutes);\n"
                "let latest = rows.iter().map(|row| row.recorded_at.minutes).max();\n"
            )
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {}),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
                contextlib.redirect_stderr(io.StringIO()) as errors,
            ):
                self.assertEqual(check_calendar_api.main(), 1)
            self.assertIn("raw calendar declarations", errors.getvalue())
            self.assertIn("type stored calendar indexes", errors.getvalue())
            self.assertEqual(errors.getvalue().count("type stored absolute minutes"), 2)
            self.assertIn("use the browser calendar adapter", errors.getvalue())
            self.assertEqual(
                errors.getvalue().count("convert wire minutes before ordering"), 2
            )

            core.write_text(
                "fn parse(accepted_at: Option<StrategicMinute>, "
                "resolved_minute: Option<StrategicMinute>) {}\n"
            )
            module.write_text(
                "pub effective_minute: StrategicMinute,\n"
                "pub active_from: StrategicMinute,\n"
                "pub gap_to: Option<StrategicMinute>,\n"
            )
            browser.write_text("const day = calendar.dayOfYear(absoluteMinute);\n")
            client.write_text(
                "rows.sort_by_key(|row| calendar_minute(&row.recorded_at));\n"
                "let latest = rows.iter().map(|row| calendar_minute(&row.recorded_at)).max();\n"
            )
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {}),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
            ):
                self.assertEqual(check_calendar_api.main(), 0)

    def test_rejects_internal_timestamp_tuple_and_calendar_arithmetic(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            module = root / "crates/adventuresim-stdb-module/src/model.rs"
            browser = root / "crates/strategic-web/static/travel-planner.js"
            module.parent.mkdir(parents=True)
            browser.parent.mkdir(parents=True)
            module.write_text(
                "fn admitted(now: StrategicMinute) -> Option<(String, u64)> {\n"
                "    Some((String::new(), now.get()))\n"
                "}\n"
                "fn enforce_temporal_scope(now: StrategicMinute) -> Result<u64, String> {\n"
                "    Ok(now.get())\n"
                "}\n"
                "fn stable_owned_open_cases(\n"
                "    rows: impl IntoIterator<Item = (u64, String, DomainCaseStatus, u64)>,\n"
                ") {}\n"
                "let day = StrategicMinute::new(index.saturating_mul(MINUTES_PER_DAY));\n"
                "let local = (u64::from(start) + elapsed) % MINUTES_PER_DAY;\n"
            )
            browser.write_text(
                "const segment = nextBoundary - absolute;\n"
                "const arrival = departure + total * fraction;\n"
                "const wake = (target - current + DAY_MINUTES) % DAY_MINUTES;\n"
            )
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {}),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
                contextlib.redirect_stderr(io.StringIO()) as errors,
            ):
                self.assertEqual(check_calendar_api.main(), 1)
            self.assertIn("keep absolute-minute tuple values typed", errors.getvalue())
            self.assertIn("construct calendar days", errors.getvalue())
            self.assertIn("wrap day time", errors.getvalue())
            self.assertIn("return typed absolute minutes", errors.getvalue())
            self.assertIn("keep case ordering timestamps typed", errors.getvalue())
            self.assertEqual(errors.getvalue().count("browser calendar adapter"), 3)

            module.write_text(
                "fn admitted(now: StrategicMinute) -> Option<(String, StrategicMinute)> {\n"
                "    Some((String::new(), now))\n"
                "}\n"
                "fn enforce_temporal_scope(now: StrategicMinute) "
                "-> Result<StrategicMinute, String> { Ok(now) }\n"
                "fn stable_owned_open_cases(\n"
                "    rows: impl IntoIterator<Item = "
                "(u64, String, DomainCaseStatus, StrategicMinute)>,\n"
                ") {}\n"
                "let day = StrategicMinute::day_start_for_index(index);\n"
                "let local = anchor.with_wrapped_time_of_day(start, elapsed);\n"
            )
            browser.write_text(
                "const segment = calendar.elapsedSince(nextBoundary, absolute);\n"
                "const arrival = calendar.addMinutes(departure, total * fraction);\n"
                "const wake = calendar.minutesUntilDailyTimeWithMinimum(current, target, DAY_MINUTES);\n"
            )
            with (
                patch.object(check_calendar_api, "ROOT", root),
                patch.object(check_calendar_api, "REQUIRED_SHARED_FIELDS", {}),
                patch.object(check_calendar_api, "REQUIRED_TYPED_RETURNS", {}),
            ):
                self.assertEqual(check_calendar_api.main(), 0)


if __name__ == "__main__":
    unittest.main()
