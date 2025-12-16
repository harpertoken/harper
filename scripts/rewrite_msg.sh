# Copyright 2025 harpertoken
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

#!/bin/bash

# Rewrite commit messages: lowercase first line and truncate to 60 chars, or set default if empty

FIRST_LINE=true
HAS_LINES=false
while IFS= read -r line; do
    HAS_LINES=true
    if $FIRST_LINE; then
        # Lowercase and truncate first line to 60 chars, or set default if empty
        if [ -n "$line" ]; then
            line=$(echo "$line" | tr '[:upper:]' '[:lower:]' | cut -c1-60)
        else
            line="no commit message"
        fi
        FIRST_LINE=false
    fi
    echo "$line"
done

# If no lines at all, output default message
if ! $HAS_LINES; then
    echo "no commit message"
fi
