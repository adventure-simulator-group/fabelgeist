// Owns contamination contribution digests and proportional updates.
fn contamination_provenance_digest(ids: &[String], loads: &[f32]) -> String {
    use sha2::Digest as _;
    let mut hash = sha2::Sha256::new();
    hash.update(b"food-contamination-provenance-v1");
    for (id, load) in ids.iter().zip(loads) {
        hash.update((id.len() as u64).to_le_bytes());
        hash.update(id.as_bytes());
        hash.update(load.to_bits().to_le_bytes());
    }
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn split_food_contamination_provenance(
    ctx: &ReducerContext,
    source_food_lot_id: u64,
    destination_food_lot_id: u64,
    child_ratio: f32,
) {
    if let Some(mut provenance) = ctx
        .db
        .food_contamination_provenance()
        .food_lot_id()
        .find(source_food_lot_id)
    {
        let child_loads = provenance
            .contribution_loads
            .iter()
            .map(|load| load * child_ratio)
            .collect::<Vec<_>>();
        provenance.contribution_loads = provenance
            .contribution_loads
            .iter()
            .map(|load| load * (1.0 - child_ratio))
            .collect();
        provenance.contribution_digest = contamination_provenance_digest(
            &provenance.contribution_ids,
            &provenance.contribution_loads,
        );
        ctx.db
            .food_contamination_provenance()
            .food_lot_id()
            .update(provenance.clone());
        ctx.db
            .food_contamination_provenance()
            .insert(FoodContaminationProvenance {
                food_lot_id: destination_food_lot_id,
                contribution_digest: contamination_provenance_digest(
                    &provenance.contribution_ids,
                    &child_loads,
                ),
                contribution_ids: provenance.contribution_ids,
                contribution_loads: child_loads,
            });
    }
}

fn consume_food_contamination_provenance(
    ctx: &ReducerContext,
    food_lot_id: u64,
    consumed_ratio: f32,
) {
    if let Some(mut provenance) = ctx
        .db
        .food_contamination_provenance()
        .food_lot_id()
        .find(food_lot_id)
    {
        if consumed_ratio >= 0.999_999 {
            ctx.db
                .food_contamination_provenance()
                .food_lot_id()
                .delete(food_lot_id);
        } else {
            for load in &mut provenance.contribution_loads {
                *load *= 1.0 - consumed_ratio;
            }
            provenance.contribution_digest = contamination_provenance_digest(
                &provenance.contribution_ids,
                &provenance.contribution_loads,
            );
            ctx.db
                .food_contamination_provenance()
                .food_lot_id()
                .update(provenance);
        }
    }
}
