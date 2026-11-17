"""Atomic generic holding and worn/held exchanges on a real registered host."""

from __future__ import annotations

from inventory_command_peer import (
    InventoryPeer, durable_inventory, equipment_change, exchange, item, remove,
)
from inventory_command_effect_acceptance import CHILD, PARENT, reject_action, total_mass


def held(hand: str) -> dict:
    return {"Held": {hand: []}}


def authored(*, parent: int | None = None) -> dict:
    targets = [] if parent is None else [{"requirement_index": 0,
        "parent_inventory_item_id": parent, "attachment_point_id": "hook"}]
    return {"Authored": {"placement_index": 0, "targets": targets}}


def unchanged_material(before: dict, after: dict) -> None:
    for table in before.keys() - {"character_equipped_item", "equipment_occupancy",
                                  "tactical_inventory_receipt_authority"}:
        removed = sorted(set(before[table]) - set(after[table]))
        added = sorted(set(after[table]) - set(before[table]))
        assert after[table] == before[table], {"table": table, "removed": removed[:4], "added": added[:4]}


def held_exchange_checks(peer: InventoryPeer, actor: int) -> None:
    initial = peer.snapshot(actor)
    tunics = sorted((row for row in initial["items"] if row["catalog"] == "linen_tunic"),
                    key=lambda row: row["id"])
    first, second = tunics[:2]
    torch = item(initial, "torch")
    before = durable_inventory(peer.client)
    peer.apply_change(actor, equipment_change(first["id"], authored(), replace=True))
    peer.apply_change(actor, equipment_change(second["id"], held("Left")))
    peer.apply_change(actor, equipment_change(torch["id"], held("Right")))
    equipped = peer.snapshot(actor)
    rows = {row["id"]: row for row in equipped["items"]}
    assert rows[first["id"]]["placement"] == {"Authored": "worn"}
    assert rows[first["id"]]["protection"]
    assert rows[second["id"]]["placement"] == {"Held": "Left"}
    assert not rows[second["id"]]["protection"]

    request = peer.apply_change(actor, exchange(first["id"], held("Left"),
                                               second["id"], authored()))
    swapped = peer.snapshot(actor)
    rows = {row["id"]: row for row in swapped["items"]}
    assert rows[first["id"]]["placement"] == {"Held": "Left"}
    assert not rows[first["id"]]["protection"]
    assert rows[second["id"]]["placement"] == {"Authored": "worn"}
    assert rows[second["id"]]["protection"]
    assert swapped["state"]["scope"] == initial["state"]["scope"]
    assert swapped["state"]["revision"] != equipped["state"]["revision"]
    unchanged_material(before, durable_inventory(peer.client))
    committed, receipts = durable_inventory(peer.client), peer.receipts()
    peer.invoke(request)
    assert durable_inventory(peer.client) == committed and peer.receipts() == receipts

    peer.apply_change(actor, exchange(first["id"], held("Right"), torch["id"], held("Left")))
    swapped = peer.snapshot(actor)
    rows = {row["id"]: row for row in swapped["items"]}
    assert rows[first["id"]]["placement"] == {"Held": "Right"}
    assert rows[torch["id"]]["placement"] == {"Held": "Left"}
    assert total_mass(swapped) == total_mass(initial)
    unchanged_material(before, durable_inventory(peer.client))

    reject_action(peer, actor, exchange(first["id"], held("Left"),
                                        torch["id"], held("Left"), replace=True))
    reject_action(peer, actor, exchange(first["id"], held("Right"),
                                        first["id"], held("Left"), replace=True))
    # The first placement commits in the reducer before contextual attachment
    # resolution rejects the second. Every vacated source and receipt rolls back.
    child = item(swapped, CHILD)
    reject_action(peer, actor, exchange(torch["id"], held("Right"), child["id"],
                                        authored(parent=second["id"]), replace=True))
    for row in (first, second, torch):
        peer.apply_change(actor, remove(row["id"]))
    unchanged_material(before, durable_inventory(peer.client))
    assert total_mass(peer.snapshot(actor)) == total_mass(initial)
    print("PASS generic held clothing/simple items, worn protection, two-hand and "
          "worn/held exchanges, exact replay, material preservation and full "
          "rollback after conflicting or invalid second destinations", flush=True)


def attachment_exchange_checks(peer: InventoryPeer, actor: int) -> None:
    current = peer.snapshot(actor)
    parent, child = item(current, PARENT), item(current, CHILD)
    peer.apply_change(actor, equipment_change(child["id"], authored(parent=parent["id"])))
    before = durable_inventory(peer.client)
    peer.apply_change(actor, exchange(parent["id"], held("Right"),
                                     child["id"], authored(parent=parent["id"])))
    moved = peer.snapshot(actor)
    assert item(moved, PARENT)["placement"] == {"Held": "Right"}
    assert item(moved, CHILD)["placement"] == {"Authored": "attached"}
    unchanged_material(before, durable_inventory(peer.client))
    reject_action(peer, actor, equipment_change(parent["id"], held("Left")))
    peer.apply_change(actor, exchange(child["id"], {"Inventory": []},
                                     parent["id"], authored()))
    assert item(peer.snapshot(actor), PARENT)["placement"] == {"Authored": "mounted"}
    print("PASS selected attachment parent/child rebuild in dependency order, "
          "unselected-child rejection and explicit child removal", flush=True)
