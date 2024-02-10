# Factorio Recipe Planner: Data Parser

This crate implements low-level parsing of Factorio data. It offers three
key facilities:

## `into-json` script

This script converts a raw Factorio data Lua dump into one or many JSON
files.

Example usage:

```sh
cargo run --bin into-json -- --split-toplevel prototype-data.lua prototype-data
```

This will produce a `prototype-data` directory containing a JSON file for
each top-level key of the input Lua definition. Using the `--split-toplevel`
flag is recommended, as otherwise the output JSON file is massive.

## [`parse_lua`] function

This function offers a programmatic interface to producing JSON from
Factorio Lua prototypes.

## [`models`] module

This module contains low-level Serde-compatible models which can be used to
parse Factorio prototypes.

This module is intentionally incomplete, as Factorio uses a very flexible
prototype system which is presumably quite nice to write by hand but which
is a pain to parse precisely. For now, the only type which is complete is
the [`Recipe`][models::Recipe] struct, as that is the focus of the
downstream tooling for which this library was initially written.

Models which have been indicated to be complete parse losslessly from
Factorio definitions. They do not quite reserialize identically to the
original definitions, but the reserialization preserves identical semantics.
(The exception is the `duration` field--serialized as
`energy_required`--which always reserializes as a float even when it has an
integral value.)

Because they have been optimized for lossless conversion from Factorio
definitions, these models can be a pain to work with in Rust code. It is
recommended to convert them into higher-level models before executing the
main logic of your program. In the future, a crate in this workspace will
provide appropriate higher-level models.
