//! Personality axes revealed by each social approach.

use adventuresim_core::social::{PersonalityAxis, SocialActionKind, SocialTopic, axis_for_topic};

pub(super) fn axes(
    action: SocialActionKind,
    topic: SocialTopic,
    is_self: bool,
) -> Vec<PersonalityAxis> {
    if is_self {
        return vec![
            PersonalityAxis::SelfKnowledge,
            axis_for_topic(topic).unwrap_or(PersonalityAxis::Outlook),
        ];
    }
    match action {
        SocialActionKind::Listen => vec![
            axis_for_topic(topic).unwrap_or(match topic {
                SocialTopic::Fatigue => PersonalityAxis::Outlook,
                SocialTopic::Hunger => PersonalityAxis::Temperance,
                _ => PersonalityAxis::Transparency,
            }),
            PersonalityAxis::Transparency,
        ],
        SocialActionKind::Commiserate => {
            vec![PersonalityAxis::Conscience, PersonalityAxis::Sociability]
        }
        SocialActionKind::Pray => vec![PersonalityAxis::Conviction],
        SocialActionKind::Reassure => Vec::new(),
        SocialActionKind::LightenMood => vec![PersonalityAxis::Mirth],
        SocialActionKind::Rally => vec![
            PersonalityAxis::Nerve,
            if topic == SocialTopic::Faith {
                PersonalityAxis::Conviction
            } else {
                PersonalityAxis::Drive
            },
        ],
        SocialActionKind::Reframe => {
            vec![PersonalityAxis::SelfRegard, PersonalityAxis::Outlook]
        }
        SocialActionKind::Flirt => {
            vec![PersonalityAxis::Courtship, PersonalityAxis::Inclination]
        }
        SocialActionKind::Reflect => unreachable!("self-only handled above"),
    }
}
