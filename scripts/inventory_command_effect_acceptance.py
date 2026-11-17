"""Real inventory effects, rollback, replay and competing read sets."""

from __future__ import annotations

import copy

from inventory_command_peer import InventoryPeer, durable_inventory, insert, item, place, remove, transfer

PARENT = "authority-command-parent"
CHILD = "authority-command-child"
BLOCKER = "authority-command-blocker"
BOX = "authority-command-box"
COUNTED = "authority-projection-counted"
CONTAINER = "authority-projection-container"
FOOD = "authority-projection-lot"


def reject_action(peer: InventoryPeer, actor: int, action: dict) -> None:
    peer.reject_unchanged(peer.prepare(peer.snapshot(actor)["state"], action))


def equipment_and_container_checks(peer: InventoryPeer, actor: int) -> None:
    source = peer.snapshot(actor)
    parent, blocker = item(source, PARENT), item(source, BLOCKER)
    box, tight = item(source, BOX), item(source, "authority-command-tight-box")
    solid = item(source, "authority-command-solid")
    children = [row for row in source["items"] if row["catalog"] == CHILD]
    first, second = sorted(children, key=lambda row: row["id"])
    peer.apply_change(actor, insert(solid["object"], box["object"]))
    reject_action(peer, actor, insert(solid["object"], tight["object"]))
    reject_action(peer, actor, insert(box["object"], box["object"]))
    peer.apply_change(actor, {"RemoveFromContainer": solid["object"]})
    reject_action(peer, actor, {"RemoveFromContainer": solid["object"]})

    peer.apply_change(actor, place(blocker["id"]))
    peer.apply_change(actor, insert(parent["object"], box["object"]))
    reject_action(peer, actor, place(parent["id"]))
    peer.apply_change(actor, place(parent["id"], replace=True))
    equipped = peer.snapshot(actor)
    assert item(equipped, PARENT)["placement"] == {"Authored": "mounted"}
    assert item(equipped, BLOCKER)["placement"] is None
    peer.apply_change(actor, place(first["id"], parent=parent["id"]))
    peer.apply_change(actor, insert(second["object"], box["object"]))
    reject_action(peer, actor, place(second["id"], parent=parent["id"]))
    peer.apply_change(actor, place(second["id"], parent=parent["id"], replace=True))
    reject_action(peer, actor, remove(parent["id"]))
    peer.apply_change(actor, remove(second["id"]))
    peer.apply_change(actor, remove(parent["id"]))
    peer.apply_change(actor, place(parent["id"]))

    combined = item(peer.snapshot(actor), "authority-command-combined")
    peer.apply_change(actor, insert(combined["object"], box["object"]))
    reject_action(peer, actor, place(combined["id"], parent=parent["id"], replace=True))
    print("PASS authored equipment, atomic replacement, attachment capacity, "
          "orphan rejection, container cycles/capacity and detachment rollback", flush=True)


def counted_transfer_checks(peer: InventoryPeer, source: int, destination: int) -> None:
    before_source, before_destination = peer.snapshot(source), peer.snapshot(destination)
    stock = item(before_source, COUNTED)
    action = transfer(stock["id"], before_destination["state"])
    requests = [peer.prepare(before_source["state"], action) for _ in range(2)]
    count_before = len(peer.receipts())
    for request in requests:
        peer.send("inventory", request=request)
    replies = [peer.receive(), peer.receive()]
    assert {reply["key"] for reply in replies} == {request["command_id"] for request in requests}
    assert sorted(reply["status"] for reply in replies) == ["accepted", "rejected"], replies
    current_source = peer.changed(source, before_source)
    current_destination = peer.changed(destination, before_destination)
    assert item(current_source, COUNTED)["quantity"] == stock["quantity"] - 1
    assert item(current_destination, COUNTED)["quantity"] == item(before_destination, COUNTED)["quantity"] + 1
    assert len(peer.receipts()) == count_before + 1

    stale_destination = copy.deepcopy(current_destination["state"])
    peer.client.call("authority_test_inventory_command_material", peer.client.bootstrap_token,
                     destination, {"foodQuality": []}, token=peer.token)
    peer.changed(destination, current_destination)
    reject_action(peer, source, transfer(item(current_source, COUNTED)["id"], stale_destination))
    reject_action(peer, source, transfer(item(current_source, COUNTED)["id"],
                                        peer.snapshot(destination)["state"], quantity=0))

    previous_source, previous_destination = peer.snapshot(source), peer.snapshot(destination)
    peer.client.call("authority_test_inventory_command_material", peer.client.bootstrap_token,
                     destination, {"countedInstance": []}, token=peer.token)
    current_source = peer.changed(source, previous_source)
    current_destination = peer.changed(destination, previous_destination)
    discrete = item(current_destination, COUNTED)
    assert discrete["quantity"] == 1 and discrete["object"] is not None
    peer.apply_change(source, transfer(item(current_source, COUNTED)["id"], current_destination["state"]))
    after = peer.changed(destination, current_destination)
    preserved = next(row for row in after["items"] if row["object"] == discrete["object"])
    assert preserved["id"] == discrete["id"] and preserved["quantity"] == 1

    before_source, before_destination = peer.snapshot(source), peer.snapshot(destination)
    remaining = item(before_source, COUNTED)
    depleted = peer.apply_change(source, transfer(remaining["id"], before_destination["state"],
                                                  quantity=remaining["quantity"]))
    peer.changed(destination, before_destination)
    assert peer.client.rows(f"SELECT id FROM inventory_item WHERE id = {remaining['id']}") == []
    committed, receipts = durable_inventory(peer.client), peer.receipts()
    peer.invoke(depleted)
    assert durable_inventory(peer.client) == committed and peer.receipts() == receipts
    print("PASS competing commands commit once, stale recipient/zero quantity "
          "rejection, discrete recipient preservation and replay after source deletion", flush=True)


def subtree_and_replay_checks(peer: InventoryPeer, source: int, destination: int) -> dict:
    current_source = peer.snapshot(source)
    food, container = item(current_source, FOOD), item(current_source, CONTAINER)
    peer.apply_change(source, insert(food["object"], container["object"]))
    before_source, before_destination = peer.snapshot(source), peer.snapshot(destination)
    object_ids = {item(before_source, catalog)["object"] for catalog in
                  (FOOD, CONTAINER, "authority-projection-stock")}
    liquid_before = peer.client.rows(
        f"SELECT * FROM container_liquid WHERE container_object_id = {container['object']}"
    )
    lot_material = "SELECT id, mass_kg, nutrition_kcal, total_value, material_revision FROM food_lot"
    lot_before = sorted(peer.client.rows(lot_material))
    assert lot_before
    fractions_before = sorted(peer.client.rows("SELECT remaining_fraction_micros FROM inventory_item_amount"))
    edges_before = sorted(peer.client.rows("SELECT * FROM inventory_containment"))
    request = peer.prepare(before_source["state"], transfer(container["id"], before_destination["state"]))
    peer.invoke(request)
    after_source, after_destination = peer.changed(source, before_source), peer.changed(destination, before_destination)
    destination_objects = {row["object"] for row in after_destination["items"]}
    assert object_ids <= destination_objects
    assert not object_ids & {row["object"] for row in after_source["items"]}
    moved = {row["object"]: row for row in after_destination["items"]}
    for row in before_source["items"]:
        if row["object"] in object_ids:
            assert moved[row["object"]] == row
    assert peer.client.rows(
        f"SELECT character_id FROM inventory_item WHERE id = {container['id']}"
    ) == [[destination]]
    assert peer.client.rows(f"SELECT * FROM container_liquid WHERE container_object_id = {container['object']}") == liquid_before
    assert sorted(peer.client.rows(lot_material)) == lot_before
    assert sorted(peer.client.rows("SELECT remaining_fraction_micros FROM inventory_item_amount")) == fractions_before
    assert sorted(peer.client.rows("SELECT * FROM inventory_containment")) == edges_before
    assert total_mass(before_source, before_destination) == total_mass(after_source, after_destination)

    committed = durable_inventory(peer.client)
    receipt = peer.receipts()
    peer.invoke(request)
    assert durable_inventory(peer.client) == committed
    assert peer.receipts() == receipt
    conflicting = copy.deepcopy(request)
    conflicting["action"]["Transfer"]["quantity"] = 2
    peer.reject_unchanged(conflicting)
    print("PASS complete subtree transfer, food/fraction/liquid and mass "
          "preservation, exact successful replay and command identity collision", flush=True)
    return request


def recipe_and_condition_transfer_checks(peer: InventoryPeer, source: int, destination: int) -> None:
    current = peer.snapshot(source)
    parent = item(current, PARENT)
    peer.apply_change(source, remove(parent["id"]))
    condition_before = peer.client.rows(
        "SELECT tier_1, tier_2, tier_3, tier_4, tier_5 FROM item_condition "
        f"WHERE inventory_item_id = {parent['id']}"
    )
    before_destination = peer.snapshot(destination)
    peer.apply_change(source, transfer(parent["id"], before_destination["state"]))
    after = peer.changed(destination, before_destination)
    moved_parent = next(row for row in after["items"] if row["object"] == parent["object"])
    assert peer.client.rows(
        "SELECT tier_1, tier_2, tier_3, tier_4, tier_5 FROM item_condition "
        f"WHERE inventory_item_id = {moved_parent['id']}"
    ) == condition_before
    for table in ("weapon_instance", "weapon_holder_instance"):
        instances = peer.client.rows(f"SELECT physical_object_id FROM {table}")
        source_items = peer.snapshot(source)["items"]
        carried = next(row for row in source_items if [row["object"]] in instances)
        before = peer.client.rows(f"SELECT * FROM {table} WHERE physical_object_id = {carried['object']}")
        before_destination = peer.snapshot(destination)
        peer.apply_change(source, transfer(carried["id"], before_destination["state"]))
        after = peer.changed(destination, before_destination)
        assert carried["object"] in {row["object"] for row in after["items"]}
        assert peer.client.rows(f"SELECT * FROM {table} WHERE physical_object_id = {carried['object']}") == before
    print("PASS condition, evaluated weapon and fitted holder recipes survive transfers", flush=True)


def total_mass(*snapshots: dict) -> float:
    return sum(row["mass_kg"] for snapshot in snapshots for row in snapshot["items"])
