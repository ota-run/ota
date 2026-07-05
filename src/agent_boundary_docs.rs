#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ParsedAgentBoundaryDoc {
    pub(crate) generated_by_ota: bool,
    pub(crate) safe_tasks: Option<Vec<String>>,
    pub(crate) verify_after_changes: Option<Vec<String>>,
    pub(crate) writable_paths: Option<Vec<String>>,
    pub(crate) protected_paths: Option<Vec<String>>,
    pub(crate) task_commands: Vec<ParsedAgentBoundaryTaskCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedAgentBoundaryTaskCommand {
    pub(crate) task_name: String,
    pub(crate) command: String,
    pub(crate) source_key: String,
}

impl ParsedAgentBoundaryDoc {
    pub(crate) fn is_empty(&self) -> bool {
        !self.generated_by_ota
            && self.safe_tasks.is_none()
            && self.verify_after_changes.is_none()
            && self.writable_paths.is_none()
            && self.protected_paths.is_none()
            && self.task_commands.is_empty()
    }
}

pub(crate) fn parse_agent_boundary_doc(contents: &str) -> ParsedAgentBoundaryDoc {
    let generated_by_ota = is_ota_generated_agent_doc(contents);
    let parse_contents = if generated_by_ota {
        contents
            .rfind("# AGENTS.md")
            .and_then(|index| contents.get(index..))
            .unwrap_or(contents)
    } else {
        contents
    };
    let mut parsed = ParsedAgentBoundaryDoc {
        generated_by_ota,
        ..ParsedAgentBoundaryDoc::default()
    };
    let lines = parse_contents.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim_end();
        if let Some(label) = parse_agent_doc_list_label(line) {
            let mut values = Vec::new();
            index += 1;
            while index < lines.len() {
                let item_line = lines[index].trim_end();
                if let Some(value) = parse_agent_doc_task_item(item_line) {
                    values.push(value);
                    index += 1;
                } else {
                    break;
                }
            }
            assign_agent_doc_list_field(&mut parsed, label, values);
            continue;
        }
        if let Some((label, values)) = parse_agent_doc_inline_list(line) {
            assign_agent_doc_list_field(&mut parsed, label, values);
        }
        index += 1;
    }
    parsed.task_commands = parse_agent_doc_task_commands(&lines);
    parsed
}

fn is_ota_generated_agent_doc(contents: &str) -> bool {
    if !contents.contains("Generated from `") {
        return false;
    }

    contents.contains("` by `ota agents`.")
        || contents.contains("## Agent Contract")
            && (contents.contains("- `safe_tasks`:")
                || contents.contains("- `verify_after_changes`:")
                || contents.contains("- `writable_paths`:")
                || contents.contains("- `protected_paths`:"))
}

fn parse_agent_doc_list_label(line: &str) -> Option<&'static str> {
    match line.trim() {
        "- `safe_tasks`:" => Some("safe_tasks"),
        "- `verify_after_changes`:" => Some("verify_after_changes"),
        _ => None,
    }
}

fn parse_agent_doc_task_item(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    let remainder = trimmed.strip_prefix("  - `")?;
    let (value, _) = remainder.split_once('`')?;
    Some(value.to_string())
}

fn parse_agent_doc_inline_list(line: &str) -> Option<(&'static str, Vec<String>)> {
    let trimmed = line.trim();
    let (label, remainder) = if let Some(remainder) = trimmed.strip_prefix("- `writable_paths`: ") {
        ("writable_paths", remainder)
    } else if let Some(remainder) = trimmed.strip_prefix("- `protected_paths`: ") {
        ("protected_paths", remainder)
    } else {
        return None;
    };

    let values = remainder
        .split(',')
        .filter_map(|segment| {
            let value = segment.trim().strip_prefix('`')?.strip_suffix('`')?;
            (!value.is_empty()).then(|| value.to_string())
        })
        .collect::<Vec<_>>();
    Some((label, values))
}

fn assign_agent_doc_list_field(
    parsed: &mut ParsedAgentBoundaryDoc,
    label: &str,
    values: Vec<String>,
) {
    match label {
        "safe_tasks" => parsed.safe_tasks = Some(values),
        "verify_after_changes" => parsed.verify_after_changes = Some(values),
        "writable_paths" => parsed.writable_paths = Some(values),
        "protected_paths" => parsed.protected_paths = Some(values),
        _ => {}
    }
}

fn parse_agent_doc_task_commands(lines: &[&str]) -> Vec<ParsedAgentBoundaryTaskCommand> {
    let mut commands = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        if !is_agent_doc_commands_heading(lines[index]) {
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() {
            if let Some(header_kind) = classify_agent_doc_task_command_table_header(lines, index) {
                index += 2;
                while index < lines.len() {
                    let line = lines[index].trim_end();
                    if !looks_like_agent_doc_table_row(line) {
                        break;
                    }
                    if let Some(command) = parse_agent_doc_table_task_command(line, header_kind) {
                        commands.push(command);
                    }
                    index += 1;
                }
                continue;
            }
            let line = lines[index].trim_end();
            if line.trim_start().starts_with('#') {
                break;
            }
            if let Some(command) = parse_agent_doc_bullet_task_command(line) {
                commands.push(command);
            } else if let Some(command) =
                parse_agent_doc_table_task_command(line, AgentDocCommandTableHeader::TaskCommand)
            {
                commands.push(command);
            }
            index += 1;
        }
    }
    commands
}

fn looks_like_agent_doc_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|')
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentDocCommandTableHeader {
    TaskCommand,
    CommandDescription,
}

fn classify_agent_doc_task_command_table_header(
    lines: &[&str],
    index: usize,
) -> Option<AgentDocCommandTableHeader> {
    let Some(header) = lines.get(index).map(|line| line.trim()) else {
        return None;
    };
    let Some(separator) = lines.get(index + 1).map(|line| line.trim()) else {
        return None;
    };
    if !header.starts_with('|') || !separator.starts_with('|') {
        return None;
    }
    let header_cells = header
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let separator_cells = separator
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    let valid_separator = separator_cells.len() >= 2
        && separator_cells.iter().all(|cell| !cell.is_empty())
        && separator_cells
            .iter()
            .all(|cell| cell.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '));
    if header_cells.len() < 2 || !valid_separator {
        return None;
    }

    match (header_cells[0].as_str(), header_cells[1].as_str()) {
        ("task", "command") => Some(AgentDocCommandTableHeader::TaskCommand),
        ("command", "what it does" | "description" | "purpose") => {
            Some(AgentDocCommandTableHeader::CommandDescription)
        }
        _ => None,
    }
}

fn is_agent_doc_commands_heading(line: &str) -> bool {
    let trimmed = line.trim();
    let heading = trimmed.trim_start_matches('#').trim();
    trimmed.starts_with('#')
        && matches!(
            heading,
            "Commands"
                | "Individual Commands"
                | "Common Commands"
                | "Common commands"
                | "Quick Reference Commands"
                | "Build/Test Commands"
                | "Build & Development Commands"
        )
}

fn parse_agent_doc_bullet_task_command(line: &str) -> Option<ParsedAgentBoundaryTaskCommand> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("- ")?;
    let (label, after_label) = remainder.split_once(':')?;
    let command = parse_backticked_command(after_label.trim())?;
    let task_name = canonical_agent_doc_task_name_from_command(&command)
        .or_else(|| canonical_agent_doc_task_name(label))?;
    Some(ParsedAgentBoundaryTaskCommand {
        source_key: task_name.clone(),
        task_name,
        command,
    })
}

fn parse_agent_doc_table_task_command(
    line: &str,
    header_kind: AgentDocCommandTableHeader,
) -> Option<ParsedAgentBoundaryTaskCommand> {
    let trimmed = line.trim();
    if !looks_like_agent_doc_table_row(trimmed) {
        return None;
    }
    if trimmed.contains("---") {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    if cells.len() < 2 {
        return None;
    }
    let (task_name, command) = match header_kind {
        AgentDocCommandTableHeader::TaskCommand => {
            let label = cells[0];
            let command = parse_backticked_command(cells[1])?;
            let task_name = canonical_agent_doc_task_name_from_command(&command)
                .or_else(|| canonical_agent_doc_task_name(label))?;
            (task_name, command)
        }
        AgentDocCommandTableHeader::CommandDescription => {
            let command = parse_backticked_command(cells[0])?;
            let task_name = canonical_agent_doc_task_name_from_command(&command)?;
            (task_name, command)
        }
    };
    Some(ParsedAgentBoundaryTaskCommand {
        source_key: task_name.clone(),
        task_name,
        command,
    })
}

fn parse_backticked_command(value: &str) -> Option<String> {
    let start = value.find('`')?;
    let tail = value.get(start + 1..)?;
    let end = tail.find('`')?;
    let command = tail.get(..end)?.trim();
    (!command.is_empty() && is_concrete_agent_doc_command(command)).then(|| command.to_string())
}

fn is_concrete_agent_doc_command(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.contains("path/to/") {
        return false;
    }

    let bytes = trimmed.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'<' {
            continue;
        }
        if let Some(relative_end) = bytes[index + 1..]
            .iter()
            .position(|candidate| *candidate == b'>')
        {
            let placeholder = &trimmed[index + 1..index + 1 + relative_end];
            if !placeholder.is_empty()
                && placeholder.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | ':' | '.')
                })
            {
                return false;
            }
        }
    }

    true
}

fn canonical_agent_doc_task_name(label: &str) -> Option<String> {
    let without_parens = strip_parenthetical_segments(label);
    let normalized = without_parens
        .trim()
        .to_ascii_lowercase()
        .replace('&', "and")
        .replace('/', " ")
        .replace('-', " ");
    let compact = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    let compact = compact
        .strip_prefix("run ")
        .or_else(|| compact.strip_prefix("execute "))
        .unwrap_or(&compact);
    match compact {
        "build" => Some(String::from("build")),
        "check" => Some(String::from("check")),
        "compile" => Some(String::from("compile")),
        "lint" => Some(String::from("lint")),
        "docs" | "doc" | "generate docs" => Some(String::from("docs")),
        "test" | "tests" | "test all" | "all unit tests" | "a specific test" | "unit tests" => {
            Some(String::from("test"))
        }
        "typecheck" | "type check" | "type checking" => Some(String::from("typecheck")),
        "format" | "fmt" | "format code" => Some(String::from("fmt")),
        _ if compact.starts_with("build ") => Some(String::from("build")),
        _ if compact.starts_with("check ") => Some(String::from("check")),
        _ if compact.starts_with("lint ") => Some(String::from("lint")),
        _ if compact.starts_with("test ") => Some(String::from("test")),
        _ if compact.ends_with(" test") => Some(String::from("test")),
        _ if compact.ends_with(" tests") => Some(String::from("test")),
        _ if compact.starts_with("docs ") => Some(String::from("docs")),
        _ if compact.starts_with("compile ") => Some(String::from("compile")),
        _ if compact.starts_with("type check") || compact.starts_with("typecheck") => {
            Some(String::from("typecheck"))
        }
        _ if compact.starts_with("format ") => Some(String::from("fmt")),
        _ => None,
    }
}

fn canonical_agent_doc_task_name_from_command(command: &str) -> Option<String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        ["pnpm" | "npm" | "yarn" | "bun", "run", script, ..] => {
            canonical_agent_doc_task_token(script)
        }
        ["pnpm" | "npm" | "yarn" | "bun", script, ..] if !script.starts_with('-') => {
            canonical_agent_doc_task_token(script)
        }
        ["cargo", subcommand, ..] if !subcommand.starts_with('-') => {
            canonical_agent_doc_task_token(subcommand)
        }
        ["pytest", ..] => Some(String::from("test")),
        ["python" | "python3", "-m", "pytest", ..] => Some(String::from("test")),
        ["uv", "run", "pytest", ..] => Some(String::from("test")),
        ["poetry", "run", "pytest", ..] => Some(String::from("test")),
        ["ruff", "check", ..] => Some(String::from("lint")),
        ["python" | "python3", "-m", "build", ..] => Some(String::from("build")),
        ["task" | "just" | "make", task, ..] if !task.starts_with('-') => {
            canonical_agent_doc_task_token(task)
        }
        ["npx", "nx", "affected", "-t", task, ..] => canonical_agent_doc_task_token(task),
        ["pnpm", "exec", "nx", "affected", "-t", task, ..] => canonical_agent_doc_task_token(task),
        ["npx", "nx", "run", task, ..] => canonical_agent_doc_task_token(task),
        ["pnpm", "exec", "nx", "run", task, ..] => canonical_agent_doc_task_token(task),
        _ => None,
    }
}

fn canonical_agent_doc_task_token(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '-' | '_' | '.'))
    {
        return Some(trimmed.to_ascii_lowercase());
    }
    canonical_agent_doc_task_name(trimmed)
}

fn strip_parenthetical_segments(value: &str) -> String {
    let mut stripped = String::with_capacity(value.len());
    let mut depth = 0usize;
    for ch in value.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => stripped.push(ch),
            _ => {}
        }
    }
    stripped
}

#[cfg(test)]
mod tests {
    use super::parse_agent_boundary_doc;

    #[test]
    fn parses_compact_markdown_separator_in_common_commands_table() {
        let parsed = parse_agent_boundary_doc(
            r#"# CLAUDE.md

## Common commands

| Task | Command |
|---|---|
| Run JS tests | `npm run test` |
| Run full RSpec suite | `bin/rails spec` |
| One-time setup | `bin/setup` |
"#,
        );

        assert!(
            parsed
                .task_commands
                .iter()
                .any(|command| command.task_name == "test" && command.command == "npm run test")
        );
    }

    #[test]
    fn skips_agent_doc_commands_with_placeholders() {
        let parsed = parse_agent_boundary_doc(
            r#"# AGENTS.md

## Commands

- Test all: `uv run --project <PROJECT> pytest path/to/test.py::TestClass::test_method -xvs`
"#,
        );

        assert!(parsed.task_commands.is_empty());
    }

    #[test]
    fn prefers_command_derived_task_name_over_prose_alias_in_task_tables() {
        let parsed = parse_agent_boundary_doc(
            r#"# CLAUDE.md

## Quick Reference Commands

| Task | Command |
| --- | --- |
| Check formatting | `pnpm run format:diff` |
| Format code | `pnpm run format` |
"#,
        );

        assert!(
            parsed
                .task_commands
                .iter()
                .any(|command| command.task_name == "format:diff"
                    && command.command == "pnpm run format:diff")
        );
        assert!(
            parsed.task_commands.iter().any(
                |command| command.task_name == "format" && command.command == "pnpm run format"
            )
        );
        assert!(
            !parsed
                .task_commands
                .iter()
                .any(|command| command.task_name == "check")
        );
        assert!(
            !parsed
                .task_commands
                .iter()
                .any(|command| command.task_name == "fmt")
        );
    }
}
