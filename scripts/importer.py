from surrealdb import Surreal
import json
import os

ROOT_DB_PASSWORD="sadf9843jqlrbq43lbadf"
def get_json():
    with open("scripts/pokemon_models.json") as f:
        return json.load(f)

async def main():
    raw_pokemon = get_json()
    password = os.environ['ROCKET_SURREAL_PASSWORD']
    async with Surreal("ws://172.21.0.2:8000/rpc") as db:
        # TODO: the root username and password can be custom
        # load from some variable
        await db.signin({"user": "root", "pass": f"{ROOT_DB_PASSWORD}"})

        await db.use("dev", "pokedraft")

        # clear table first
        await db.delete("pokemon")
        await save_pokemon_to_db(db, raw_pokemon)

        await db.delete("pokemon_draft_set")
        await db.delete("contains")
        await create_pokemon_lists(db)

        await db.delete("draft_rules")
        await create_draft_rules(db)
        result = await db.query(f"DEFINE USER pokedraft_api ON DATABASE PASSWORD '{password}' ROLES OWNER;")
        print(result)

# TODO maybe create a fixed ID for the initial sets?
async def save_pokemon_to_db(db, pokemon):
    for pk in pokemon:
        pk['evolves_from'] = 0 if pk["evolves_from"] == "" else int(pk["evolves_from"])
        pk['type2'] = "NONE" if pk['type2'] == "" else pk['type2'].upper()
        result = await db.create("pokemon",
            {
                "id": f"pokemon:{pk['id']}",
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

async def create_list(db, gen, filters):
    suffix, is_legendary, is_mythic, base_form = filters
    set_name = f"Pokemon Gen {gen} {suffix}"
    result = await db.create("pokemon_draft_set", {"name": set_name})
    sub_sql = pokemon_select(gen, is_legendary, is_mythic, base_form)
    if len(result) == 1:
        print(f"Creating Set: {set_name}")
        await db.query(f"RELATE {result[0]['id']}->contains->({sub_sql})")
    else:
        print(f"Result was not a single item, but expected only 1: {result}")

async def create_pokemon_lists(db):
    pokemon_ds_filters = [
        ("Full Roster", None, None, False),
        ("Base Only", None, None, True),
        ("No Legends", False, False, False),
        ("No Legends and Base Only", False, False, True),
    ]

    for f in pokemon_ds_filters:
        for gen in range(1,10):
            await create_list(db, gen, f)

    for f in pokemon_ds_filters:
        result = await db.create("pokemon_draft_set", {"name": f"Pokemon All Gens {f[0]}"})
        sub_sql = pokemon_select()
        if len(result) == 1:
            print(f"Creating Set: Pokemon All Gens {f[0]}")
            await db.query(f"RELATE {result[0]['id']}->contains->({sub_sql})")
        else:
            print(f"Result was not a single item, but expected only 1: {result}")

    debug_sql = "SELECT id FROM pokemon WHERE dex_id < 10"
    result = await db.create("pokemon_draft_set", {"name": "Debug Set"})
    await db.query(f"RELATE {result[0]['id']}->contains->({debug_sql})")

async def create_draft_rules(db):
    await db.create("draft_rules", {
        "name": "Showdown Snake",
        "picks_per_round": 1,
        "bans_per_round": 3,
        "max_pokemon": 6,
        "starting_phase": "Ban",
        "turn_type": "Snake"
    })

    await db.create("draft_rules", {
        "name": "Showdown Round Robin",
        "picks_per_round": 1,
        "bans_per_round": 3,
        "max_pokemon": 6,
        "starting_phase": "Ban",
        "turn_type": "RoundRobin"
    })

    await db.create("draft_rules", {
        "name": "Nuzlocke Snake",
        "picks_per_round": 1,
        "bans_per_round": 2,
        "max_pokemon": 15,
        "starting_phase": "Ban",
        "turn_type": "Snake"
    })

    await db.create("draft_rules", {
        "name": "Nuzlocke Round Robin",
        "picks_per_round": 1,
        "bans_per_round": 2,
        "max_pokemon": 15,
        "starting_phase": "Ban",
        "turn_type": "RoundRobin"
    })

    await db.create("draft_rules", {
        "name": "Intergration Test Snake",
        "picks_per_round": 1,
        "bans_per_round": 1,
        "max_pokemon": 1,
        "starting_phase": "Ban",
        "turn_type": "Snake"
    })

    await db.create("draft_rules", {
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
