"""Repair preserves generic held or authored placement through real custody."""

from __future__ import annotations

from inventory_command_peer import InventoryPeer, durable_inventory, equipment_change, remove
from inventory_equipment_exchange_acceptance import authored, held


def repair_equipment_checks(peer: InventoryPeer, settlement: str) -> None:
    client, actor = peer.client, 735100
    client.fixture("authority_test_settlement_actor", actor, settlement, True)
    [[tunic], *_] = client.rows(
        f"SELECT id FROM inventory_item WHERE character_id = {actor} AND item_id = 'linen_tunic'")
    [[torch], *_] = client.rows(
        f"SELECT id FROM inventory_item WHERE character_id = {actor} AND item_id = 'torch'")

    def change(action: dict) -> None:
        client.call("change_equipment", actor, peer.equipment_argument(action))

    for destination in (held("Left"), authored()):
        change(equipment_change(tunic, destination, replace=True))
        original = client.rows(f"SELECT placement FROM character_equipped_item WHERE inventory_item_id = {tunic}")
        objects = client.rows(f"SELECT id FROM inventory_object WHERE item_id = 'linen_tunic'")
        client.fixture("authority_test_repair_setup", actor, tunic)
        client.call("submit_item_for_repair", actor, settlement, "clothing", tunic)
        [[order]] = client.rows(f"SELECT id FROM repair_order WHERE inventory_item_id = {tunic}")
        assert not client.rows(f"SELECT * FROM character_equipped_item WHERE inventory_item_id = {tunic}")
        client.fixture("authority_test_repair_ready", order)
        if "Held" in destination:
            change(equipment_change(torch, held("Left")))
            before = durable_inventory(client)
            orders = client.rows("SELECT * FROM repair_order")
            client.call("retrieve_repaired_item", actor, order,
                        error="saved equipment graph cannot be restored")
            assert durable_inventory(client) == before
            assert client.rows("SELECT * FROM repair_order") == orders
            change(remove(torch))
        client.call("retrieve_repaired_item", actor, order)
        assert client.rows(f"SELECT placement FROM character_equipped_item WHERE inventory_item_id = {tunic}") == original
        assert client.rows(f"SELECT id FROM inventory_object WHERE item_id = 'linen_tunic'") == objects
        assert not client.rows(f"SELECT * FROM repair_order WHERE id = {order}")
    print("PASS held and worn repair restoration, physical identity and complete "
          "fee/condition/custody rollback when the saved hand is occupied", flush=True)
