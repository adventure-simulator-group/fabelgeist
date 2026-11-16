"""Ordinary-scene authority and live transport checks for isolated profiles."""

from __future__ import annotations

from contextlib import contextmanager
import hashlib
import os
from pathlib import Path
import subprocess
import time
from typing import TYPE_CHECKING

import dev_stack as stack

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient

@contextmanager
def persistent_scene_peer(client: AcceptanceClient, instance: str, claim: str, token: str):
    executable = stack.ROOT / "target/debug/examples/settlement-authority-peer"
    if os.name == "nt":
        executable = executable.with_suffix(".exe")
    environment = {**os.environ, "SPACETIMEDB_TOKEN": token,
                   "ADVENTURESIM_TACTICAL_CLAIM": claim}
    peer_log = stack.ROOT / "target/authority-check/peer.log"
    peer_log.parent.mkdir(parents=True, exist_ok=True)
    with peer_log.open("w", encoding="utf-8") as errors:
        peer = subprocess.Popen([
            str(executable), "--url", f"http://{client.http._host}:{client.http._port}",
            "--database", client.database, "--instance-id", instance,
        ], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors,
            text=True, env=environment)
        try:
            assert peer.stdout is not None
            assert peer.stdout.readline().strip() == "READY", peer_log.read_text()
            yield peer
        finally:
            if peer.poll() is None:
                try:
                    peer.communicate(input="stop\n", timeout=5)
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
        client.call("retire_settlement_scene", token=server_token)
        client.call("retire_settlement_scene", token=server_token)
        assert client.rows(f"SELECT instance_id FROM tactical_server_authority WHERE instance_id = '{literal}'") == []
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
        # An abrupt serving-transport loss releases the same slot without a result.
        peer.kill()
        peer.wait(timeout=5)
    deadline = time.monotonic() + 10
    while client.rows(f"SELECT instance_id FROM tactical_server_authority WHERE instance_id = '{literal}'"):
        assert time.monotonic() < deadline, "Lost serving connection retained its settlement slot"
        time.sleep(0.05)
    assert client.rows(f"SELECT server FROM character WHERE id = {solo}") == [[["0x0"]]]
    for table, before in baseline.items():
        assert client.rows(f"SELECT COUNT(*) AS count FROM {table}") == before
    print("PASS ordinary solo/cross-party shared admission, duplicate requests/joins, foreign presence rejection, no combat outcome, retirement and fresh request", flush=True)


def live_settlement_scene_checks(client: AcceptanceClient, base_port: int) -> None:
    """Exercise the real dispatcher, Bevy server and two headless game clients."""
    [[settlement], *_] = client.rows("SELECT id FROM settlement")
    # Use a flat source fixture to isolate session lifecycle from terrain grading.
    # The canonical town generator, dispatcher, server and transports remain real.
    client.fixture("authority_test_settlement_location", settlement, 105_000_000, 505_000_000)
    actors = (734010, 734011)
    for actor in actors:
        client.fixture("authority_test_settlement_actor", actor, settlement, True)
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
            for index, actor in enumerate(actors):
                peer_log = (output / f"peer-{index}.log").open("w", encoding="utf-8")
                command = [str(peer_binary), "--character", str(actor), "--server", address,
                           "--expected-players", "2", "--hold-open"]
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
            while not all("ACCEPTED" in (output / f"peer-{index}.log").read_text() for index in range(2)):
                exited = [(index, peer.returncode) for index, (peer, _) in enumerate(peers)
                          if peer.poll() is not None]
                assert not exited, f"Headless peer exited before co-presence acceptance: {exited}; inspect peer logs"
                assert time.monotonic() < deadline, "Headless scene peers did not accept co-presence"
                time.sleep(0.1)
            for peer, peer_log in peers:
                peer.communicate(input="stop\n", timeout=10)
                assert peer.returncode == 0, "Headless scene peer failed; inspect peer log"
                peer_log.flush()
            assert "ACCEPTED players=2 reconnects=2" in (output / "peer-0.log").read_text()
            assert "ACCEPTED players=2 reconnects=1" in (output / "peer-1.log").read_text()
            deadline = time.monotonic() + 60
            while client.rows(f"SELECT instance_id FROM tactical_server_authority WHERE instance_id = '{literal}'"):
                assert time.monotonic() < deadline, "Last-disconnect retirement did not release town scene"
                time.sleep(0.1)
            for actor in actors:
                assert client.rows(f"SELECT server FROM character WHERE id = {actor}") == [[["0x0"]]]
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

