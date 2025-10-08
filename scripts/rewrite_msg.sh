#!/bin/bash

# Rewrite commit messages: lowercase first line and truncate to 60 chars

FIRST_LINE=true
while IFS= read -r line; do
    if $FIRST_LINE; then
        # Lowercase and truncate first line to 60 chars
        line=$(echo "$line" | tr '[:upper:]' '[:lower:]' | cut -c1-60)
        FIRST_LINE=false
    fi
    echo "$line"
done