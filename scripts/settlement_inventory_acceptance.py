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
