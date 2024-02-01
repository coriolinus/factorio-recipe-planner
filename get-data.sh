#!/usr/bin/env bash

curl \
    -o prototype-data.lua \
    --compressed \
    --compressed-ssh \
    --max-redirs 10 \
    --progress-bar \
    'https://gist.githubusercontent.com/Bilka2/6b8a6a9e4a4ec779573ad703d03c1ae7/raw/2758ed720e42cdcd9e5c3b2708a0f51b38a92daa/Data.raw%25201.1.91'
