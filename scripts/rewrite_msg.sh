#!/bin/bash

# Rewrite commit messages: lowercase first line and truncate to 60 chars

FIRST_LINE=true
while IFS= read -r line; do
    if $FIRST_LINE; then
        # Lowercase and truncate first line to 60 chars, but only if not empty
        if [ -n "$line" ]; then
            line=$(echo "$line" | tr '[:upper:]' '[:lower:]' | cut -c1-60)
        fi
        FIRST_LINE=false
    fi
    echo "$line"
done