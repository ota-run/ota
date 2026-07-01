//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//     ░░██████   ░░█████ ░░████████
//      ░░░░░░     ░░░░░   ░░░░░░░░
//
//   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
//
//   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
//
//   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
//   You may not use this file except in compliance with that License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

use serde_json::Value as JsonValue;

pub(crate) fn parse_jsonc_value(contents: &str) -> Result<JsonValue, serde_json::Error> {
    serde_json::from_str(&strip_trailing_commas(&strip_jsonc_comments(contents)))
}

fn strip_jsonc_comments(contents: &str) -> String {
    let chars = contents.chars().collect::<Vec<_>>();
    let mut cleaned = String::with_capacity(contents.len());
    let mut index = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while index < chars.len() {
        let ch = chars[index];
        let next = chars.get(index + 1).copied();

        if line_comment {
            if ch == '\n' {
                line_comment = false;
                cleaned.push(ch);
            }
            index += 1;
            continue;
        }

        if block_comment {
            if ch == '*' && next == Some('/') {
                block_comment = false;
                index += 2;
                continue;
            }
            if ch == '\n' {
                cleaned.push('\n');
            }
            index += 1;
            continue;
        }

        if in_string {
            cleaned.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            cleaned.push(ch);
            index += 1;
            continue;
        }

        if ch == '/' && next == Some('/') {
            line_comment = true;
            index += 2;
            continue;
        }

        if ch == '/' && next == Some('*') {
            block_comment = true;
            index += 2;
            continue;
        }

        cleaned.push(ch);
        index += 1;
    }

    cleaned
}

fn strip_trailing_commas(contents: &str) -> String {
    let chars = contents.chars().collect::<Vec<_>>();
    let mut cleaned = String::with_capacity(contents.len());
    let mut index = 0usize;
    let mut in_string = false;
    let mut escape = false;

    while index < chars.len() {
        let ch = chars[index];

        if in_string {
            cleaned.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            cleaned.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut lookahead = index + 1;
            let mut skip_comma = false;
            while let Some(next) = chars.get(lookahead) {
                if next.is_whitespace() {
                    lookahead += 1;
                    continue;
                }
                if *next == '}' || *next == ']' {
                    skip_comma = true;
                }
                break;
            }
            if skip_comma {
                index += 1;
                continue;
            }
        }

        cleaned.push(ch);
        index += 1;
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::parse_jsonc_value;

    #[test]
    fn parses_jsonc_comments_and_trailing_commas() {
        let json = parse_jsonc_value(
            r#"{
  // comment
  "image": "mcr.microsoft.com/devcontainers/python:1-3.12-bullseye",
  "features": {
    "ghcr.io/devcontainers/features/node:1": {},
  },
}"#,
        )
        .expect("jsonc parse");

        assert_eq!(
            json["image"].as_str(),
            Some("mcr.microsoft.com/devcontainers/python:1-3.12-bullseye")
        );
        assert!(
            json["features"]
                .get("ghcr.io/devcontainers/features/node:1")
                .is_some()
        );
    }
}
