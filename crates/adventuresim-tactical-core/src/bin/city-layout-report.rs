//! Export the actual city layout as inspectable JSON without starting a game server.
use adventuresim_tactical_core::prelude::generate_city;
use adventuresim_world_schema::{
    FallbackIndustry, IndustryEvidence, InferredIndustryProfile, infer_settlement_economy,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let population = arguments
        .next()
        .map(|s| s.parse::<u32>())
        .transpose()?
        .unwrap_or(6_500);
    let seed = arguments
        .next()
        .map(|s| s.parse::<u64>())
        .transpose()?
        .unwrap_or(42);
    if arguments.next().is_some() {
        return Err("usage: city-layout-report [population] [seed]".into());
    }
    let level = match population {
        0..=249 => 1,
        250..=1_999 => 2,
        2_000..=9_999 => 3,
        _ => 4,
    };
    let industries = InferredIndustryProfile::new(vec![IndustryEvidence::Fallback(
        FallbackIndustry::CroplandGrain,
    )])
    .unwrap();
    let economy = infer_settlement_economy(level, population, 3, level >= 3, &industries)?;
    let city = generate_city(seed, population, &economy);
    let lots = city.lots.iter().map(|lot| {
        let dimensions = lot.dimensions_metres();
        json!({
            "id": lot.id,
            "use": lot.building_use(),
            "label": lot.building_use().map(|usage| usage.definition().label).unwrap_or("Dwelling"),
            "archetype": lot.archetype(),
            "capacity": lot.service.map(|service| service.capacity.0),
            "residents": if lot.service.is_none() { lot.house_class.resident_capacity() } else { 0 },
            "centre": lot.centre_metres.to_array(),
            "dimensions": dimensions.to_array(),
            "yaw_radians": lot.orientation.yaw_radians(),
        })
    }).collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "seed": seed, "population": population, "economy": economy,
            "unhoused_population": city.unhoused_population,
            "unplaced_services": city.unplaced_services.iter().map(|demand| json!({"use": demand.usage, "capacity": demand.capacity.0})).collect::<Vec<_>>(),
            "streets": city.streets, "yards": city.yards, "lots": lots,
        }))?
    );
    Ok(())
}
