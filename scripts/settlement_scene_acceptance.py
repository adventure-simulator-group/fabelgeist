"""Ordinary-scene authority and live transport checks for isolated profiles."""

from __future__ import annotations

from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import uuid
from typing import TYPE_CHECKING

import dev_stack as stack
from settlement_resident_acceptance import resident_roster_checks
from settlement_inventory_acceptance import empty_container_identity_checks, inventory_projection_checks

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient

@contextmanager
def persistent_scene_peer(client: AcceptanceClient, instance: str, claim: str, token: str):
    executable = stack.ROOT / "target/debug/examples/settlement-authority-peer"
    if os.name == "nt":
        executable = executable.with_suffix(".exe")
    environment = {**os.environ, "SPACETIMEDB_TOKEN": token,
                   "ADVENTURESIM_TACTICAL_CLAIM": claim}
    peer_log = stack.ROOT / "target/authority-check" / f"peer-{uuid.uuid4().hex}.log"
    peer_log.parent.mkdir(parents=True, exist_ok=True)
    with peer_log.open("w", encoding="utf-8") as errors:
        peer = subprocess.Popen([
            str(executable), "--url", f"http://{client.http._host}:{client.http._port}",
            "--database", client.database, "--instance-id", instance,
        ], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors,
            text=True, env=environment)
        try:
            assert peer.stdout is not None
            assert peer.stdout.readline().strip() == "READY", peer_log.read_text(encoding="utf-8")
            yield peer
        finally:
            if peer.poll() is None:
                try:
                    peer.communicate(input=json.dumps({"operation": "stop"}) + "\n", timeout=5)
                except subprocess.TimeoutExpired:
                    peer.kill()
                    peer.wait(timeout=5)


def settlement_scene_checks(client: AcceptanceClient) -> None:
    """Run admission without creating any investigation or combat fixture."""
    [[settlement], [elsewhere], *_] = client.rows("SELECT id FROM settlement")
    solo, party_a, party_b, absent = 732010, 732011, 732012, 732013
    for actor, place, party in [(solo, settlement, False), (party_a, settlement, True),
                                (party_b, settlement, True), (absent, elsewhere, False)]:
        client.fixture("authority_test_settlement_actor", actor, place, party)
    assert client.rows(f"SELECT id FROM party_member WHERE character_id = {solo}") == []
    empty_container_identity_checks(client, solo)
    parties = client.rows(f"SELECT party_id FROM character WHERE id = {party_a} OR id = {party_b}")
    assert len(parties) == 2 and parties[0] != parties[1]
    baseline = {table: client.rows(f"SELECT COUNT(*) AS count FROM {table}") for table in
                ("mission_authority", "autoresolve_report", "outcome_source_authority")}
    _, outsider = client.identity()
    client.call("request_settlement_scene", solo, token=outsider, error="gateway")
    client.call("request_settlement_scene", solo)
    client.call("request_settlement_scene", party_a)
    client.call("request_settlement_scene", party_b)
    instance = f"scene:settlement:{settlement}"
    literal = instance.replace("'", "''")
    assert client.rows("SELECT instance_id FROM tactical_server_request_authority") == [[instance]]
    original_progress = client.rows("SELECT provisioning FROM tactical_server_request_authority")
    client.call("record_tactical_scene_provisioning_failure", instance, token=outsider, error="gateway")
    client.call("record_tactical_scene_provisioning_failure", instance)
    failed_progress = client.rows("SELECT provisioning FROM tactical_server_request_authority")
    assert failed_progress != original_progress
    client.call("record_tactical_scene_provisioning_failure", instance)
    assert client.rows("SELECT provisioning FROM tactical_server_request_authority") == failed_progress
    client.call("request_settlement_scene", solo)
    assert client.rows("SELECT provisioning FROM tactical_server_request_authority") == original_progress
    claim = "ordinary-town-claim"
    client.call("authorize_tactical_server_claim", instance, list(hashlib.sha256(claim.encode()).digest()))
    server_identity, server_token = client.identity()
    with persistent_scene_peer(client, instance, claim, server_token):
        resident_roster_checks(client, settlement, server_identity, server_token, outsider, solo)
        client.call("create_tactical_server_for_request", instance, claim, "127.0.0.1:1", "", token=server_token,
                    error="no authorized claim")
        assert client.rows(f"SELECT id FROM character WHERE server = 0x{server_identity}") == []
        for actor in (solo, party_a, party_b, solo):
            client.call("enter_tactical_scene", actor, ["0x" + server_identity], token=server_token)
        client.call("enter_tactical_scene", absent, ["0x" + server_identity], token=server_token,
                    error="not present")
        client.call("request_settlement_scene", solo)
        client.call("end_tactical_server", {"failed": []}, [[], []], token=server_token,
                    error="mission")
        assert len(client.rows(f"SELECT id FROM character WHERE server = 0x{server_identity}")) == 3
        client.call("leave_tactical_scene", solo, token=outsider, error="registered tactical server")
        client.call("leave_tactical_scene", solo, token=server_token, error="visit receipt")
        client.call("retire_settlement_scene", token=server_token, error="Finish local actor visits")
        visits = client.rows(
            "SELECT id, character_id FROM local_scene_visit "
            f"WHERE character_id = {solo} OR character_id = {party_a} OR character_id = {party_b}"
        )
        assert len(visits) == 3, "Duplicate enrollment created another time owner"
        solo_visit = next(visit for visit, actor in visits if actor == solo)
        inventory_projection_checks(client, solo, server_token, outsider)
        client.call("finish_local_scene_visit", solo_visit, {"unprojected": []},
                    token=outsider, error="owning tactical server")
        client.call("rest_at_settlement_hours", solo, 60, {"inn": []},
                    error="owned by a live settlement")
        client.call("authority_test_local_visit_handoff", client.bootstrap_token, solo,
                    token=server_token)
        for visit, actor in visits:
            if actor != solo:
                client.call("finish_local_scene_visit", visit, {"unprojected": []}, token=server_token)
                client.call("finish_local_scene_visit", visit, {"unprojected": []}, token=server_token)
        client.call("retire_settlement_scene", token=server_token)
        client.call("retire_settlement_scene", token=server_token)
        assert client.rows(f"SELECT instance_id FROM tactical_server_authority WHERE instance_id = '{literal}'") == []
        assert client.rows("SELECT server FROM scene_resident_roster") == []
        assert client.rows("SELECT inventory FROM connected_players", token=server_token) == []
        for actor in (solo, party_a, party_b):
            assert client.rows(f"SELECT server FROM character WHERE id = {actor}") == [[["0x0"]]]
        for table, before in baseline.items():
            assert client.rows(f"SELECT COUNT(*) AS count FROM {table}") == before
    client.call("request_settlement_scene", solo)
    assert client.rows("SELECT instance_id FROM tactical_server_request_authority") == [[instance]]
    client.call("authorize_tactical_server_claim", instance, list(hashlib.sha256(claim.encode()).digest()))
    lost_identity, lost_token = client.identity()
    with persistent_scene_peer(client, instance, claim, lost_token) as peer:
        client.call("enter_tactical_scene", solo, ["0x" + lost_identity], token=lost_token)
        clock_before = client.rows(f"SELECT minutes FROM character_time WHERE character_id = {solo}")
        fatigue_before = client.rows(f"SELECT calories_used FROM character_stats WHERE character_id = {solo}")
        # The actor remains time-owned during retained/disconnected play.
        time.sleep(1.25)
        # An abrupt serving-transport loss releases the same slot without a result.
        peer.kill()
        peer.wait(timeout=5)
    deadline = time.monotonic() + 10
    while client.rows(f"SELECT instance_id FROM tactical_server_authority WHERE instance_id = '{literal}'"):
        assert time.monotonic() < deadline, "Lost serving connection retained its settlement slot"
        time.sleep(0.05)
    assert client.rows(f"SELECT server FROM character WHERE id = {solo}") == [[["0x0"]]]
    assert client.rows(f"SELECT minutes FROM character_time WHERE character_id = {solo}") != clock_before
    assert client.rows(f"SELECT calories_used FROM character_stats WHERE character_id = {solo}") == fatigue_before
    for table, before in baseline.items():
        assert client.rows(f"SELECT COUNT(*) AS count FROM {table}") == before
    print("PASS ordinary solo/cross-party shared admission, duplicate requests/joins, foreign presence rejection, no combat outcome, retirement and fresh request", flush=True)
    print("PASS exact local clock/needs closure, replay/conflict/foreign rejection, local-journey-local fractional carry, and server-loss time without fabricated fatigue recovery", flush=True)


def live_settlement_scene_checks(client: AcceptanceClient, base_port: int) -> None:
    """Exercise the real dispatcher, Bevy server and two headless game clients."""
    [[settlement]] = client.rows("SELECT id FROM settlement WHERE id = 'riverdale'")
    client.fixture("authority_test_merchant_area", settlement)
    [[population]] = client.rows(f"SELECT population_estimate FROM settlement WHERE id = '{settlement}'")
    print(f"Live merchant fixture settlement={settlement} population={population}", flush=True)
    # Use a flat source fixture to isolate session lifecycle from terrain grading.
    # The canonical town generator, dispatcher, server and transports remain real.
    client.fixture("authority_test_settlement_location", settlement, 105_000_000, 505_000_000)
    actors = (734010, 734011)
    for actor in actors:
        client.fixture("authority_test_settlement_actor", actor, settlement, True)
    clocks_before = {actor: client.rows(
        f"SELECT minutes, fraction FROM character_time WHERE character_id = {actor}"
    ) for actor in actors}
    needs_before = {actor: client.rows(
        f"SELECT food_balance_kcal, water_balance_ml FROM character_needs WHERE character_id = {actor}"
    ) for actor in actors}
    baseline = {table: client.rows(f"SELECT COUNT(*) AS count FROM {table}") for table in
                ("mission_authority", "autoresolve_report", "outcome_source_authority")}
    client.call("request_settlement_scene", actors[0])
    client.call("request_settlement_scene", actors[1])
    suffix = ".exe" if os.name == "nt" else ""
    binary = stack.ROOT / "target/debug"
    server_binary = binary / f"adventuresim-tactical-server{suffix}"
    dispatcher_binary = binary / f"adventuresim-tactical-server-dispatcher{suffix}"
    peer_binary = binary / f"examples/live-scene-peer{suffix}"
    environment = {**os.environ, "SPACETIMEDB_TOKEN": client.http._headers["Authorization"].removeprefix("Bearer ")}
    output = stack.ROOT / "target/authority-check/live-settlement"
    output.mkdir(parents=True, exist_ok=True)
    terrain = output / "flat-terrain"
    exported = subprocess.run([
        "cargo", "test", "-p", "adventuresim-tactical-server-dispatcher", "--lib",
        "export_flat_acceptance_terrain", "--", "--ignored", "--nocapture",
    ], env={**os.environ, "SETTLEMENT_ACCEPTANCE_TERRAIN_DIR": str(terrain)},
        capture_output=True, text=True)
    assert exported.returncode == 0, exported.stdout + exported.stderr
    instance = f"scene:settlement:{settlement}"
    literal = instance.replace("'", "''")
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
            deadline = time.monotonic() + 120
            while True:
                rows = client.rows(f"SELECT addr FROM tactical_server_authority WHERE instance_id = '{literal}'")
                if rows:
                    [[address]] = rows
                    break
                assert dispatcher.poll() is None, "Dispatcher exited before scene registration"
                assert time.monotonic() < deadline, "Dispatcher did not register ordinary settlement scene"
                time.sleep(0.1)
            port = int(address.rsplit(":", 1)[1])
            serving_process = stack.listener_process_snapshot(port)
            assert serving_process and Path(serving_process["executable"]).resolve() == server_binary.resolve()
            documents = list((output / "scenes").glob("scene-*.json"))
            assert documents, "Dispatcher did not capture its playable scene document"
            scene = json.loads(max(documents, key=lambda path: path.stat().st_mtime_ns).read_text(encoding="utf-8"))
            assert scene["landform"] is None, "Flat transport fixture must not import geological landforms"
            physical = {building["id"] for building in scene["buildings"]}
            operators = [business for business in scene["establishments"]
                         if business["building_id"] in physical]
            merchants = [business for business in operators
                         if business["business_id"]["key"]["usage"] == "general_shop"
                         and business["business_id"]["key"]["ordinal"] == 0]
            assert merchants, "Small playable settlement must include its canonical general merchant"
            merchant = min(merchants, key=lambda business: business["operator_character_id"])
            operator = merchant["operator_character_id"]
            [[operator_name]] = client.rows(f"SELECT name FROM character WHERE id = {operator}")
            assert operator_name == merchant["operator_name"]
            assert client.rows(f"SELECT service_id FROM settlement_resident_profile WHERE character_id = {operator}") == [["merchants"]]
            for business in operators:
                client.fixture("authority_test_resident_admission", business["operator_character_id"], {"available": []})
            for index, actor in enumerate(actors):
                peer_log = (output / f"peer-{index}.log").open("w", encoding="utf-8")
                command = [str(peer_binary), "--character", str(actor), "--server", address,
                           "--expected-players", "2", "--expected-resident", str(operator), "--hold-open"]
                if index == 0:
                    command.append("--reconnect")
                peer = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=peer_log, stderr=subprocess.STDOUT, text=True)
                peers.append((peer, peer_log))
            deadline = time.monotonic() + 30
            while client.rows(f"SELECT id FROM character WHERE id = {actors[0]} AND server = 0x{'0' * 64}"):
                assert time.monotonic() < deadline, "First peer did not enter scene"
                time.sleep(0.05)
            duplicate = subprocess.run([
                str(peer_binary), "--character", str(actors[0]), "--server", address,
                "--expect-rejection",
            ], capture_output=True, text=True, timeout=65)
            assert duplicate.returncode == 0 and "REJECTED" in duplicate.stdout, duplicate.stdout + duplicate.stderr
            deadline = time.monotonic() + 65
            while not all("ACCEPTED" in (output / f"peer-{index}.log").read_text(encoding="utf-8") for index in range(2)):
                exited = [(index, peer.returncode) for index, (peer, _) in enumerate(peers)
                          if peer.poll() is not None]
                if exited:
                    snapshot = {
                        "expected_operator": operator,
                        "rosters": client.rows("SELECT character_ids FROM scene_resident_roster"),
                        "operator": client.rows(f"SELECT id, alive, server FROM character WHERE id = {operator}"),
                        "presence": client.rows(f"SELECT location_id, start_minute, end_minute, context_suppressed, health_suppressed FROM settlement_resident_presence WHERE character_id = {operator}"),
                        "npc_authority": client.rows(f"SELECT character_id FROM npc_policy WHERE character_id = {operator}"),
                        "visits": client.rows(f"SELECT id, state FROM local_scene_visit WHERE character_id = {operator}"),
                    }
                    (output / "resident-state.json").write_text(json.dumps(snapshot, indent=2), encoding="utf-8")
                assert not exited, f"Headless peer exited before co-presence acceptance: {exited}; inspect peer logs"
                assert time.monotonic() < deadline, "Headless scene peers did not accept co-presence"
                time.sleep(0.1)
            assert len(client.rows(f"SELECT id FROM local_scene_visit WHERE character_id = {actors[0]} OR character_id = {actors[1]}")) == 2
            operator_visits = client.rows(f"SELECT id, state FROM local_scene_visit WHERE character_id = {operator}")
            assert len(operator_visits) == 1, "Merchant must retain one visit across player reconnects"
            for actor in actors:
                assert client.rows(f"SELECT minutes, fraction FROM character_time WHERE character_id = {actor}") == clocks_before[actor]
                assert client.rows(f"SELECT food_balance_kcal, water_balance_ml FROM character_needs WHERE character_id = {actor}") == needs_before[actor]
            for index in range(2):
                assert "CALENDAR first=" in (output / f"peer-{index}.log").read_text(encoding="utf-8")
                assert f"RESIDENT id={operator} name={operator_name}" in (output / f"peer-{index}.log").read_text(encoding="utf-8")
            client.fixture("authority_test_resident_admission", operator, {"healthUnavailable": []})
            deadline = time.monotonic() + 20
            while not all(f"RESIDENT_DEPARTED id={operator}" in (output / f"peer-{index}.log").read_text(encoding="utf-8")
                          for index in range(2)):
                assert time.monotonic() < deadline, "Unavailable merchant remained projected to a client"
                assert all(peer.poll() is None for peer, _ in peers), "Client exited during merchant departure"
                time.sleep(0.1)
            assert client.rows(f"SELECT server FROM character WHERE id = {operator}") == [[["0x0"]]]
            assert client.rows(f"SELECT id, state FROM local_scene_visit WHERE character_id = {operator}") != operator_visits
            client.fixture("authority_test_resident_admission", operator, {"available": []})
            deadline = time.monotonic() + 20
            while not all(f"RESIDENT_RETURNED id={operator}" in (output / f"peer-{index}.log").read_text(encoding="utf-8")
                          for index in range(2)):
                assert time.monotonic() < deadline, "Available merchant did not re-enter exactly once on both clients"
                assert all(peer.poll() is None for peer, _ in peers), "Client exited during merchant re-entry"
                time.sleep(0.1)
            operator_visits = client.rows(f"SELECT id, state FROM local_scene_visit WHERE character_id = {operator}")
            assert len(operator_visits) == 2, "Merchant re-entry must start one fresh visit"
            for peer, peer_log in peers:
                peer.communicate(input="stop\n", timeout=10)
                assert peer.returncode == 0, "Headless scene peer failed; inspect peer log"
                peer_log.flush()
            assert "ACCEPTED players=2 reconnects=2" in (output / "peer-0.log").read_text(encoding="utf-8")
            assert "ACCEPTED players=2 reconnects=1" in (output / "peer-1.log").read_text(encoding="utf-8")
            deadline = time.monotonic() + 60
            while client.rows(f"SELECT instance_id FROM tactical_server_authority WHERE instance_id = '{literal}'"):
                assert time.monotonic() < deadline, "Last-disconnect retirement did not release town scene"
                time.sleep(0.1)
            for actor in actors:
                assert client.rows(f"SELECT server FROM character WHERE id = {actor}") == [[["0x0"]]]
                assert client.rows(f"SELECT minutes, fraction FROM character_time WHERE character_id = {actor}") != clocks_before[actor]
                assert client.rows(f"SELECT food_balance_kcal, water_balance_ml FROM character_needs WHERE character_id = {actor}") != needs_before[actor]
            assert client.rows(f"SELECT server FROM character WHERE id = {operator}") == [[["0x0"]]]
            assert client.rows(f"SELECT id, state FROM local_scene_visit WHERE character_id = {operator}") != operator_visits
            assert client.rows("SELECT server FROM scene_resident_roster") == []
            for table, before in baseline.items():
                assert client.rows(f"SELECT COUNT(*) AS count FROM {table}") == before
            documents = {path: path.read_bytes() for path in (output / "scenes").glob("scene-*.json")}
            client.call("request_settlement_scene", actors[0])
            deadline = time.monotonic() + 120
            while not client.rows(f"SELECT addr FROM tactical_server_authority WHERE instance_id = '{literal}'"):
                assert dispatcher.poll() is None
                assert time.monotonic() < deadline, "Retired town did not reopen"
                time.sleep(0.1)
            [[next_address]] = client.rows(f"SELECT addr FROM tactical_server_authority WHERE instance_id = '{literal}'")
            serving_process = stack.listener_process_snapshot(int(next_address.rsplit(":", 1)[1]))
            assert serving_process
            assert any(path not in documents for path in (output / "scenes").glob("scene-*.json")), "Reopened town reused stale captured time"
            assert all(path.read_bytes() == data for path, data in documents.items()), "Reopening mutated an immutable scene document"
            for table, before in baseline.items():
                assert client.rows(f"SELECT COUNT(*) AS count FROM {table}") == before
            print("PASS live canonical town on flat terrain: cross-party co-presence, duplicate-actor rejection, reconnect rotation, retirement and reopening with fresh immutable input without battle outcomes", flush=True)
            print("PASS live merchant identity on both clients, unavailable removal, duplicate-free re-entry, visit closure and roster retirement", flush=True)
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

