use adventuresim_core::identity::CharacterId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SqlQuery(String);

impl SqlQuery {
    /// Encode the complete statement only at the HTTP provider boundary.
    pub(crate) fn request_body(&self) -> reqwest::Body {
        self.0.clone().into()
    }
}

impl From<String> for SqlQuery {
    fn from(statement: String) -> Self {
        Self(statement)
    }
}

impl From<&str> for SqlQuery {
    fn from(statement: &str) -> Self {
        Self::from(statement.to_owned())
    }
}

impl std::fmt::Display for SqlQuery {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub(crate) fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

macro_rules! string_key_query {
    ($name:ident, $prefix:literal) => {
        pub(crate) fn $name(value: &str) -> SqlQuery {
            SqlQuery::from(format!(concat!($prefix, "{}"), sql_string_literal(value)))
        }
    };
}

macro_rules! u64_key_query {
    ($name:ident, $prefix:literal) => {
        pub(crate) fn $name(value: u64) -> SqlQuery {
            SqlQuery::from(format!(concat!($prefix, "{}"), value))
        }
    };
}

string_key_query!(settlement_by_id, "SELECT * FROM settlement WHERE id = ");
string_key_query!(party_by_id, "SELECT * FROM party WHERE id = ");
string_key_query!(
    contract_by_id,
    "SELECT * FROM backend_contracts WHERE id = "
);
string_key_query!(
    party_journey_by_party_id,
    "SELECT * FROM party_journey WHERE party_id = "
);
string_key_query!(
    party_journey_route_by_party_id,
    "SELECT * FROM party_journey_route WHERE party_id = "
);
string_key_query!(
    strategic_encounter_by_party_id,
    "SELECT * FROM strategic_encounter WHERE party_id = "
);
string_key_query!(
    battle_result_by_battle_id,
    "SELECT * FROM battle_result WHERE battle_id = "
);
string_key_query!(
    autoresolve_report_by_battle_id,
    "SELECT * FROM autoresolve_report WHERE battle_id = "
);
string_key_query!(
    tactical_server_request_by_mission_id,
    "SELECT * FROM tactical_server_request WHERE mission_id = "
);
string_key_query!(
    settlement_smith_by_settlement_id,
    "SELECT * FROM settlement_smith WHERE settlement_id = "
);
string_key_query!(item_by_id, "SELECT * FROM item WHERE id = ");
string_key_query!(
    fireplace_station_by_key,
    "SELECT * FROM backend_fireplace_stations WHERE key = "
);
string_key_query!(
    fireplace_dish_by_station_key,
    "SELECT * FROM backend_fireplace_dishes WHERE station_key = "
);
string_key_query!(
    character_affinity_by_id,
    "SELECT * FROM backend_character_affinities WHERE id = "
);
string_key_query!(
    character_familiarity_by_id,
    "SELECT * FROM backend_character_familiarities WHERE id = "
);
string_key_query!(
    automatic_social_chat_by_id,
    "SELECT * FROM backend_automatic_social_chats WHERE id = "
);
string_key_query!(
    social_address_by_id,
    "SELECT * FROM backend_social_addresses WHERE id = "
);
string_key_query!(
    case_site_pin_by_case_site_id,
    "SELECT * FROM backend_case_site_pins WHERE case_site_id = "
);
pub(crate) fn case_site_pin_by_case_site_id_and_owner(
    case_site_id: &str,
    owner_character_id: CharacterId,
) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_case_site_pins WHERE case_site_id = {} AND owner_character_id = {owner_character_id}",
        sql_string_literal(case_site_id)
    ))
}
string_key_query!(
    tactical_server_by_mission_id,
    "SELECT * FROM tactical_server WHERE mission_id = "
);

pub(crate) fn character_by_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_characters WHERE id = {character_id}"
    ))
}
pub(crate) fn character_time_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_times WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_stats_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_stats WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_skills_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_skills WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_attributes_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_attributes WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_capability_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_capabilities WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_limbs_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_limbs WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_condition_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_conditions WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_strategic_condition_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_strategic_conditions WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_death_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_deaths WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_needs_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_needs WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_personality_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_personalities WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_training_schedule_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_training_schedules WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_relationship_status_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_relationship_statuses WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_residence_status_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_residence_statuses WHERE character_id = {character_id}"
    ))
}
pub(crate) fn character_case_site_location_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_character_case_site_locations WHERE character_id = {character_id}"
    ))
}
pub(crate) fn organization_presentation_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM organization_presentation WHERE character_id = {character_id}"
    ))
}
pub(crate) fn settlement_resident_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_settlement_residents WHERE character_id = {character_id}"
    ))
}
pub(crate) fn settlement_resident_presence_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM settlement_resident_presence WHERE character_id = {character_id}"
    ))
}
pub(crate) fn forage_attempt_state_by_character_id(character_id: CharacterId) -> SqlQuery {
    SqlQuery::from(format!(
        "SELECT * FROM backend_forage_attempt_states WHERE character_id = {character_id}"
    ))
}
u64_key_query!(
    inventory_item_by_id,
    "SELECT * FROM inventory_item WHERE id = "
);
u64_key_query!(
    inventory_object_by_id,
    "SELECT * FROM inventory_object WHERE id = "
);
u64_key_query!(
    party_action_request_by_id,
    "SELECT * FROM party_action_request WHERE id = "
);
u64_key_query!(
    party_recruitment_role_by_id,
    "SELECT * FROM party_recruitment_role WHERE id = "
);
u64_key_query!(
    weapon_instance_by_physical_object_id,
    "SELECT * FROM backend_weapon_instances WHERE physical_object_id = "
);
u64_key_query!(
    weapon_holder_instance_by_physical_object_id,
    "SELECT * FROM backend_weapon_holder_instances WHERE physical_object_id = "
);

pub(crate) fn world_clock_singleton() -> SqlQuery {
    SqlQuery::from("SELECT * FROM world_clock WHERE id = 0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_admission_and_http_encoding_preserve_exact_utf8_bytes() {
        let expected = "SELECT * FROM settlement WHERE id = '港の''家'";
        let borrowed = SqlQuery::from(expected);
        let owned = SqlQuery::from(expected.to_owned());
        assert_eq!(borrowed, owned);
        assert_eq!(borrowed.to_string(), expected);
        assert_eq!(
            borrowed.request_body().as_bytes(),
            Some(expected.as_bytes())
        );
        assert_eq!(owned.request_body().as_bytes(), Some(expected.as_bytes()));
    }

    #[test]
    fn string_primary_key_queries_escape_ids() {
        let cases = [
            (
                settlement_by_id("St. John's"),
                "SELECT * FROM settlement WHERE id = 'St. John''s'",
            ),
            (
                party_by_id("pilgrims' guild"),
                "SELECT * FROM party WHERE id = 'pilgrims'' guild'",
            ),
            (
                contract_by_id("contract'oath"),
                "SELECT * FROM backend_contracts WHERE id = 'contract''oath'",
            ),
            (
                party_journey_by_party_id("party'oath"),
                "SELECT * FROM party_journey WHERE party_id = 'party''oath'",
            ),
            (
                party_journey_route_by_party_id("party'oath"),
                "SELECT * FROM party_journey_route WHERE party_id = 'party''oath'",
            ),
            (
                strategic_encounter_by_party_id("party'oath"),
                "SELECT * FROM strategic_encounter WHERE party_id = 'party''oath'",
            ),
            (
                battle_result_by_battle_id("battle'oath"),
                "SELECT * FROM battle_result WHERE battle_id = 'battle''oath'",
            ),
            (
                autoresolve_report_by_battle_id("battle'oath"),
                "SELECT * FROM autoresolve_report WHERE battle_id = 'battle''oath'",
            ),
            (
                tactical_server_request_by_mission_id("mission'oath"),
                "SELECT * FROM tactical_server_request WHERE mission_id = 'mission''oath'",
            ),
            (
                tactical_server_by_mission_id("mission'oath"),
                "SELECT * FROM tactical_server WHERE mission_id = 'mission''oath'",
            ),
            (
                settlement_smith_by_settlement_id("smith's town"),
                "SELECT * FROM settlement_smith WHERE settlement_id = 'smith''s town'",
            ),
            (
                item_by_id("smith's hammer"),
                "SELECT * FROM item WHERE id = 'smith''s hammer'",
            ),
            (
                fireplace_station_by_key("station'oath"),
                "SELECT * FROM backend_fireplace_stations WHERE key = 'station''oath'",
            ),
            (
                fireplace_dish_by_station_key("station'oath"),
                "SELECT * FROM backend_fireplace_dishes WHERE station_key = 'station''oath'",
            ),
            (
                character_affinity_by_id("affinity'oath"),
                "SELECT * FROM backend_character_affinities WHERE id = 'affinity''oath'",
            ),
            (
                character_familiarity_by_id("familiarity'oath"),
                "SELECT * FROM backend_character_familiarities WHERE id = 'familiarity''oath'",
            ),
            (
                automatic_social_chat_by_id("chat'oath"),
                "SELECT * FROM backend_automatic_social_chats WHERE id = 'chat''oath'",
            ),
            (
                social_address_by_id("address'oath"),
                "SELECT * FROM backend_social_addresses WHERE id = 'address''oath'",
            ),
            (
                case_site_pin_by_case_site_id("site'oath"),
                "SELECT * FROM backend_case_site_pins WHERE case_site_id = 'site''oath'",
            ),
            (
                case_site_pin_by_case_site_id_and_owner("site'oath", CharacterId::from(17)),
                "SELECT * FROM backend_case_site_pins WHERE case_site_id = 'site''oath' AND owner_character_id = 17",
            ),
        ];

        for (query, expected) in cases {
            assert_eq!(query.to_string(), expected);
        }
    }

    #[test]
    fn character_primary_key_queries_name_the_gateway_rows() {
        let cases = [
            (
                character_by_id(CharacterId::from(17)),
                "SELECT * FROM backend_characters WHERE id = 17",
            ),
            (
                character_time_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_times WHERE character_id = 17",
            ),
            (
                character_stats_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_stats WHERE character_id = 17",
            ),
            (
                character_skills_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_skills WHERE character_id = 17",
            ),
            (
                character_attributes_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_attributes WHERE character_id = 17",
            ),
            (
                character_capability_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_capabilities WHERE character_id = 17",
            ),
            (
                character_limbs_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_limbs WHERE character_id = 17",
            ),
            (
                character_condition_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_conditions WHERE character_id = 17",
            ),
            (
                character_strategic_condition_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_strategic_conditions WHERE character_id = 17",
            ),
            (
                character_death_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_deaths WHERE character_id = 17",
            ),
            (
                character_needs_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_needs WHERE character_id = 17",
            ),
            (
                character_personality_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_personalities WHERE character_id = 17",
            ),
            (
                character_training_schedule_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_training_schedules WHERE character_id = 17",
            ),
            (
                character_relationship_status_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_relationship_statuses WHERE character_id = 17",
            ),
            (
                character_residence_status_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_residence_statuses WHERE character_id = 17",
            ),
            (
                character_case_site_location_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_character_case_site_locations WHERE character_id = 17",
            ),
            (
                organization_presentation_by_character_id(CharacterId::from(17)),
                "SELECT * FROM organization_presentation WHERE character_id = 17",
            ),
            (
                settlement_resident_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_settlement_residents WHERE character_id = 17",
            ),
            (
                settlement_resident_presence_by_character_id(CharacterId::from(17)),
                "SELECT * FROM settlement_resident_presence WHERE character_id = 17",
            ),
            (
                forage_attempt_state_by_character_id(CharacterId::from(17)),
                "SELECT * FROM backend_forage_attempt_states WHERE character_id = 17",
            ),
        ];

        for (query, expected) in cases {
            assert_eq!(query.to_string(), expected);
        }
    }

    #[test]
    fn character_query_selectors_keep_full_unsigned_width_and_empty_identity_word() {
        for raw in [0, u64::MAX] {
            let character = CharacterId::from(raw);
            assert_eq!(
                character_by_id(character).to_string(),
                format!("SELECT * FROM backend_characters WHERE id = {raw}"),
            );
            assert_eq!(
                character_time_by_character_id(character).to_string(),
                format!("SELECT * FROM backend_character_times WHERE character_id = {raw}"),
            );
            assert_eq!(
                case_site_pin_by_case_site_id_and_owner("site'oath", character).to_string(),
                format!(
                    "SELECT * FROM backend_case_site_pins WHERE case_site_id = 'site''oath' AND owner_character_id = {raw}"
                ),
            );
        }
    }

    #[test]
    fn other_numeric_primary_key_queries_are_unquoted() {
        let cases = [
            (
                inventory_item_by_id(17),
                "SELECT * FROM inventory_item WHERE id = 17",
            ),
            (
                inventory_object_by_id(17),
                "SELECT * FROM inventory_object WHERE id = 17",
            ),
            (
                party_action_request_by_id(17),
                "SELECT * FROM party_action_request WHERE id = 17",
            ),
            (
                party_recruitment_role_by_id(17),
                "SELECT * FROM party_recruitment_role WHERE id = 17",
            ),
            (
                weapon_instance_by_physical_object_id(17),
                "SELECT * FROM backend_weapon_instances WHERE physical_object_id = 17",
            ),
            (
                weapon_holder_instance_by_physical_object_id(17),
                "SELECT * FROM backend_weapon_holder_instances WHERE physical_object_id = 17",
            ),
            (
                world_clock_singleton(),
                "SELECT * FROM world_clock WHERE id = 0",
            ),
        ];

        for (query, expected) in cases {
            assert_eq!(query.to_string(), expected);
        }
    }

    #[test]
    fn query_helpers_never_interpolate_table_or_key_names_at_runtime() {
        let source = include_str!("queries.rs");
        let runtime_table_placeholder = ["{", "table", "}"].concat();
        let runtime_key_placeholder = ["{", "key", "}"].concat();
        let dynamic_string_helper = ["fn string_key_query", "(table"].concat();
        let dynamic_numeric_helper = ["fn u64_key_query", "(table"].concat();
        assert!(!source.contains(&runtime_table_placeholder));
        assert!(!source.contains(&runtime_key_placeholder));
        assert!(!source.contains(&dynamic_string_helper));
        assert!(!source.contains(&dynamic_numeric_helper));
    }
}
