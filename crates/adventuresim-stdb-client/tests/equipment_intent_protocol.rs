//! Conformance between the portable command boundary and reducer arguments.

use adventuresim_core::equipment_command::EquipmentIntent;
use adventuresim_stdb_client::{
    EquipmentChangeSetRequest, EquipmentChangesRequest, EquipmentConflictPolicy,
    EquipmentDestinationRequest, EquipmentHand, json_reducer_argument,
};
use serde_json::{Value, json};
use spacetimedb_sats::serde::SerdeWrapper;

fn exchange() -> Value {
    json!({
        "changes": {"Exchange": {
            "source": {"inventory": 41, "destination": {"Authored": {
                "placement": 3,
                "targets": [{"requirement": 2, "parent": 17, "point": "blade"}]
            }}},
            "destination": {"inventory": 29, "destination": {"Held": "Left"}}
        }},
        "conflicts": "RejectOccupied"
    })
}

#[test]
fn serialized_exchange_preserves_both_changes_and_authored_selection_in_reducer_arguments() {
    let request = exchange();
    let intent: EquipmentIntent = serde_json::from_value(request.clone()).unwrap();
    assert_eq!(serde_json::to_value(&intent).unwrap(), request);
    let submitted = EquipmentChangeSetRequest::from(intent);
    let argument = json_reducer_argument(&submitted).unwrap();
    let decoded: SerdeWrapper<EquipmentChangeSetRequest> =
        serde_json::from_value(argument).unwrap();
    assert_eq!(decoded.0, submitted);
    assert_eq!(decoded.0.conflicts, EquipmentConflictPolicy::RejectOccupied);
    let EquipmentChangesRequest::Exchange(exchange) = decoded.0.changes else {
        panic!("both changes must be submitted atomically");
    };
    assert_eq!(exchange.source.inventory_item_id, 41);
    assert_eq!(exchange.destination.inventory_item_id, 29);
    assert_eq!(
        exchange.destination.destination,
        EquipmentDestinationRequest::Held(EquipmentHand::Left)
    );
    let EquipmentDestinationRequest::Authored(authored) = exchange.source.destination else {
        panic!("an authored selection must remain distinct from a generic hand");
    };
    assert_eq!(authored.placement_index, 3);
    assert_eq!(authored.targets.len(), 1);
    assert_eq!(authored.targets[0].requirement_index, 2);
    assert_eq!(authored.targets[0].parent_inventory_item_id, 17);
    assert_eq!(authored.targets[0].attachment_point_id, "blade");
}

#[test]
fn portable_admission_rejects_malformed_and_ambiguous_attachment_requests() {
    let original = exchange();
    let pointer = "/changes/Exchange/source/destination/Authored/targets/0";
    let mut requests = Vec::new();
    for (field, invalid) in [
        ("parent", json!(0)),
        ("point", json!("bad/point")),
        ("requirement", json!(-1)),
    ] {
        let mut request = original.clone();
        request.pointer_mut(pointer).unwrap()[field] = invalid;
        requests.push(request);
    }
    let mut duplicate = original.clone();
    let target = duplicate.pointer(pointer).unwrap().clone();
    duplicate
        .pointer_mut("/changes/Exchange/source/destination/Authored/targets")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(target);
    requests.push(duplicate);
    let mut zero_row = original.clone();
    zero_row["changes"]["Exchange"]["source"]["inventory"] = json!(0);
    requests.push(zero_row);
    let mut extra = original;
    extra["revision"] = json!("do-not-inject-a-current-snapshot");
    requests.push(extra);
    for request in requests {
        assert!(
            serde_json::from_value::<EquipmentIntent>(request.clone()).is_err(),
            "{request}"
        );
    }
}

#[test]
fn inventory_and_both_generic_hands_keep_their_destination_and_conflict_policy() {
    for (destination, expected) in [
        (json!("Inventory"), EquipmentDestinationRequest::Inventory),
        (
            json!({"Held": "Left"}),
            EquipmentDestinationRequest::Held(EquipmentHand::Left),
        ),
        (
            json!({"Held": "Right"}),
            EquipmentDestinationRequest::Held(EquipmentHand::Right),
        ),
    ] {
        let intent: EquipmentIntent = serde_json::from_value(json!({
            "changes": {"Single": {"inventory": 41, "destination": destination}},
            "conflicts": "ReplaceOccupied"
        }))
        .unwrap();
        let submitted = EquipmentChangeSetRequest::from(intent);
        let argument = json_reducer_argument(&submitted).unwrap();
        let decoded: SerdeWrapper<EquipmentChangeSetRequest> =
            serde_json::from_value(argument).unwrap();
        assert_eq!(decoded.0, submitted);
        assert_eq!(
            decoded.0.conflicts,
            EquipmentConflictPolicy::ReplaceOccupied
        );
        let EquipmentChangesRequest::Single(change) = decoded.0.changes else {
            panic!("a single change must remain one change");
        };
        assert_eq!(change.inventory_item_id, 41);
        assert_eq!(change.destination, expected);
    }
}
