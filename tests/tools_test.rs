// Copyright 2025 harpertoken
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unit tests for tool modules

use harper::tools::parsing;

#[test]
fn test_extract_tool_arg() {
    let response = "[READ_FILE src/main.rs]";
    let result = parsing::extract_tool_arg(response, "[READ_FILE");
    assert_eq!(result.unwrap(), "src/main.rs");
}

#[test]
fn test_extract_tool_args() {
    let response = "[SEARCH_REPLACE file.rs old new]";
    let args = parsing::extract_tool_args(response, "[SEARCH_REPLACE", 3).unwrap();
    assert_eq!(args, vec!["file.rs", "old", "new"]);
}
