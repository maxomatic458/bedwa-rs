> [!NOTE]
> Parts of the server logic will be seperated into [this](https://github.com/maxomatic458/valence-extra) repository (to make them more reusable).
> So a rewrite will probably happen before larger features can be added.  

# Bedwa-rs
A minecraft bedwars server written entirely in rust with [valence](https://github.com/valence-rs/valence)

# Current/Future Features 
- [X] Mostly vanilla combat system, that supports most pvp enchants and also bows & arrows
- [X] Use custom bedwars maps
- [X] Configurable shops
- [X] Configurable resource spawners
- [X] Chests & Enderchets (still some bugs with that)
- [ ] Potions
- [ ] Custom Items

# Getting started

* Download the binary from the release page or build it yourself with `cargo build --release`.
* Setup the directory as shown below (the world folder should be a 1.20.1 bedwars map with all blocks, beds and chests placed already)
  ```
  world/
  shop.json
  ```
* Run the ``bedwa-rs`` binary in the directory.

# Configuring the server
When you run the server for the first time, you will be placed in an edit mode.
Now you can use the items in your hotbar and the chat commands to configure the server.

## Chat commands
* `/bwa arenabounds <pos1> <pos2>`: Set the arena bounds.

* `/bwa team add <team_name> <team_color>`: Add a team.

* `/bwa team remove <team_name>`: Remove a team.

* `/bwa team spawn <team_name> <pos>`: Set the spawn of a team.

* `/bwa team bed <team_name> <pos>`: Set the bed of a team.

* `/bwa shop <team_name> <pos> <yaw> <team?>`: Place a shop (team is optional, when it is set, then the shop will only spawn if the team is in the match).

* `/bwa shop remove <pos>`: Remove a shop.

* `/bwa spawner add <pos> <resource> <interval> <amount> <team?>`: Add a resource spawner, `resource` is the minecraft item id, like `iron_ingot`, `interval` is the time in seconds between spawns, `amount` is the amount of items spawned, `team` is optional, when it is set, then the spawner will only spawn if the team is in the match.

* `/bwa spawner remove <pos>`: Remove a resource spawner.

* `/bwa lobby spawn <pos>`: Set the lobby spawn.

* `/bwa spectator spawn <pos>`: Set the spectator spawn.

* `/bwa summary`: Print a summary of the current configuration.

* ~~`/bwa help`: Print a list of all commands.~~ (not implemented yet)

* `/bwa save`: Save the configuration to disk, then you can restart the server to go into play mode.

## Shop configuration
The shop configuration is stored in the `shop.json` file in the server directory.
The file has this structure:
```jsonc
{
    "shop_items": {
        "BlockCategory": [ // Name of the category in the shop
            {
                "item": "white_wool", // Item for that category
                "count": 1,
                "nbt": null,
            },
            [ // List of items that can be bought
                {
                    "offer": {
                        "item": "white_wool",
                        "count": 4,
                        "nbt": null,
                    },
                    "price": {
                        "item": "iron_ingot",
                        "count": 1,
                        "nbt": null,
                    }
                }
            ]
        ]
    }
}
```

This is how an enchanted item would look like:
```jsonc
    {
        "offer": {
        "item": "diamond_sword",
        "count": 1,
        "nbt": {
            "display": {
                "Lore": [
                    "{\"text\":\"10 Gold\", \"italic\": \"false\", \"bold\": \"true\", \"color\": \"gold\"}'}}"
                ]
            },
            "Enchantments": [
            {
                "id": "minecraft:sharpness",
                "lvl": 1
            },
            {
                "id": "minecraft:fire_aspect",
                "lvl": 1
            },
            {
                "id": "minecraft:knockback",
                "lvl": 1
            }
            ]
        }  
        },
        "price": {
        "item": "gold_ingot",
        "count": 10,
        "nbt": null
        }
    }
```

