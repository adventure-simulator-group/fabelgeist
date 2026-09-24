// Owns kinship, household membership, birth records, and effective age.
fn kinship_id(subject_id: u64, related_id: u64, kind: KinshipKind) -> String {
    format!("kinship:{subject_id}:{related_id}:{}", kind.stable_id())
}

fn ensure_kinship(
    ctx: &ReducerContext,
    subject_id: u64,
    related_id: u64,
    kind: KinshipKind,
    minute: u64,
) {
    let id = kinship_id(subject_id, related_id, kind);
    if ctx.db.character_kinship().id().find(&id).is_none() {
        ctx.db.character_kinship().insert(CharacterKinship {
            id,
            subject_id,
            related_id,
            kind,
            established_minute: minute,
        });
    }
}

fn leave_household(ctx: &ReducerContext, character_id: u64) {
    if let Some(member) = ctx.db.household_member().character_id().find(character_id) {
        ctx.db.household_member().id().delete(&member.id);
    }
}

fn join_household(
    ctx: &ReducerContext,
    household_id: &str,
    character_id: u64,
    minute: u64,
    role: HouseholdRole,
) {
    if ctx
        .db
        .household_member()
        .character_id()
        .find(character_id)
        .is_some_and(|member| member.household_id == household_id)
    {
        return;
    }
    leave_household(ctx, character_id);
    ctx.db.household_member().insert(HouseholdMember {
        id: format!("household:{household_id}:{character_id}"),
        household_id: household_id.to_owned(),
        character_id,
        joined_minute: minute,
        role,
    });
}

pub fn record_character_birth(ctx: &ReducerContext, character_id: u64, birth_minute: i64) {
    if ctx
        .db
        .character_birth()
        .character_id()
        .find(character_id)
        .is_none()
    {
        ctx.db.character_birth().insert(CharacterBirth {
            character_id,
            birth_minute,
        });
    }
}

pub fn set_seeded_character_birth_from_age(
    ctx: &ReducerContext,
    character_id: u64,
    age_years: u16,
) {
    let row = CharacterBirth {
        character_id,
        birth_minute: -(i64::from(age_years)
            * i64::try_from(MINUTES_PER_YEAR).unwrap_or(i64::MAX)),
    };
    if ctx
        .db
        .character_birth()
        .character_id()
        .find(character_id)
        .is_some()
    {
        ctx.db.character_birth().character_id().update(row);
    } else {
        ctx.db.character_birth().insert(row);
    }
}

pub fn effective_age_years(ctx: &ReducerContext, character_id: u64, minute: u64) -> Option<u16> {
    let character = ctx.db.character().id().find(character_id)?;
    let Some(birth) = ctx.db.character_birth().character_id().find(character_id) else {
        return Some(character.age_years);
    };
    let elapsed = i128::from(minute).saturating_sub(i128::from(birth.birth_minute));
    Some((elapsed.max(0) as u128 / u128::from(MINUTES_PER_YEAR)).min(u128::from(u16::MAX)) as u16)
}

/// Refresh the cached display age from the authoritative birth coordinate.
/// Calling this at every lifecycle boundary naturally promotes dependents at
/// their yearly boundary without granting newborn starter equipment.
pub fn settle_character_age(ctx: &ReducerContext, character_id: u64, minute: u64) {
    let Some(mut character) = ctx.db.character().id().find(character_id) else {
        return;
    };
    let Some(age_years) = effective_age_years(ctx, character_id, minute) else {
        return;
    };
    if character.age_years != age_years {
        character.age_years = age_years;
        ctx.db.character().id().update(character);
    }
}

/// Turn the deterministic resident roster into coherent authoritative family
/// units. Demographics are finalized by settlement population planning before
/// character creation; this pass only records roles and relationships.
#[derive(Debug, PartialEq, Eq)]
struct SeededHouseholdPlan {
    household_id: String,
    family_key: String,
    members: Vec<(u64, HouseholdRole)>,
    kinships: Vec<(u64, u64, KinshipKind)>,
}

fn seeded_household_plan(
    settlement_id: &str,
    family: &[u64],
) -> Result<Option<SeededHouseholdPlan>, String> {
    let Some(first_character_id) = family.first() else {
        return Ok(None);
    };
    let roles = [
        HouseholdRole::Head,
        HouseholdRole::Spouse,
        HouseholdRole::AdultChild,
        HouseholdRole::AdultChild,
    ];
    if family.len() > roles.len() {
        return Err("Seeded household exceeds the supported family shape".into());
    }
    let members = family.iter().copied().zip(roles).collect();
    let kinships = if family.len() == roles.len() {
        let mut edges = Vec::with_capacity(10);
        for child in [family[2], family[3]] {
            for parent in [family[0], family[1]] {
                edges.push((child, parent, KinshipKind::Parent));
                edges.push((parent, child, KinshipKind::Child));
            }
        }
        edges.push((family[2], family[3], KinshipKind::Sibling));
        edges.push((family[3], family[2], KinshipKind::Sibling));
        edges
    } else {
        Vec::new()
    };
    Ok(Some(SeededHouseholdPlan {
        household_id: format!("household:seeded:{settlement_id}:{first_character_id}"),
        family_key: format!("seeded:{settlement_id}:{first_character_id}"),
        members,
        kinships,
    }))
}

pub fn ensure_seeded_family_households(
    ctx: &ReducerContext,
    settlement_id: &str,
    household_groups: &[Vec<u64>],
) -> Result<(), String> {
    for family in household_groups {
        let Some(plan) = seeded_household_plan(settlement_id, family)? else {
            continue;
        };
        if ctx.db.household().id().find(&plan.household_id).is_none() {
            ctx.db.household().insert(Household {
                id: plan.household_id.clone(),
                home_settlement_id: settlement_id.to_owned(),
                created_minute: 0,
            });
        }
        for &(character_id, role) in &plan.members {
            join_household(ctx, &plan.household_id, character_id, 0, role);
        }
        let noble = plan.members.iter().any(|(character_id, _)| {
            crate::social_roles::character_has_profession(ctx, *character_id, "noble")
                .unwrap_or(false)
        });
        for &(character_id, _) in &plan.members {
            crate::social_roles::ensure_character_family_role(
                ctx,
                character_id,
                &plan.family_key,
                noble,
            )?;
        }
        let family_ids: Vec<u64> = plan.members.iter().map(|(character_id, _)| *character_id).collect();
        assign_seeded_family_names(ctx, &family_ids)?;
        for &(subject_id, related_id, kind) in &plan.kinships {
            ensure_kinship(ctx, subject_id, related_id, kind, 0);
        }
    }
    Ok(())
}

fn assign_seeded_family_names(ctx: &ReducerContext, family: &[u64]) -> Result<(), String> {
    let mut surname = family.iter().copied().find_map(|character_id| {
        crate::character::character_hereditary_surname(
            ctx,
            crate::character::CharacterId::new(character_id),
        )
    });
    for character_id in family.iter().copied() {
        if crate::character::character_name_is_authored(
            ctx,
            crate::character::CharacterId::new(character_id),
        ) {
            continue;
        }
        let age_years = ctx
            .db
            .character()
            .id()
            .find(character_id)
            .ok_or("Seeded family member is missing its Character")?
            .age_years;
        let seed = fabelgeist_determinism::Seed::derive(
            &character_id.to_le_bytes(),
            fabelgeist_determinism::StreamId::new("resident.personal-name"),
            &[&2u16.to_le_bytes()],
        )
        .to_u64();
        surname = Some(crate::character::assign_generated_historical_name(
            ctx,
            crate::character::CharacterId::new(character_id),
            crate::character::NameSeed::new(seed),
            adventuresim_world_schema::person_names::NameBirthYear::new(
                adventuresim_core::strategic_time::birth_year_from_age(0, age_years),
            ),
            surname,
        )?);
    }
    Ok(())
}

fn father_of_at(ctx: &ReducerContext, child_id: u64, minute: u64) -> Result<Option<u64>, String> {
    let father = ctx.db.character_kinship().iter().find_map(|edge| {
        (edge.subject_id == child_id
            && edge.kind == KinshipKind::Parent
            && edge.established_minute <= minute)
            .then(|| {
                ctx.db
                    .character_personality()
                    .character_id()
                    .find(edge.related_id)
                    .filter(|personality| personality.sex == Sex::Male)
                    .map(|_| edge.related_id)
            })
            .flatten()
    });
    let Some(father) = father else {
        return Ok(None);
    };
    if canonical_now(ctx, father)? != minute {
        return Err("The prospective bride's father has not reached the relationship date".into());
    }
    Ok(character_alive_at(ctx, father, minute).then_some(father))
}

fn relationship_conflicts_at(
    ctx: &ReducerContext,
    character_id: u64,
    minute: u64,
    permitted_courtship_id: Option<&str>,
) -> bool {
    let courtship_conflict = ctx.db.courtship().iter().any(|row| {
        (row.first_character_id == character_id || row.second_character_id == character_id)
            && Some(row.id.as_str()) != permitted_courtship_id
            && row.started_minute <= minute
            && row.resolved_minute.is_none_or(|resolved| resolved > minute)
    });
    let commitment_conflict = ctx.db.exclusive_commitment().iter().any(|row| {
        (row.first_character_id == character_id || row.second_character_id == character_id)
            && row.created_minute <= minute
            && row.resolved_minute.is_none_or(|resolved| resolved > minute)
    });
    let marriage_conflict = ctx.db.marriage().iter().any(|row| {
        (row.first_character_id == character_id || row.second_character_id == character_id)
            && row.married_minute <= minute
            && row.resolved_minute.is_none_or(|resolved| resolved > minute)
    });
    courtship_conflict || commitment_conflict || marriage_conflict
}

fn formal_dowry_amount(father_wealth: u64) -> u32 {
    if father_wealth >= 300 {
        100
    } else if father_wealth >= 100 {
        45
    } else if father_wealth >= 30 {
        15
    } else {
        0
    }
}
