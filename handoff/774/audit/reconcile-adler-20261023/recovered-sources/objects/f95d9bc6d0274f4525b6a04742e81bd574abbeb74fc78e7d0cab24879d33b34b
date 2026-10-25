// Ordinary residence portfolio and exact-property actions.
#[derive(Default, Deserialize)]
pub(super) struct ResidencePageQuery {
    residence_notice: Option<String>,
}

#[derive(Default, Deserialize)]
pub(super) struct ResidenceActionForm {
    holding_id: Option<String>,
    property_id: Option<String>,
}

fn residence_notice(code: Option<&str>) -> Option<&'static str> {
    match code {
        Some("rented") => Some("The residence is now rented and ready to use."),
        Some("bought") => Some("You bought the residence."),
        Some("relinquished") => Some("You relinquished the residence."),
        Some("designated") => Some("This residence is now your designated home."),
        Some("recovered") => Some("The owned residence is active again."),
        Some("funds") => Some("You do not have enough coin for that."),
        Some("location") => Some("You must be in this settlement to do that."),
        Some("overdue") => Some("Settle the overdue housing cost before doing that."),
        Some("unavailable") => Some("That housing change is not available."),
        _ => None,
    }
}

fn relationship_date_label(minute: StrategicMinute) -> String {
    let year = minute.calendar_year();
    let day_of_year = minute.day_of_year();
    format!("year {year}, day {day_of_year}")
}

fn housing_error_code(_error: &str) -> &'static str {
    "unavailable"
}

pub(super) async fn settlement_resident_place(
    State(state): State<AppState>,
    Path((id, place)): Path<(String, String)>,
    Query(page_query): Query<ResidencePageQuery>,
    session: Session,
) -> Html<String> {
    let organization_chapter =
        adventuresim_core::organization::organization_chapter_at(&id, &place);
    if !matches!(place.as_str(), "residences" | "keep") && organization_chapter.is_none() {
        return Html("<h1>Settlement place not found</h1>".into());
    }
    let settlement_query = settlement_by_id(&id);
    let settlement = state
        .db
        .query_one_sats_into::<DbSettlement, SettlementView>(settlement_query.as_str())
        .await
        .ok()
        .flatten();
    let Some(settlement) = settlement else {
        return Html("<h1>Settlement not found</h1>".into());
    };
    if let Some((organization, chapter)) = organization_chapter
        && !adventuresim_core::organization::chapter_has_standalone_building(
            organization,
            chapter,
            &settlement.economy,
        )
    {
        return Html("<h1>Settlement place not found</h1>".into());
    }
    let active = get_active_character(&state, session.character_id_u64()).await;
    let Some((character, _)) = active.as_ref() else {
        return Html("<h1>Choose a character first</h1>".into());
    };
    if character.current_settlement_id.as_deref() != Some(id.as_str()) {
        return Html("<h1>You are not in this settlement</h1>".into());
    }
    if place == "keep"
        && !matches!(
            settlement.category,
            db::SettlementCategory::Town
                | db::SettlementCategory::City
                | db::SettlementCategory::Capital
        )
    {
        return Html("<h1>This settlement has no keep</h1>".into());
    }
    let party_members = get_active_party_members(&state, Some(character)).await;
    if place == "residences" {
        return render_residences(
            &state,
            &settlement,
            character,
            &party_members,
            &session,
            residence_notice(page_query.residence_notice.as_deref()),
        )
        .await;
    }

    Html(
        settlement_resident_location_page(
            &settlement,
            character,
            &party_members,
            &place,
            Some(&character.name),
        )
        .into_string(),
    )
}

async fn render_residences(
    state: &AppState,
    settlement: &SettlementView,
    character: &CharacterView,
    party_members: &[CharacterView],
    session: &Session,
    notice: Option<&str>,
) -> Html<String> {
    if let Err(error) = super::super::settlement_properties::ensure(state, settlement).await {
        tracing::error!(%error, settlement_id = %settlement.id, "physical homes unavailable");
        return Html("<h1>Residence properties are unavailable</h1>".into());
    }
    let offers_sql = format!(
        "SELECT * FROM backend_available_residence_properties WHERE settlement_id = {}",
        sql_string_literal(&settlement.id)
    );
    let residence_sql = db::character_residence_status_by_character_id(character.id);
    let (offers, residences, (presentation, character_minute)) = tokio::join!(
        state
            .db
            .query_sats::<adventuresim_stdb_client::AvailableResidenceProperty>(&offers_sql),
        state
            .db
            .query_sats::<BackendCharacterResidenceStatus>(&residence_sql),
        read_family(state, character, session.owner_key().unwrap_or_default()),
    );
    let (Ok(mut offers), Ok(mut residences)) = (offers, residences) else {
        return Html("<h1>Residence properties are unavailable</h1>".into());
    };
    offers.sort_by_key(|offer| match offer.tier {
        adventuresim_stdb_client::HousingTier::Cheap => 0,
        adventuresim_stdb_client::HousingTier::Moderate => 1,
        adventuresim_stdb_client::HousingTier::Fancy => 2,
    });
    offers.retain(|offer| offer.available_from_minute.minutes <= character_minute.get());
    residences.retain(|holding| holding.character_id == character.id);
    residences.sort_by(|left, right| {
        (
            !left.primary,
            left.settlement_id != settlement.id,
            left.holding_id.as_str(),
        )
            .cmp(&(
                !right.primary,
                right.settlement_id != settlement.id,
                right.holding_id.as_str(),
            ))
    });
    let can_rest_at_home = residences
        .iter()
        .any(|home| home.active && home.occupied && home.settlement_id == settlement.id);
    Html(
        settlement_residence_page(
            settlement,
            character,
            party_members,
            Some(&character.name),
            &offers,
            &residences,
            presentation.as_ref(),
            can_rest_at_home,
            notice,
        )
        .into_string(),
    )
}

async fn read_family(
    state: &AppState,
    character: &CharacterView,
    owner_key: &str,
) -> (Option<RelationshipPresentation>, StrategicMinute) {
    let relationship_sql = db::character_relationship_status_by_character_id(character.id);
    let family_sql = format!(
        "SELECT * FROM backend_family_children WHERE owner_key = {} AND observer_character_id = {}",
        sql_string_literal(owner_key),
        character.id
    );
    let clock_sql = db::character_time_by_character_id(character.id);
    let (relationship, children, clock) = tokio::join!(
        state
            .db
            .query_one_sats::<BackendCharacterRelationshipStatus>(&relationship_sql),
        state.db.query_sats::<BackendFamilyChild>(&family_sql),
        state.db.query_one_sats::<CharacterTime>(&clock_sql)
    );
    let minute = clock.ok().flatten().map_or(StrategicMinute::ZERO, |time| {
        StrategicMinute::new(time.minutes.minutes)
    });
    let relationship = relationship.ok().flatten();
    let mut children = children.unwrap_or_default();
    children.retain(|child| {
        child.owner_key == owner_key && child.observer_character_id == character.id
    });
    children.sort_by_key(|child| child.child_id);
    let mut related_characters = Vec::new();
    let related_ids = relationship
        .iter()
        .flat_map(|row| [row.spouse_id, row.courtship_partner_id])
        .flatten()
        .collect::<Vec<_>>();
    for related_id in related_ids {
        if let Ok(Some(related)) = state
            .db
            .query_one_sats_into::<DbCharacter, CharacterView>(&db::character_by_id(related_id))
            .await
        {
            related_characters.push(related);
        }
    }
    let presentation = relationship.as_ref().map(|status| {
        let name = |id: Option<u64>| {
            id.and_then(|id| {
                related_characters
                    .iter()
                    .find(|character| character.id == id)
                    .map(|character| character.name.clone())
            })
        };
        RelationshipPresentation {
            spouse_name: name(status.spouse_id),
            courtship_partner_name: name(status.courtship_partner_id),
            courtship_kind: status.courtship_kind,
            courtship_exposed: status.courtship_exposed,
            wedding: status
                .wedding_effective_minute
                .as_ref()
                .map(|date| WeddingPresentation {
                    days_remaining: minute.days_until_ceil(StrategicMinute::new(date.minutes)),
                    date_label: relationship_date_label(StrategicMinute::new(date.minutes)),
                }),
            pregnancy_due_days: status
                .pregnancy_due_minute
                .as_ref()
                .map(|due| minute.days_until_ceil(StrategicMinute::new(due.minutes))),
            children: children
                .iter()
                .map(|child| ChildPresentation {
                    name: child.child_name.clone(),
                    stage: child.stage,
                    focus: child.focus,
                    maturity_basis_points: child.maturity_basis_points,
                    adult_playable: child.adult_playable,
                    alive: child.alive,
                })
                .collect(),
        }
    });
    (presentation, minute)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResidenceTier {
    Cheap,
    Moderate,
    Fancy,
    Current,
}

impl ResidenceTier {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "cheap" => Some(Self::Cheap),
            "moderate" => Some(Self::Moderate),
            "fancy" => Some(Self::Fancy),
            "current" => Some(Self::Current),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResidenceOperation {
    Rent,
    Buy,
    Relinquish,
    Designate,
    Recover,
}

impl ResidenceOperation {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "rent" => Some(Self::Rent),
            "buy" => Some(Self::Buy),
            "relinquish" => Some(Self::Relinquish),
            "designate" => Some(Self::Designate),
            "recover" => Some(Self::Recover),
            _ => None,
        }
    }

    const fn reducer(self) -> &'static str {
        match self {
            Self::Rent => "rent_residence",
            Self::Buy => "buy_residence",
            Self::Relinquish => "relinquish_residence",
            Self::Designate => "designate_residence",
            Self::Recover => "recover_owned_residence",
        }
    }

    const fn success_notice(self) -> &'static str {
        match self {
            Self::Rent => "rented",
            Self::Buy => "bought",
            Self::Relinquish => "relinquished",
            Self::Designate => "designated",
            Self::Recover => "recovered",
        }
    }
}

pub(super) async fn change_residence(
    State(state): State<AppState>,
    Path((id, action, tier)): Path<(String, String, String)>,
    session: Session,
    Form(form): Form<ResidenceActionForm>,
) -> Redirect {
    let fallback = paths::SETTLEMENT_PLACE.url([&id, &("residences")]);
    let Some(character_id) = session.character_id_u64() else {
        return Redirect::to("/characters");
    };
    let (Some(operation), Some(tier)) = (
        ResidenceOperation::parse(&action),
        ResidenceTier::parse(&tier),
    ) else {
        return Redirect::to(&format!("{fallback}?residence_notice=unavailable"));
    };
    let selected_holding = form
        .holding_id
        .filter(|holding_id| !holding_id.trim().is_empty());
    let args = match operation {
        ResidenceOperation::Rent | ResidenceOperation::Buy => {
            let Some(property_id) = form.property_id.filter(|id| !id.is_empty()) else {
                return Redirect::to(&format!("{fallback}?residence_notice=unavailable"));
            };
            let property = state
                .db
                .query_one_sats::<adventuresim_stdb_client::SettlementProperty>(&format!(
                    "SELECT * FROM settlement_property WHERE id = {}",
                    sql_string_literal(&property_id)
                ))
                .await;
            let expected_tier = match tier {
                ResidenceTier::Cheap => Some(adventuresim_stdb_client::HousingTier::Cheap),
                ResidenceTier::Moderate => Some(adventuresim_stdb_client::HousingTier::Moderate),
                ResidenceTier::Fancy => Some(adventuresim_stdb_client::HousingTier::Fancy),
                ResidenceTier::Current => None,
            };
            if !matches!(property, Ok(Some(ref property)) if property.settlement_id == id && Some(property.tier) == expected_tier)
            {
                return Redirect::to(&format!("{fallback}?residence_notice=unavailable"));
            }
            vec![json!(character_id), json!(property_id)]
        }
        ResidenceOperation::Relinquish
        | ResidenceOperation::Designate
        | ResidenceOperation::Recover => {
            let Some(holding_id) = selected_holding.filter(|_| tier == ResidenceTier::Current)
            else {
                return Redirect::to(&format!("{fallback}?residence_notice=unavailable"));
            };
            vec![json!(character_id), json!(holding_id)]
        }
    };
    match state.db.call(operation.reducer(), &args).await {
        Ok(()) => Redirect::to(&format!(
            "{fallback}?residence_notice={}",
            operation.success_notice()
        )),
        Err(error) => {
            tracing::warn!(character_id, ?operation, ?tier, %error, "residence acquisition rejected");
            Redirect::to(&format!(
                "{fallback}?residence_notice={}",
                housing_error_code(&error.to_string())
            ))
        }
    }
}

#[cfg(test)]
mod residence_route_tests {
    use super::{ResidenceOperation, ResidenceTier};

    #[test]
    fn residence_route_tags_map_to_fixed_typed_operations() {
        assert_eq!(
            ["rent", "buy", "relinquish", "designate", "recover"].map(|tag| {
                let operation = ResidenceOperation::parse(tag).unwrap();
                (operation.reducer(), operation.success_notice())
            }),
            [
                ("rent_residence", "rented"),
                ("buy_residence", "bought"),
                ("relinquish_residence", "relinquished"),
                ("designate_residence", "designated"),
                ("recover_owned_residence", "recovered"),
            ]
        );
        assert_eq!(ResidenceOperation::parse("remove"), None);
        assert_eq!(
            ResidenceTier::parse("current"),
            Some(ResidenceTier::Current)
        );
        assert_eq!(ResidenceTier::parse("luxury"), None);
    }
}
