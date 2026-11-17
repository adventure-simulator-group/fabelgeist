"""Live dialogue authority through real reducers in an isolated scene."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

import dev_stack as stack

if TYPE_CHECKING:
    from check_semantic_authorities import AcceptanceClient


def scene_dialogue_checks(client: AcceptanceClient, resident: int, settlement: str,
                          server_identity: str, server_token: str, outsider: str) -> None:
    player = 732014
    client.fixture("authority_test_settlement_actor", player, settlement, False)
    result = stack.run_checked(["cargo", "run", "-p", "adventuresim-dialogue",
                                "--bin", "dialogue-check", "--", "check"])
    assert result.returncode == 0, result.stdout
    match = re.search(r"dialogue catalog ([0-9a-f]{64}):", result.stdout)
    assert match is not None, result.stdout
    revision = match.group(1)
    client.call("enter_tactical_scene", player, ["0x" + server_identity], token=server_token)
    [[actor_visit]] = client.rows(f"SELECT id FROM local_scene_visit WHERE character_id = {player}")
    [[resident_visit]] = client.rows(f"SELECT id FROM local_scene_visit WHERE character_id = {resident}")
    session = f"dialogue:{player}:live-authority"
    request = [actor_visit, resident_visit, session, revision]
    for token in (None, outsider):
        client.call("start_tactical_dialogue", *request, token=token, error="registered")
    client.call("start_tactical_dialogue", resident_visit, actor_visit, session, revision,
                token=server_token, error="participation role")
    client.call("start_tactical_dialogue", actor_visit, resident_visit,
                f"dialogue:{resident}:wrong-owner", revision, token=server_token,
                error="Invalid dialogue session ID")
    before = client.rows(f"SELECT minutes, fraction FROM character_time WHERE character_id = {player}")
    client.call("start_tactical_dialogue", *request, token=server_token)
    client.call("start_tactical_dialogue", *request, token=server_token)
    assert client.rows(f"SELECT id FROM dialogue_session WHERE id = '{session}'") == [[session]]
    participants = client.rows(f"SELECT actor_id FROM dialogue_participant WHERE session_id = '{session}'")
    assert len(participants) == 2 and {row[0] for row in participants} == {
        f"character:{player}", str(resident)}, participants
    assert client.rows(f"SELECT minutes, fraction FROM character_time WHERE character_id = {player}") == before
    [[conversation, location]] = client.rows(
        f"SELECT conversation_id, location_id FROM dialogue_session WHERE id = '{session}'")
    client.call("start_dialogue", player, f"dialogue:{player}:forged-venue", conversation,
                str(resident), location, revision, error="owned by a live")
    [[topic], *_] = client.rows(f"SELECT topic_id FROM dialogue_topic_option WHERE session_id = '{session}'")
    for token in (None, outsider):
        client.call("choose_dialogue_topic", player, session, topic, "foreign", 0, revision,
                    token=token, error="owning tactical host")
    client.call("choose_dialogue_topic", player, session, topic, "topic-one", 0, revision,
                token=server_token)
    client.call("choose_dialogue_topic", player, session, topic, "topic-one", 0, revision,
                token=server_token)
    [[current_revision]] = client.rows(f"SELECT revision FROM dialogue_session WHERE id = '{session}'")
    assert current_revision == 1
    assert client.rows(f"SELECT minutes, fraction FROM character_time WHERE character_id = {player}") == before
    if conversation == "service-professions":
        client.call("choose_dialogue_topic", player, session, "return-recovered-property",
                    "deferred-investigation", 1, revision, token=server_token,
                    error="not available in a live scene")
        assert client.rows(f"SELECT id FROM dialogue_investigation_binding WHERE session_id = '{session}'") == []
    for state in ("away", "healthUnavailable", "elsewhere"):
        client.fixture("authority_test_resident_admission", resident, {state: []})
        client.call("choose_dialogue_topic", player, session, topic, "unavailable", 1, revision,
                    token=server_token, error="absent or unavailable")
    client.fixture("authority_test_resident_admission", resident, {"available": []})
    assert client.rows(f"SELECT revision FROM dialogue_session WHERE id = '{session}'") == [[1]]
    closed_visit_checks(client, player, actor_visit, resident_visit, session, topic,
                        revision, server_identity, server_token, conversation, location, resident)
    print("PASS live dialogue host/visit binding, current NPC authority, solo contact, idempotent actions, and unchanged stored clocks", flush=True)


def closed_visit_checks(client: AcceptanceClient, player: int, actor_visit: int,
                        resident_visit: int, session: str, topic: str, revision: str,
                        server_identity: str, server_token: str, conversation: str,
                        location: str, resident: int) -> None:
    client.call("finish_local_scene_visit", actor_visit, {"unprojected": []}, token=server_token)
    for token, error in ((None, "owning tactical host"), (server_token, "visit is closed")):
        client.call("choose_dialogue_topic", player, session, topic, "after-departure", 1,
                    revision, token=token, error=error)
    client.call("start_dialogue", player, session, conversation, str(resident), location,
                revision, error="owned by a live")
    client.call("enter_tactical_scene", player, ["0x" + server_identity], token=server_token)
    next_visit = max(row[0] for row in client.rows(
        f"SELECT id FROM local_scene_visit WHERE character_id = {player}"))
    assert next_visit != actor_visit
    client.call("start_tactical_dialogue", next_visit, resident_visit, session, revision,
                token=server_token, error="conflicts")
    client.call("close_tactical_dialogue", session, error="original tactical host")
    client.call("close_tactical_dialogue", session, token=server_token)
    client.call("close_tactical_dialogue", session, token=server_token)
    assert client.rows(f"SELECT state, revision FROM dialogue_session WHERE id = '{session}'") == [["closed", 2]]
    client.call("choose_dialogue_topic", player, session, topic, "after-close", 2, revision,
                token=server_token, error="closed")
    client.call("finish_local_scene_visit", next_visit, {"unprojected": []}, token=server_token)
    print("PASS dialogue stays bound after departure/re-entry and only its original host can close it", flush=True)
