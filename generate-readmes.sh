#!/usr/bin/env bash

# Generate readme files with `cargo readme`.

set -eu

cd "$(dirname "$0")"

for workspace_member in $(toml2json Cargo.toml | jq -r '.workspace.members[]'); do
    (
        cd "$workspace_member"
        cargo readme \
            --no-badges \
            --no-indent-headings \
            --no-title \
        > README.md
    )
done
