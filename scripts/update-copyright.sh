#!/bin/bash
# Update copyright year in harpertoken files
# Also adds headers to files missing them

YEAR=$(date +%Y)
HEADER="// Copyright $YEAR harpertoken
//
// Licensed under the Apache License, Version 2.0 (the \"License\");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an \"AS IS\" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
"

# Find all .rs files excluding target/
find . -type f -name "*.rs" -not -path "*/target/*" | sort > /tmp/rs_files.txt

# Find files with existing copyright
find . -type f -name "*.rs" -not -path "*/target/*" -exec grep -l "Copyright.*harpertoken" {} \; 2>/dev/null | sort > /tmp/has_copyright.txt

# Find files missing copyright
comm -23 /tmp/rs_files.txt /tmp/has_copyright.txt > /tmp/missing.txt

if [ -s /tmp/missing.txt ]; then
    echo "Files missing copyright header:"
    cat /tmp/missing.txt
    echo ""
    read -p "Add headers to these files? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        while IFS= read -r f; do
            # Prepend header to file
            { echo "$HEADER"; cat "$f"; } > "$f.tmp" && mv "$f.tmp" "$f"
            echo "Added header to $f"
        done < /tmp/missing.txt
    fi
else
    echo "All .rs files have copyright header"
fi

# Update existing copyright years
while IFS= read -r f; do
    sed -i '' "s/Copyright [0-9]* harpertoken/Copyright $YEAR harpertoken/g" "$f"
    echo "Updated $f"
done < /tmp/has_copyright.txt

rm /tmp/rs_files.txt /tmp/has_copyright.txt /tmp/missing.txt 2>/dev/null
