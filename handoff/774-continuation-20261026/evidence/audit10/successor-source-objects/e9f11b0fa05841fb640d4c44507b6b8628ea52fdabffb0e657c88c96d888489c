//! Shared segmented skill scale and its current accessible reading.
use super::{Markup, SkillTooltip, html};

pub(in crate::templates::settlement) fn skill_rank_tier(rank: f32) -> u8 {
    let rank = super::finite_rank(rank);
    match rank {
        rank if rank <= 0.0 => 0,
        rank if rank <= 1.0 => 1,
        rank if rank <= 2.0 => 2,
        rank if rank <= 3.0 => 3,
        rank if rank <= 4.0 => 4,
        _ => 5,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::templates::settlement) struct SkillRankBarOptions<'a> {
    pub(in crate::templates::settlement) extra_class: Option<&'a str>,
    pub(in crate::templates::settlement) aria_label: Option<&'a str>,
}

pub(in crate::templates::settlement) fn skill_rank_bar(
    rank: f32,
    effective_rank: f32,
    title: &str,
    options: SkillRankBarOptions<'_>,
) -> Markup {
    skill_rank_bar_markup(rank, effective_rank, Some(title), None, options)
}

pub(super) fn skill_rank_bar_with_tooltip(
    rank: f32,
    effective_rank: f32,
    tooltip: &SkillTooltip,
    options: SkillRankBarOptions<'_>,
) -> Markup {
    skill_rank_bar_markup(rank, effective_rank, None, Some(tooltip), options)
}

fn skill_rank_bar_markup(
    rank: f32,
    effective_rank: f32,
    title: Option<&str>,
    skill_tooltip: Option<&SkillTooltip>,
    options: SkillRankBarOptions<'_>,
) -> Markup {
    let rank = rank.clamp(0.0, 5.0);
    let effective_rank = effective_rank.clamp(0.0, rank);
    let class = options.extra_class.map_or_else(
        || "skill-rank-bar".to_owned(),
        |extra| format!("skill-rank-bar {extra}"),
    );
    let aria_label = options
        .aria_label
        .map_or_else(|| format!("{effective_rank:.1} out of 5"), str::to_owned);
    let tooltip_description = skill_tooltip.map_or_else(
        || format!("{}\n{aria_label}", title.unwrap_or_default()),
        |tooltip| format!("{}\n{aria_label}", tooltip.accessible_description()),
    );
    let tooltip_json = skill_tooltip
        .map(|tooltip| serde_json::to_string(tooltip).expect("skill tooltip data serializes"));
    html! {
        div class=(class) aria-label=(aria_label)
            data-strategic-tooltip=(tooltip_description) data-tooltip-pinnable
            data-skill-tooltip=[tooltip_json]
            tabindex="0" aria-keyshortcuts="Enter Space"
            role="meter" aria-valuemin="0" aria-valuemax="5" aria-valuenow=(format!("{effective_rank:.1}")) {
            span class="skill-rank-track" aria-hidden="true" {
                @for tier in 1..=5 {
                    @let offset = (tier - 1) as f32;
                    @let current = (effective_rank - offset).clamp(0.0, 1.0) * 100.0;
                    @let trained = (rank - offset).clamp(0.0, 1.0) * 100.0;
                    @let damaged = (trained - current).max(0.0);
                    span class=(format!("skill-rank-segment skill-rank-segment-{tier}")) {
                        span class="rank-current" style=(format!("width:{current:.1}%")) {}
                        span class="rank-damage" style=(format!("left:{current:.1}%;width:{damaged:.1}%")) {}
                    }
                }
            }
        }
    }
}
