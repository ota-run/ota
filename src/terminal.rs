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

use std::env;

pub(crate) fn supports_dynamic_stderr_ui(is_terminal: bool) -> bool {
    is_terminal && terminal_supports_dynamic_ui(env::var("TERM").ok().as_deref())
}

fn terminal_supports_dynamic_ui(term: Option<&str>) -> bool {
    !matches!(term, Some(value) if value.eq_ignore_ascii_case("dumb"))
}

#[cfg(test)]
mod tests {
    use super::terminal_supports_dynamic_ui;

    #[test]
    fn dumb_term_disables_dynamic_terminal_ui() {
        assert!(!terminal_supports_dynamic_ui(Some("dumb")));
        assert!(!terminal_supports_dynamic_ui(Some("DUMB")));
    }

    #[test]
    fn missing_or_normal_term_enables_dynamic_terminal_ui() {
        assert!(terminal_supports_dynamic_ui(None));
        assert!(terminal_supports_dynamic_ui(Some("xterm-256color")));
    }
}
