from asyncio import sleep
from surrealdb import Surreal
import json
import os
import requests
import toml, sys

def get_json():
    with open("scripts/pokemon_models.json") as f:
        return json.load(f)

# TODO: this should be better, and shouldn't just check the http endpoint
async def wait_for_surreal(addr):
    for _ in range(10):
        try:
            resp = requests.get(f"http://{addr}", allow_redirects=False)
            if resp.status_code == 307:
                return True
        except requests.exceptions.ConnectionError:
            print("SurrealDB is not update yet. Waiting and trying again.")

        await sleep(2)
    return False

async def main():
    raw_pokemon = get_json()
    rocket_toml = toml.load("Rocket.toml")
    profile = os.environ['ROCKET_PROFILE']
    password = os.environ['ROCKET_SURREAL_PASSWORD']
    root_pwd = os.environ['ROOT_DB_PASSWORD']
    namespace = rocket_toml[profile]['surreal_namespace']
    addr = rocket_toml[profile]['surreal_addr']
    username = rocket_toml['default']['surreal_username']
    surreal_addr = f"ws://{addr}/rpc"

    surreal_ready = await wait_for_surreal(addr)
    if not surreal_ready:
        print("SurrealDB did not start.")
        return

    with Surreal(surreal_addr) as db:
        db.signin({"username": "root", "password": f"{root_pwd}"})

        db.use(namespace, "pokedraft")
        # Check if user exists, if yes exists
        resp = db.query("SELECT * FROM canary:surreal")
        if len(resp) >= 1:
            print("Canary found. Not moving forth with import.")
            return

        save_pokemon_to_db(db, raw_pokemon)
        create_pokemon_lists(db)
        create_draft_rules(db)

        result = db.create("canary", {"id": "surreal"})
        result = db.query(f"DEFINE USER {username} ON DATABASE PASSWORD '{password}' ROLES OWNER;")
        print(result)

    db.close()

# TODO maybe create a fixed ID for the initial sets?
def save_pokemon_to_db(db, pokemon):
    for pk in pokemon:
        pk['evolves_from'] = 0 if pk["evolves_from"] == "" else int(pk["evolves_from"])
        pk['type2'] = "NONE" if pk['type2'] == "" else pk['type2'].upper()
        db.create("pokemon",
            {
                "id": f"{pk['id']}",
                "dex_id": int(pk['id']),
                "name": pk["name"],
                "is_mythic": bool(pk['is_mythical']),
                "is_legendary": bool(pk['is_legendary']),
                "gen": int(pk['gen']),
                "evolves_from": int(pk['evolves_from']), 
                "type1": pk['type1'].upper(), 
                "type2": pk['type2'].upper()
            } 
        )

def pokemon_select(before_gen=None, is_legendary=None, is_mythic=None, base_evolution=False):
    sql = "SELECT id FROM pokemon"
    filters = []
    if before_gen is not None:
        filters.append(f"gen <= {before_gen}")
    if is_legendary is not None:
        filters.append(f"is_legendary = {is_legendary}")
    if is_mythic is not None:
        filters.append(f"is_mythic = {is_mythic}")
    if base_evolution:
        filters.append("evolves_from = 0")

    if len(filters) != 0:
        sql += " WHERE " + " and ".join(filters)

    return sql

def create_list(db, gen, filters):
    suffix, is_legendary, is_mythic, base_form = filters
    set_name = f"Pokemon Gen {gen} {suffix}"
    result = db.create("pokemon_draft_set", {"name": set_name})
    sub_sql = pokemon_select(gen, is_legendary, is_mythic, base_form)
    if result:
        print(f"Creating Set: {set_name}")
        db.query(f"RELATE {result['id']}->contains->({sub_sql})")
    else:
        print(f"There was an issue creating the draft list: {result}")

def create_pokemon_lists(db):
    pokemon_ds_filters = [
        ("Full Roster", None, None, False),
        ("Base Only", None, None, True),
        ("No Legends", False, False, False),
        ("No Legends and Base Only", False, False, True),
    ]

    for f in pokemon_ds_filters:
        for gen in range(1,10):
            create_list(db, gen, f)

    for f in pokemon_ds_filters:
        result = db.create("pokemon_draft_set", {"name": f"Pokemon All Gens {f[0]}"})
        sub_sql = pokemon_select()
        if result:
            print(f"Creating Set: Pokemon All Gens {f[0]}")
            db.query(f"RELATE {result['id']}->contains->({sub_sql})")
        else:
            print(f"There was an issue creating the draft list: {result}")

    debug_sql = "SELECT id FROM pokemon WHERE dex_id < 10"
    result = db.create("pokemon_draft_set", {"name": "Debug Set"})
    db.query(f"RELATE {result['id']}->contains->({debug_sql})")

def create_draft_rules(db):
    db.create("draft_rules", {
        "name": "Showdown Snake",
        "picks_per_round": 1,
        "bans_per_round": 3,
        "max_pokemon": 6,
        "starting_phase": "Ban",
        "turn_type": "Snake"
    })

    db.create("draft_rules", {
        "name": "Showdown Round Robin",
        "picks_per_round": 1,
        "bans_per_round": 3,
        "max_pokemon": 6,
        "starting_phase": "Ban",
        "turn_type": "RoundRobin"
    })

    db.create("draft_rules", {
        "name": "Nuzlocke Snake",
        "picks_per_round": 1,
        "bans_per_round": 2,
        "max_pokemon": 15,
        "starting_phase": "Ban",
        "turn_type": "Snake"
    })

    db.create("draft_rules", {
        "name": "Nuzlocke Round Robin",
        "picks_per_round": 1,
        "bans_per_round": 2,
        "max_pokemon": 15,
        "starting_phase": "Ban",
        "turn_type": "RoundRobin"
    })

    db.create("draft_rules", {
        "name": "Intergration Test Snake",
        "picks_per_round": 1,
        "bans_per_round": 1,
        "max_pokemon": 1,
        "starting_phase": "Ban",
        "turn_type": "Snake"
    })

    db.create("draft_rules", {
        "name": "Intergration Test Round Robin",
        "picks_per_round": 1,
        "bans_per_round": 1,
        "max_pokemon": 1,
        "starting_phase": "Ban",
        "turn_type": "RoundRobin"
    })

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
