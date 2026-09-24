use crate::location_urls::LocationKind;

use adventuresim_world_schema::SettlementEconomyProfile;
use maud::Markup;

use crate::spacetimedb::SettlementCategory;
use crate::templates::{
    quest_location_layout_with_session, settlement_building_available,
    settlement_layout_with_session,
};

#[derive(Clone, Debug)]
pub struct LocationView {
    pub kind: LocationKind,
    pub id: String,
    pub name: String,
    pub religion_id: Option<String>,
    pub category: Option<SettlementCategory>,
    pub economy: Option<SettlementEconomyProfile>,
    pub active_building: Option<String>,
}
impl LocationView {
    pub fn valid_building<'a>(&self, building: &'a str) -> Option<&'a str> {
        (self.kind == LocationKind::Settlement
            && crate::location_urls::SettlementPlace::parse(&self.id, building).is_some()
            && settlement_building_available(
                &self.id,
                self.category
                    .as_ref()
                    .unwrap_or(&SettlementCategory::Unknown),
                self.economy.as_ref(),
                building,
            ))
        .then_some(building)
    }
    pub fn base_path(&self) -> String {
        self.kind.path(&self.id)
    }

    pub fn preserve_building(&self, path: String) -> String {
        self.active_building.as_deref().map_or_else(
            || path.clone(),
            |building| crate::location_urls::with_query(&path, "building", building),
        )
    }

    pub(super) fn render_layout(
        &self,
        title: &str,
        content: Markup,
        logged_in_as: Option<&str>,
    ) -> Markup {
        if self.kind == LocationKind::Settlement {
            settlement_layout_with_session(
                title,
                &self.name,
                &self.id,
                self.category
                    .as_ref()
                    .unwrap_or(&SettlementCategory::Unknown),
                self.active_building.as_deref().unwrap_or(""),
                self.religion_id.as_deref(),
                self.economy.as_ref(),
                content,
                logged_in_as,
            )
        } else {
            quest_location_layout_with_session(
                title,
                &self.name,
                &self.id,
                "",
                content,
                logged_in_as,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_kind_rejects_unknown_path_segments() {
        assert_eq!("case-site".parse(), Ok(LocationKind::CaseSite));
        assert!("quest".parse::<LocationKind>().is_err());
        assert!("merchant".parse::<LocationKind>().is_err());
    }

    #[test]
    fn settlement_location_layout_respects_its_economy_services() {
        let location = LocationView {
            kind: LocationKind::Settlement,
            id: "small-place".into(),
            name: "Small Place".into(),
            religion_id: None,
            category: Some(SettlementCategory::Village),
            economy: Some(SettlementEconomyProfile::stage_placeholder()),
            active_building: Some("inn".into()),
        };
        let markup = location
            .render_layout("Party", maud::html! {}, None)
            .into_string();
        assert!(markup.contains("data-building-id=\"inn\""));
        assert!(!markup.contains("data-building-id=\"merchants\""));
    }
}
