"""Personal equipment through the real dispatcher, Bevy host and native clients.

This scenario does not require resident navigation: personal hand actions have
no pathfinding prerequisite. The complete resident/navigation acceptance remains
in settlement_scene_acceptance.py, with its original requirements unchanged.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import time
from typing import TYPE_CHECKING

import dev_stack as stack

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient


def normalized(rows: list) -> list[str]:
    return sorted(json.dumps(row, sort_keys=True) for row in rows)


def live_equipment_checks(client: AcceptanceClient, base_port: int) -> None:
    settlement = "riverdale"
    actors = (734010, 734011)
    client.fixture("authority_test_merchant_area", settlement)
    client.fixture("authority_test_settlement_location", settlement, 105_000_000, 505_000_000)
    for actor in actors:
        client.fixture("authority_test_settlement_actor", actor, settlement, True)
    equipment_query = (
        f"SELECT inventory_item_id, placement FROM character_equipped_item WHERE character_id = {actors[0]}"
    )
    inventory_query = f"SELECT id, item_id, quantity FROM inventory_item WHERE character_id = {actors[0]}"
    equipment_before = normalized(client.rows(equipment_query))
    inventory_before = normalized(client.rows(inventory_query))
    output = stack.ROOT / "target/authority-check/native-equipment-live"
    output.mkdir(parents=True, exist_ok=True)
    terrain = output / "flat-terrain"
    exported = subprocess.run([
        "cargo", "test", "-p", "adventuresim-tactical-server-dispatcher", "--lib",
        "export_flat_acceptance_terrain", "--", "--ignored", "--nocapture",
    ], env={**os.environ, "SETTLEMENT_ACCEPTANCE_TERRAIN_DIR": str(terrain)},
        capture_output=True, text=True)
    assert exported.returncode == 0, exported.stdout + exported.stderr
    for actor in actors:
        client.call("request_settlement_scene", actor)
    suffix = ".exe" if os.name == "nt" else ""
    binary = stack.ROOT / "target/debug"
    server_binary = binary / f"adventuresim-tactical-server{suffix}"
    dispatcher_binary = binary / f"adventuresim-tactical-server-dispatcher{suffix}"
    peer_binary = binary / f"examples/live-scene-peer{suffix}"
    environment = {**os.environ, "SPACETIMEDB_TOKEN": client.http._headers["Authorization"].removeprefix("Bearer ")}
    peers = []
    serving_process = None
    with (output / "dispatcher.log").open("w", encoding="utf-8") as log:
        dispatcher = subprocess.Popen([
            str(dispatcher_binary), "--spacetimedb-url", f"http://{client.http._host}:{client.http._port}",
            "--spacetimedb-module", client.database, "--tactical-server-bin", str(server_binary),
            "--base-port", str(base_port + 2), "--host", "127.0.0.1",
            "--scene-input-dir", str(output / "scenes"),
            "--terrain-manifest", str(terrain / "terrain.manifest.json"),
            "--terrain-pack", str(terrain / "terrain.pack"),
        ], stdout=log, stderr=subprocess.STDOUT, env=environment)
        try:
            instance = f"scene:settlement:{settlement}"
            deadline = time.monotonic() + 120
            while True:
                rows = client.rows(f"SELECT addr FROM tactical_server_authority WHERE instance_id = '{instance}'")
                if rows:
                    [[address]] = rows
                    break
                assert dispatcher.poll() is None, "Dispatcher exited before scene registration"
                assert time.monotonic() < deadline, "Settlement host did not register"
                time.sleep(0.1)
            serving_process = stack.listener_process_snapshot(int(address.rsplit(":", 1)[1]))
            assert serving_process and Path(serving_process["executable"]).resolve() == server_binary.resolve()
            for index, actor in enumerate(actors):
                peer_log = (output / f"peer-{index}.log").open("w", encoding="utf-8")
                command = [str(peer_binary), "--character", str(actor), "--server", address,
                           "--expected-players", "2", "--hold-open"]
                if index == 0:
                    command.extend(["--reconnect", "--equipment"])
                peer = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=peer_log,
                                        stderr=subprocess.STDOUT, text=True)
                peers.append((peer, peer_log))
            deadline = time.monotonic() + 65
            while not all("ACCEPTED players=2" in (output / f"peer-{index}.log").read_text(encoding="utf-8")
                          for index in range(2)):
                exited = [index for index, (peer, _) in enumerate(peers) if peer.poll() is not None]
                diagnostics = {index: (output / f"peer-{index}.log").read_text(encoding="utf-8")[-2000:]
                               for index in exited}
                assert not exited, f"Native equipment peer exited: {diagnostics}"
                assert time.monotonic() < deadline, "Native equipment acceptance timed out"
                time.sleep(0.1)
            first = (output / "peer-0.log").read_text(encoding="utf-8")
            assert "EQUIPMENT_ACCEPTED swaps=2 replay=1 stale=1 drop=1 foreign=1" in first
            assert "ACCEPTED players=2 reconnects=2" in first
            receipts = client.rows(
                f"SELECT command_id FROM tactical_inventory_receipt_authority WHERE actor_character_id = {actors[0]}"
            )
            assert len([command for [command] in receipts if command.startswith("native-")]) == 2
            assert normalized(client.rows(equipment_query)) == equipment_before, "Durable placement changed after hand round trip"
            assert normalized(client.rows(inventory_query)) == inventory_before, "Durable ownership or quantities changed after hand round trip"
            for peer, peer_log in peers:
                peer.communicate(input="stop\n", timeout=10)
                assert peer.returncode == 0, "Equipment peer failed after acceptance"
                peer_log.flush()
            print("PASS native live equipment: two durable hand commits, unchanged physical entities/materials, replay/stale/drop/foreign rejection, reconnect and round-trip ownership", flush=True)
        finally:
            for peer, peer_log in peers:
                if peer.poll() is None:
                    peer.kill()
                    peer.wait(timeout=5)
                peer_log.close()
            dispatcher.terminate()
            dispatcher.wait(timeout=5)
            if serving_process is not None:
                stack.terminate_verified_or_accept_exit(serving_process)
