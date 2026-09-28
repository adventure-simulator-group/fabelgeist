use super::*;
use adventuresim_building_generator::BuildingArchetype;

#[test]
fn massive_city_workers_prepare_only_shared_exteriors() {
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(input).unwrap();
    assert!(requests.len() <= 1 + BuildingArchetype::ALL.len() * 3);
    assert!(matches!(
        serde_json::from_str::<GenerationJob>(&requests[0]).unwrap(),
        GenerationJob::Scene(_)
    ));
    assert_eq!(requests, jobs(input).unwrap());
    println!("massive city generation jobs: {}", requests.len());
}

#[test]
fn worker_products_round_trip_geometry_and_reject_wrong_inputs() {
    let program = BuildingProgram::fixture(BuildingArchetype::FachwerkCottage, u64::MAX);
    let job = serde_json::to_string(&GenerationJob::Building(Box::new(program.clone()))).unwrap();
    let bytes = generate(&job).unwrap();
    receive(&job, &bytes).unwrap();
    let actual = take_facade(&program).unwrap();
    let expected = PreparedFacade::generate(program.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&actual).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert!(take_facade(&program).is_err());
    assert!(receive(&job, b"truncated").is_err());
    let mut changed = program;
    changed.seed = 42;
    let changed = serde_json::to_string(&GenerationJob::Building(Box::new(changed))).unwrap();
    assert!(receive(&changed, &bytes).is_err());
}

#[test]
fn scene_transport_preserves_static_assets_and_full_width_seed() {
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.seed = u64::MAX;
    let request = serde_json::to_string(&input).unwrap();
    let jobs = jobs(&request).unwrap();
    assert!(jobs[0].contains(&u64::MAX.to_string()));
    let bytes = generate(&jobs[0]).unwrap();
    receive(&jobs[0], &bytes).unwrap();
    let actual = take_scene(&input).unwrap();
    let expected = input.generate().unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert!(take_scene(&input).is_err());
    assert!(super::jobs("{}").is_err());
}
