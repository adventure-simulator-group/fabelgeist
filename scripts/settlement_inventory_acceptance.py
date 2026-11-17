"""Registered-host inventory projection through real isolated reducers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient


def inventory_projection_checks(client: AcceptanceClient, actor: int,
                                host: str, outsider: str) -> None:
    query = "SELECT inventory FROM connected_players"
    assert client.rows(query, token=outsider) == []
    client.call("authority_test_inventory_projection", client.bootstrap_token,
                actor, {"seed": []}, token=outsider, error="not projected")
    previous = client.rows(query, token=host)
    for stage in ("seed", "consume", "spirit", "corrupt", "repair",
                  "corruptIdentity", "repairIdentity", "corruptContainment",
                  "repairContainment", "remove"):
        client.call("authority_test_inventory_projection", client.bootstrap_token,
                    actor, {stage: []}, token=host)
        current = client.rows(query, token=host)
        assert len(current) == 3 and current != previous, stage
        assert client.rows(query, token=host) == current, stage
        assert client.rows(query, token=outsider) == [], stage
        previous = current
    print("PASS host-scoped inventory, counted/food/fraction/liquid mass, "
          "nested contents counted once, stable physical identity, committed "
          "refresh, corruption and recovery", flush=True)


def empty_container_identity_checks(client: AcceptanceClient, actor: int | None = None) -> None:
    if actor is None:
        [[settlement], *_] = client.rows("SELECT id FROM settlement")
        actor = 732015
        client.fixture("authority_test_settlement_actor", actor, settlement, False)
    client.fixture("authority_test_empty_inventory_containers", actor)
    query = ("SELECT id, quantity FROM inventory_item "
             f"WHERE character_id = {actor} AND item_id = 'authority-empty-container'")
    rows = sorted(client.rows(query))
    objects = client.rows("SELECT id FROM inventory_object "
                          "WHERE item_id = 'authority-empty-container'")
    assert len(rows) == 2 and all(quantity == 1 for _, quantity in rows)
    assert len(objects) == 2
    source = min(object_id for [object_id] in objects)
    client.call("discard_container_water", actor, source, 1)
    assert sorted(client.rows(query)) == rows, "Emptying a container merged its individual inventory row"
    assert client.rows("SELECT id FROM inventory_object "
                       "WHERE item_id = 'authority-empty-container'") == objects, \
        "Emptying a container destroyed its stable physical identity"
    client.call("discard_container_water", actor, source, 1, error="Container has no water")
    child_row, parent_row = sorted(row_id for row_id, _ in rows)
    client.call("put_inventory_item_in_container", actor, "personal", child_row,
                "personal", parent_row)
    client.call("remove_inventory_item_from_container", actor, source)
    assert sorted(client.rows(query)) == rows
    assert client.rows("SELECT id FROM inventory_object "
                       "WHERE item_id = 'authority-empty-container'") == objects
    client.fixture("authority_test_empty_inventory_mass", actor)
    print("PASS drained and unnested containers retain individual rows, physical identities and tare mass", flush=True)
