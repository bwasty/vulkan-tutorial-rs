#!/bin/bash
git diff --color-words --no-index "$1" "$2"
git diff --no-index "$1" "$2" > "$2".diff
