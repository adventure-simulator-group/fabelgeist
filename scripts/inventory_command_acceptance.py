"""Live inventory authority through real database transactions and SDK hosts."""

from __future__ import annotations

import copy
import hashlib
from typing import TYPE_CHECKING

import dev_stack as stack
from inventory_command_peer import InventoryPeer, durable_inventory, insert, item, place, remove, transfer
from inventory_command_effect_acceptance import (
    BOX, CONTAINER, COUNTED, PARENT, counted_transfer_checks,
    equipment_and_container_checks, recipe_and_condition_transfer_checks,
    reject_action, subtree_and_replay_checks, total_mass,
)
from settlement_scene_acceptance import persistent_scene_peer
from inventory_equipment_exchange_acceptance import held_exchange_checks, attachment_exchange_checks
from equipment_repair_acceptance import repair_equipment_checks

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient


def inventory_command_checks(client: AcceptanceClient) -> None:
    [[settlement], [elsewhere], *_] = client.rows("SELECT id FROM settlement")
    source, recipient, foreign, absent, elsewhere_actor = 735000, 735001, 735002, 735003, 735004
    for actor, location, party in (
        (source, settlement, True), (recipient, settlement, False),
        (foreign, settlement, True), (absent, settlement, False),
        (elsewhere_actor, elsewhere, True),
    ):
        client.fixture("authority_test_settlement_actor", actor, location, party)
    for member in (recipient, absent):
        client.fixture("authority_test_inventory_party", source, member)
    instance = f"scene:settlement:{settlement}"
    claim = "inventory-authority-claim"
    client.call("request_settlement_scene", source)
    authorize(client, instance, claim)
    identity, token = client.identity()
    with persistent_scene_peer(client, instance, claim, token) as process:
        peer = InventoryPeer(process, client, token)
        repair_equipment_checks(peer, settlement)
        for actor in (source, recipient, foreign):
            peer.control("enter", f"enter:{actor}", character_id=actor)
            client.call("authority_test_inventory_projection", client.bootstrap_token,
                        actor, {"seed": []}, token=token)
            client.call("authority_test_inventory_command_seed", client.bootstrap_token,
                        actor, token=token)
            peer.snapshot(actor, ready=lambda response: response["state"] is not None
                          and any(row["catalog"] == PARENT for row in response["items"]))
        token_checks(peer, source)
        scalar_replay_checks(peer, source)
        ownership_checks(peer, source, recipient, foreign, absent)
        other_host_checks(client, peer, source, elsewhere_actor, elsewhere)
        equipment_and_container_checks(peer, source)
        held_exchange_checks(peer, source)
        attachment_exchange_checks(peer, source)
        counted_transfer_checks(peer, source, recipient)
        subtree_request = subtree_and_replay_checks(peer, source, recipient)
        recipe_and_condition_transfer_checks(peer, source, recipient)
        lifecycle_checks(peer, source, recipient, subtree_request)
        retire_and_replace(client, peer, source, recipient, foreign, instance, identity)
    mission_reentry_checks(client)


def authorize(client: AcceptanceClient, instance: str, claim: str) -> None:
    client.call("authorize_tactical_server_claim", instance,
                list(hashlib.sha256(claim.encode()).digest()))


def token_checks(peer: InventoryPeer, actor: int) -> None:
    for stage in ("foodQuality", "repartitionMaterial", "condition", "recipe"):
        previous = peer.snapshot(actor)
        container = item(previous, CONTAINER)
        prepared = peer.prepare(previous["state"], {"DiscardWater": {
            "container_object_id": container["object"], "amount_ml": 1}})
        peer.client.call("authority_test_inventory_command_material", peer.client.bootstrap_token,
                         actor, {stage: []}, token=peer.token)
        current = peer.changed(actor, previous)
        assert current["state"]["scope"] == previous["state"]["scope"]
        assert current["state"]["revision"] != previous["state"]["revision"], stage
        assert total_mass(current) == total_mass(previous), stage
        peer.reject_unchanged(prepared)
    before = durable_inventory(peer.client)
    peer.client.call("authority_test_inventory_construction_validation", peer.client.bootstrap_token,
                     actor, token=peer.token)
    assert durable_inventory(peer.client) == before
    print("PASS revisions include equal-mass material, food quality, condition "
          "and authenticated recipe edits; stale source and warmed corrupt "
          "construction cannot commit", flush=True)


def scalar_replay_checks(peer: InventoryPeer, actor: int) -> None:
    container = item(peer.snapshot(actor), CONTAINER)
    action = {"DiscardWater": {"container_object_id": container["object"], "amount_ml": 1}}
    request = peer.apply_change(actor, action)
    before, receipts = durable_inventory(peer.client), peer.receipts()
    peer.invoke(request)
    assert durable_inventory(peer.client) == before and peer.receipts() == receipts
    collision = copy.deepcopy(request)
    collision["action"]["DiscardWater"]["amount_ml"] = 2
    peer.reject_unchanged(collision)
    for command_id in ("", "invalid/request", "x" * 257):
        peer.reject_unchanged(peer.prepare(peer.snapshot(actor)["state"], action, command_id=command_id))
    malformed = peer.prepare(peer.snapshot(actor)["state"], action)
    malformed["actor"]["scope"] = []
    peer.reject_unchanged(malformed)
    malformed = peer.prepare(peer.snapshot(actor)["state"], action)
    malformed["actor"]["revision"] = []
    peer.reject_unchanged(malformed)
    reject_action(peer, actor, {"DiscardWater": {"container_object_id": container["object"], "amount_ml": 0}})
    print("PASS water disposal, immutable successful receipt, conflicting reuse "
          "and invalid identity/token rejection", flush=True)


def ownership_checks(peer: InventoryPeer, source: int, recipient: int,
                     foreign: int, absent: int) -> None:
    current, other = peer.snapshot(source), peer.snapshot(foreign)
    parent, box = item(current, PARENT), item(current, BOX)
    reject_action(peer, source, place(item(other, PARENT)["id"]))
    reject_action(peer, source, insert(item(other, PARENT)["object"], box["object"]))
    reject_action(peer, source, transfer(item(current, COUNTED)["id"], other["state"]))
    reject_action(peer, source, transfer(item(current, COUNTED)["id"], current["state"]))
    inactive = copy.deepcopy(peer.snapshot(recipient)["state"])
    inactive["character_id"] = absent
    reject_action(peer, source, transfer(item(current, COUNTED)["id"], inactive))
    for reducer, arguments in (
        ("change_equipment", (source, peer.equipment_argument(place(parent["id"])))),
        ("change_equipment", (source, peer.equipment_argument(remove(parent["id"])))),
        ("put_inventory_item_in_container", (source, "personal", parent["id"], "personal", box["id"])),
        ("remove_inventory_item_from_container", (source, parent["object"])),
        ("discard_container_water", (source, item(current, CONTAINER)["object"], 1)),
        ("transfer_party_item", (source, recipient, parent["id"], 1)),
    ):
        before = durable_inventory(peer.client)
        peer.client.call(reducer, *arguments, error="owned by a live")
        assert durable_inventory(peer.client) == before, reducer
    [[absent_row], *_] = peer.client.rows(f"SELECT id FROM inventory_item WHERE character_id = {absent}")
    peer.client.call("transfer_party_item", absent, source, absent_row, 1, error="owned by a live")

    request = peer.prepare(current["state"], {"DiscardWater": {
        "container_object_id": item(current, CONTAINER)["object"], "amount_ml": 1}})
    # HTTP has the same logical identity, but lacks the registered SDK connection.
    http_request = copy.deepcopy(request)
    http_request["action"] = {"discardWater": next(iter(request["action"].values()))}
    before = durable_inventory(peer.client)
    peer.client.call("commit_tactical_inventory", http_request, token=peer.token,
                     error="serving connection")
    _, outsider = peer.client.identity()
    peer.client.call("commit_tactical_inventory", http_request, token=outsider,
                     error="registered tactical host")
    assert peer.client.rows("SELECT * FROM tactical_inventory_receipt", token=outsider) == []
    assert durable_inventory(peer.client) == before
    print("PASS serving-connection authority, foreign ownership/party, absent "
          "peers and non-live entry-point bypass rejection", flush=True)


def other_host_checks(client: AcceptanceClient, source_peer: InventoryPeer,
                      source: int, actor: int, settlement: str) -> None:
    client.call("request_settlement_scene", actor)
    instance, claim = f"scene:settlement:{settlement}", "inventory-other-host"
    authorize(client, instance, claim)
    _, token = client.identity()
    with persistent_scene_peer(client, instance, claim, token) as process:
        peer = InventoryPeer(process, client, token)
        current = source_peer.snapshot(source)
        request = peer.prepare(current["state"], {"DiscardWater": {
            "container_object_id": item(current, CONTAINER)["object"], "amount_ml": 1}})
        peer.reject_unchanged(request)
    print("PASS another registered host cannot mutate this scene's actors", flush=True)


def lifecycle_checks(peer: InventoryPeer, source: int, recipient: int, replay: dict) -> None:
    peer.client.call("authority_test_inventory_command_material", peer.client.bootstrap_token,
                     recipient, {"revokeMembership": []}, token=peer.token)
    peer.reject_unchanged(replay)
    peer.client.call("authority_test_inventory_command_material", peer.client.bootstrap_token,
                     recipient, {"restoreMembership": []}, token=peer.token)
    before = durable_inventory(peer.client)
    peer.invoke(replay)
    assert durable_inventory(peer.client) == before
    previous = peer.snapshot(recipient)
    finish(peer, recipient)
    peer.snapshot(recipient, ready=lambda response: response["state"] is None)
    peer.reject_unchanged(replay)
    peer.control("enter", f"enter:{recipient}", character_id=recipient)
    current = peer.snapshot(recipient, ready=lambda response: response["state"] is not None
                            and response["state"]["scope"] != previous["state"]["scope"])
    assert current["state"]["scope"] != previous["state"]["scope"]
    peer.reject_unchanged(replay)
    print("PASS canonical membership, closed admission and replaced visit "
          "are checked before successful replay", flush=True)


def finish(peer: InventoryPeer, actor: int) -> None:
    visits = peer.client.rows(f"SELECT id FROM local_scene_visit WHERE character_id = {actor}")
    visit = max(value for [value] in visits)
    peer.control("finish_visit", f"finish:{visit}", visit_id=visit)


def retire_and_replace(client: AcceptanceClient, old: InventoryPeer,
                       source: int, recipient: int, foreign: int,
                       instance: str, identity: str) -> None:
    current = old.snapshot(source)
    box, solid = item(current, BOX), item(current, "authority-command-solid")
    prepared = old.prepare(current["state"], insert(solid["object"], box["object"]))
    for actor in (source, recipient, foreign):
        finish(old, actor)
    assert old.receipts()
    client.call("retire_settlement_scene", token=old.token)
    assert old.receipts() == []
    old.reject_unchanged(prepared)
    client.call("request_settlement_scene", source)
    claim = "inventory-replacement-host"
    authorize(client, instance, claim)
    replacement_identity, token = client.identity()
    assert replacement_identity != identity
    with persistent_scene_peer(client, instance, claim, token) as process:
        new = InventoryPeer(process, client, token)
        new.control("enter", f"enter:{source}", character_id=source)
        new.snapshot(source)
        assert new.receipts() == []
        new.reject_unchanged(prepared)
        old.reject_unchanged(prepared)
        new.apply_change(source, insert(solid["object"], box["object"]))
        assert len(new.receipts()) == 1 and old.receipts() == []
    print("PASS retired and replacement host isolation, private receipt "
          "visibility and fresh-admission commands", flush=True)


def mission_reentry_checks(client: AcceptanceClient) -> None:
    actor, mission, claim = 735090, "mission:inventory-admissions", "inventory-mission-host"
    enemy_yaml = stack.read_enemy_fixture(stack.PASSIVE_ENEMY_FIXTURE)
    client.fixture("seed_standalone_tactical_mission", actor, mission, "woodland", enemy_yaml, claim)
    _, token = client.identity()
    with persistent_scene_peer(client, f"scene:mission:{mission}", claim, token) as process:
        peer = InventoryPeer(process, client, token)
        peer.control("enter", f"enter:{actor}", character_id=actor)
        client.call("authority_test_inventory_projection", client.bootstrap_token,
                    actor, {"seed": []}, token=token)
        before = peer.snapshot(actor, ready=lambda response: response["state"] is not None
                               and any(row["catalog"] == CONTAINER for row in response["items"]))
        peer.control("enter", f"enter:{actor}", character_id=actor)
        assert peer.snapshot(actor)["state"] == before["state"]
        action = {"DiscardWater": {"container_object_id": item(before, CONTAINER)["object"], "amount_ml": 1}}
        accepted = peer.apply_change(actor, action)
        before_leave = peer.snapshot(actor)
        prepared = peer.prepare(before_leave["state"], action)
        client.call("discard_container_water", actor, item(before, CONTAINER)["object"], 1,
                    error="live tactical scene")
        peer.control("leave", f"leave:{actor}", character_id=actor)
        peer.control("enter", f"enter:{actor}", character_id=actor)
        rejoined = peer.snapshot(actor, ready=lambda response: response["state"] is not None
                                 and response["state"]["scope"] != before_leave["state"]["scope"])
        assert rejoined["state"]["revision"] == before_leave["state"]["revision"]
        peer.reject_unchanged(prepared)
        peer.reject_unchanged(accepted)
        peer.apply_change(actor, action)
        peer.control("leave", f"leave:{actor}", character_id=actor)
    print("PASS mission enrollment retries, same-connection reentry generation, "
          "unchanged-content stale-admission rejection and fresh commands", flush=True)
