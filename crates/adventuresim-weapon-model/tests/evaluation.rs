use adventuresim_weapon_model::*;

#[test]
fn physical_evaluations_match_rendered_weapons_and_fitted_holders() {
    for id in MELEE_CATALOG_IDS {
        let design = default_design(id).unwrap();
        let evaluation = EvaluatedWeapon::new(design.clone()).unwrap();
        assert_eq!(
            evaluation.derived(),
            generate(&design).unwrap().derived,
            "{id}"
        );
        assert_eq!(evaluation.encode().unwrap(), encode(&design).unwrap());
        let decoded = EvaluatedWeapon::decode(&evaluation.encode().unwrap()).unwrap();
        assert_eq!(decoded.design(), &design);
        assert_eq!(decoded.derived(), evaluation.derived());
        if let Some(holder) = default_holder_design(&design) {
            let evaluated = EvaluatedHolder::new(holder.clone()).unwrap();
            assert_eq!(
                evaluated.derived(),
                generate_holder(&holder).unwrap().derived,
                "{id}"
            );
            assert_eq!(evaluated.encode().unwrap(), encode_holder(&holder).unwrap());
            let decoded = EvaluatedHolder::decode(&evaluated.encode().unwrap()).unwrap();
            assert_eq!(decoded.design(), &holder);
            assert_eq!(decoded.derived(), evaluated.derived());
        }
    }
}

#[test]
fn evaluated_recipes_reject_corruption_and_invalid_construction() {
    let mut design = default_design("longsword").unwrap();
    let bytes = encode(&design).unwrap();
    assert!(EvaluatedWeapon::decode(&bytes[..bytes.len() - 1]).is_err());
    design.recipe.components[1].attach.as_mut().unwrap().offset = Some([
        recipe::Metres::new(1.0).unwrap(),
        recipe::Metres::new(0.0).unwrap(),
        recipe::Metres::new(0.0).unwrap(),
    ]);
    assert!(EvaluatedWeapon::new(design).is_err());
    let mut holder = default_holder_design(&default_design("longsword").unwrap()).unwrap();
    holder.clearance = Millimeters(0);
    assert!(EvaluatedHolder::new(holder).is_err());
}
