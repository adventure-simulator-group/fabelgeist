"""Selected-area resident identity and host-scoped projection acceptance."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient


def resident_roster_checks(client: AcceptanceClient, settlement: str,
                           server_identity: str, server_token: str,
                           outsider: str, player_id: int) -> None:
    literal = settlement.replace("'", "''")
    [[operator], *_] = client.rows(
        "SELECT operator_character_id FROM settlement_business_operator "
        f"WHERE settlement_id = '{literal}'"
    )
    empty = {"operator_character_ids": [], "common_venues": []}
    area = {**empty, "operator_character_ids": [operator]}
    client.call("configure_settlement_residents", area, token=outsider,
                error="registered settlement host")
    client.call("configure_settlement_residents",
                {**empty, "operator_character_ids": [player_id]}, token=server_token,
                error="outside the captured settlement businesses")
    assert client.rows("SELECT server FROM scene_resident_roster") == []
    client.call("configure_settlement_residents", area, token=server_token)
    client.call("configure_settlement_residents", area, token=server_token)
    selected = client.rows(
        "SELECT character_ids FROM scene_resident_roster "
        f"WHERE server = 0x{server_identity}"
    )
    assert len(selected) == 1
    [characters] = selected[0]
    assert operator in characters and len(characters) == len(set(characters))
    client.call("configure_settlement_residents", empty, token=server_token,
                error="immutable")
    client.call("configure_settlement_residents",
                {**empty, "operator_character_ids": [operator, operator]},
                token=server_token, error="invalid or repeated identities")
    assert client.rows("SELECT character_ids FROM scene_resident_roster") == selected
    candidates = client.rows("SELECT character_id, name FROM scene_resident_candidates",
                             token=server_token)
    assert {row[0] for row in candidates} == set(characters)
    for character, name in candidates:
        assert client.rows(f"SELECT name FROM character WHERE id = {character}") == [[name]]
        assert client.rows(
            f"SELECT home_settlement_id FROM settlement_resident_profile WHERE character_id = {character}"
        ) == [[settlement]]
    assert client.rows("SELECT character_id FROM scene_resident_candidates", token=outsider) == []
    client.call("enter_tactical_scene", operator, ["0x" + server_identity],
                token=server_token, error="scene-owned admission")
    client.call("enter_settlement_resident", operator, token=outsider,
                error="registered settlement host")
    client.call("enter_settlement_resident", player_id, token=server_token,
                error="not a scene-owned resident")
    for state in ("away", "healthUnavailable", "elsewhere"):
        client.fixture("authority_test_resident_admission", operator, {state: []})
        client.call("enter_settlement_resident", operator, token=server_token,
                    error="absent or unavailable")
        if state == "elsewhere":
            assert operator not in {row[0] for row in client.rows(
                "SELECT character_id FROM scene_resident_candidates", token=server_token
            )}
    client.fixture("authority_test_resident_admission", operator, {"available": []})
    client.call("enter_settlement_resident", operator, token=server_token)
    client.call("enter_settlement_resident", operator, token=server_token)
    [[visit]] = client.rows(f"SELECT id FROM local_scene_visit WHERE character_id = {operator}")
    role_before = client.rows("SELECT role FROM connected_players", token=server_token)
    assert len(role_before) == 1
    client.fixture("authority_test_resident_admission", operator, {"transferToPlayer": []})
    assert operator not in {row[0] for row in client.rows(
        "SELECT character_id FROM scene_resident_candidates", token=server_token
    )}
    assert client.rows("SELECT role FROM connected_players", token=server_token) == role_before
    client.call("enter_tactical_scene", operator, ["0x" + server_identity],
                token=server_token, error="another scene participation role")
    client.call("finish_local_scene_visit", visit, {"unprojected": []}, token=server_token)
    client.call("enter_tactical_scene", operator, ["0x" + server_identity], token=server_token)
    role_after = client.rows("SELECT role FROM connected_players", token=server_token)
    assert len(role_after) == 1 and role_after != role_before
    client.call("finish_local_scene_visit", visit, {"unprojected": []}, token=server_token)
    assert client.rows("SELECT role FROM connected_players", token=server_token) == role_after
    next_visits = client.rows(f"SELECT id FROM local_scene_visit WHERE character_id = {operator}")
    next_visit = max(row[0] for row in next_visits)
    assert next_visit != visit
    client.call("finish_local_scene_visit", next_visit, {"unprojected": []}, token=server_token)
    print("PASS immutable selected-area resident roster, real character names, unique local identities, and foreign-host projection rejection", flush=True)
    print("PASS separate resident/player admission, absence/health gates, repeated enrollment, and visit-role ownership across authority transfer", flush=True)
