// Persistent semantic personal-name authority and projection helpers.

use adventuresim_world_schema::person_names::{
    GivenNameResolution, NameBirthYear, NameCulture, NameGenerationContext, NameRegister, NameSex,
    NameStableSeed, PersonalNameIdentity, SurnameId,
    generate_personal_name, render_personal_name,
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

fn character_name_sex(ctx: &ReducerContext, character_id: CharacterId) -> Result<NameSex, String> {
    let personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(character_id.get())
        .ok_or_else(|| format!("Character {} has no personality", character_id.get()))?;
    Ok(match personality.sex {
        crate::personality::Sex::Female => NameSex::Female,
        crate::personality::Sex::Male => NameSex::Male,
    })
}

fn authored_name_identity(name: String) -> PersonalNameIdentity {
    PersonalNameIdentity::authored(name, NameCulture::German)
}

/// Persist a semantic identity and refresh `Character.name`'s native-everyday
/// projection.
pub(crate) fn assign_character_name_identity(
    ctx: &ReducerContext,
    character_id: CharacterId,
    identity: PersonalNameIdentity,
) -> Result<(), String> {
    let sex = character_name_sex(ctx, character_id)?;
    let display = render_personal_name(
        &identity,
        identity.native_culture,
        NameRegister::Everyday,
        sex,
    )
    .map_err(|error| error.to_string())?;
    let mut character = ctx
        .db
        .character()
        .id()
        .find(character_id.get())
        .ok_or_else(|| format!("Character {} not found", character_id.get()))?;
    character.name = display.into_string();
    ctx.db.character().id().update(character);
    let row = CharacterNameIdentity {
        character_id: character_id.get(),
        identity_json: NameIdentityJson::from_identity(&identity)?,
    };
    if ctx
        .db
        .character_name_identity()
        .character_id()
        .find(character_id.get())
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
) -> Result<(), String> {
    let mut identity = authored_name_identity(name);
    identity.surname_id = character_hereditary_surname(ctx, character_id);
    assign_character_name_identity(ctx, character_id, identity)
}

pub(crate) fn character_name_is_authored(ctx: &ReducerContext, character_id: CharacterId) -> bool {
    ctx.db
        .character_name_identity()
        .character_id()
        .find(character_id.get())
        .and_then(|row| NameIdentityJson::parse(row.identity_json.as_str()).ok())
        .is_some_and(|identity| matches!(identity.given, GivenNameResolution::Authored { .. }))
}

pub(crate) fn assign_generated_historical_name(
    ctx: &ReducerContext,
    character_id: CharacterId,
    stable_seed: NameSeed,
    birth_year: NameBirthYear,
    inherited_surname: Option<SurnameId>,
) -> Result<SurnameId, String> {
    let sex = character_name_sex(ctx, character_id)?;
    let identity = generate_personal_name(
        NameGenerationContext::german_lutheran(sex, birth_year),
        NameStableSeed::new(stable_seed.get()),
        inherited_surname,
    )
    .map_err(|error| error.to_string())?;
    let surname = identity
        .surname_id
        .clone()
        .ok_or("Generated German identity has no hereditary surname")?;
    assign_character_name_identity(ctx, character_id, identity)?;
    Ok(surname)
}

pub(crate) fn assign_generated_historical_name_for_age(
    ctx: &ReducerContext,
    character_id: CharacterId,
    stable_seed: NameSeed,
    minute: WorldMinute,
    inherited_surname: Option<SurnameId>,
) -> Result<SurnameId, String> {
    let age_years = ctx
        .db
        .character()
        .id()
        .find(character_id.get())
        .ok_or("Named character was not created")?
        .age_years;
    assign_generated_historical_name(
        ctx,
        character_id,
        stable_seed,
        NameBirthYear::new(adventuresim_core::strategic_time::birth_year_from_age(
            minute.get(),
            age_years,
        )),
        inherited_surname,
    )
}

pub(crate) fn assign_newborn_historical_name(
    ctx: &ReducerContext,
    child_id: CharacterId,
    father_id: CharacterId,
    mother_id: CharacterId,
    due_minute: WorldMinute,
    stable_seed: NameSeed,
    sex: crate::personality::Sex,
) -> Result<(), String> {
    let mut personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(child_id.get())
        .ok_or("Newborn has no personality")?;
    personality.sex = sex;
    personality.presentation = match sex {
        crate::personality::Sex::Female => crate::personality::Presentation::Woman,
        crate::personality::Sex::Male => crate::personality::Presentation::Man,
    };
    ctx.db
        .character_personality()
        .character_id()
        .update(personality);
    let inherited_surname = character_hereditary_surname(ctx, father_id)
        .or_else(|| character_hereditary_surname(ctx, mother_id));
    assign_generated_historical_name(
        ctx,
        child_id,
        stable_seed,
        NameBirthYear::new(adventuresim_core::strategic_time::world_year_at(due_minute.get())),
        inherited_surname,
    )?;
    Ok(())
}

pub(crate) fn assign_generated_name_demographics(
    ctx: &ReducerContext,
    character_id: CharacterId,
    sex: crate::personality::Sex,
    age_years: u16,
) -> Result<(), String> {
    let mut personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(character_id.get())
        .ok_or("Resident character has no personality")?;
    personality.sex = sex;
    personality.presentation = match sex {
        crate::personality::Sex::Female => crate::personality::Presentation::Woman,
        crate::personality::Sex::Male => crate::personality::Presentation::Man,
    };
    ctx.db
        .character_personality()
        .character_id()
        .update(personality);
    crate::relationship::set_seeded_character_birth_from_age(ctx, character_id.get(), age_years);
    Ok(())
}

pub(crate) fn character_hereditary_surname(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Option<SurnameId> {
    ctx.db
        .character_name_identity()
        .character_id()
        .find(character_id.get())
        .and_then(|row| NameIdentityJson::parse(row.identity_json.as_str()).ok())
        .and_then(|identity| identity.surname_id)
}

fn delete_character_name_data(ctx: &ReducerContext, character_id: CharacterId) {
    if ctx
        .db
        .character_name_identity()
        .character_id()
        .find(character_id.get())
        .is_some()
    {
        ctx.db
            .character_name_identity()
            .character_id()
            .delete(character_id.get());
    }
}

fn persistent_character_has_name_identity(ctx: &ReducerContext, character_id: CharacterId) -> bool {
    ctx.db
        .character()
        .id()
        .find(character_id.get())
        .is_none_or(|character| {
            character.temporary
                || ctx
                    .db
                    .character_name_identity()
                    .character_id()
                    .find(character_id.get())
                    .is_some()
        })
}

fn missing_name_identity(ctx: &ReducerContext, character_id: CharacterId) -> Vec<&'static str> {
    (!persistent_character_has_name_identity(ctx, character_id))
        .then_some("name_identity")
        .into_iter()
        .collect()
}
