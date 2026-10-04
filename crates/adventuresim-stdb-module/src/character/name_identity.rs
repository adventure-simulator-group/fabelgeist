// Persistent semantic personal-name authority and projection helpers.

use adventuresim_world_schema::{
    Culture, Sex,
    calendar::CalendarYear,
    person_names::{
        NameCatalogError, NameGenerationContext, NameRegister, NameStableSeed,
        PersonalNameIdentity, SurnameId, generate_personal_name, render_personal_name,
    },
};

/// Private semantic authority behind [`Character::name`]. Resolved generated
/// identities project to the native-culture everyday form; authored and
/// unresolved identities preserve their supplied display form for existing UI.
#[derive(Clone, Debug)]
#[table(accessor = character_name_identity)]
pub struct CharacterNameIdentity {
    #[primary_key]
    pub character_id: u64,
    /// JSON serialization of the shared, strongly typed semantic identity.
    /// SpacetimeDB 2.6 sums cannot represent the resolved/authored variants
    /// directly, so this private row preserves the clean shared schema intact.
    pub identity_json: NameIdentityJson,
}

fn character_name_sex(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<Sex, CharacterNameError> {
    let personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(CharacterNameError::MissingPersonality(character_id))?;
    Ok(personality.sex)
}

fn authored_name_identity(name: String) -> PersonalNameIdentity {
    PersonalNameIdentity::authored(name, Culture::German)
}

/// Persist a semantic identity and refresh the character's display name.
pub(crate) fn assign_character_name_identity(
    ctx: &ReducerContext,
    character_id: CharacterId,
    identity: PersonalNameIdentity,
) -> Result<(), CharacterNameError> {
    let sex = character_name_sex(ctx, character_id)?;
    let display = render_personal_name(
        &identity,
        identity.native_culture,
        NameRegister::Everyday,
        sex,
    )?;
    let identity_json = NameIdentityJson::from_identity(&identity)?;
    let mut character = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .ok_or(CharacterNameError::MissingCharacter(character_id))?;
    character.name = display.into_string();
    ctx.db.character().id().update(character);
    let row = CharacterNameIdentity {
        character_id: u64::from(character_id),
        identity_json,
    };
    if ctx
        .db
        .character_name_identity()
        .character_id()
        .find(u64::from(character_id))
        .is_some()
    {
        ctx.db.character_name_identity().character_id().update(row);
    } else {
        ctx.db.character_name_identity().insert(row);
    }
    Ok(())
}

pub(crate) fn assign_authored_character_name(
    ctx: &ReducerContext,
    character_id: CharacterId,
    name: String,
) -> Result<(), CharacterNameError> {
    let mut identity = authored_name_identity(name);
    identity.surname_id = character_hereditary_surname(ctx, character_id)?;
    assign_character_name_identity(ctx, character_id, identity)
}

pub(crate) fn assign_generated_historical_name(
    ctx: &ReducerContext,
    character_id: CharacterId,
    stable_seed: NameStableSeed,
    birth_year: CalendarYear,
    inherited_surname: Option<SurnameId>,
) -> Result<SurnameId, CharacterNameError> {
    let sex = character_name_sex(ctx, character_id)?;
    let identity = generated_historical_identity(sex, stable_seed, birth_year, inherited_surname)?;
    let surname = identity
        .surname_id
        .clone()
        .ok_or(CharacterNameError::MissingGeneratedSurname)?;
    assign_character_name_identity(ctx, character_id, identity)?;
    Ok(surname)
}

pub(crate) fn generated_historical_identity(
    sex: Sex,
    stable_seed: NameStableSeed,
    birth_year: CalendarYear,
    inherited_surname: Option<SurnameId>,
) -> Result<PersonalNameIdentity, NameCatalogError> {
    generate_personal_name(
        NameGenerationContext::german_lutheran(sex, birth_year),
        stable_seed,
        inherited_surname,
    )
}

pub(crate) fn assign_generated_historical_name_for_age(
    ctx: &ReducerContext,
    character_id: CharacterId,
    stable_seed: NameStableSeed,
    minute: StrategicMinute,
    inherited_surname: Option<SurnameId>,
) -> Result<SurnameId, CharacterNameError> {
    let age_years = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .ok_or(CharacterNameError::MissingCharacter(character_id))?
        .age_years;
    assign_generated_historical_name(
        ctx,
        character_id,
        stable_seed,
        minute
            .birth_year_for_age(age_years)
            .ok_or(CharacterNameError::AgePredatesCalendar)?,
        inherited_surname,
    )
}

pub(crate) fn assign_newborn_historical_name(
    ctx: &ReducerContext,
    child_id: CharacterId,
    father_id: CharacterId,
    mother_id: CharacterId,
    due_minute: StrategicMinute,
    stable_seed: NameStableSeed,
    sex: Sex,
) -> Result<(), CharacterNameError> {
    let mut personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(u64::from(child_id))
        .ok_or(CharacterNameError::MissingPersonality(child_id))?;
    personality.sex = sex;
    personality.presentation = match sex {
        Sex::Female => crate::personality::Presentation::Woman,
        Sex::Male => crate::personality::Presentation::Man,
    };
    ctx.db
        .character_personality()
        .character_id()
        .update(personality);
    let inherited_surname = match character_hereditary_surname(ctx, father_id)? {
        Some(surname) => Some(surname),
        None => character_hereditary_surname(ctx, mother_id)?,
    };
    assign_generated_historical_name(
        ctx,
        child_id,
        stable_seed,
        due_minute.calendar_year(),
        inherited_surname,
    )?;
    Ok(())
}

pub(crate) fn character_hereditary_surname(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<Option<SurnameId>, CharacterNameError> {
    let row = ctx
        .db
        .character_name_identity()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(CharacterNameError::MissingIdentity(character_id))?;
    let identity = NameIdentityJson::parse(row.identity_json.as_str())?;
    render_personal_name(
        &identity,
        identity.native_culture,
        NameRegister::Everyday,
        character_name_sex(ctx, character_id)?,
    )?;
    Ok(identity.surname_id)
}

fn delete_character_name_data(ctx: &ReducerContext, character_id: CharacterId) {
    if ctx
        .db
        .character_name_identity()
        .character_id()
        .find(u64::from(character_id))
        .is_some()
    {
        ctx.db
            .character_name_identity()
            .character_id()
            .delete(u64::from(character_id));
    }
}

fn persistent_character_has_name_identity(ctx: &ReducerContext, character_id: CharacterId) -> bool {
    ctx.db
        .character()
        .id()
        .find(u64::from(character_id))
        .is_none_or(|character| {
            character.temporary
                || ctx
                    .db
                    .character_name_identity()
                    .character_id()
                    .find(u64::from(character_id))
                    .is_some()
        })
}

fn missing_name_identity(ctx: &ReducerContext, character_id: CharacterId) -> Vec<&'static str> {
    (!persistent_character_has_name_identity(ctx, character_id))
        .then_some("name_identity")
        .into_iter()
        .collect()
}
