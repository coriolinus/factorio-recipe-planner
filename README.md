# Factorio Recipe Planner

This project is comprised of two parts. One is a more general parser for the recipe data. The other uses that data to interactively design factory modules.

Note that everything is WIP for now; neither of the features have yet been fully implemented.

## Data Parser

Factorio recipe data is not included in this repository, but can be found in various places online. Either use a precompiled data set, or extract your own given your currnet mod list.

A script, `get-data.sh`, is provided, which fetches data for the base game. Not affiliated, no guarantee of correctness, etc. It emits a file in the current working directory called `prototype-data.lua`.

## Module Planner

You tell it things like the assembler type, belt type, modules used, and any assumed-infinite inputs; it tells you what you'll get from one assembler, what you'll need to fill one belt, and what you'll need to
