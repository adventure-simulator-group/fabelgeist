/// Local problem activity and consequence scaling.
fn is_active(row: &LocalProblemAuthority, minute: StrategicMinute) -> bool {
    minute >= row.starts_at
        && (row.recurring_hostile || minute < row.ends_at)
        && row
            .resolved_at
            .is_none_or(|at| minute < at)
        && row.mitigation_bps < adventuresim_world_schema::BASIS_POINTS_PER_WHOLE
}
fn scaled(value: i32, mitigation: u16) -> i32 {
    (i64::from(value)
        * i64::from(
            adventuresim_world_schema::BASIS_POINTS_PER_WHOLE
                .saturating_sub(mitigation.min(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE)),
        )
        / i64::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE)) as i32
}
