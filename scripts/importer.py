from surrealdb import Surreal
import json

def get_json():
    with open("pokemon_models.json") as f:
        return json.load(f)

async def main():
    raw_pokemon = get_json()
    async with Surreal("ws://localhost:8000/rpc") as db:
        await db.signin({"user": "root", "pass": "root"})
        await db.use("test", "test")

        # clear table first
        await db.delete("pokemon")

        await save_pokemon_to_db(db, raw_pokemon)

async def save_pokemon_to_db(db, pokemon):
    for pk in pokemon:
        pk['evolves_from'] = 0 if pk["evolves_from"] == "" else int(pk["evolves_from"])
        pk['type2'] = "NONE" if pk['type2'] == "" else pk['type2'].upper()
        await db.create("pokemon",
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

async def create_pokemon_lists(db):
    db.create("pokemon_draft_set", {"name": "Pokemon Gen 1"})
    return ""

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
