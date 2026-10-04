// Owns ordered deletion of character-owned strategic state.

#[derive(Clone, Copy, Eq, PartialEq)]
enum CharacterPartyDeletion {
    DeleteTemporaryParty,
    PreserveParty,
}

/// Transactionally delete a temporary tactical character and every durable
/// strategic row that is owned by it. This must run before deleting Character
/// so no orphan can survive a successful reducer commit.
pub(crate) fn delete_temporary_character(
    ctx: &ReducerContext,
    character: Character,
) -> Result<(), CharacterDeletionError> {
    if !character.temporary {
        return Err(CharacterDeletionError::PersistentCharacter(
            character.id.into(),
        ));
    }
    delete_character_name_data(ctx, CharacterId::from(character.id));
    delete_character_data(ctx, character, CharacterPartyDeletion::DeleteTemporaryParty)
}

pub(crate) fn delete_character_for_world_import(
    ctx: &ReducerContext,
    character: Character,
) -> Result<(), CharacterDeletionError> {
    delete_character_name_data(ctx, CharacterId::from(character.id));
    delete_character_data(ctx, character, CharacterPartyDeletion::PreserveParty)
}

fn delete_character_data(
    ctx: &ReducerContext,
    character: Character,
    party_deletion: CharacterPartyDeletion,
) -> Result<(), CharacterDeletionError> {
    if party_deletion == CharacterPartyDeletion::DeleteTemporaryParty {
        if let Some(party_id) = character.party_id.as_deref() {
            crate::strategic::delete_temporary_character_party(ctx, character.id, party_id)?;
        } else {
            for membership in ctx
                .db
                .party_member()
                .character_id()
                .filter(character.id)
                .collect::<Vec<_>>()
            {
                ctx.db.party_member().id().delete(membership.id);
            }
        }
    }

    // Repair custody changes InventoryItem.character_id to zero while retaining
    // the real owner on RepairOrder. Remove those custody rows before scanning
    // ordinary owned inventory so a temporary character cannot leave orphaned
    // smith inventory behind.
    for order in ctx
        .db
        .repair_order()
        .owner_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        crate::inventory_container::delete_repair_object_for_row(ctx, order.inventory_item_id)?;
        if ctx
            .db
            .item_condition()
            .inventory_item_id()
            .find(order.inventory_item_id)
            .is_some()
        {
            ctx.db
                .item_condition()
                .inventory_item_id()
                .delete(order.inventory_item_id);
        }
        ctx.db.inventory_item().id().delete(order.inventory_item_id);
        ctx.db.repair_order().id().delete(order.id);
    }

    let inventory = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>();
    for row in inventory {
        if crate::inventory_container::delete_carried_object_for_row(
            ctx,
            adventuresim_core::physical_object::CarriedInventoryScope::Personal,
            row.id,
        )? {
            continue;
        }
        if ctx
            .db
            .item_condition()
            .inventory_item_id()
            .find(row.id)
            .is_some()
        {
            ctx.db.item_condition().inventory_item_id().delete(row.id);
        }
        for repair in ctx
            .db
            .repair_order()
            .iter()
            .filter(|repair| repair.inventory_item_id == row.id)
            .collect::<Vec<_>>()
        {
            ctx.db.repair_order().id().delete(repair.id);
        }
        ctx.db.inventory_item().id().delete(row.id);
        crate::food::delete_personal_food_lot(ctx, row.id);
    }
    for row in ctx
        .db
        .physiology_administration()
        .administration_patient_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.physiology_administration().id().delete(row.id);
    }

    for row in ctx
        .db
        .infection_episode()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.infection_episode().id().delete(row.id);
    }
    for row in ctx
        .db
        .committed_cut()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.committed_cut().id().delete(row.id);
    }
    for row in ctx
        .db
        .disease_notice()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.disease_notice().id().delete(&row.id);
    }
    for row in ctx
        .db
        .morale_event()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.morale_event().id().delete(row.id);
    }
    for row in ctx
        .db
        .character_morale_source()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.character_morale_source().id().delete(&row.id);
    }
    crate::social::cleanup_character_social(ctx, character.id);
    for row in ctx
        .db
        .religious_demand()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.religious_demand().id().delete(row.id);
    }
    for row in ctx
        .db
        .inventory_quantity_target()
        .owner_character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.inventory_quantity_target().id().delete(&row.id);
    }
    for row in ctx
        .db
        .alcohol_consumption()
        .by_character()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.alcohol_consumption().id().delete(&row.id);
    }
    for row in ctx
        .db
        .organization_membership()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.organization_membership().id().delete(row.id);
    }
    crate::social_roles::delete_character_social_roles(ctx, character.id);
    if ctx
        .db
        .organization_presentation()
        .character_id()
        .find(character.id)
        .is_some()
    {
        ctx.db
            .organization_presentation()
            .character_id()
            .delete(character.id);
    }

    ctx.db.character_stats().character_id().delete(character.id);
    ctx.db
        .character_skills()
        .character_id()
        .delete(character.id);
    ctx.db.character_time().character_id().delete(character.id);
    ctx.db
        .character_training_schedule()
        .character_id()
        .delete(character.id);
    crate::reputation::delete_character_reputation(ctx, character.id);
    crate::strategic::delete_activity_incident_entropy(ctx, character.id);
    for injury in ctx
        .db
        .limb_injury()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.limb_injury().id().delete(injury.id);
    }
    for projectile in ctx
        .db
        .retained_projectile()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db.retained_projectile().id().delete(projectile.id);
    }
    ctx.db.character_limbs().character_id().delete(character.id);
    for row in ctx
        .db
        .character_equipped_item()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        unequip_wearable(ctx, row.inventory_item_id);
    }
    ctx.db
        .character_attributes()
        .character_id()
        .delete(character.id);
    ctx.db
        .character_personality()
        .character_id()
        .delete(character.id);
    ctx.db
        .character_personality_scores()
        .character_id()
        .delete(character.id);
    // Development is character-owned audit history in this pre-launch model;
    // canonical deletion removes it so a reused development source cannot
    // point at a character that no longer exists.
    for event in ctx
        .db
        .personality_development_event()
        .character_id()
        .filter(character.id)
        .collect::<Vec<_>>()
    {
        ctx.db
            .personality_development_event()
            .source_id()
            .delete(&event.source_id);
    }
    if ctx
        .db
        .npc_policy()
        .character_id()
        .find(character.id)
        .is_some()
    {
        ctx.db.npc_policy().character_id().delete(character.id);
    }
    ctx.db
        .character_capability()
        .character_id()
        .delete(character.id);
    ctx.db
        .character_condition()
        .character_id()
        .delete(character.id);
    ctx.db.character_needs().character_id().delete(character.id);
    ctx.db
        .character_exposure()
        .character_id()
        .delete(character.id);
    ctx.db
        .character_strategic_condition()
        .character_id()
        .delete(character.id);
    if ctx
        .db
        .character_illness_status()
        .character_id()
        .find(character.id)
        .is_some()
    {
        ctx.db
            .character_illness_status()
            .character_id()
            .delete(character.id);
    }
    if ctx
        .db
        .character_death()
        .character_id()
        .find(character.id)
        .is_some()
    {
        ctx.db.character_death().character_id().delete(character.id);
    }
    ctx.db.character().delete(character);
    Ok(())
}
