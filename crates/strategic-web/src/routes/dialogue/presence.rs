//! Daily NPC presence check at a strategic calendar minute.

pub(super) fn npc_presence_contains(
    start_minute: u16,
    end_minute: u16,
    minute: adventuresim_world_schema::calendar::StrategicMinute,
) -> bool {
    minute.contains_daily_window(start_minute, end_minute)
}
