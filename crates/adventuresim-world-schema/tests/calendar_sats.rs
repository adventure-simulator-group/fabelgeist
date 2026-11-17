#![cfg(feature = "spacetimedb")]

use adventuresim_world_schema::calendar::CalendarYear;
use spacetimedb_lib::{
    AlgebraicType, SpacetimeType, bsatn, de::Deserialize, sats::typespace::TypespaceBuilder,
};

#[test]
fn database_calendar_year_preserves_its_product_and_rejects_invalid_values() {
    struct Schema;
    impl TypespaceBuilder for Schema {
        fn add(
            &mut self,
            _type_id: std::any::TypeId,
            name: Option<&'static str>,
            make_type: impl FnOnce(&mut Self) -> AlgebraicType,
        ) -> AlgebraicType {
            assert_eq!(name, Some("CalendarYear"));
            make_type(self)
        }
    }

    let AlgebraicType::Product(record) = CalendarYear::make_type(&mut Schema) else {
        panic!("calendar year must remain a named product");
    };
    let [year] = record.elements.as_ref() else {
        panic!("calendar year must have one field");
    };
    assert_eq!(year.name.as_deref(), Some("year"));
    assert_eq!(year.algebraic_type, AlgebraicType::I32);

    for raw in [1_i32, i32::MAX] {
        let year = CalendarYear::new(raw).unwrap();
        let bytes = bsatn::to_vec(&year).unwrap();
        assert_eq!(bytes, raw.to_le_bytes());
        assert_eq!(bsatn::from_slice::<CalendarYear>(&bytes).unwrap(), year);
        assert_eq!(year.years_since_first(), raw - 1);
    }
    for raw in [0_i32, -1, i32::MIN] {
        let bytes = raw.to_le_bytes();
        assert!(bsatn::from_slice::<CalendarYear>(&bytes).is_err());
        let mut input = bytes.as_slice();
        assert!(CalendarYear::validate(bsatn::Deserializer::new(&mut input)).is_err());

        let nested = bsatn::to_vec(&vec![1_i32, raw]).unwrap();
        assert!(bsatn::from_slice::<Vec<CalendarYear>>(&nested).is_err());
    }

    let valid = bsatn::to_vec(&vec![1_i32, i32::MAX]).unwrap();
    let years = bsatn::from_slice::<Vec<CalendarYear>>(&valid).unwrap();
    assert_eq!(
        years.iter().map(|year| year.get()).collect::<Vec<_>>(),
        [1, i32::MAX]
    );
}
