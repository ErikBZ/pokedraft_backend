#!/usr/bin/env
import requests, sys
from functools import wraps

from requests.api import post

def test(func):
    @wraps(func)
    def wrapper(*args, **kwargs):
        print(f"{func.__module__}.{func.__qualname__}")
        return func(*args, **kwargs)
    return wrapper

LOCAL_URL = "http://localhost:8080"
API_URL = f"{LOCAL_URL}/api/v1"

def get_draft_obj_id(name, obj):
    draft_set_url = f"{API_URL}/draft_{obj}/"
    res = requests.get(draft_set_url)
    data = res.json()
    y = [x for x in data if x["name"] == name]

    if len(y) != 0:
        return y[0]["id"]["id"]["String"]
    else:
        return ""

def get_draft_set_pokemon(draft_set_id):
    draft_set_url = f"{API_URL}/draft_set/{draft_set_id}"
    res = requests.get(draft_set_url)
    json_data = res.json()

    if res.status_code != 200:
        return ""
    else:
        return json_data["pokemon"]["Ids"]

DRAFT_DEBUG_SET_ID = get_draft_obj_id("Debug Set", "set")
assert DRAFT_DEBUG_SET_ID != ""
RULE_NUZLOCKE_SNAKE_ID = get_draft_obj_id("Intergration Test Snake", "rules")
assert RULE_NUZLOCKE_SNAKE_ID != ""

DEBUG_POKEMON_SET = get_draft_set_pokemon(DRAFT_DEBUG_SET_ID)

def create_draft_session(set_id, rules_id, draft_name):
    draft_session_url = f"{API_URL}/draft_session/create"
    post_data = {
        "name": draft_name,
        "draft_set": set_id,
        "draft_rules": rules_id,
        "min_num_players": 2,
        "max_num_players": 4,
    }
    res = requests.post(draft_session_url, json=post_data)
    if res.status_code != 200:
        print(f"Unable to create Draft Session with name {draft_name}. {res.status_code}")
        return ""
    else:
        return res.json()

def create_player(session, player_name):
    draft_session_url = f"{API_URL}/draft_session/{session}/create-user"
    post_data = {
        "name": player_name
    }
    res = requests.post(draft_session_url, json=post_data)
    if res.status_code != 200:
        print(f"Unable to create Draft Player with name {player_name} in Draft {session}. {res.status_code}")
    else:
        return res.json()

def player_select_pokemon(session, player, pokemon, action="SELECT"):
    select_pokemon_url = f"{API_URL}/draft_session/{session}/select-pokemon"
    post_data = {
        "user_id": player['user_id'],
        "pokemon_id": pokemon,
        "action": action,
        "secret": player['key'],
    }
    res = requests.post(select_pokemon_url, json=post_data)
    return res.json(), res.status_code

def player_ban_pokemon(session, player, pokemon):
    return player_select_pokemon(session, player, pokemon, action="Ban")

def player_pick_pokemon(session, player, pokemon):
    return player_select_pokemon(session, player, pokemon, action="Pick")

# TESTS
@test
def test_create_session_4_player_join_select_snake():
    session = create_draft_session(DRAFT_DEBUG_SET_ID, RULE_NUZLOCKE_SNAKE_ID, "TEST 1")

    if session == "":
        return
    else:
        session = session['id']

    players = []

    print("Creating Players")
    for i in range(4):
        name = f"Player {i + 1}" 
        player = create_player(session, name)

        if player != "":
            players.append(player)
            print(f"Created User: {name} in Session: {session}")
        else:
            print(f"Unable to  create User: {name} in Session: {session}")
            return False

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[0])
    assert status == 200
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1],
        "selected_pokemon": [],
    }, f"{res_data}"

    res_data, status = player_ban_pokemon(session, players[1], DEBUG_POKEMON_SET[1])
    assert status == 200
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1, 2],
        "selected_pokemon": [],
    }, f"{res_data}"

    res_data, status = player_ban_pokemon(session, players[2], DEBUG_POKEMON_SET[2])
    assert status == 200
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1, 2, 3],
        "selected_pokemon": [],
    }, f"{res_data}"

    res_data, status = player_ban_pokemon(session, players[3], DEBUG_POKEMON_SET[3])
    assert status == 200
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1, 2, 3, 4],
        "selected_pokemon": [],
    }, f"{res_data}"

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[0])
    assert status == 404
    assert res_data == { "message": "It is not your turn" }

    res_data, status = player_ban_pokemon(session, players[3], DEBUG_POKEMON_SET[3])
    assert status == 200
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1, 2, 3, 4],
        "selected_pokemon": [5],
    }, f"{res_data}"

@test
def test_create_session_3_player_join_select():
    pass

@test
def test_create_session_2_player_join_select():
    pass

@test
def test_create_session_1_player_join_select():
    pass

def extra():
    pass

def run_tests():
    test_create_session_4_player_join_select_snake()
    test_create_session_1_player_join_select()
    test_create_session_2_player_join_select()
    test_create_session_3_player_join_select()

if __name__ == "__main__":
    run_tests()

