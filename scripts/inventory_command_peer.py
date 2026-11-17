"""Bounded JSON-lines transport for the registered inventory acceptance peer."""

from __future__ import annotations

import copy
import json
import time
from typing import TYPE_CHECKING, Callable

if TYPE_CHECKING:
    import subprocess
    from check_semantic_authorities import AcceptanceClient

PROJECTION_TIMEOUT_SECONDS = 15
PROJECTION_POLL_SECONDS = 0.02
INVENTORY_TABLES = (
    "inventory_item", "inventory_object", "inventory_containment",
    "container_liquid", "inventory_item_amount", "food_lot", "item_condition",
    "weapon_instance", "weapon_holder_instance", "character_equipped_item",
    "equipment_occupancy", "tactical_inventory_receipt_authority",
)


class InventoryPeer:
    def __init__(self, process: subprocess.Popen[str], client: AcceptanceClient,
                 token: str) -> None:
        self.process = process
        self.client = client
        self.token = token
        self.sequence = 0

    def send(self, operation: str, **payload: object) -> None:
        assert self.process.stdin is not None
        self.process.stdin.write(json.dumps({"operation": operation, **payload}) + "\n")
        self.process.stdin.flush()

    def receive(self, key: str | None = None) -> dict:
        assert self.process.stdout is not None
        line = self.process.stdout.readline()
        assert line, f"Acceptance peer stopped: {self.process.poll()}"
        response = json.loads(line)
        if key is not None:
            assert response["key"] == key, response
        return response

    def control(self, operation: str, key: str, **payload: object) -> None:
        self.send(operation, **payload)
        response = self.receive(key)
        assert response["status"] == "accepted", response

    def snapshot(self, actor: int, *,
                 ready: Callable[[dict], bool] | None = None) -> dict:
        deadline = time.monotonic() + PROJECTION_TIMEOUT_SECONDS
        while True:
            self.send("snapshot", character_id=actor)
            response = self.receive(f"snapshot:{actor}")
            assert response["status"] == "snapshot", response
            is_ready = ready(response) if ready is not None else response["state"] is not None
            if is_ready:
                return response
            assert time.monotonic() < deadline, response
            time.sleep(PROJECTION_POLL_SECONDS)

    def changed(self, actor: int, previous: dict) -> dict:
        return self.snapshot(actor, ready=lambda response: response["state"] is not None
                             and response["state"] != previous["state"])

    def prepare(self, state: dict, action: dict, *, command_id: str | None = None) -> dict:
        self.sequence += 1
        return {"command_id": f"inventory-intent-{self.sequence}" if command_id is None else command_id,
                "actor": copy.deepcopy(state), "action": copy.deepcopy(action)}

    def invoke(self, request: dict, *, rejected: bool = False) -> dict:
        self.send("inventory", request=request)
        response = self.receive(request["command_id"])
        assert response["status"] == ("rejected" if rejected else "accepted"), response
        return response

    def apply_change(self, actor: int, action: dict) -> dict:
        before = self.snapshot(actor)
        request = self.prepare(before["state"], action)
        self.invoke(request)
        self.changed(actor, before)
        return request

    def reject_unchanged(self, request: dict) -> None:
        before = durable_inventory(self.client)
        self.invoke(request, rejected=True)
        assert durable_inventory(self.client) == before, request

    def receipts(self) -> list:
        return self.client.rows("SELECT * FROM tactical_inventory_receipt", token=self.token)


def durable_inventory(client: AcceptanceClient) -> dict:
    return {table: sorted(json.dumps(row, sort_keys=True) for row in client.rows(f"SELECT * FROM {table}"))
            for table in INVENTORY_TABLES}


def item(snapshot: dict, catalog: str) -> dict:
    return next(row for row in snapshot["items"] if row["catalog"] == catalog)


def place(row: int, *, replace: bool = False, parent: int | None = None) -> dict:
    targets = [] if parent is None else [{"requirement_index": 0,
                "parent_inventory_item_id": parent, "attachment_point_id": "hook"}]
    return {"PlaceEquipment": {"inventory_item_id": row, "placement_index": 0,
            "targets": targets, "conflicts": {
                "ReplaceOccupied" if replace else "RejectOccupied": []}}}


def insert(child: int, parent: int) -> dict:
    return {"InsertIntoContainer": {"child_object_id": child, "parent_object_id": parent}}


def transfer(row: int, recipient: dict, quantity: int = 1) -> dict:
    return {"Transfer": {"inventory_item_id": row, "quantity": quantity,
                          "recipient": copy.deepcopy(recipient)}}
