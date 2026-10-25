#!/usr/bin/env python3
"""Exercise semantic authority invariants in a guarded disposable database."""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
from pathlib import Path

import dev_stack as stack


class AcceptanceClient:
    def __init__(self, server: str, database: str, bootstrap_token: str) -> None:
        self.http = stack._SeedHttpClient(server, database, bootstrap_token)
        self.database = database
        self.bootstrap_token = bootstrap_token

    def call(self, reducer: str, *args: object, token: str | None = None,
             error: str | None = None) -> None:
        headers = dict(self.http._headers)
        if token is not None:
            headers["Authorization"] = f"Bearer {token}"
        status, body = self.http._request(
            f"/v1/database/{self.database}/call/{reducer}", json.dumps(args), headers
        )
        if error is not None:
            assert status // 100 != 2 and error in body, (reducer, status, body)
        else:
            assert status // 100 == 2, (reducer, status, body)

    def fixture(self, reducer: str, *args: object, error: str | None = None) -> None:
        self.call(reducer, self.bootstrap_token, *args, error=error)

    def rows(self, query: str) -> list[list[object]]:
        headers = dict(self.http._headers)
        headers["Content-Type"] = "text/plain"
        status, body = self.http._request(
            f"/v1/database/{self.database}/sql", query, headers
        )
        assert status // 100 == 2, (status, body)
        return json.loads(body)[0]["rows"]

    def identity(self) -> tuple[str, str]:
        connection = http.client.HTTPConnection(self.http._host, self.http._port)
        try:
            connection.request("POST", "/v1/identity")
            response = connection.getresponse()
            assert response.status // 100 == 2
            identity = json.loads(response.read())
            return identity["identity"], identity["token"]
        finally:
            connection.close()


def identity_checks(client: AcceptanceClient) -> None:
    owner = "a" * 64
    args = [owner, 8, "00112233445566778899aabbccddeeff", {"young": []}, 0]
    client.call("register_strategic_gateway", None, 0)
    client.call("create_starting_character", *args)
    [[character_id, request_key]] = client.rows("SELECT character_id, request_key FROM starting_character_claim")
    client.fixture("authority_test_name_projection", character_id)
    client.call("update_character", character_id, "Renamed Candidate")
    client.fixture("authority_test_name_projection", character_id)
    before = client.rows(f"SELECT name FROM character WHERE id = {character_id}")
    identity_before = client.rows(
        f"SELECT identity_json FROM character_name_identity WHERE character_id = {character_id}"
    )
    assert before == [["Renamed Candidate"]]
    assert "Renamed Candidate" in json.dumps(identity_before)
    client.call("create_starting_character", *args)
    client.call("create_starting_character", "b" * 64, *args[1:], error="different browser owner")
    assert client.rows(f"SELECT name FROM character WHERE id = {character_id}") == before
    assert len(client.rows("SELECT character_id FROM starting_character_claim")) == 1
    for fault, error in [("malformedJson", "Invalid character name identity"),
                         ("missingIdentity", "has no semantic name identity"),
                         ("invalidRendering", "rendered"),
                         ("invalidFamily", "missing-family"),
                         ("mismatchedFamily", "disagrees"),
                         ("invalidSurname", "missing-surname")]:
        client.fixture("authority_test_name_failure", character_id, {fault: []}, error=error)
        if fault in {"invalidFamily", "mismatchedFamily", "invalidSurname"}:
            client.fixture("authority_test_invalid_name_assignment", character_id,
                           {fault: []}, error=error)
        assert client.rows(
            f"SELECT identity_json FROM character_name_identity WHERE character_id = {character_id}"
        ) == identity_before
        assert client.rows(f"SELECT name FROM character WHERE id = {character_id}") == before
    client.call("create_named_character_with_id", 732003, "Unrelated Character")
    for target, error in [(0, "claim exists without its character"),
                          (732003, "does not match regenerated character")]:
        client.fixture("authority_test_claim_target", request_key, target)
        client.call("create_starting_character", *args, error=error)
    client.fixture("authority_test_claim_target", request_key, character_id)
    client.call("update_character", character_id, "\n", error="rendered")
    assert client.rows(f"SELECT name FROM character WHERE id = {character_id}") == before
    client.call("create_named_character_with_id", 732001, "\n", error="rendered")
    assert client.rows("SELECT id FROM character WHERE id = 732001") == []
    assert client.rows("SELECT character_id FROM character_personality WHERE character_id = 732001") == []
    client.fixture("authority_test_absent_surname", character_id)
    client.call("create_default_character", owner)
    [[default_id]] = client.rows(
        f"SELECT character_id FROM starting_character_claim WHERE request_key = 'default:3:{owner}'"
    )
    client.fixture("authority_test_name_projection", default_id)
    client.call("update_character", default_id, "Renamed Default")
    client.call("create_default_character", owner)
    assert client.rows(f"SELECT name FROM character WHERE id = {default_id}") == [["Renamed Default"]]
    # Browser grant validation fails after insertion; the entire claim and its
    # component rows must roll back.
    count_before = client.rows("SELECT COUNT(*) AS count FROM character")
    claims_before = client.rows("SELECT COUNT(*) AS count FROM starting_character_claim")
    client.call("create_starting_character", "invalid", *args[1:-1], 1,
                error="owner key is malformed")
    assert client.rows("SELECT COUNT(*) AS count FROM character") == count_before
    assert client.rows("SELECT COUNT(*) AS count FROM starting_character_claim") == claims_before
    print("PASS identity projections, renamed retries, ownership, absent surname, and rollback", flush=True)


def enrollment_checks(client: AcceptanceClient) -> None:
    character_id = 732002
    mission = "mission:diagnostic-authority-check"
    claim = "authority-check-claim"
    enemy_yaml = stack.read_enemy_fixture(stack.PASSIVE_ENEMY_FIXTURE)
    client.fixture("seed_standalone_tactical_mission", character_id, mission,
                   "woodland", enemy_yaml, claim)
    for tag, members in [("empty", []), ("duplicate", [character_id, character_id]),
                         ("oversized", list(range(17))), ("conflict", [character_id])]:
        clone = f"mission:authority-{tag}"
        client.fixture("authority_test_request_roster", mission, clone, members)
        client.call("authorize_tactical_server_claim", clone, list(hashlib.sha256(claim.encode()).digest()))
    server_identity, server_token = client.identity()
    other_identity, other_token = client.identity()
    for tag, error in [("empty", "at least one"), ("duplicate", "duplicate members"),
                       ("oversized", "participant limit")]:
        clone = f"mission:authority-{tag}"
        client.call("create_tactical_server_for_request", clone, claim, "127.0.0.1:1", "test",
                    token=other_token, error=error)
        assert len(client.rows(
            f"SELECT mission_id FROM tactical_server_request_authority WHERE mission_id = '{clone}'"
        )) == 1
        assert len(client.rows(
            f"SELECT mission_id FROM tactical_server_claim WHERE mission_id = '{clone}'"
        )) == 1
    client.call("create_tactical_server_for_request", mission, claim, "127.0.0.1:1", "test", token=server_token)
    client.call("create_tactical_server_for_request", "mission:authority-conflict", claim,
                "127.0.0.1:2", "test", token=other_token)
    client.call("enter_mission", character_id, ["0x" + server_identity], token=server_token)
    client.call("enter_mission", character_id, ["0x" + server_identity], token=server_token)
    client.call("enter_mission", character_id, ["0x" + other_identity], token=other_token,
                error="another active tactical server")
    client.call("leave_mission", character_id, token=other_token, error="owning tactical server")
    client.call("start_poppy_tincture", character_id, 0, error="tactical")
    client.call("leave_mission", character_id, token=server_token)
    zero = ["0x0"]
    assignment = client.rows(f"SELECT server FROM character WHERE id = {character_id}")
    assert assignment == [[zero]], assignment
    stale_identity, _ = client.identity()
    client.fixture("authority_test_stale_assignment", character_id, ["0x" + stale_identity])
    client.call("enter_mission", character_id, ["0x" + server_identity], token=server_token)
    client.call("end_tactical_server", {"failed": []}, [[], []], token=server_token)
    assignment = client.rows(f"SELECT server FROM character WHERE id = {character_id}")
    assert assignment == [[zero]], assignment
    assert client.rows(f"SELECT mission_id FROM tactical_server_authority WHERE mission_id = '{mission}'") == []
    print("PASS roster rejection and rollback, enrollment conflicts, leave, stale reclamation, cleanup", flush=True)


def resident_checks(client: AcceptanceClient) -> None:
    client.fixture("authority_test_seed_resident")
    [[resident_id]] = client.rows("SELECT character_id FROM settlement_resident_profile LIMIT 1")
    client.fixture("authority_test_name_projection", resident_id)
    client.fixture("authority_test_resident_projection", resident_id)
    client.fixture("authority_test_name_projection", resident_id)
    print("PASS current resident identity/demographics, traversal key, historical provenance", flush=True)


def run(profile: str, base_port: int) -> None:
    # The runner only uses a derived local profile and holds its ownership lock.
    os.environ["ADVENTURESIM_RUNTIME_ROOT"] = str(stack.ROOT / "target/authority-check/runtime")
    values = stack.profile_values(profile, base_port)
    root = stack.runtime_root()
    profile_dir = stack.ensure_secure_directory(Path(str(values["profile_dir"])), root)
    run_dir = stack.ensure_secure_directory(profile_dir / "run", root)
    data_dir = stack.ensure_secure_directory(profile_dir / "spacetimedb-data", root)
    with stack.ProfileLock(profile_dir / "lifecycle.lock") as lock:
        assert not stack.ports_in_use([base_port]), "isolated port is already occupied"
        server = f"http://127.0.0.1:{base_port}"
        database = str(values["database"])
        metadata = run_dir / "spacetime.identity.json"
        log = run_dir / "spacetime.log"
        config = {"role": "authority-check", "server": server, "database": database, "data_dir": str(data_dir)}
        process = stack.spawn_recorded([
            "spacetime", "start", "--non-interactive", "--listen-addr",
            f"127.0.0.1:{base_port}", "--data-dir", str(data_dir),
        ], metadata, log, config)
        client = None
        try:
            listener = stack.wait_for_spacetime(process, metadata, log, base_port)
            capability = stack.ResetCapability(profile, base_port, server, database, lock, listener)
            token = stack.dev_bootstrap_token()
            previous = os.environ.get("ADVENTURESIM_DEV_BOOTSTRAP_TOKEN")
            os.environ["ADVENTURESIM_DEV_BOOTSTRAP_TOKEN"] = token
            try:
                assert stack.reset_publish(capability, build_options="--features authority-tests") == 0
            finally:
                if previous is None:
                    os.environ.pop("ADVENTURESIM_DEV_BOOTSTRAP_TOKEN", None)
                else:
                    os.environ["ADVENTURESIM_DEV_BOOTSTRAP_TOKEN"] = previous
            client = AcceptanceClient(server, database, token)
            client.fixture("dev_bootstrap_base")
            identity_checks(client)
            enrollment_checks(client)
            resident_checks(client)
            client.fixture("authority_test_context_intervals")
            print("PASS context intervals, historical membership, idempotent closure", flush=True)
            client.fixture("authority_test_conception_chronology")
            print("PASS conception birth/death chronology and stale display projections", flush=True)
            client.fixture("authority_test_leisure_checkpoints")
            print("PASS leisure checkpoint partitions, trial crossings, repeated settlement", flush=True)
            client.fixture("authority_test_witness_resolutions")
            print("PASS unresolved/resolved witness admission, both outcomes, exact replay and collisions", flush=True)
            client.fixture("authority_test_witness_request_keys")
            print("PASS all witness keys, executing outcomes, authored response gates and exact/conflicting retries", flush=True)
            client.fixture("authority_test_treatment_receipts")
            print("PASS completed/interrupted treatment, exact replay and collisions", flush=True)
            client.fixture("authority_test_preparation_operations")
            print("PASS cut/grind conservation, digest vectors, replay, clipped generation", flush=True)
            client.fixture("authority_test_contact_existence")
            print("PASS contact existence, scoped awareness, surprise choice and revision retries", flush=True)
            client.fixture("authority_test_projectile_identity")
            print("PASS both projectile kinds, retained injury facts and zero-damage rejection", flush=True)
            client.fixture("authority_test_catalog_item_classification")
            print("PASS all materialized catalog kinds and authored capability projection", flush=True)
            client.fixture("authority_test_investigation_keys")
            for method, terrain, error in [("", "road", "Unknown investigation action method"),
                                            ("FollowTracks", "road", "Unknown investigation action method"),
                                            ("watch", "", "Unknown investigation terrain"),
                                            ("watch", "Road", "Unknown investigation terrain")]:
                client.fixture("authority_test_invalid_investigation_key", method, terrain, error=error)
                assert client.rows("SELECT id FROM investigation_action_capability WHERE id = 'authority-investigation:invalid'") == []
            print("PASS investigation storage keys, generated bindings, tracking prerequisites and malformed rollback", flush=True)
            client.fixture("authority_test_social_action_keys")
            print("PASS all social request keys, executing receipts, conditional skills and malformed rejection", flush=True)
            client.fixture("authority_test_automatic_chat_preferences")
            print("PASS automatic chat opt-in, opt-out, bounded attempts, party and death cleanup", flush=True)
            client.fixture("authority_test_seed_errantry_issuer")
            client.fixture("authority_test_challenge_lifecycle")
            print("PASS puzzle/road closure, wrong answers, exact retries, camp binding, delayed combat", flush=True)
            client.fixture("authority_test_mission_roster")
            print("PASS normal mission roster, invalid snapshots, planning drift, tactical launch, autoresolve", flush=True)
            [[scene_key]] = client.rows("SELECT scene_key FROM settlement LIMIT 1")
            enemy_fixture = (stack.ROOT / "assets/tactical-enemies/passive-bandit.yaml").read_text()
            client.fixture("seed_standalone_tactical_mission", 732082, "mission:diagnostic-authority-roster",
                           scene_key, enemy_fixture, "authority-standalone-claim")
            client.fixture("authority_test_standalone_roster")
            print("PASS standalone captured roster and launch cardinality", flush=True)
            client.fixture("authority_test_finale_receipts")
            print("PASS absent/selected/executed finales and exact/conflicting retries", flush=True)
            client.fixture("authority_test_offense_policy")
            print("PASS offense preflight, retries, fine policy, charge snapshots, downstream settlement", flush=True)
            client.fixture("authority_test_outcome_fact_payload")
            print("PASS outcome payload chronology, window evaluation, source retries and index attribution", flush=True)
            client.fixture("authority_test_disease_keys")
            for key in ["", "ShroudFever", "shroud-fever", "unknown"]:
                client.fixture("authority_test_invalid_disease_key", key, error="Unknown disease")
                assert client.rows("SELECT id FROM infection_episode WHERE id = 732888") == []
            print("PASS stored disease keys, ordered exposure sources, unknown-key rejection and rollback", flush=True)
            client.fixture("authority_test_outbreak_patient_lifecycle")
            print("PASS outbreak materialization, context release, recovery/death and historical suppression", flush=True)
            client.fixture("authority_test_finale_world_effects")
            print("PASS one-time world-event fame and local-problem resolution with conflicting retry rejection", flush=True)
        finally:
            if client is not None:
                client.http.close()
            stack.stop_recorded(metadata, config)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="authority-check")
    parser.add_argument("--base-port", type=int, default=25370)
    args = parser.parse_args()
    run(args.profile, args.base_port)
