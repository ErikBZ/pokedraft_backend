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

def unwrap_id(data):
    return data['id']['String']

def get_draft_obj_id(name, obj):
    draft_set_url = f"{API_URL}/draft_{obj}/"
    res = requests.get(draft_set_url)
    data = res.json()
    y = [x for x in data if x["name"] == name]

    if len(y) != 0:
        return unwrap_id(y[0]['id'])
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
RULE_NUZLOCKE_SNAKE_PICK_ID = get_draft_obj_id("Intergration Test Snake Pick First", "rules")

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

def check_draft_update(session_id):
    draft_session_url = f"{API_URL}/draft_session/{session_id}/update"
    res = requests.get(draft_session_url)
    return res.json(), res.status_code

def create_player(session, player_name):
    draft_session_url = f"{API_URL}/draft_session/{session}/create-user"
    post_data = {
        "name": player_name
    }
    res = requests.post(draft_session_url, json=post_data)
    if res.status_code != 200:
        print(f"Unable to create Draft Player with name {player_name} in Draft {session}. {res.status_code}")
        print(res.json())
        return ""
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

def toggle_user(session, player):
    toggle_url = f"{API_URL}/draft_session/{session}/ready"
    post_data = {
        "user_id": unwrap_id(player['user_id'])
    }
    res = requests.post(toggle_url, json=post_data)
    return res.json(), res.status_code

def start_session(session, player):
    toggle_url = f"{API_URL}/draft_session/{session}/start"
    post_data = {
        "user_id": unwrap_id(player['user_id'])
    }
    res = requests.post(toggle_url, json=post_data)
    return res.json(), res.status_code

# TESTS
@test
def test_create_session_4_player_join_select_snake():
    session = create_draft_session(DRAFT_DEBUG_SET_ID, RULE_NUZLOCKE_SNAKE_ID, "TEST 1")

    if session == "":
        return
    else:
        session = unwrap_id(session['id'])

    players = []

    print("Creating Players")
    for i in range(4):
        name = f"Player {i + 1}" 
        player = create_player(session, name)

        if player != "":
            players.append(player)
            print(f"Created User: {name} {player['user_id']} in Session: {session}")
        else:
            print(f"Unable to  create User: {name} in Session: {session}")
            return False

    for player in players:
        toggle_url = f"{API_URL}/draft_session/{session}/ready"
        post_data = {
            "user_id": unwrap_id(player['user_id'])
        }
        res = requests.post(toggle_url, json=post_data)
        assert res.status_code == 200

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[0])
    assert status == 404, f"{status}"
    assert res_data == {
            "message": "Draft has not yet started"
    }, f"{res_data}"
    print(f"Passed: Tried banning pokemon before draft has started")

    res_data, status = start_session(session, players[0])
    assert status == 200, f"{res_data}"

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[0])
    assert status == 200, f"{res_data}"
    assert res_data == {
        "phase": "Ban",
        "banned_pokemon": [1],
        "selected_pokemon": [],
    }, f"{res_data}"
    print(f"Passed: {players[0]['name']} banning pokemon 1.")

    res_data, status = player_ban_pokemon(session, players[1], DEBUG_POKEMON_SET[1])
    assert status == 200, f"{res_data}"
    assert res_data == {
        "phase": "Ban",
        "banned_pokemon": [1, 2],
        "selected_pokemon": [],
    }, f"{res_data}"
    print(f"Passed: {players[1]['name']} banning pokemon 2.")

    res_data, status = player_ban_pokemon(session, players[2], DEBUG_POKEMON_SET[2])
    assert status == 200, f"{res_data}"
    assert res_data == {
        "phase": "Ban",
        "banned_pokemon": [1, 2, 3],
        "selected_pokemon": [],
    }, f"{res_data}"
    print(f"Passed: {players[2]['name']} banning pokemon 3.")

    res_data, status = player_ban_pokemon(session, players[3], DEBUG_POKEMON_SET[3])
    assert status == 200, f"{res_data}"
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1, 2, 3, 4],
        "selected_pokemon": [],
    }, f"{res_data}"
    print(f"Passed: {players[3]['name']} banning pokemon 4.")

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[4])
    assert status == 404, f"{res_data}"
    assert res_data == { "message": "Current action not allowed" }, f"{res_data}"
    print(f"Passed: {players[0]['name']} failing to pick pokemon 5.")

    res_data, status = player_pick_pokemon(session, players[0], DEBUG_POKEMON_SET[4])
    assert status == 404, f"{res_data}"
    assert res_data == { "message": "It is not your turn" }, f"{res_data}"
    print(f"Passed: {players[0]['name']} failing to pick pokemon 5.")

    res_data, status = player_pick_pokemon(session, players[3], DEBUG_POKEMON_SET[4])
    assert status == 200, f"{res_data}"
    assert res_data == {
        "phase": "Pick",
        "banned_pokemon": [1, 2, 3, 4, 5],
        "selected_pokemon": [5],
    }, f"{res_data}"
    print(f"Passed: {players[3]['name']} picking pokemon 5.")

    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Pick', 'banned_pokemon': [1, 2, 3, 4, 5], 'current_player': 'Player 3', 'state': 'InProgress','players': [{'name': 'Player 1', 'pokemon': [], "ready": True}, {'name': 'Player 2', 'pokemon': [], "ready": True}, {'name': 'Player 3', 'pokemon': [], "ready": True}, {'name': 'Player 4', 'pokemon': [5], "ready": True}]}, f"{res_data}"

@test
def test_toggle_ready_on_pokemon():
    session = create_draft_session(DRAFT_DEBUG_SET_ID, RULE_NUZLOCKE_SNAKE_ID, "TEST 1")

    if session == "":
        return
    else:
        session = unwrap_id(session['id'])

    players = []

    print("Creating Players")
    for i in range(3):
        name = f"Player {i + 1}" 
        player = create_player(session, name)

        if player != "":
            players.append(player)
            print(f"Created User: {name} {player['user_id']} in Session: {session}")
        else:
            print(f"Unable to  create User: {name} in Session: {session}")
            return False
    
    res_data, status = check_draft_update(session)
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [], 'current_player': 'Player 1', 'state': 'Open', 'players': [{'name': 'Player 1', 'pokemon': [], 'ready': False},{'name': 'Player 2', 'pokemon': [], 'ready': False}, {'name': 'Player 3', 'pokemon': [], 'ready': False}]}, f"{res_data}"
    print("Passed: All players ready status is set to false.")

    res_data, status = toggle_user(session, players[0])
    assert status == 200, f"{res_data}"
    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [], 'current_player': 'Player 1', 'state': 'Open', 'players': [{'name': 'Player 1', 'pokemon': [], 'ready': True},{'name': 'Player 2', 'pokemon': [], 'ready': False}, {'name': 'Player 3', 'pokemon': [], 'ready': False}]}, f"{res_data}"
    print("Passed: Setting Player 1 Ready Status")

    res_data, status = toggle_user(session, players[1])
    assert status == 200, f"{res_data}"
    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [], 'current_player': 'Player 1','state':'Open', 'players': [{'name': 'Player 1', 'pokemon': [], 'ready': True},{'name': 'Player 2', 'pokemon': [], 'ready': True}, {'name': 'Player 3', 'pokemon': [], 'ready': False}]}, f"{res_data}"
    print("Passed: Setting Player 2 Ready Status")

    res_data, status = toggle_user(session, players[2])
    assert status == 200, f"{res_data}"
    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [], 'current_player': 'Player 1','state':'Ready', 'players': [{'name': 'Player 1', 'pokemon': [], 'ready': True},{'name': 'Player 2', 'pokemon': [], 'ready': True}, {'name': 'Player 3', 'pokemon': [], 'ready': True}]}, f"{res_data}"
    print("Passed: Setting Player 3 Ready Status")

    res_data, status = toggle_user(session, players[1])
    assert status == 200, f"{res_data}"
    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [], 'current_player': 'Player 1','state':'Open', 'players': [{'name': 'Player 1', 'pokemon': [], 'ready': True},{'name': 'Player 2', 'pokemon': [], 'ready': False}, {'name': 'Player 3', 'pokemon': [], 'ready': True}]}, f"{res_data}"
    print("Passed: Setting Player 1 Ready Status")

    res_data, status = toggle_user(session, players[1])
    assert status == 200, f"{res_data}"
    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [], 'current_player': 'Player 1','state':'Ready', 'players': [{'name': 'Player 1', 'pokemon': [], 'ready': True},{'name': 'Player 2', 'pokemon': [], 'ready': True}, {'name': 'Player 3', 'pokemon': [], 'ready': True}]}, f"{res_data}"
    print("Passed: Setting Player 1 Ready Status")


@test
def test_full_game():
    session = create_draft_session(DRAFT_DEBUG_SET_ID, RULE_NUZLOCKE_SNAKE_ID, "TEST 1")

    if session == "":
        return
    else:
        session = unwrap_id(session['id'])

    players = []

    print("Creating Players")
    for i in range(3):
        name = f"Player {i + 1}"
        player = create_player(session, name)

        if player != "":
            players.append(player)
            print(f"Created User: {name} {player['user_id']} in Session: {session}")
        else:
            print(f"Unable to  create User: {name} in Session: {session}")
            return False
        res_data, status = toggle_user(session, player)
        assert status == 200, f"{res_data}"

    print("Passed: Created all users and toggled them.")

    res_data, status = start_session(session, players[0])
    assert status == 200, f"{res_data}"
    print("Passed: Started draft session.")

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[0])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[0]['name']} banned {DEBUG_POKEMON_SET[0]}")

    res_data, status = player_ban_pokemon(session, players[1], DEBUG_POKEMON_SET[1])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[1]['name']} banned {DEBUG_POKEMON_SET[1]}")

    res_data, status = player_ban_pokemon(session, players[2], DEBUG_POKEMON_SET[2])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[2]['name']} banned {DEBUG_POKEMON_SET[2]}")

    res_data, status = player_pick_pokemon(session, players[2], DEBUG_POKEMON_SET[3])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[2]['name']} picked {DEBUG_POKEMON_SET[3]}")

    res_data, status = player_pick_pokemon(session, players[1], DEBUG_POKEMON_SET[4])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[1]['name']} picked {DEBUG_POKEMON_SET[4]}")

    res_data, status = player_pick_pokemon(session, players[0], DEBUG_POKEMON_SET[5])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[0]['name']} picked {DEBUG_POKEMON_SET[5]}")

    res_data, status = player_ban_pokemon(session, players[0], DEBUG_POKEMON_SET[6])
    assert status == 404, f"{res_data}"
    print(f"Passed: {players[0]['name']} failed in banning {DEBUG_POKEMON_SET[6]}")

    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [1,2,3,4,5,6], 'current_player': 'Player 1','state':'Ended', 'players': [{'name': 'Player 1', 'pokemon': [6], 'ready': True},{'name': 'Player 2', 'pokemon': [5], 'ready': True}, {'name': 'Player 3', 'pokemon': [4], 'ready': True}]}, f"{res_data}"
    print("Passed: Checking state update")

@test
def test_full_game_pick_first():
    session = create_draft_session(DRAFT_DEBUG_SET_ID, RULE_NUZLOCKE_SNAKE_PICK_ID, "TEST 1")

    if session == "":
        return
    else:
        session = unwrap_id(session['id'])

    players = []

    print("Creating Players")
    for i in range(3):
        name = f"Player {i + 1}"
        player = create_player(session, name)

        if player != "":
            players.append(player)
            print(f"Created User: {name} {player['user_id']} in Session: {session}")
        else:
            print(f"Unable to  create User: {name} in Session: {session}")
            return False
        res_data, status = toggle_user(session, player)
        assert status == 200, f"{res_data}"

    print("Passed: Created all users and toggled them.")

    res_data, status = start_session(session, players[0])
    assert status == 200, f"{res_data}"
    print("Passed: Started draft session.")

    res_data, status = player_pick_pokemon(session, players[0], DEBUG_POKEMON_SET[0])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[0]['name']} banned {DEBUG_POKEMON_SET[0]}")

    res_data, status = player_pick_pokemon(session, players[1], DEBUG_POKEMON_SET[1])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[1]['name']} banned {DEBUG_POKEMON_SET[1]}")

    res_data, status = player_pick_pokemon(session, players[2], DEBUG_POKEMON_SET[2])
    assert status == 200, f"{res_data}"
    print(f"Passed: {players[2]['name']} banned {DEBUG_POKEMON_SET[2]}")

    res_data, status = player_ban_pokemon(session, players[2], DEBUG_POKEMON_SET[3])
    assert status == 404, f"{res_data}"
    print(f"Passed: {players[2]['name']} picked {DEBUG_POKEMON_SET[3]}")

    res_data, status = check_draft_update(session)
    assert status == 200, f"{res_data}"
    assert res_data == {'current_phase': 'Ban', 'banned_pokemon': [1,2,3], 'current_player': 'Player 3','state':'Ended', 'players': [{'name': 'Player 1', 'pokemon': [1], 'ready': True},{'name': 'Player 2', 'pokemon': [2], 'ready': True}, {'name': 'Player 3', 'pokemon': [3], 'ready': True}]}, f"{res_data}"
    print("Passed: Setting Player 1 Ready Status")

def extra():
    pass

def run_all():
    test_create_session_4_player_join_select_snake()
    test_toggle_ready_on_pokemon()
    test_full_game()
    test_full_game_pick_first()

def print_usage():
    print("intergration_run_draft_session.py [all|list<int>]")

if __name__ == "__main__":
    args = sys.argv[1:]

    if len(args) < 1:
        print_usage()
        sys.exit()

    tests_to_run = set(args[0].split(','))

    if 'all' in tests_to_run:
        run_all()
    else:
        if '1' in tests_to_run:
            test_create_session_4_player_join_select_snake()
        if '2' in tests_to_run:
            test_toggle_ready_on_pokemon()
        if '3' in tests_to_run:
            test_full_game()
        if '4' in tests_to_run:
            test_full_game_pick_first()

