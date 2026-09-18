// Persistent semantic personal-name authority and projection helpers.

use adventuresim_world_schema::person_names::{
    NameCulture, NameGenerationContext, NameRegister, NameSex, PersonalNameIdentity, SurnameId,
    generate_personal_name, render_personal_name,
};

/// Private semantic authority behind [`Character::name`]. The public string is
/// only the invariant native-culture everyday projection used by existing UI.
#[derive(Clone, Debug)]
#[table(accessor = character_name_identity)]
pub struct CharacterNameIdentity {
    #[primary_key]
    pub character_id: u64,
    /// JSON serialization of the shared, strongly typed semantic identity.
    /// SpacetimeDB 2.6 sums cannot represent the resolved/authored variants
    /// directly, so this private row preserves the clean shared schema intact.
    pub identity_json: String,
}

fn character_name_sex(ctx: &ReducerContext, character_id: u64) -> Result<NameSex, String> {
    let personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(character_id)
        .ok_or_else(|| format!("Character {character_id} has no personality"))?;
    Ok(match personality.sex {
        crate::personality::Sex::Female => NameSex::Female,
        crate::personality::Sex::Male => NameSex::Male,
    })
}

fn authored_name_identity(name: String) -> PersonalNameIdentity {
    PersonalNameIdentity::authored(name, NameCulture::German)
}

/// Persist a semantic identity and refresh the compatibility display string.
pub(crate) fn assign_character_name_identity(
    ctx: &ReducerContext,
    character_id: u64,
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
        .find(character_id)
        .ok_or_else(|| format!("Character {character_id} not found"))?;
    character.name = display.into_string();
    ctx.db.character().id().update(character);
    let row = CharacterNameIdentity {
        character_id,
        identity_json: serde_json::to_string(&identity)
            .map_err(|error| format!("Could not serialize character name identity: {error}"))?,
    };
    if ctx
        .db
        .character_name_identity()
        .character_id()
        .find(character_id)
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
    character_id: u64,
    name: String,
) -> Result<(), String> {
    assign_character_name_identity(ctx, character_id, authored_name_identity(name))
}

pub(crate) fn assign_generated_historical_name(
    ctx: &ReducerContext,
    character_id: u64,
    stable_seed: u64,
    birth_year: i32,
    inherited_surname: Option<SurnameId>,
) -> Result<SurnameId, String> {
    let sex = character_name_sex(ctx, character_id)?;
    let identity = generate_personal_name(
        NameGenerationContext::german_lutheran(sex, birth_year),
        stable_seed,
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
    character_id: u64,
    stable_seed: u64,
    minute: u64,
    inherited_surname: Option<SurnameId>,
) -> Result<SurnameId, String> {
    let age_years = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .ok_or("Named character was not created")?
        .age_years;
    assign_generated_historical_name(
        ctx,
        character_id,
        stable_seed,
        adventuresim_core::strategic_time::birth_year_from_age(minute, age_years),
        inherited_surname,
    )
}

pub(crate) fn assign_newborn_historical_name(
    ctx: &ReducerContext,
    child_id: u64,
    father_id: u64,
    mother_id: u64,
    due_minute: u64,
    stable_seed: u64,
    female: bool,
) -> Result<(), String> {
    let mut personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(child_id)
        .ok_or("Newborn has no personality")?;
    personality.sex = if female {
        crate::personality::Sex::Female
    } else {
        crate::personality::Sex::Male
    };
    personality.presentation = if female {
        crate::personality::Presentation::Woman
    } else {
        crate::personality::Presentation::Man
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
        adventuresim_core::strategic_time::world_year_at(due_minute),
        inherited_surname,
    )?;
    Ok(())
}

pub(crate) fn assign_generated_name_demographics(
    ctx: &ReducerContext,
    character_id: u64,
    female: bool,
    age_years: u16,
) -> Result<(), String> {
    let mut personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(character_id)
        .ok_or("Resident character has no personality")?;
    personality.sex = if female {
        crate::personality::Sex::Female
    } else {
        crate::personality::Sex::Male
    };
    personality.presentation = if female {
        crate::personality::Presentation::Woman
    } else {
        crate::personality::Presentation::Man
    };
    ctx.db
        .character_personality()
        .character_id()
        .update(personality);
    crate::relationship::set_seeded_character_birth_from_age(ctx, character_id, age_years);
    Ok(())
}

pub(crate) fn character_hereditary_surname(
    ctx: &ReducerContext,
    character_id: u64,
) -> Option<SurnameId> {
    ctx.db
        .character_name_identity()
        .character_id()
        .find(character_id)
        .and_then(|row| serde_json::from_str::<PersonalNameIdentity>(&row.identity_json).ok())
        .and_then(|identity| identity.surname_id)
}

fn delete_character_name_data(ctx: &ReducerContext, character_id: u64) {
    if ctx
        .db
        .character_name_identity()
        .character_id()
        .find(character_id)
        .is_some()
    {
        ctx.db
            .character_name_identity()
            .character_id()
            .delete(character_id);
    }
}

fn persistent_character_has_name_identity(ctx: &ReducerContext, character_id: u64) -> bool {
    ctx.db
        .character()
        .id()
        .find(character_id)
        .is_none_or(|character| {
            character.temporary
                || ctx
                    .db
                    .character_name_identity()
                    .character_id()
                    .find(character_id)
                    .is_some()
        })
}

fn missing_name_identity(ctx: &ReducerContext, character_id: u64) -> Vec<&'static str> {
    (!persistent_character_has_name_identity(ctx, character_id))
        .then_some("name_identity")
        .into_iter()
        .collect()
}
