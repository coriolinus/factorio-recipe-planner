# Factorio Recipe Planner

This project is comprised of two parts. One is a more general parser for the recipe data. The other uses that data to interactively design factory modules.

Note that everything is WIP for now; neither of the features have yet been fully implemented.

## Data Parser

Factorio recipe data is not included in this repository, but can be found in various places online. Either use a precompiled data set, or extract your own given your currnet mod list.

A script, `get-data.sh`, is provided, which fetches data for the base game. Not affiliated, no guarantee of correctness, etc. It emits a file in the current working directory called `prototype-data.lua`.

To transform this data into a collection of JSON files, run

```sh
cargo run --bin into-json --release -- --split-toplevel prototype-data.lua prototype-data
```

You can now inspect individual recipes as desired:

```sh
$ jq '."copper-cable"' prototype-data/recipe.json
{
  "ingredients": [
    [
      "copper-plate",
      1
    ]
  ],
  "name": "copper-cable",
  "result": "copper-cable",
  "result_count": 2,
  "type": "recipe"
}
$ jq '."uranium-processing"' prototype-data/recipe.json
Fri 02 Feb 2024 01:18:10 AM CET
{
  "category": "centrifuging",
  "enabled": false,
  "energy_required": 12,
  "icon": "__base__/graphics/icons/uranium-processing.png",
  "icon_mipmaps": 4,
  "icon_size": 64,
  "ingredients": [
    [
      "uranium-ore",
      10
    ]
  ],
  "name": "uranium-processing",
  "order": "k[uranium-processing]",
  "results": [
    {
      "amount": 1,
      "name": "uranium-235",
      "probability": 0.007000000000000001
    },
    {
      "amount": 1,
      "name": "uranium-238",
      "probability": 0.993
    }
  ],
  "subgroup": "raw-material",
  "type": "recipe"
}
```

Note that there is variance between `"result"` and `"results"`. `"energy_required"` appears to be recipe duration in seconds.

## Module Planner

You tell it things like the assembler type, belt type, modules used, and any assumed-infinite inputs; it tells you what you'll get from one assembler, what you'll need to fill one belt, and what you'll need to
