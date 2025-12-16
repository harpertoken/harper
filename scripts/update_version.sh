#!/bin/bash

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

# Script to update Harper version across all relevant files
# Usage: ./scripts/update_version.sh <new_version>

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 0.3.3"
    exit 1
fi

NEW_VERSION=$1

echo "Updating version to $NEW_VERSION..."

# Update VERSION file
echo "$NEW_VERSION" > VERSION

# Update Cargo.toml (only the package version, not dependency versions)
sed -i.bak "/^\[package\]/,/^\[/{ s/^version = \".*\"/version = \"$NEW_VERSION\"/; }" Cargo.toml && rm Cargo.toml.bak

# Update config/default.toml comment
sed -i.bak "s/# Updated for v.*/# Updated for v$NEW_VERSION/" config/default.toml && rm config/default.toml.bak

# Build to update Cargo.lock
cargo build --quiet

echo "Version updated to $NEW_VERSION in VERSION, Cargo.toml, config/default.toml, and Cargo.lock"
echo "Don't forget to update CHANGELOG.md manually for the new version section!"
