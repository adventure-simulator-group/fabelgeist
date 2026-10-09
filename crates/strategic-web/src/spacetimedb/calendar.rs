//! Convert generated database minutes before calendar calculations.

use adventuresim_stdb_client::StrategicMinute as WireMinute;
use adventuresim_world_schema::calendar::StrategicMinute;

pub(crate) fn calendar_minute(value: &WireMinute) -> StrategicMinute {
    StrategicMinute::new(value.minutes)
}

pub(crate) fn calendar_countdown_days(now: StrategicMinute, target: Option<&WireMinute>) -> u64 {
    now.days_until_ceil(target.map_or(now, calendar_minute))
}
